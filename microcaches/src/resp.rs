use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;

// Représente une valeur RESP
#[derive(Debug)]
pub enum ValeurResp {
    SimpleString(String),   // +OK
    Error(String),          // -ERR message
    Integer(i64),           // :42
    BulkString(String),     // $6\r\nfoobar
    Null,                   // $-1 (valeur nulle)
    Array(Vec<ValeurResp>), // *3\r\n...
}

pub async fn lire_valeur(reader: &mut BufReader<OwnedReadHalf>) -> Result<ValeurResp, String> {
    let mut ligne = String::new();
    reader.read_line(&mut ligne).await.map_err(|e| e.to_string())?;
    let ligne = ligne.trim_end_matches("\r\n").trim_end_matches("\n");

    if ligne.is_empty() {
        return Err("Ligne vide".to_string());
    }

    match &ligne[0..1] {
        // Simple String : +OK
        "+" => Ok(ValeurResp::SimpleString(ligne[1..].to_string())),

        // Error : -ERR message
        "-" => Ok(ValeurResp::Error(ligne[1..].to_string())),

        // Integer : :42
        ":" => {
            let n = ligne[1..].parse::<i64>().map_err(|e| e.to_string())?;
            Ok(ValeurResp::Integer(n))
        }

        // Bulk String : $6\r\nfoobar\r\n
        "$" => {
            let taille: i64 = ligne[1..].parse::<i64>().map_err(|e| e.to_string())?;
            if taille == -1 {
                return Ok(ValeurResp::Null);
            }
            let mut buf = vec![0u8; taille as usize + 2]; // +2 pour \r\n
            reader.read_exact(&mut buf).await.map_err(|e| e.to_string())?;
            let contenu = String::from_utf8_lossy(&buf[..taille as usize]).to_string();
            Ok(ValeurResp::BulkString(contenu))
        }

        // Array : *3\r\n...
        "*" => {
            let nb: i64 = ligne[1..].parse::<i64>().map_err(|e| e.to_string())?;
            if nb == -1 {
                return Ok(ValeurResp::Null);
            }
            let mut elements = Vec::new();
            for _ in 0..nb {
                let element = Box::pin(lire_valeur(reader)).await?;
                elements.push(element);
            }
            Ok(ValeurResp::Array(elements))
        }

        autre => Err(format!("Type RESP inconnu : {}", autre)),
    }
}


// Encoder une réponse simple (+OK, -ERR, etc.)
pub fn encoder_ok() -> String {
    "+OK\r\n".to_string()
}

pub fn encoder_erreur(message: &str) -> String {
    format!("-ERR {}\r\n", message)
}

pub fn encoder_entier(n: i64) -> String {
    format!(":{}\r\n", n)
}

pub fn encoder_chaine(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

pub fn encoder_null() -> String {
    "$-1\r\n".to_string()
}

pub fn encoder_tableau(elements: Vec<String>) -> String {
    let mut resultat = format!("*{}\r\n", elements.len());
    for element in elements {
        resultat.push_str(&element);
    }
    resultat
}

// Convertir une ValeurResp en commande texte qu'on peut passer au parser
pub fn resp_vers_commande(valeur: ValeurResp) -> Result<String, String> {
    match valeur {
        ValeurResp::Array(elements) => {
            let mut mots: Vec<String> = Vec::new();
            for e in elements {
                match e {
                    ValeurResp::BulkString(s) => mots.push(s),
                    ValeurResp::SimpleString(s) => mots.push(s),
                    autre => return Err(format!("Element inattendu : {:?}", autre)),
                }
            }
            Ok(mots.join(" "))
        }
        ValeurResp::BulkString(s) => Ok(s),
        ValeurResp::SimpleString(s) => Ok(s),
        autre => Err(format!("Format de commande invalide : {:?}", autre)),
    }
}