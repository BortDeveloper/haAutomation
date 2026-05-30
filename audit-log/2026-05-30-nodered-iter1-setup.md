# Audit-Log 2026-05-30 — Node-RED Iter-1-Setup

**Anlass:** User-Auftrag „erstes Test-Setup" (2026-05-29). Auftrag deckte
sieben Test-Bereiche ab; Repo-Realitaet erfuellte vier davon (Setup,
Konnektivitaet, Devices auslesen, Katalogisieren), die Node-RED-Bereiche
(Flow-Auslesen, Versionierung, KI-Analyse, Update-Prozess) fehlten
vollstaendig. Diese Iteration schliesst die vier Luecken auf Software-Ebene.

**Scope-Abgrenzung:** Software-Vorarbeit; physische Hardware-Beschaffung
laeuft asynchron auf User-Seite (~1-2 Wo Lieferzeit). Iter-2-Themen
(SQLite-Tabellen, Token-Rotation-Skript, Allow-List/Audit-Mode) bewusst
nicht angerueht.

## Pfadentscheidungen (User-bestaetigt)

| Entscheidung | Wahl | Begruendung |
|---|---|---|
| Hardware-Modus | Voll-physisch nach `docs/test-umgebung-real-hardware.md` | Deckt RF-Funk-Pfade realistisch ab; ~480 EUR, 1-2 Wo Lieferzeit |
| Node-RED-Integrationspfad | Rust-Backend (`inventory sync nodered`) | Konsistent mit ADR-0001/0002, ein Token-Set fuer HA + Node-RED |
| API-Zugriff | HA-Supervisor-Ingress + HA-LLAT | Keine zusaetzliche `:1880`-Exposition, BSI APP.3.2.A1 erfuellt |
| Persistenz Iter-1 | YAML-only (keine SQLite-Tabellen) | `devices`-Schema passt semantisch nicht; Iter-2-Folge-ADR bei Cross-Source-Bedarf |
| Commit-Strategie | Ein einziger Commit fuer Iter-1 + authgate-Fix | User-Wunsch |

## Geliefert

### Neu (6 Dateien)

| Pfad | Zweck |
|---|---|
| `docs/decisions/0009-nodered-sync-source.md` | ADR Iter-1 + Folge-Pfad fuer Iter-2 |
| `inventory/src/sync/nodered.rs` | `fetch_flows` + `sanitize` (Defense-in-Depth) + `write_flows_yaml` (deterministisch) + 13 Tests |
| `inventory/fixtures/nodered_flows.json` | Fixture mit 4 absichtlichen Klartext-Credentials |
| `docs/runbooks/nodered-flow-analysis-with-claude.md` | Prompt-Template + Audit-Anker, Rechtsanker ADR-0007 |
| `docs/runbooks/nodered-flow-update.md` | Pre-Snapshot → Test → Diff → Deploy → Rollback, ISO 27001 A.8.32 |
| `audit-log/2026-05-30-nodered-iter1-setup.md` | dieser Eintrag |

### Geaendert (7 Dateien)

| Pfad | Aenderung |
|---|---|
| `docs/decisions/README.md` | Index-Eintrag ADR-0009 |
| `docs/test-umgebung-real-hardware.md` | §8.7 Node-RED-Setup + Checkliste #15/#16 |
| `inventory/src/sync/mod.rs` | `pub mod nodered` |
| `inventory/src/main.rs` | `SyncSource::Nodered` + Handler + 2 Parse-Tests |
| `inventory/smoke-test.sh` | Schritt 4/4 (nodered) |
| `inventory/test-setup.env.example` | `NODERED_INGRESS_PATH` |
| `inventory/Cargo.toml` | hmac 0.13→0.12, getrandom 0.4→0.2 (authgate-Fix) |

## Build- und Test-Status

```text
cargo test               -> 104/104 (25 authgate + 79 inventory)
cargo check --tests      -> green (nur 1 unused-import-Warning, vorbestehend)
cargo check --bin inventory -> green
```

Test-Decken-Erweiterung: +15 neue Tests (13 in `sync::nodered`, 2 in
`main::tests` fuer CLI-Parse). Coverage des Sanitizers: alle 4 Credential-
Patterns der Fixture werden maskiert, plus Edge-Case verschachteltes
`credentials: { user, password }`-Objekt (Defense-in-Depth bestaetigt).

## Aufgedeckte Befunde

### Befund 1 — authgate-Bug (Dependabot-induziert, parallel gefixt)

