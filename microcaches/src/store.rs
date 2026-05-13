use std::time::{Instant, SystemTime, UNIX_EPOCH, Duration};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

pub struct CacheEntry {
    pub valeur: Vec<u8>,
    pub expiration: Option<Instant>,
}

impl CacheEntry {
    pub fn est_expiree(&self) -> bool {
        match self.expiration {
            Some(exp) => Instant::now() > exp,
            None => false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CacheEntrySerializable {
    pub valeur: Vec<u8>,
    pub expiration_timestamp: Option<u64>,
}

pub struct Store {
    pub donnees: HashMap<String, CacheEntry>,
}

impl Store {
    pub fn nouveau() -> Self {
        Store {
            donnees: HashMap::new(),
        }
    }

    pub fn set(&mut self, cle: String, valeur: Vec<u8>, ttl_secs: Option<u64>) {
        let expiration = ttl_secs.map(|secs| {
            Instant::now() + std::time::Duration::from_secs(secs)
        });
        self.donnees.insert(cle, CacheEntry { valeur, expiration });
    }

    pub fn get(&self, cle: &str) -> Option<&Vec<u8>> {
        match self.donnees.get(cle) {
            Some(entry) if !entry.est_expiree() => Some(&entry.valeur),
            _ => None,
        }
    }

    pub fn delete(&mut self, cle: &str) -> bool {
        self.donnees.remove(cle).is_some()
    }

    pub fn exists(&self, cle: &str) -> bool {
        self.get(cle).is_some()
    }

    pub fn ttl(&self, cle: &str) -> Option<i64> {
        match self.donnees.get(cle) {
            Some(entry) => match entry.expiration {
                Some(exp) => {
                    let now = Instant::now();
                    if exp > now {
                        Some((exp - now).as_secs() as i64)
                    } else {
                        Some(-2)
                    }
                }
                None => Some(-1),
            },
            None => None,
        }
    }

    pub fn flush(&mut self) -> usize {
        let nb = self.donnees.len();
        self.donnees.clear();
        nb
    }

    pub fn nettoyer_expires(&mut self) -> usize {
        let avant = self.donnees.len();
        self.donnees.retain(|_, entry| !entry.est_expiree());
        avant - self.donnees.len()
    }
}

pub type SharedStore = Arc<RwLock<Store>>;

pub fn nouveau_store_partage() -> SharedStore {
    Arc::new(RwLock::new(Store::nouveau()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_et_get() {
        let mut store = Store::nouveau();
        store.set("nom".to_string(), b"Alice".to_vec(), None);
        assert_eq!(store.get("nom"), Some(&b"Alice".to_vec()));
    }

    #[test]
    fn test_get_cle_inexistante() {
        let store = Store::nouveau();
        assert_eq!(store.get("inexistant"), None);
    }

    #[test]
    fn test_delete() {
        let mut store = Store::nouveau();
        store.set("nom".to_string(), b"Alice".to_vec(), None);
        assert!(store.delete("nom"));
        assert_eq!(store.get("nom"), None);
    }

    #[test]
    fn test_exists() {
        let mut store = Store::nouveau();
        store.set("nom".to_string(), b"Alice".to_vec(), None);
        assert!(store.exists("nom"));
        store.delete("nom");
        assert!(!store.exists("nom"));
    }

    #[test]
    fn test_ttl_expire() {
        let mut store = Store::nouveau();
        store.set("nom".to_string(), b"Alice".to_vec(), Some(0));
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(store.get("nom"), None);
    }

    #[test]
    fn test_flush() {
        let mut store = Store::nouveau();
        store.set("a".to_string(), b"1".to_vec(), None);
        store.set("b".to_string(), b"2".to_vec(), None);
        assert_eq!(store.flush(), 2);
        assert_eq!(store.get("a"), None);
    }
}