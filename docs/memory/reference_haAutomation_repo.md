---
name: reference-hasite-repo
description: haAutomation-Repo-Koordinaten und SSH-Setup auf der Windows-Workstation.
metadata: 
  node_type: memory
  type: reference
  originSessionId: f0b8800a-334e-4fb1-8ba0-6d7fd3505322
---

- **Repo:** https://github.com/BortDeveloper/haAutomation (privat)
- **Lokaler Klon:** `C:\Users\guebr\haAutomation`
- **Default-Branch:** `main`
- **Git-User in dem Klon:** `BortDeveloper <<email@example.org>>` (lokal pro Repo gesetzt)

**SSH-Setup auf dieser Windows-Maschine:**

- Default-Identitaet `~/.ssh/id_rsa` (RSA-4096, Kommentar `guebr@Schosshund`) ist **Deploy-Key fuer `BortDeveloper/ansible-vps-stack`** — gibt also nur Zugriff auf dieses eine Repo.
- Fuer `haAutomation` wurde ein **dedizierter Deploy-Key** `~/.ssh/id_ed25519_haAutomation` (Ed25519, ohne Passphrase) generiert und am Repo mit Write-Recht registriert.
- `~/.ssh/config` enthaelt einen Host-Alias `github-haAutomation`, der diesen Key nutzt.
- Clone/Push fuer haAutomation geht ueber: `git@github-haAutomation:BortDeveloper/haAutomation.git`
- Beim Initial-Clone musste der GitHub Ed25519-Hostkey explizit nach `known_hosts` geschrieben werden — ssh-keyscan failte wegen KEX-Inkompatibilitaet von Windows OpenSSH 9.5 mit GitHub.
- Zusaetzlicher Workstation-Key `~/.ssh/id_ed25519_inventory` (Ed25519) fuer den dedizierten `inventory`-Tailscale-Server — siehe [[reference-inventory-server]].

**gh CLI:** authentifiziert als `BortDeveloper` via Keyring, Protocol = ssh.

**VPS-Host (ansible-vps-stack):**
- Hostname: `vps.example.org` (Debian 12.x kernel 6.1, Docker 29.4.1)
- Erreichbar per SSH-Alias `vps` (Windows ~/.ssh/config) als User **`deploy`** (UID 1001, in Gruppe `docker`)
- Lokaler Key dafuer: `~/.ssh/id_ed25519_vps` (Ed25519, ohne Passphrase, fingerprint `SHA256:ti0vvgxWXR/SGMY4BKTWtTgtPEAtxIL8wkQxO9FiGrU`)
- Root-Login geht weiterhin per default `id_rsa` (urspruenglicher Setup-User), `deploy` reicht fuer Docker-Builds und compose ohne sudo
- Repo-Klon auf VPS: `/home/deploy/haAutomation` — gepullt ueber **read-only** Deploy-Key (`~/.ssh/id_ed25519_github_haAutomation` auf VPS, am Repo als `vps-deploy` registriert mit `read_only=true`)
- VPS pulled also nur, pusht NICHT — schreibender Sync-Pfad (git_publish.rs in S12) braucht spaeter einen separaten Write-Key oder PAT
- Inventory-Image-Build: `cd ~/haAutomation/inventory && docker build -f docker/Dockerfile -t inventory:dev .` → ~10 MB Image
- VPN (initial Tailscale) ans Heimnetz, hostet kuenftig Inventory-Stack via docker compose
- Age-Privatkey gehoert ausschliesslich auf diesen Host unter `/etc/inventory/age.key` (chmod 400) — noch nicht angelegt

Siehe [[project-hasite]] fuer Stack-Details.
