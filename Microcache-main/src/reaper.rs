use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::store::SharedStore;

pub struct Reaper {
    arret: mpsc::Sender<()>,
}

impl Reaper {
    pub fn demarrer(store: SharedStore, intervalle_secs: u64) -> Self {
        let (envoyeur, receveur) = mpsc::channel();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(intervalle_secs));

                if receveur.try_recv().is_ok() {
                    println!("Reaper : arret demande, bye !");
                    break;
                }

                let mut store_guard = store.write().unwrap();
                let nb = store_guard.nettoyer_expires();
                if nb > 0 {
                    println!("Reaper : {} entree(s) expiree(s) supprimee(s)", nb);
                }
            }
        });

        Reaper { arret: envoyeur }
    }

    pub fn arreter(self) {
        let _ = self.arret.send(());
    }
}