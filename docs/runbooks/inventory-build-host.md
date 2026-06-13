# Runbook: Linux-Build-Host (`inventory`) aufsetzen

**Zweck:** Einen Linux-Host als **Build- und Test-Ziel** für den
`inventory`-Crate provisionieren. Er bildet die CI-Umgebung nach und
dient als Host für Arbeiten, die die Windows-Workstation nicht bauen
kann (Dependency-Migrationen, `cargo test`, `Cargo.lock`-Regen).

**Geltungsbereich:** Dedizierter Tailscale-Host `inventory`
(Debian 12 bookworm, x86_64). Die Schritte gelten analog für jeden
frischen Debian/Ubuntu-x86_64-Host.

> ℹ️ **Warum überhaupt ein eigener Host:** Der Crate hängt über `ring`
> (TLS) und die gebündelte SQLite (`rusqlite` Feature `bundled`) an
> C/ASM-Code. Der mingw-`gcc` der Windows-Workstation ist defekt →
> `cargo build`/`cargo test` schlagen dort **immer** fehl. Nur das
> Dependency-Resolve (`Cargo.lock` schreiben) und der reine Shell-Notices-
> Generator laufen auf Windows. Alles, was kompiliert, gehört auf diesen
> Linux-Host.

## Voraussetzungen

- [ ] Debian 12 (bookworm) x86_64, frisch installiert, mit `sudo`-fähigem
      User (oder root).
- [ ] Tailscale-Account fürs Tailnet (Backbone zum Heimnetz).
- [ ] Von der Workstation: SSH-Key `~/.ssh/id_ed25519_inventory` und
      Host-Alias `inventory` in `~/.ssh/config` (siehe Memory
      `reference_inventory_server.md`).

## Schritt 1 — Host ins Tailnet bringen

```sh
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up            # einmaliger Login-Link im Browser bestaetigen
tailscale ip -4              # IP notieren
```

> **MagicDNS löst auf der Windows-Workstation nicht auf** — von dort
> immer über die **IP** verbinden, nicht über den MagicDNS-Namen.

## Schritt 2 — SSH-Zugang von der Workstation

In `~/.ssh/config` der Workstation (einmalig):

```
Host inventory
    HostName <inventory-tailscale-ip>
    User root
    IdentityFile ~/.ssh/id_ed25519_inventory
    IdentitiesOnly yes
```

Dann:

```sh
ssh inventory        # bzw. ssh root@<inventory-tailscale-ip>
```

## Schritt 3 — System aktualisieren und Build-Pakete installieren

```sh
sudo apt update && sudo apt full-upgrade -y
sudo apt install -y build-essential pkg-config git curl ca-certificates
```

- `build-essential` — C/ASM-Compiler für `ring` (TLS) und die gebündelte
  SQLite. **Das ist die einzige hart benötigte Build-Abhängigkeit.**
- `libssl-dev`/`libsqlite3-dev` aus älteren Notizen sind für den
  aktuellen Feature-Satz **nicht** nötig (rustls statt OpenSSL,
  `rusqlite` `bundled`) — schaden aber nicht, falls schon installiert.

## Schritt 4 — Rust-Toolchain via rustup

Das `apt`-`rustc` ist zu alt (Cargo.toml verlangt `rust-version = 1.91`).
Die Toolchain ist via `inventory/rust-toolchain.toml` auf **1.91** gepinnt —
rustup lädt sie beim ersten `cargo`-Aufruf im Repo automatisch nach.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
```

## Schritt 5 — Repo read-only auschecken

Das Repo ist öffentlich → HTTPS-Clone ist read-only, ohne Credentials:

```sh
cd ~
git clone https://github.com/BortDeveloper/haAutomation.git
cd haAutomation/inventory
```

> Schreibender Sync-Pfad (`git_publish`) braucht später einen separaten
> Write-Key/PAT — für Build/Test nicht erforderlich.

## Schritt 6 — Build verifizieren

```sh
cargo build --release --locked --bin inventory
```

- `--locked` erzwingt exakt die `Cargo.lock`-Versionen (reproduzierbar,
  identisch zur CI).
- Erfolgskontrolle: `./target/release/inventory --help`.

## Schritt 7 — Volle Test-Suite (für Dependency-Migrationen)

Die Secrets-Tests brauchen `sops` + `age`. `age` ist in bookworm-apt;
`sops` nicht → `.deb` vom GitHub-Release.

```sh
sudo apt install -y age
curl -fsSLo /tmp/sops.deb \
  https://github.com/getsops/sops/releases/latest/download/sops_3.13.1_amd64.deb
sudo dpkg -i /tmp/sops.deb

cargo test --workspace --locked       # erwartet: alle Tests gruen
```

## Schritt 8 — Dependency-Migration + Notices-Regen (typischer Use-Case)

Wenn eine Dependency gebumpt wird, die Code-Änderungen erfordert
(z.B. `rusqlite`, RustCrypto-`digest`-Familie), läuft das auf diesem Host:

```sh
# 1. Lock aktualisieren
cargo update -p <crate> --precise <version>
# 2. Code an neue API anpassen, bis Build + Tests gruen sind
cargo build --locked && cargo test --workspace --locked
# 3. THIRD-PARTY-NOTICES.md regenerieren (Pflicht — CI-Drift-Check)
cd ..  # Repo-Root
bash inventory/scripts/generate-notices.sh > THIRD-PARTY-NOTICES.md
# 4. Cargo.lock + Notices + Code-Aenderung in einem Branch committen
```

> Der Notices-Generator ist reines Shell+awk und parst nur `Cargo.lock` —
> kein Build, kein Netz. Der CI-Job `THIRD-PARTY-NOTICES drift check`
> re-runt ihn und `diff`t gegen die committete Datei; jede Abweichung
> blockt den Merge.

## Aktualisieren / Wiederverwenden

```sh
cd ~/haAutomation && git fetch origin && git checkout <branch>
cd inventory && cargo build --release --locked
```

## Pin-Hinweis (nicht versehentlich lösen)

`hmac` (0.12) und `getrandom` (0.2) sind in `inventory/Cargo.toml`
**bewusst gepinnt**. Beim `cargo update` darauf achten, dass diese
nicht ungewollt mitgezogen werden — der Bump auf die RustCrypto-
`digest`-0.11-Familie (sha2 0.11 / hmac 0.13) ist eine **koordinierte,
bewusste** Migration, kein Beifang.

## Bezug

- Memory `reference_inventory_server.md` — Host-Koordinaten, SSH-Key
- `docs/runbooks/raspberry-pi-install.md` — analoge ARM-/Sync-Variante
- `inventory/rust-toolchain.toml` — gepinnte Toolchain (1.91)
- `inventory/scripts/generate-notices.sh` — Notices-Generator
- `.github/workflows/security.yml` — CI-Drift-Check (Job `license-notices`)
