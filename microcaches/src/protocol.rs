#[derive(Debug, PartialEq)]
pub enum Commande {
    Ping,
    Get(String),
    Set {
        cle: String,
        valeur: Vec<u8>,
        ttl: Option<u64>,
    },
    Del(String),
    Keys(String),
    Ttl(String),
    Flush,
}

#[derive(Debug)]
pub enum ErreurParsing {
    CommandeInconnue(String),
    ArgumentsManquants,
    NombreInvalide(String),
}

pub fn parser_commande(ligne: &str) -> Result<Commande, ErreurParsing> {
    let mots: Vec<&str> = ligne.trim().split_whitespace().collect();

    if mots.is_empty() {
        return Err(ErreurParsing::ArgumentsManquants);
    }

    match mots[0].to_uppercase().as_str() {
        "PING" => Ok(Commande::Ping),

        "GET" => {
            if mots.len() < 2 {
                return Err(ErreurParsing::ArgumentsManquants);
            }
            Ok(Commande::Get(mots[1].to_string()))
        }

        "DEL" => {
            if mots.len() < 2 {
                return Err(ErreurParsing::ArgumentsManquants);
            }
            Ok(Commande::Del(mots[1].to_string()))
        }

        "SET" => {
            if mots.len() < 3 {
                return Err(ErreurParsing::ArgumentsManquants);
            }
            let cle = mots[1].to_string();
            let valeur = mots[2].as_bytes().to_vec();

            let ttl = if mots.len() >= 5 && mots[3].to_uppercase() == "EX" {
                let secs = mots[4].parse::<u64>().map_err(|_| {
                    ErreurParsing::NombreInvalide(mots[4].to_string())
                })?;
                Some(secs)
            } else {
                None
            };

            Ok(Commande::Set { cle, valeur, ttl })
        }

        "KEYS" => {
            let pattern = if mots.len() >= 2 {
                mots[1].to_string()
            } else {
                "*".to_string()
            };
            Ok(Commande::Keys(pattern))
        }

        "TTL" => {
            if mots.len() < 2 {
                return Err(ErreurParsing::ArgumentsManquants);
            }
            Ok(Commande::Ttl(mots[1].to_string()))
        }

        "FLUSH" => Ok(Commande::Flush),

        autre => Err(ErreurParsing::CommandeInconnue(autre.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping() {
        assert_eq!(parser_commande("PING").unwrap(), Commande::Ping);
    }

    #[test]
    fn test_ping_minuscule() {
        assert_eq!(parser_commande("ping").unwrap(), Commande::Ping);
    }

    #[test]
    fn test_get() {
        assert_eq!(
            parser_commande("GET nom").unwrap(),
            Commande::Get("nom".to_string())
        );
    }

    #[test]
    fn test_set_sans_ttl() {
        assert_eq!(
            parser_commande("SET nom Alice").unwrap(),
            Commande::Set {
                cle: "nom".to_string(),
                valeur: b"Alice".to_vec(),
                ttl: None,
            }
        );
    }

    #[test]
    fn test_set_avec_ttl() {
        assert_eq!(
            parser_commande("SET nom Alice EX 60").unwrap(),
            Commande::Set {
                cle: "nom".to_string(),
                valeur: b"Alice".to_vec(),
                ttl: Some(60),
            }
        );
    }

    #[test]
    fn test_del() {
        assert_eq!(
            parser_commande("DEL nom").unwrap(),
            Commande::Del("nom".to_string())
        );
    }

    #[test]
    fn test_commande_inconnue() {
        assert!(parser_commande("BONJOUR").is_err());
    }

    #[test]
    fn test_get_sans_argument() {
        assert!(parser_commande("GET").is_err());
    }

    #[test]
    fn test_ttl_invalide() {
        assert!(parser_commande("SET nom Alice EX abc").is_err());
    }
}