`src/bin/authgate.rs` war seit Dependabot-PRs `1f7597a` (hmac 0.13) und
`5600779` (getrandom 0.4) build-broken: hmac 0.13 erwartet `digest 0.11`,
`sha2 0.10` haengt aber an `digest 0.10` (Type-Mismatch auf
`EagerHash`/`CoreProxy`-Bounds). Plus: `getrandom::getrandom()` wurde in
0.4 zu `getrandom::fill()` umbenannt.

**Fix:** Downgrade beider Direkt-Deps in `Cargo.toml`. Pinning-Kommentar
ergaenzt; Dependabot wird die Bumps erneut anbieten, die sollten bis
sha2 0.11 stable geblockt werden.

**Risiko-Status:** RUSTSEC-Pruefung der 0.12er/0.2er-Tracks im naechsten
`cargo audit`-Lauf erforderlich. Fuer aktuelle Iteration: keine bekannten
Advisories auf den Downgrades.

### Befund 2 — Inline-Klartext-Credentials in CCU-Connection-Nodes

Bestaetigt den CRITICAL-Befund 2026-05-22 (Memory
`project_ha_smarthome_mqtt_architecture`). Sanitizer maskiert die
konventionellen Felder; Custom-Node-Schemata mit ungewoehnlichen
Schluesselnamen bleiben blind — Backlog Iter-2: Allow-List + `--audit`-Mode.

### Befund 3 — Ingress-Path-Form nicht final festgelegt

ADR-0009 dokumentiert die Architektur-Entscheidung (HA-LLAT + Supervisor-
Ingress), laesst aber die konkrete Path-Form (`api/hassio_ingress/<session>`
vs. add-on-spezifischer Reverse-Proxy-Pfad) offen — beide sind via
HA-LLAT autorisiert, kein architektonischer Hebel. Verifikation gegen die
laufende Test-HA, sobald die Hardware steht (Backlog Hardware-#8).

### Befund 4 — Session-Token-Rotation fuer Cron-Sync nicht geloest

Wenn der Ingress-Path die Session-Token-Variante ist, rotiert das Token
pro Login. Fuer einen Cron-getriebenen `inventory sync nodered` braucht es
ein Token-Refresh-Skript oder die `addressable: true`-Add-on-Option mit
internem Hostname `core-node-red:1880` (nicht VPN-erreichbar). Backlog
Iter-2; fuer Iter-1-Smoke (einmaliger Lauf) nicht blockierend.

## Verbindlichkeits-Anker dieser Iteration

- **ADR-0001** (synchroner Stack) — `nodered.rs` nutzt ureq blocking, kein async
- **ADR-0002** (SQLite-Cache, YAML-SoT) — YAML-only in Iter-1, SoT-Disziplin gewahrt
- **ADR-0007** (EU-KI-VO) — Claude-Analyse-Runbook explizit als GPAI-Tool-Nutzung verankert
- **ADR-0009** (neu) — Node-RED-Sync-Pfad
- **BSI APP.3.2.A1** — keine `:1880`-Exposition
- **ISO 27001:2022 A.8.32** — Flow-Update-Runbook (Change Management)
- **CRITICAL-Befund 2026-05-22** — Sanitizer adressiert Klartext-Credentials in CCU-Nodes

## Nicht-Ziele dieser Iteration (explizit ausgeklammert)

- SQLite-Tabellen fuer Flows/Nodes (→ Iter-2-Folge-ADR bei Cross-Source-Bedarf)
- Token-Rotation-Skript (→ Iter-2)
- Allow-List + `--audit`-Mode fuer Sanitizer (→ Iter-2)
- Schreib-Pfad `POST /flows` als Backend-Subkommando (Runbook beschreibt
  manuellen `curl`-Aufruf; falls Backend-Integration spaeter sinnvoll: eigenes ADR)
- Migration der CCU-internen Node-RED-Instanz (Schmerzpunkt-Pattern,
  separater Architektur-Task im Rahmen der ccu-jack-Adoption)

## Bezug

- Vorgaenger: `audit-log/2026-05-26-ha-automation-dev-ist-stand-adr-0009.md`
  (Cockpit-ADR-0009 Strato-Mode — disjunkt zu diesem Repo-ADR-0009)
- Cockpit-Disziplin: `42ae7d6 chore(test-setup): smoke-test + env-example +
  ADR-0009-Audit nachgezogen` (logischer Vorlauf)
- Memory: `project_ha_smarthome_mqtt_architecture.md`, `project_haautomation_home_topology.md`
