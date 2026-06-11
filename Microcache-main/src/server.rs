use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::store::SharedStore;
use crate::protocol::{parser_commande, Commande};
use crate::resp::{lire_valeur, resp_vers_commande, ValeurResp};
use crate::resp::{encoder_ok, encoder_erreur, encoder_entier, encoder_chaine, encoder_null, encoder_tableau};

pub async fn demarrer(store: SharedStore) {
    let listener = TcpListener::bind("0.0.0.0:6379").await.unwrap();
    println!("MicroCache en ecoute sur 0.0.0.0:6379...");

    loop {
        let (socket, adresse) = listener.accept().await.unwrap();
        println!("Nouveau client connecte : {}", adresse);
        let store_clone = store.clone();
        tokio::spawn(async move {
            traiter_client(socket, store_clone).await;
        });
    }
}

async fn traiter_client(socket: tokio::net::TcpStream, store: SharedStore) {
    let (lecteur, mut ecrivain) = socket.into_split();
    let mut reader = BufReader::new(lecteur);

    loop {
        // Lire le premier octet pour détecter le protocole
        let mut premier_octet = [0u8; 1];
        match tokio::io::AsyncReadExt::read_exact(&mut reader, &mut premier_octet).await {
            Ok(_) => {}
            Err(_) => {
                println!("Client deconnecte");
                break;
            }
        }

        let reponse = if premier_octet[0] == b'*' {
            // Format RESP : commence par '*'
            // On remet le '*' dans un buffer et on lit le reste
            let mut ligne = String::new();
            reader.read_line(&mut ligne).await.unwrap_or(0);
            let nb: i64 = ligne.trim().parse().unwrap_or(0);

            // Lire les elements du tableau RESP
            let mut elements = Vec::new();
            for _ in 0..nb {
                match lire_valeur(&mut reader).await {
                    Ok(val) => elements.push(val),
                    Err(_) => break,
                }
            }

            // Convertir en commande texte
            let commande_texte = match resp_vers_commande(ValeurResp::Array(elements)) {
                Ok(cmd) => cmd,
                Err(e) => {
                    ecrivain.write_all(encoder_erreur(&e).as_bytes()).await.ok();
                    continue;
                }
            };

            println!("Commande RESP recue : {}", commande_texte);
            executer_commande(&commande_texte, &store, true)

        } else {
            // Format textuel simple : lire le reste de la ligne
            let mut reste = String::new();
            reader.read_line(&mut reste).await.unwrap_or(0);
            let ligne = format!("{}{}", premier_octet[0] as char, reste);
            let ligne = ligne.trim().to_string();

            println!("Commande texte recue : {}", ligne);
            executer_commande(&ligne, &store, false)
        };

        if ecrivain.write_all(reponse.as_bytes()).await.is_err() {
            println!("Client deconnecte");
            break;
        }
    }
}

fn executer_commande(ligne: &str, store: &SharedStore, format_resp: bool) -> String {
    match parser_commande(ligne) {
        Ok(Commande::Ping) => {
            if format_resp { "+PONG\r\n".to_string() }
            else { "+PONG\r\n".to_string() }
        }

        Ok(Commande::Get(cle)) => {
            let store_guard = store.read().unwrap();
            match store_guard.get(&cle) {
                Some(val) => {
                    let s = String::from_utf8_lossy(val).to_string();
                    if format_resp { encoder_chaine(&s) }
                    else { format!("+{}\r\n", s) }
                }
                None => {
                    if format_resp { encoder_null() }
                    else { "-ERR cle introuvable\r\n".to_string() }
                }
            }
        }

        Ok(Commande::Set { cle, valeur, ttl }) => {
            store.write().unwrap().set(cle, valeur, ttl);
            if format_resp { encoder_ok() }
            else { "+OK\r\n".to_string() }
        }

        Ok(Commande::Del(cle)) => {
            let supprime = store.write().unwrap().delete(&cle);
            if format_resp { encoder_entier(if supprime { 1 } else { 0 }) }
            else if supprime { ":1\r\n".to_string() }
            else { ":0\r\n".to_string() }
        }

        Ok(Commande::Ttl(cle)) => {
            let store_guard = store.read().unwrap();
            match store_guard.ttl(&cle) {
                Some(secondes) => {
                    if format_resp { encoder_entier(secondes) }
                    else { format!(":{}\r\n", secondes) }
                }
                None => {
                    if format_resp { encoder_entier(-2) }
                    else { "-ERR cle introuvable\r\n".to_string() }
                }
            }
        }

        Ok(Commande::Keys(_pattern)) => {
            let store_guard = store.read().unwrap();
            let cles: Vec<String> = store_guard.donnees.keys().cloned().collect();
            if format_resp {
                let elements: Vec<String> = cles.iter()
                    .map(|c| encoder_chaine(c))
                    .collect();
                encoder_tableau(elements)
            } else {
                format!("+{}\r\n", cles.join(", "))
            }
        }

        Ok(Commande::Flush) => {
            let nb = store.write().unwrap().flush();
            if format_resp { encoder_entier(nb as i64) }
            else { format!(":{} entree(s) supprimee(s)\r\n", nb) }
        }

        Err(e) => {
            if format_resp { encoder_erreur(&format!("{:?}", e)) }
            else { format!("-ERR {:?}\r\n", e) }
        }
    }
}