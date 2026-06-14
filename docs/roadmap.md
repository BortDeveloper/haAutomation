# Roadmap

Schritte mit Status. Phase 1 & 2 & 3 sind vollstaendig (Library + Tests).
Die als "vertagt" markierten Live-Tests gegen reale Heim-Systeme werden
gebuendelt am Ende von Phase 3 ausgefuehrt, sobald alle Quellen da sind.

Legende: ✓ erledigt + Gate gruen · ✓ (lib) Library + Mock-Tests fertig,
Live-Smoke noch offen · ☐ offen.

## Phase 1 — Lokales Skelett (kein Netz, kein VPN) — ✓

| Step | Inhalt | Status |
|---|---|---|
| S1 | Cargo-Crate `home-inventory`, `clap`-Subkommandos `serve / sync / migrate` | ✓ |
| S2 | `db.rs` mit Migrations, Tabellen `devices`, `firmware_snapshot`, `software`, `manual_meta` | ✓ (3 Tests) |
| S3 | YAML-Fixture -> `upsert_devices` -> Roundtrip-Read | ✓ (3 Tests) |
| S4 | `tiny_http`-Server, Route `/health` | ✓ (2 Tests + Smoke) |
| S5 | `/api/devices` (JSON), `/` (HTML-Tabelle aus `format!`) | ✓ (4 Tests + Smoke) |

## Phase 2 — Containerisierung & Auth-Huelle — ✓

| Step | Inhalt | Status |
|---|---|---|
| S6 | Dockerfile multi-stage `rust:alpine` -> `alpine:3.20`, ~10 MB | ✓ (VPS build + curl) |
| S7 | Auth-Middleware: `X-Authentik-Username` Pflicht, `AUTH_BYPASS=1` fuer Dev | ✓ (4 Tests + Smoke) |
| S8 | `secrets.rs` shell-outet auf `sops -d`, parst K=V | ✓ (5 Tests) |

## Phase 3 — Sync gegen Heim-Systeme — ✓ (Lib), Live-Smoke vertagt

| Step | Inhalt | Status |
|---|---|---|
| S9 | `sync/ha.rs` + Mapping `HaEntity -> Device`, gegen Mock-Server im Test | ✓ (6 Tests) |
| S10 | HA-Sync gegen echtes HA: CLI-Wiring `home-inventory sync ha` | ✓ (lib) · Live-Smoke vertagt |
| S11 | CCU-Sync (XML-API), Firmware in `firmware_snapshot` mit Timestamp | ✓ (8 Tests, davon 4 FW-Diff) |
| S11b | Philips-Hue-Sync gegen mehrere Bridges (`sync hue --config`) | ✓ (4 Tests) |
| S11c | Shelly-Sync via mDNS-Discovery + Per-Device-Fetch (Gen1+Gen2) | ✓ (4 Tests, mDNS manuell) |
| S12a | YAML-Export pro Sync-Quelle, stable sortiert, deterministisch | ✓ (4 Tests) |
| S12b | `git_publish.rs`: `git add/commit/push` nur bei YAML-Diff | ✓ (5 Tests, incl. bare-repo push) |

**Gesamt nach Phase 3: 52 cargo-Tests gruen.**

## Phase 4 — VPS Deployment — ☐

| Step | Inhalt | Test-Gate |
|---|---|---|
| S13 | Compose-Base + VPN-Abstraktion + `justfile`-Dispatcher | `just up tailscale` -> `vpn`+`inventory`+`caddy` healthy; ohne Overlay startet inventory nicht |
| S13a | Tailscale-Overlay komplett (Auth-Key, ACL nur HA:8123, CCU:80) | `just ping-home tailscale <ha-ip>:8123` -> 200; State persistiert |
| S13b | NetBird-Overlay (SaaS + self-hosted Switch via `NB_MANAGEMENT_URL`) | wie S13a; plus: Switch zwischen SaaS und self-hosted nur per env, kein Code |
| S13c | WireGuard-Overlay mit sops-Init-Container fuer `wg0.conf` | wie S13a; `wg show wg0` zeigt Handshake; `wg0.conf` nie auf Host-Disk |
| S13d | `authgate`: eigenstaendiges Forward-Auth-Sidecar als SSO-Behelf (Login-Formular, HMAC-signiertes Session-Cookie); `Caddyfile` mit `forward_auth` | Code + Tests stehen; Build/Test auf dem Server noch offen |
| S14 | Caddy + Authentik Forward-Auth (loest `authgate` ab), Subdomain mit Let's-Encrypt | anonymer Request -> Login-Redirect; nach Login UI sichtbar; `X-Authentik-Username` korrekt; `curl` ohne Cookie -> 401 |

`authgate` (S13d) und Authentik (S14) teilen denselben Header-Vertrag
(`X-Authentik-Username`) — der Umstieg ist ein Einzeiler im `Caddyfile`.

## Phase 5 (optional, post-V1)

- S15 cron im Container fuer periodischen Sync
- S16 Z2M-Sync (MQTT)
- S17 Node-RED-Sync (`/flows` + Palette via `npm list`)
- S18 Firmware-Diff-Highlight im UI
- S19 CI: `cargo test` + Image-Build auf push
- S20 Migration der bestehenden Automationen je Domaene (Licht -> Heizung -> Anwesenheit -> …)
