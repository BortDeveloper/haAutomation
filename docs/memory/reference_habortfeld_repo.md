---
name: reference-habortfeld-repo
description: haBortfeld-Repo-Koordinaten und SSH-Setup auf der Windows-Workstation.
metadata: 
  node_type: memory
  type: reference
  originSessionId: f0b8800a-334e-4fb1-8ba0-6d7fd3505322
---

- **Repo:** https://github.com/BortDeveloper/haBortfeld (privat)
- **Lokaler Klon:** `C:\Users\guebr\haBortfeld`
- **Default-Branch:** `main`
- **Git-User in dem Klon:** `BortDeveloper <guebraun@gmail.com>` (lokal pro Repo gesetzt)

**SSH-Setup auf dieser Windows-Maschine:**

- Default-Identitaet `~/.ssh/id_rsa` (RSA-4096, Kommentar `guebr@Schosshund`) ist **Deploy-Key fuer `BortDeveloper/ansible-strato-stack`** — gibt also nur Zugriff auf dieses eine Repo.
- Fuer `haBortfeld` wurde ein **dedizierter Deploy-Key** `~/.ssh/id_ed25519_haBortfeld` (Ed25519, ohne Passphrase) generiert und am Repo mit Write-Recht registriert.
- `~/.ssh/config` enthaelt einen Host-Alias `github-haBortfeld`, der diesen Key nutzt.
- Clone/Push fuer haBortfeld geht ueber: `git@github-haBortfeld:BortDeveloper/haBortfeld.git`
- Beim Initial-Clone musste der GitHub Ed25519-Hostkey explizit nach `known_hosts` geschrieben werden — ssh-keyscan failte wegen KEX-Inkompatibilitaet von Windows OpenSSH 9.5 mit GitHub.
- Zusaetzlicher Workstation-Key `~/.ssh/id_ed25519_inventory` (Ed25519) fuer den dedizierten `inventory`-Tailscale-Server — siehe [[reference-inventory-server]].

**gh CLI:** authentifiziert als `BortDeveloper` via Keyring, Protocol = ssh.

**Strato-Host (ansible-strato-stack):**
- Hostname: `paperless.guebraun.org` (Debian 12.x kernel 6.1, Docker 29.4.1)
- Erreichbar per SSH-Alias `strato` (Windows ~/.ssh/config) als User **`deploy`** (UID 1001, in Gruppe `docker`)
- Lokaler Key dafuer: `~/.ssh/id_ed25519_strato` (Ed25519, ohne Passphrase, fingerprint `SHA256:ti0vvgxWXR/SGMY4BKTWtTgtPEAtxIL8wkQxO9FiGrU`)
- Root-Login geht weiterhin per default `id_rsa` (urspruenglicher Setup-User), `deploy` reicht fuer Docker-Builds und compose ohne sudo
- Repo-Klon auf Strato: `/home/deploy/haBortfeld` — gepullt ueber **read-only** Deploy-Key (`~/.ssh/id_ed25519_github_haBortfeld` auf Strato, am Repo als `strato-deploy` registriert mit `read_only=true`)
- Strato pulled also nur, pusht NICHT — schreibender Sync-Pfad (git_publish.rs in S12) braucht spaeter einen separaten Write-Key oder PAT
- Inventory-Image-Build: `cd ~/haBortfeld/inventory && docker build -f docker/Dockerfile -t inventory:dev .` → ~10 MB Image
- VPN (initial Tailscale) ans Heimnetz, hostet kuenftig Inventory-Stack via docker compose
- Age-Privatkey gehoert ausschliesslich auf diesen Host unter `/etc/inventory/age.key` (chmod 400) — noch nicht angelegt

Siehe [[project-habortfeld]] fuer Stack-Details.
