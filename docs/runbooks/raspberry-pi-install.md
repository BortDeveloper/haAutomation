# Runbook: HA-Automation-Suite auf einem Raspberry Pi installieren

**Zweck:** Den aktuellen Stand des Repos auf einem Raspberry Pi
**read-only auschecken**, das Inventory-Backend (Rust) dort bauen und
einen ersten **Home-Assistant-Sync** durchführen.

**Geltungsbereich:** Frisch aufgesetzter Raspberry Pi als Test-/Sync-Host.
Der Pi liest nur (Clone über HTTPS, kein Schreibrecht aufs Repo). Geschrieben
wird ausschließlich lokal in die Inventory-Datenhaltung (SQLite + YAML).

> ℹ️ **Architektur-Hinweis:** Diese Anleitung baut die Binary lokal auf dem
> Pi. Das ist bewusst — die Windows-Workstation baut den Crate nicht
> (`ring`/`libsqlite3-sys` brauchen einen funktionierenden C-Compiler), der
> Pi mit `build-essential` kann es. Der Crate kompiliert SQLite gebündelt
> (`rusqlite` Feature `bundled`) und nutzt rustls/`ring` statt OpenSSL —
> deshalb genügt `build-essential`, es braucht **kein** `libsqlite3-dev`
> oder `libssl-dev`.

## Voraussetzungen

- [ ] **Raspberry Pi mit 64-bit-OS.** Empfohlen: Raspberry Pi 4/5 (oder
      Pi 3 mit ≥ 1 GB RAM) und **Raspberry Pi OS 64-bit (aarch64)** bzw.
      Debian 12 arm64. 32-bit-OS wird nicht empfohlen — der `ring`-Build
      ist auf aarch64 deutlich problemloser.
- [ ] **Netzwerk-Erreichbarkeit zur HA-Instanz.** Der Pi muss die Home-
      Assistant-URL erreichen (gleiches LAN oder über VPN/Tailnet). Kurz
      testen: `curl -sS http://<ha-host>:8123/ -o /dev/null -w '%{http_code}\n'`.
- [ ] **HA Long-Lived Access Token (LLAT).** Wird in Schritt 7 erstellt.
- [ ] **SSH- oder Konsolen-Zugang** zum Pi mit einem Nicht-root-User
      (Standard `pi`), der `sudo` darf.

## Schritt 1 — Raspberry Pi OS aufsetzen

1. Mit dem **Raspberry Pi Imager** „Raspberry Pi OS (64-bit)" (Lite reicht,
   kein Desktop nötig) auf die SD-Karte / SSD schreiben.
2. Im Imager unter „Erweiterte Optionen" Hostname, SSH (Public-Key bevorzugt)
   und WLAN/Netzwerk vorkonfigurieren.
3. Pi booten und per SSH verbinden:

   ```sh
   ssh pi@<pi-host>
   ```

## Schritt 2 — System aktualisieren und Build-Pakete installieren

```sh
sudo apt update && sudo apt full-upgrade -y
sudo apt install -y build-essential pkg-config git curl ca-certificates
```

- `build-essential` — C/C++-Compiler für `ring` (TLS) und die gebündelte
  SQLite.
- `git`, `curl`, `ca-certificates` — Clone bzw. rustup-Bootstrap.

> Für den **vollständigen Testlauf** (`cargo test`, enthält Secrets-Tests)
> zusätzlich `sops` und `age` installieren — für den reinen HA-Sync **nicht**
> nötig. Siehe Schritt 9 (optional).

## Schritt 3 — Rust-Toolchain via rustup installieren

