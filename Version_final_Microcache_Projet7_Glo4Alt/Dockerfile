# =========================================================
# Etape 1 : Compilation du binaire Rust avec cache des deps
# =========================================================
FROM rust:1.85-slim AS builder

WORKDIR /app

# 1) Copier d'abord uniquement les manifestes pour mettre en cache la compilation des dependances
COPY Cargo.toml Cargo.lock ./

# 2) Creer un main.rs factice pour permettre la compilation des dependances seules
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/microcache*

# 3) Copier les vraies sources et compiler
COPY src ./src
RUN cargo build --release

# =========================================================
# Etape 2 : Image finale legere
# =========================================================
FROM debian:bookworm-slim

# Utilisateur non-root pour la securite
RUN useradd --system --create-home --uid 1001 microcache

WORKDIR /app
COPY --from=builder /app/target/release/microcache /usr/local/bin/microcache

# Le binaire ecrit snapshot.bin dans le WORKDIR — on s'assure qu'il appartient a l'utilisateur
RUN chown -R microcache:microcache /app

USER microcache

EXPOSE 6379

CMD ["microcache"]
