# haAutomation — Home Automation Site

Single Source of Truth fuer die Heim-Automatisierung. Erfasst, dokumentiert
und refaktorisiert die ueber Home Assistant, Node-RED und Homematic verteilte
Logik. Bietet ein eigenes Inventarisierungs-Backend (in Rust) auf einem
VPS-VPS, das ueber VPN das heimische Netz inspiziert.

**Status:** in Aufbau — siehe [docs/roadmap.md](docs/roadmap.md). Aktuell vor S1.

## Komponenten

| Bereich | Rolle | Wohin im Repo |
|---|---|---|
| Home Assistant | UI, State-Registry, einfache Automationen | `homeassistant/` (geplant) |
| Node-RED | komplexe Flows, Timer, Notifications | `nodered/` (geplant) |
| Homematic CCU | Direktverknuepfungen, latenzkritische Geraete | `homematic/` (geplant) |
| Inventory-Backend | sammelt Geraete/Firmware/Software, Web-UI | `inventory/` |
| Doku & Mapping | Ist-Stand, Ownership, ADRs | `docs/` |

## Quick Links

- [Anforderungen](docs/requirements.md) — was gebaut wird und warum
- [Architektur](docs/architecture.md) — Komponenten, Datenfluss, Trust
- [Roadmap](docs/roadmap.md) — Schrittplan S1–S14
- [Inventory Backend](inventory/) — Rust-App
- [Docker / VPN](inventory/docker/README.md) — Sidecar-Setup, Provider-Wechsel
- [Secrets](inventory/secrets/) — sops+age-Layout

## Konventionen

- **GitOps:** alles ausser dem age-Privatkey lebt im Repo
- **PRs mit Test-Gate:** jeder Schritt aus der Roadmap = ein PR, mit reproduzierbarem Test
- **Sprache:** Code & Identifier englisch, Doku deutsch
- **Lizenz:** privat / TBD
