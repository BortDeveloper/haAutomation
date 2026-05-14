# haBortfeld — Home Automation Bortfeld

Single Source of Truth fuer die Heim-Automatisierung. Erfasst, dokumentiert
und refaktorisiert die ueber Home Assistant, Node-RED und Homematic verteilte
Logik. Bietet ein eigenes Inventarisierungs-Backend (in Rust) auf einem
Strato-VPS, das ueber VPN das heimische Netz inspiziert.

**Status:** Phase 1 + 2 + 3 abgeschlossen, 52 cargo-Tests gruen. Strato-Deploy
(VPN-Sidecar + Caddy + Authentik forward_auth) folgt in Phase 4. Details:
[docs/roadmap.md](docs/roadmap.md).

## Komponenten

| Bereich | Rolle | Wohin im Repo |
|---|---|---|
| Home Assistant (HAOS) | UI, State-Registry, einfache Automationen | `homeassistant/` (geplant) |
| Node-RED (HA-Addon) | komplexe Flows, Timer, Notifications | `nodered/` (geplant) |
| Homematic CCU (RaspberryMatic) | Direktverknuepfungen, latenzkritische Geraete | `homematic/` (geplant) |
| Philips Hue (Multi-Bridge) | dimmbare Lichter + Sensoren, ueber v1-REST | abgebildet im Inventory |
| Shellys (20+, mDNS) | Schalt-/Roller-/Sensoren, Gen1+Gen2 HTTP | abgebildet im Inventory |
| Zigbee2MQTT (HA-Addon) | Zigbee-Devices ueber MQTT (Phase-5-Sync) | `nodered/` (geplant) |
| Inventory-Backend (Rust) | sammelt Geraete/Firmware/Software, Web-UI | `inventory/` |
| Doku & Mapping | Ist-Stand, Ownership, ADRs | `docs/` |

## Sync-Quellen (aktiv)

| Source | CLI | Liefert | Firmware-Tracking |
|---|---|---|---|
| Home Assistant | `inventory sync ha --url --token` | Devices via `/api/states` (14 Domains gefiltert) | nein (HA-API) |
| Homematic CCU | `inventory sync ccu --url` | Devices via `xmlapi/devicelist.cgi` | ja, diff-basiert |
| Philips Hue | `inventory sync hue --config <yaml>` | Lights+Sensoren, mehrere Bridges | ja, diff-basiert |
| Shelly | `inventory sync shelly [--ip ...] [--discover-seconds N]` | Gen1+Gen2 via HTTP, mDNS-Auto-Discovery | ja, diff-basiert |

## Quick Links

- [Anforderungen](docs/requirements.md) — was gebaut wird und warum
- [Architektur](docs/architecture.md) — Komponenten, Datenfluss, Trust
- [Roadmap](docs/roadmap.md) — Schrittplan S1–S14
- [Strato-Setup](docs/strato-setup.md) — Server-Bootstrap, SSH/Keys, Operations, DR
- [Inventory Backend](inventory/) — Rust-App
- [Docker / VPN](inventory/docker/README.md) — Sidecar-Setup, Provider-Wechsel
- [Secrets](inventory/secrets/) — sops+age-Layout

## Konventionen

- **GitOps:** alles ausser dem age-Privatkey lebt im Repo
- **PRs mit Test-Gate:** jeder Schritt aus der Roadmap = ein PR, mit reproduzierbarem Test
- **Sprache:** Code & Identifier englisch, Doku deutsch
- **Lizenz:** privat / TBD
