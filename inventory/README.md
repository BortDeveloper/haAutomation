# inventory — Rust-Backend

Synchroner, framework-armer Rust-Code. Eine ausfuehrbare Datei mit drei
Subkommandos:

```
inventory serve     # HTTP-Server (S4/S5/S7)
inventory sync ha   |  ccu  |  hue  |  shelly      # Sync-Quellen
inventory migrate   # DB-Migrationen
```

## Build & Run

```bash
cargo build --release
./target/release/inventory --help
```

Auf dem VPS-Host wird das Image gebaut, nicht das Binary direkt — siehe
[docker/README.md](docker/README.md) und [../docs/strato-setup.md](../docs/strato-setup.md).

## Globale Optionen

| Option | Env | Default | Zweck |
|---|---|---|---|
| `--db` | `INVENTORY_DB` | `inventory.db` | SQLite-Datei (wird angelegt) |
| `--yaml-dir` | `INVENTORY_YAML_DIR` | `yaml` | Verzeichnis fuer die per-source YAML-Snapshots |
| `--publish` | `INVENTORY_PUBLISH` | `false` | Nach Sync `git add/commit/push` ausfuehren |

## Sync-Quellen

| Quelle | CLI | Erforderlich |
|---|---|---|
| HA | `sync ha` | `--url`, `--token` (env `HA_URL`, `HA_TOKEN`) |
| CCU/RaspberryMatic | `sync ccu` | `--url` (env `CCU_URL`) |
| Philips Hue | `sync hue` | `--config` (YAML: `[{ip, token, name?}, ...]`) |
| Shelly | `sync shelly` | `--ip ip1,ip2` und/oder `--discover-seconds N` |

## Crate-Struktur (aktuell)

```
src/
├── main.rs           # entry + clap dispatch
├── auth.rs           # X-Authentik-Username header / AUTH_BYPASS
├── db.rs             # rusqlite: open + migrate + upsert + firmware-diff
├── git_publish.rs    # git add/commit/push shell-out, only on diff
├── http.rs           # tiny_http routes /, /api/devices, /health
├── secrets.rs        # sops -d shell-out, dotenv parsing
├── types.rs          # Device struct (kanonisches Schema)
├── views.rs          # HTML rendering (format!), XSS-escape
├── yaml_io.rs        # per-source deterministic write
└── sync/
    ├── mod.rs
    ├── ha.rs         # Home Assistant /api/states
    ├── ccu.rs        # CCU XML-API + virtuelle Empfaenger-Filter
    ├── hue.rs        # Hue v1 REST, Multi-Bridge
    └── shelly.rs     # Gen1+Gen2 HTTP, mDNS-Discovery

migrations/
└── 0001_init.sql     # devices, firmware_snapshot, software, manual_meta

fixtures/             # Test-Fixtures (nicht im Image)
├── devices.yaml
└── ccu_devicelist.xml
```

## Tests

```bash
cargo test --release
```

Aktueller Stand: **52 Tests gruen**. Sops/age muessen im PATH liegen
(Tests im `secrets`-Modul rufen sie via shell-out auf). Auf Windows liegen
die Pfade unter `~/AppData/Local/Microsoft/WinGet/Packages/...`.

Real-System-Smokes (echte HA-Instanz, echte CCU, echte Hue-Bridges, echte
Shellys) sind als gesammelter Pass am Ende von Phase 3 vorgesehen — vor
S13/S14 muessen die Daten einmal vollstaendig durch den Stack geflossen
sein.
