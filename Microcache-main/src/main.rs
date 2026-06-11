mod store;
mod protocol;
mod reaper;
mod server;
mod persistence;
mod resp;

use store::nouveau_store_partage;
use reaper::Reaper;

const FICHIER_SNAPSHOT: &str = "snapshot.bin";
const INTERVALLE_SAUVEGARDE: u64 = 10; // sauvegarder toutes les 10 secondes

#[tokio::main]
async fn main() {
    let store = nouveau_store_partage();

    // Charger le snapshot au démarrage si il existe
    match persistence::charger(&store, FICHIER_SNAPSHOT) {
        Ok(nb) => println!("Demarrage : {} entree(s) rechargee(s)", nb),
        Err(_) => println!("Demarrage : aucun snapshot trouve, on repart de zero"),
    }

    // Démarrer le Reaper
    let _reaper = Reaper::demarrer(store.clone(), 5);

    // Sauvegarde automatique toutes les 10 secondes
    let store_sauvegarde = store.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(INTERVALLE_SAUVEGARDE)).await;
            match persistence::sauvegarder(&store_sauvegarde, FICHIER_SNAPSHOT) {
                Ok(_) => println!("Sauvegarde automatique effectuee"),
                Err(e) => println!("Erreur sauvegarde : {}", e),
            }
        }
    });

    // Démarrer le serveur TCP
    server::demarrer(store).await;
}