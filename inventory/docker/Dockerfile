# syntax=docker/dockerfile:1.7
#
# Multi-stage Build fuer das inventory-Binary.
# Stage 1 baut statisch gegen musl (Default bei rust:1-alpine).
# Stage 2 ist alpine:3.20 + ca-certificates + tini + non-root user.
#
# Build aus dem Crate-Root:
#   docker build -f docker/Dockerfile -t inventory:dev .
#
# Ziel-Imagegroesse: < 30 MB (NFR-9).

# === Build stage ===
FROM rust:1-alpine AS builder

# rusqlite-bundled kompiliert SQLite-C aus dem Quelltext.
# Dafuer brauchen wir musl-dev + gcc + make.
RUN apk add --no-cache musl-dev gcc make

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Baut beide Binaries: `inventory` (Backend) und `authgate` (Auth-Sidecar).
RUN cargo build --release --locked && \
    strip target/release/inventory target/release/authgate

# === Runtime stage ===
FROM alpine:3.20

# Build-Args: vom CI/Build-Skript zu setzen (siehe scripts/trivy-scan.sh / GHCR-Workflow).
# Defaults erlauben einen lokalen `docker build` ohne extra Flags.
ARG VERSION=0.0.0-dev
ARG REVISION=unknown
ARG BUILD_DATE=

LABEL org.opencontainers.image.title="haBortfeld inventory" \
      org.opencontainers.image.description="Asset- und Prozess-Inventory fuer haBortfeld (Standalone & strato-stack)" \
      org.opencontainers.image.source="https://github.com/BortDeveloper/haAutomation" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.authors="bortdeveloper <guebraun@gmail.com>" \
      org.opencontainers.image.vendor="bortdeveloper" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${REVISION}" \
      org.opencontainers.image.created="${BUILD_DATE}"

RUN apk add --no-cache ca-certificates tini && \
    addgroup -S inventory && \
    adduser -S -G inventory -H -h /var/lib/inventory inventory && \
    mkdir -p /var/lib/inventory && \
    chown inventory:inventory /var/lib/inventory

USER inventory
WORKDIR /var/lib/inventory

COPY --from=builder /app/target/release/inventory /usr/local/bin/inventory
COPY --from=builder /app/target/release/authgate  /usr/local/bin/authgate

ENV INVENTORY_DB=/var/lib/inventory/inventory.db \
    INVENTORY_LISTEN=0.0.0.0:8080 \
    RUST_LOG=info

EXPOSE 8080

# Default-Healthcheck zielt auf das `inventory`-Binary (Standard-Entrypoint).
# Wer dasselbe Image als `authgate` startet, ueberschreibt den Healthcheck im
# Compose (siehe docker-compose.yml: authgate -> /auth/health auf :9000).
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget -q -O- http://127.0.0.1:8080/health || exit 1

# tini reicht SIGTERM korrekt durch fuer schnelles docker stop.
ENTRYPOINT ["/sbin/tini", "--", "/usr/local/bin/inventory"]
CMD ["serve"]
