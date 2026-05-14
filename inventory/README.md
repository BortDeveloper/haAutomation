# inventory — Rust-Backend

Synchroner, framework-armer Rust-Code. Eine ausfuehrbare Datei mit drei
Subkommandos:

```
inventory serve     # HTTP-Server (S4)
inventory sync      # Quell-Sync (S9+)
inventory migrate   # DB-Migrationen (S2)
```

## Build & Run

```bash
cargo build --release
./target/release/inventory --help
```

## Crate-Struktur (geplant, kommt schrittweise mit der Roadmap)

```
src/
├── main.rs         # entry + clap dispatch (S1, aktuell)
├── db.rs           # rusqlite: open + migrate + queries (S2)
├── http.rs         # tiny_http-Routen (S4)
├── auth.rs         # X-Authentik-Username header (S7)
├── views.rs        # HTML-Rendering via format! (S5)
├── secrets.rs      # sops -d shell-out (S8)
├── types.rs        # Device, Firmware, Software
└── sync/
    ├── mod.rs
    ├── ha.rs       # Home Assistant (S9/S10)
    ├── ccu.rs      # Homematic XML-API (S11)
    ├── z2m.rs      # Zigbee2MQTT (S16)
    ├── nodered.rs  # Node-RED (S17)
    ├── yaml_io.rs  # YAML read/write
    └── git_publish.rs  # commit + push bei Diff (S12)
```

Aktuelle Stufe: **S1** — Skelett mit drei leeren Subkommandos.