Die im Repo gepinnte Toolchain ist **1.95** (`home-inventory/rust-toolchain.toml`).
Das `apt`-`rustc` ist zu alt — daher rustup verwenden. rustup lädt die
korrekte Version beim ersten `cargo`-Aufruf im Repo automatisch nach.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
rustc --version    # Bootstrap-Version; im Repo wird automatisch 1.95 genutzt
```

## Schritt 4 — Repo read-only auschecken

Das Repo ist öffentlich → Clone über **HTTPS** ist read-only und braucht
keine Credentials/Keys:

```sh
cd ~
git clone https://github.com/BortDeveloper/haAutomation.git
cd haAutomation/home-inventory
```

> Alle folgenden Befehle laufen im Verzeichnis `~/haAutomation/home-inventory`.

## Schritt 5 — Inventory-Binary bauen

```sh
cargo build --release --locked --bin home-inventory
```

- `--locked` erzwingt exakt die `Cargo.lock`-Versionen (reproduzierbar).
- Erster Build dauert auf dem Pi einige Minuten (gebündelte SQLite + `ring`).
- **Bei wenig RAM (Pi 3 / 1 GB):** falls der Build mit OOM/„signal 9"
  abbricht, einmalig Swap erhöhen
  (`sudo dphys-swapfile swapoff && sudo sed -i 's/^CONF_SWAPSIZE=.*/CONF_SWAPSIZE=2048/' /etc/dphys-swapfile && sudo dphys-swapfile setup && sudo dphys-swapfile swapon`)
  und neu bauen.

Erfolgskontrolle:

```sh
./target/release/home-inventory --help
```

## Schritt 6 — Konfiguration anlegen

```sh
cp test-setup.env.example local/test-setup.env
chmod 600 local/test-setup.env
```

`local/` ist in `.gitignore` — die Datei wird nie committet. Anschließend
`local/test-setup.env` editieren und mindestens setzen:

```sh
HA_URL=<ha-host>:8123        # mit oder ohne http:// — der Runner normalisiert es
HA_TOKEN=<llat-aus-schritt-7>
```

> **Bekannte Stolperfalle (gelöst):** `HA_URL` **nicht** doppelt mit
> Schema versehen. `sync-ha.sh` ergänzt ein fehlendes `http://`, doppelt
> aber nie ein vorhandenes `http(s)://` und schneidet ein versehentliches
> `/api`-Suffix ab.

## Schritt 7 — HA Long-Lived Access Token erstellen

1. In der HA-Weboberfläche unten links auf den **Benutzernamen** klicken
   (Profil).
2. Tab **Sicherheit** → ganz unten **„Long-Lived Access Tokens"** →
   **„Token erstellen"**.
3. Namen vergeben (z.B. `inventory-pi`), Token **sofort kopieren** (wird nur
   einmal angezeigt) und als `HA_TOKEN` in `local/test-setup.env` eintragen.

> ⚠️ Ein LLAT läuft nicht ab und ist effektiv Vollzugriff auf die HA-Instanz.
> Nur in der `chmod 600`-Datei ablegen, nie als CLI-Argument übergeben.

## Schritt 8 — HA-Sync ausführen

```sh
./sync-ha.sh
```

Das Script lädt `local/test-setup.env`, normalisiert `HA_URL`, legt die
SQLite-DB an (`migrate`) und führt `home-inventory sync ha` aus. Die Binary holt
`/api/states`, filtert auf Geräte-Domains und schreibt das Ergebnis als
**SQLite-Upsert + YAML-Snapshot**.

Erwartete Ausgabe (Beispiel):

```
=== 2/2: Home Assistant Sync (http://<ha-host>:8123) ===
HA sync ok: <N> entities, <M> devices upserted, yaml: ./local/yaml/ha.yaml
  -> HA sync OK
HA-Sync gruen.
```

Exit-Codes: `0` OK · `1` Config-Fehler · `2` Binary fehlt · `3` Sync-Fehler.

## Schritt 9 — Ergebnis verifizieren

```sh
cat local/yaml/ha.yaml | head        # Geräte-Snapshot
ls -la local/inventory.db            # SQLite-Cache liegt vor
```

Optional die Web-UI lokal starten (read-only ansehen):

```sh
AUTH_BYPASS=1 ./target/release/home-inventory serve --addr 127.0.0.1:8080
# dann im Browser des Pi / per SSH-Tunnel: http://127.0.0.1:8080
```

> `AUTH_BYPASS=1` schaltet die Authentisierung der UI ab — nur für lokalen
> Test auf `127.0.0.1` verwenden, nie offen ins Netz.

## Optional — weitere Quellen und voller Testlauf

- **Weitere Sync-Quellen** (CCU, Shelly, Hue, Node-RED) in derselben
  `local/test-setup.env` konfigurieren und mit `./smoke-test.sh` gesammelt
  ausführen.
- **Vollständige Test-Suite:** zusätzlich `sudo apt install -y` von `sops`
  und `age` (für die Secrets-Tests), dann:

  ```sh
  cargo test --workspace --locked
  ```

## Aktualisieren (späterer Stand)

```sh
cd ~/haAutomation
git pull --ff-only
cd home-inventory && cargo build --release --locked --bin home-inventory
```

## Bezug

- `home-inventory/sync-ha.sh` — der fokussierte HA-Runner (Schritt 8)
- `home-inventory/test-setup.env.example` — Konfigurations-Vorlage
- `docs/getting-started.md` — First-Run-PoC (sicherheitsgeprüfter Durchlauf)
- `docs/test-umgebung-real-hardware.md` — Aufbau der realen Test-Umgebung
- `home-inventory/rust-toolchain.toml` — gepinnte Toolchain (1.95)
