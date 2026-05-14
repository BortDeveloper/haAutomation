# Roadmap

14 Schritte, je ein PR mit reproduzierbarem Test-Gate. Vor "gruen" nicht
weitergehen.

## Phase 1 — Lokales Skelett (kein Netz, kein VPN)

| Step | Inhalt | Test-Gate |
|---|---|---|
| S1 | Cargo-Crate `inventory`, `clap`-Subkommandos `serve | sync | migrate` | `cargo build --release` gruen, `inventory --help` zeigt Subkommandos |
| S2 | `db.rs` mit Migrations, Tabellen `devices`, `firmware_snapshot`, `software`, `manual_meta` | Unit-Test prueft alle Tabellen in `sqlite_master` |
| S3 | YAML-Fixture -> `upsert_devices` -> Roundtrip-Read | Test: `SELECT COUNT(*) = 3`, Felder Bytes-gleich |
| S4 | `tiny_http`-Server, Route `/health` | `curl :8080/health` -> `200 ok` |
| S5 | `/api/devices` (JSON), `/` (HTML-Tabelle aus `format!`) | `curl … | jq 'length == 3'`, Browser-Check |

## Phase 2 — Containerisierung & Auth-Huelle

| Step | Inhalt | Test-Gate |
|---|---|---|
| S6 | Dockerfile multi-stage `rust:alpine` -> `alpine:3.20` | Image < 30 MB, `docker run` -> `/health` ok |
| S7 | Auth-Middleware: `X-Authentik-Username` Pflicht, `AUTH_BYPASS=1` fuer Dev | 3 Tests: ohne Header `401`, mit `200`, Bypass `200` |
| S8 | `secrets.rs` shell-outet auf `sops -d`, parst K=V | Fixture-Decrypt liefert `HA_TOKEN=foo`; falscher Key -> klarer Fehler |

## Phase 3 — Sync gegen Heim-Systeme

| Step | Inhalt | Test-Gate |
|---|---|---|
| S9 | `sync/ha.rs` + Mapping `HaEntity -> Device`, gegen Mock-Server im Test | Mock-JSON -> N erwartete Devices |
| S10 | HA-Sync gegen echtes HA (lokal im LAN) | Reale Devices in UI, idempotenter Re-Run |
| S11 | CCU-Sync (XML-API), Firmware in `firmware_snapshot` mit Timestamp | Re-Run ohne FW-Aenderung -> 0 neue Snapshots; FW geaendert -> 1 neuer |
| S11b | Philips-Hue-Sync gegen mehrere Bridges (`sync hue --config`) | Mock-Bridge -> N Lights/Sensoren mit Firmware; Diff-basierte FW-Snapshots |
| S11c | Shelly-Sync via mDNS-Discovery + Per-Device-Fetch (Gen1+Gen2) | Mock-Device -> Devices+Firmware; ohne Discovery laeuft `--ip` Liste |
| S12a | YAML-Export pro Sync-Quelle nach `inventory/yaml/<source>.yaml` | nach Sync schreibt Datei; stable sortiert; deterministisch |
| S12b | `git_publish.rs`: `git add/commit/push` nur bei YAML-Diff | Synth. Datenaenderung -> Commit + Push; kein Diff -> kein Commit |

## Phase 4 — Strato-Deployment

| Step | Inhalt | Test-Gate |
|---|---|---|
| S13 | Compose-Base + VPN-Abstraktion + `justfile`-Dispatcher | `just up tailscale` -> `vpn`+`inventory`+`caddy` healthy; ohne Overlay startet inventory nicht |
| S13a | Tailscale-Overlay komplett (Auth-Key, ACL nur HA:8123, CCU:80) | `just ping-home tailscale <ha-ip>:8123` -> 200; State persistiert |
| S13b | NetBird-Overlay (SaaS + self-hosted Switch via `NB_MANAGEMENT_URL`) | wie S13a; plus: Switch zwischen SaaS und self-hosted nur per env, kein Code |
| S13c | WireGuard-Overlay mit sops-Init-Container fuer `wg0.conf` | wie S13a; `wg show wg0` zeigt Handshake; `wg0.conf` nie auf Host-Disk |
| S14 | Caddy + Authentik Forward-Auth, Subdomain mit Let's-Encrypt | anonymer Request -> Login-Redirect; nach Login UI sichtbar; `X-Authentik-Username` korrekt; `curl` ohne Cookie -> 401 |

## Phase 5 (optional, post-V1)

- S15 cron im Container fuer periodischen Sync
- S16 Z2M-Sync (MQTT)
- S17 Node-RED-Sync (`/flows` + Palette via `npm list`)
- S18 Firmware-Diff-Highlight im UI
- S19 CI: `cargo test` + Image-Build auf push
- S20 Migration der bestehenden Automationen je Domaene (Licht -> Heizung -> Anwesenheit -> …)
