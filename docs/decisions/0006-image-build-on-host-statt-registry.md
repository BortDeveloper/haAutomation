# ADR-0006: Image-Build auf dem Zielhost statt Registry-Pull

- **Status:** accepted
- **Datum:** 2026-05-17 (retroaktiv dokumentiert)
- **Bezug:** Architekt-Audit R6, Finding 7 (Tag-Inkonsistenz)

## Kontext

Das `inventory`-Image könnte aus einer Container-Registry (z.B. GHCR) gezogen
oder direkt auf dem Zielhost gebaut werden. Eine Registry bringt zusätzliche
Komplexität: Auth, Push-Workflow, Tag-/Digest-Verwaltung.

## Entscheidung

- In V1 wird das `inventory`-Image **auf dem Zielhost gebaut**
  (`docker build -f docker/Dockerfile`), nicht aus einer Registry gezogen.
- Die Standalone-Compose-Variante (`docker-compose.yml`) nutzt entsprechend
  `build:` + `image: inventory:dev`.
- Die vps-Variante (`docker-compose.vps.yml`) erwartet dagegen ein per
  **Digest gepinntes** Image (`INVENTORY_IMAGE=...@sha256:<digest>`) — für den
  Fall, dass doch eine Registry genutzt wird.

## Folgen

**Positiv**

- Keine Registry-Infrastruktur, kein Push-Schritt im Deploy.

**Negativ / Kosten**

- Kein zentrales, signiertes Artefakt; Reproduzierbarkeit hängt am Build-Host.
- **Bekannte Inkonsistenz:** Standalone baut lokal, vps pinnt per Digest.
  Die *fremden* Images (Caddy, VPN-Sidecars, sops) werden in allen Overlays
  per Digest gepinnt (Architekt-R4) — siehe `docker/`-Compose-Dateien. Die
  vollständige Auflösung Richtung Registry-Push ist mit der CI-Einführung
  (Phase 2, S19) zu treffen.
