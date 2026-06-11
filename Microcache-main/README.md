# MicroCache

Serveur de cache clé/valeur en Rust, compatible avec le protocole Redis (RESP).

---

## Prérequis

Avant de lancer le projet, assure-toi d'avoir installé :

- **Rust** — https://rustup.rs
- **Python 3** (optionnel, pour les tests) — https://python.org
- **Docker** (optionnel, pour la containerisation) — https://docker.com

Pour vérifier que Rust est bien installé :

```bash
rustc --version
cargo --version
```

---

## Installation

### 1. Cloner ou télécharger le projet

```bash
git clone <url-du-repo>
cd microcache
```

Ou si tu as reçu le projet en archive ZIP, décompresse-le et ouvre un terminal dans le dossier `microcache/`.

### 2. Compiler le projet

```bash
cargo build --release
```

La première compilation peut prendre quelques minutes car Cargo télécharge et compile les dépendances.

---

## Lancer le serveur

```bash
cargo run
```

Le serveur démarre sur **127.0.0.1:6379** (le même port que Redis).

Tu dois voir :

```
MicroCache en ecoute sur 127.0.0.1:6379...
```

Le serveur sauvegarde automatiquement les données toutes les **10 secondes** dans un fichier `snapshot.bin`. Au redémarrage, les données sont automatiquement rechargées.

Pour arrêter le serveur : **Ctrl+C**

---

## Lancer les tests unitaires

```bash
cargo test
```

Tu dois voir **13 tests** passer avec succès :

```
running 13 tests
test protocol::tests::test_ping ... ok
test protocol::tests::test_set_avec_ttl ... ok
...
test result: ok. 13 passed; 0 failed
```

---

## Utiliser le serveur

### Depuis PowerShell (Windows)

Ouvre un nouveau terminal PowerShell et connecte-toi :

```powershell
$tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", 6379)
$stream = $tcp.GetStream()
$writer = New-Object System.IO.StreamWriter($stream)
$reader = New-Object System.IO.StreamReader($stream)
$writer.AutoFlush = $true
```

Envoie des commandes :

```powershell
$writer.WriteLine("PING")
$reader.ReadLine()
# -> +PONG

$writer.WriteLine("SET ville Yaounde EX 60")
$reader.ReadLine()
# -> +OK

$writer.WriteLine("GET ville")
$reader.ReadLine()
# -> +Yaounde

$writer.WriteLine("TTL ville")
$reader.ReadLine()
# -> :59

$writer.WriteLine("DEL ville")
$reader.ReadLine()
# -> :1

$writer.WriteLine("FLUSH")
$reader.ReadLine()
# -> :0 entree(s) supprimee(s)
```

### Depuis Linux/Mac (netcat)

```bash
nc 127.0.0.1 6379
PING
GET nom
SET nom Alice EX 30
```

---

## Commandes disponibles

| Commande | Description | Exemple |
|---|---|---|
| `PING` | Vérifie que le serveur répond | `PING` |
| `SET cle valeur` | Stocke une valeur | `SET nom Alice` |
| `SET cle valeur EX secondes` | Stocke avec expiration | `SET session abc EX 60` |
| `GET cle` | Récupère une valeur | `GET nom` |
| `DEL cle` | Supprime une entrée | `DEL nom` |
| `TTL cle` | Temps restant avant expiration | `TTL session` |
| `KEYS *` | Liste toutes les clés | `KEYS *` |
| `FLUSH` | Vide tout le store | `FLUSH` |

### Codes de retour

| Code | Signification |
|---|---|
| `+OK` | Succès |
| `+PONG` | Réponse au PING |
| `+valeur` | Valeur retournée |
| `:N` | Nombre entier (TTL, compteur) |
| `:-1` | Clé sans expiration (TTL infini) |
| `:-2` | Clé expirée |
| `-ERR message` | Erreur |

---

## Utiliser avec Python (redis-py)

### Installer la bibliothèque

```bash
python -m pip install redis
```

### Script de test

```python
import redis

client = redis.Redis(host='127.0.0.1', port=6379, decode_responses=True)

# Test de connexion
print(client.ping())           # True

# Stocker et récupérer
client.set('ville', 'Yaounde')
print(client.get('ville'))     # Yaounde

# Avec expiration
client.set('session', 'abc123', ex=60)
print(client.ttl('session'))   # ~60

# Supprimer
client.delete('ville')
```

Lancer le script :

```bash
python test_redis.py
```

---

## Utiliser avec Docker

### Lancer avec Docker Compose

```bash
docker-compose up
```

Cette commande lance MicroCache dans un conteneur et expose le port 6379.

### Lancer manuellement avec Docker

```bash
# Construire l'image
docker build -t microcache .

# Lancer le conteneur
docker run -p 6379:6379 -v $(pwd)/snapshot.bin:/app/snapshot.bin microcache
```

---

## Structure du projet

```
microcache/
├── Cargo.toml           # Configuration et dépendances Rust
├── Dockerfile           # Configuration Docker
├── docker-compose.yml   # Orchestration des conteneurs
├── .dockerignore        # Fichiers exclus de l'image Docker
├── test_redis.py        # Script de test Python
├── snapshot.bin         # Fichier de persistance (créé automatiquement)
└── src/
    ├── main.rs          # Point d'entrée — assemble tous les modules
    ├── store.rs         # Partie 1 — Store en mémoire thread-safe
    ├── reaper.rs        # Partie 2 — Thread de nettoyage TTL
    ├── protocol.rs      # Partie 3 — Parser de commandes textuelles
    ├── server.rs        # Partie 4 — Serveur TCP multi-connexions
    ├── persistence.rs   # Partie 5 — Sauvegarde et rechargement sur disque
    └── resp.rs          # Extension — Protocole RESP (compatibilité Redis)
```

---

## Dépendances

| Bibliothèque | Version | Usage |
|---|---|---|
| `tokio` | 1.x | Runtime asynchrone pour le serveur TCP |
| `serde` | 1.x | Sérialisation des structures de données |
| `bincode` | 1.x | Format binaire pour la persistance sur disque |

---

## Dépannage

### Le serveur ne démarre pas — port déjà utilisé

Le port 6379 est peut-être occupé par une autre application (Redis, une autre instance de MicroCache). Vérifie avec :

```bash
# Windows
netstat -ano | findstr :6379

# Linux/Mac
lsof -i :6379
```

### cargo build échoue — erreur réseau

Relance simplement la commande, Cargo reprend là où il s'est arrêté :

```bash
cargo build
```

### Les données ne sont pas sauvegardées

La sauvegarde automatique s'effectue toutes les 10 secondes. Si tu arrêtes le serveur moins de 10 secondes après avoir stocké des données, elles ne seront pas sauvegardées. Attends au moins 10 secondes avant d'arrêter le serveur.

---

## Auteurs

Projet réalisé dans le cadre du cours de programmation système en Rust.
