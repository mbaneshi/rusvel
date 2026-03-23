# ── Stage 1: Build frontend ──
FROM node:22-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ── Stage 2: Build Rust binary ──
FROM rust:1.87-bookworm AS builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY --from=frontend /app/frontend/build frontend/build

RUN cargo build --release --bin rusvel-app

# ── Stage 3: Runtime ──
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rusvel-app /usr/local/bin/rusvel

ENV RUST_LOG=info
EXPOSE 3000

ENTRYPOINT ["rusvel"]
