---
name: reference-inventory-server
description: "Dedizierter Tailscale-Host `inventory` als Build- und Deploy-Ziel fuer das Inventory-Backend; die Windows-Workstation kann den Crate nicht bauen."
metadata:
  node_type: memory
  type: reference
---

Seit 2026-05-15 gibt es einen **eigenen Server `inventory`** im Tailscale-Tailnet,
bereitgestellt als Build- und Deploy-Ziel fuer das Inventory-Backend.

- **Tailscale-Name:** `inventory`, IP `<inventory-tailscale-ip>`. MagicDNS loest in dieser
  Windows-Umgebung nicht auf — Verbindung ueber die IP.
- **OS:** Debian 12 (bookworm), x86_64.
- **SSH:** `root@<inventory-tailscale-ip>` mit dediziertem Workstation-Key
  `~/.ssh/id_ed25519_inventory` (Ed25519, ohne Passphrase). Host-Alias
  `inventory` in `~/.ssh/config`.
- Server ist initial **blank** ausgeliefert: kein git / cargo / rustc / docker.
  Toolchain wird bei Bedarf via `apt` + `rustup` installiert (apt-`rustc` zu
  alt — Cargo.toml verlangt rust-version 1.85).

**Die Windows-Workstation baut den Crate NICHT.** Der `inventory`-Crate haengt
ueber `ureq`→`ring` und `rusqlite`→`libsqlite3-sys` an C-Code; der mingw-`gcc`
der Workstation ist defekt (`gcc -E` scheitert schon an Trivialdateien).
`cargo build`/`cargo test` schlagen lokal **immer** fehl — auch
`cargo build --bin authgate` zieht den ganzen Crate-Dependency-Graphen.
→ **Build und Tests gehoeren auf einen Linux-Host** (`inventory`-Server oder
<vps-provider>). Nur das Dependency-Resolve (das `Cargo.lock` schreibt) laeuft auf der
Workstation durch.

Siehe [[reference-habortfeld-repo]] fuer den separaten VPS und das
SSH-Schluessel-Inventar, [[project-habortfeld]] fuer den Stack.
