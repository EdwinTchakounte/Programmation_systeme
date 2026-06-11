use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH, Duration};
use crate::store::{Store, SharedStore, CacheEntry, CacheEntrySerializable};

pub fn sauvegarder(store: &SharedStore, chemin: &str) -> Result<(), String> {
    let store_guard = store.read().map_err(|e| e.to_string())?;

    // Convertir toutes les entrées en format sérialisable
    let serialisable: HashMap<String, CacheEntrySerializable> = store_guard
        .donnees
        .iter()
        .map(|(cle, entry)| {
            let ts = entry.expiration.map(|instant| {
                let unix_now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let now = Instant::now();
                if instant > now {
                    unix_now + (instant - now).as_secs()
                } else {
                    0
                }
            });
            (cle.clone(), CacheEntrySerializable {
                valeur: entry.valeur.clone(),
                expiration_timestamp: ts,
            })
        })
        .collect();

    // Sérialiser en binaire
    let octets = bincode::serialize(&serialisable)
        .map_err(|e| e.to_string())?;

    // Écrire dans le fichier
    std::fs::write(chemin, octets).map_err(|e| e.to_string())?;
    println!("Snapshot sauvegarde dans '{}'", chemin);
    Ok(())
}

pub fn charger(store: &SharedStore, chemin: &str) -> Result<usize, String> {
    let octets = std::fs::read(chemin).map_err(|e| e.to_string())?;

    let snapshot: HashMap<String, CacheEntrySerializable> = bincode::deserialize(&octets)
        .map_err(|e| e.to_string())?;

    let mut store_guard = store.write().map_err(|e| e.to_string())?;
    let mut nb = 0;

    for (cle, entry_ser) in snapshot {
        let unix_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Ne recharger que les entrées non expirées
        let encore_valide = entry_ser.expiration_timestamp
            .map_or(true, |ts| ts > unix_now);

        if encore_valide {
            let expiration = entry_ser.expiration_timestamp.map(|ts| {
                Instant::now() + Duration::from_secs(ts - unix_now)
            });
            store_guard.donnees.insert(cle, CacheEntry {
                valeur: entry_ser.valeur,
                expiration,
            });
            nb += 1;
        }
    }

    println!("{} entree(s) rechargee(s) depuis '{}'", nb, chemin);
    Ok(nb)
}