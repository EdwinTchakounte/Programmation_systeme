import os
import time
import redis

host = os.environ.get('REDIS_HOST', '127.0.0.1')
port = int(os.environ.get('REDIS_PORT', '6379'))

# Petite attente pour laisser le serveur demarrer dans le compose
for tentative in range(10):
    try:
        client = redis.Redis(host=host, port=port, decode_responses=True, socket_connect_timeout=2)
        client.ping()
        break
    except Exception:
        if tentative == 9:
            raise
        time.sleep(1)

print(f"=== Connecte a {host}:{port} ===")

print("\n=== Test PING ===")
print(f"PING -> {client.ping()}")

print("\n=== Test SET et GET ===")
client.set('ville', 'Yaounde')
print(f"SET ville Yaounde -> OK")
print(f"GET ville -> {client.get('ville')}")

print("\n=== Test SET avec TTL ===")
client.set('session', 'abc123', ex=60)
print(f"SET session abc123 EX 60 -> OK")
print(f"TTL session -> {client.ttl('session')} secondes")

print("\n=== Test DEL ===")
client.delete('ville')
print(f"DEL ville -> OK")
print(f"GET ville apres suppression -> {client.get('ville')}")

print("\n=== Tous les tests sont passes ! ===")
