# ADR-0001: Rust + synchroner Minimal-Stack (kein tokio)

- **Status:** accepted
- **Datum:** 2026-05-17 (retroaktiv dokumentiert; Entscheidung getroffen in S1)
- **Bezug:** Architekt-Audit R6, NFR-2

## Kontext

Das Inventory-Backend braucht einen HTTP-Server (Web-UI), einen HTTP-Client
(Sync gegen HA/CCU/Hue/Shelly), eine eingebettete Datenbank, CLI-Parsing,
XML- und mDNS-Unterstützung. Der Eigentümer betreibt das System solo und hat
als ausdrückliche Präferenz: **lesbarer Code vor idiomatischer Eleganz** — „ich
will das verstehen können". Das Lastprofil ist niedrig: periodische
Cron-getriggerte Syncs, eine Handvoll gleichzeitiger UI-Requests.

## Entscheidung

Implementierung in **Rust, durchgängig synchron/blocking**, mit minimalen
Crates:

- `tiny_http` — HTTP-Server, ohne Framework
- `ureq` (blocking) — HTTP-Client
- `rusqlite` (`bundled`) — SQLite
- `clap` — CLI
- `serde` / `serde_json` / `serde_yaml_ng` — Serialisierung
- `roxmltree` — CCU-XML
- `mdns-sd` — Shelly-Discovery
- `hmac` / `sha2` / `pbkdf2` — Krypto im `authgate`-Sidecar

**Explizit nicht** verwendet: `tokio`, `axum`, `sqlx` oder vergleichbare
async-Frameworks.

## Folgen

**Positiv**

- Kein async-Färbung, kein Runtime-Setup — der Kontrollfluss ist linear lesbar.
- Kleine Abhängigkeitsbäume, kleines Release-Image (Ziel <30 MB, NFR-9).

**Negativ / Kosten**

- `rusqlite`-`bundled` und `ureq`→`ring` kompilieren C-Code. Die Windows-
  Workstation kann den Crate deshalb nicht bauen → siehe [ADR-0005](0005-build-host-inventory-statt-ci.md).
- Keine async-Skalierung. Bei dem gegebenen Lastprofil bewusst akzeptiert; ein
  Wechsel wäre ein neues ADR.
