# VPS-Host Setup

Operative Dokumentation des VPS-VPS, auf dem das Inventory-Backend
laeuft. Enthaelt vollstaendige Bootstrap-Anleitung zur Neuaufsetzung.

## Server-Kontext

| Eigenschaft | Wert |
|---|---|
| Hostname (oeffentlich) | `vps.example.org` |
| Rolle im Repo | "ansible-vps-stack" (managed von einem separaten Ansible-Repo gleichen Namens) |
| Betriebssystem | Debian 12 (Kernel 6.1.x) |
| Docker-Engine | 29.x |
| VPN-Backbone zum Heimnetz | Tailscale (initial), Overlays fuer NetBird / WireGuard liegen vor |
| Hostkey Ed25519 | `SHA256:y2llW7O1YaO/xOfRJMhONCoqRo7mwOf2rOi+rWqm0uA` |

## Benutzer- und Rechte-Modell

Bewusste Trennung in zwei Linux-Accounts:

| Account | Zweck | Hat sudo? | Mitglied in `docker`? | Login mit |
|---|---|---|---|---|
| `root` | Bootstrap, System-Pakete, neue User, Routing-Konfig | n/a | ja | `~/.ssh/id_rsa` (Workstation) |
| `deploy` | Tagesbetrieb: `git pull`, `docker build`, `docker compose up/down` | nein | **ja** (994) | `~/.ssh/id_ed25519_vps` (Workstation) |

**Warum getrennt?** Mit `deploy` kann der gesamte Container-Lifecycle ohne
Root-Privilegien laufen. Systemaenderungen erfordern explizit den Wechsel
auf `root` und sind dadurch sichtbar. Die deploy-Keys haben keinen
Schreibzugriff auf `/etc` oder die Docker-Konfiguration.

## Schluessel-Inventar

Im Projekt entstehen mehrere SSH-Keys mit jeweils klarem Scope. Diese Tabelle
ist die Quelle der Wahrheit darueber, "welcher Key macht was":

| Privatkey | Wo | Scope | Erlaubt |
|---|---|---|---|
| `~/.ssh/id_rsa` (Workstation) | Win | Workstation-Identitaet | Login `root@vps`; Deploy-Key fuer `ansible-vps-stack`-Repo |
| `~/.ssh/id_ed25519_vps` (Workstation) | Win | VPS-Login-Identitaet | Login `deploy@vps` |
| `~/.ssh/id_ed25519_haAutomation` (Workstation) | Win | haAutomation-Git-Identitaet | **Schreiben** auf `BortDeveloper/haAutomation` |
| `~/.ssh/id_ed25519_github_haAutomation` (VPS) | Lin | VPS-Git-Identitaet | **Lesen** von `BortDeveloper/haAutomation` |

Jeder Key hat genau eine Aufgabe. Rotationen betreffen jeweils nur einen
Server / ein Repo.

## SSH-Setup: Workstation → VPS

### Lokal: Key + Config

```bash
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_vps -N "" -C "deploy@vps"
```

In `~/.ssh/config` anhaengen:

```
Host vps
    HostName vps.example.org
    User deploy
    IdentityFile ~/.ssh/id_ed25519_vps
    IdentitiesOnly yes
```

### Host-Key verifizieren

```bash
ssh-keyscan -t ed25519 vps.example.org
# Fingerprint via:
ssh-keygen -lf <(ssh-keyscan -t ed25519 vps.example.org 2>/dev/null)
```

Erwartet: `SHA256:y2llW7O1YaO/xOfRJMhONCoqRo7mwOf2rOi+rWqm0uA`.
Dann `>> ~/.ssh/known_hosts`.

### deploy-User auf VPS anlegen (als root)

```bash
useradd -m -s /bin/bash -G docker deploy
install -d -m700 -o deploy -g deploy /home/deploy/.ssh
echo "<inhalt von id_ed25519_vps.pub>" > /home/deploy/.ssh/authorized_keys
chmod 600 /home/deploy/.ssh/authorized_keys
chown deploy:deploy /home/deploy/.ssh/authorized_keys
```

Pruefen:

```bash
ssh vps 'whoami && groups && docker ps'
```

Erwartet: `deploy`, Groups enthaelt `docker`, `docker ps` listet bestehende
Container.

## SSH-Setup: VPS → GitHub

VPS pullt **read-only** aus `haAutomation`. Schreibender Sync-Pfad
(git_publish.rs in S12) bekommt spaeter einen separaten Write-Key oder
nutzt ein PAT.

### VPS-seitig: Key + ssh-Config + known_hosts

```bash
ssh vps bash <<'BASH'
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_github_haAutomation -N "" \
  -C "deploy@vps-haAutomation"
ssh-keyscan -t ed25519 github.com >> ~/.ssh/known_hosts

cat > ~/.ssh/config <<EOF
Host github.com
    IdentityFile ~/.ssh/id_ed25519_github_haAutomation
    IdentitiesOnly yes
EOF
chmod 600 ~/.ssh/config
BASH
```

### Pubkey als Deploy-Key registrieren

Von der Workstation aus (gh-CLI als `BortDeveloper` authentifiziert):

```bash
PUB=$(ssh vps 'cat ~/.ssh/id_ed25519_github_haAutomation.pub')
gh api repos/BortDeveloper/haAutomation/keys \
  -f title="vps-deploy" \
  -f key="$PUB" \
  -F read_only=true
```

### Pruefen

```bash
ssh vps 'ssh -T git@github.com'
```

Erwartet: `Hi BortDeveloper/haAutomation! You've successfully authenticated...`

## Repo klonen

```bash
ssh vps 'git clone git@github.com:BortDeveloper/haAutomation.git ~/haAutomation'
```

Aktualisieren spaeter:

```bash
ssh vps 'cd ~/haAutomation && git pull'
```

## Erster Image-Build und Test

```bash
ssh vps 'cd ~/haAutomation/inventory && \
  docker build -f docker/Dockerfile -t inventory:dev . | tail -5'
```

Pruefen:

```bash
ssh vps 'docker image inspect inventory:dev --format "{{.Size}}"' \
  | awk '{printf "%.2f MB\n", $1/1048576}'
```

Erwartet: < 30 MB (NFR-9). Aktuell ~10 MB.

End-to-End-Test:

```bash
ssh vps 'docker rm -f inv-test 2>/dev/null
  docker run -d --name inv-test -p 127.0.0.1:18080:8080 inventory:dev
  sleep 2
  curl -s http://127.0.0.1:18080/health
  echo
  curl -s http://127.0.0.1:18080/api/devices
  docker rm -f inv-test'
```

Erwartet: `ok` und `[]`.

## Operative Routinen

### Komplette Update-Iteration

```bash
ssh vps bash -c '
  cd ~/haAutomation &&
  git pull &&
  cd inventory &&
  docker build -f docker/Dockerfile -t inventory:dev . &&
  docker compose -f docker/docker-compose.yml \
                 -f docker/docker-compose.vpn.tailscale.yml \
                 up -d
'
```

(Compose-Aufruf wird in S13 ueber `just` vereinfacht.)

### Logs anschauen

```bash
ssh vps 'docker logs -f inventory'
```

### Image-Cleanup

```bash
ssh vps 'docker image prune -f'
```

### Build-Cache aufraeumen

```bash
ssh vps 'docker builder prune -f'
```

## Trust Boundaries

```
+--------- Workstation (Windows) ---------+
|                                          |
|  id_rsa  ----------+ -> root@vps      |
|  id_ed25519_vps + -> deploy@vps    |
|  id_ed25519_haAutomation -> github (write) |
|                                          |
+--------------------|---------------------+
                     | SSH (ed25519 hostkey)
                     v
+--------- VPS VPS (Debian) -----------+
|                                          |
|  root          (System, sudo)            |
|  deploy        (Container-Lifecycle)     |
|  id_ed25519_github_haAutomation            |
|                  -> github (read-only)   |
|                                          |
|  ~deploy/haAutomation/  <-- git pull       |
|  /etc/inventory/age.key   (geplant)      |
|                                          |
+-------- Docker --------------------------+
|                                          |
|  inventory:dev   (non-root, alpine)      |
|  vpn-tailscale   (sidecar, NET_ADMIN)    |
|  caddy           (Reverse-Proxy + TLS)   |
|                                          |
+------------------------------------------+
```

Niemals committen: `id_rsa`, `id_ed25519_vps`, `age.key`, alle privaten
Keys auf VPS.

## Geplante Erweiterungen

Diese Punkte sind heute **noch nicht** auf dem VPS-Host vorhanden und
kommen mit den entsprechenden Steps aus der Roadmap:

- `/etc/inventory/age.key` (root:deploy 0440) — fuer sops-Decryption (mit S8)
- Tailscale-Sidecar inkl. Auth-Key (mit S13a)
- Caddy + TLS-Cert + Authentik-Forward-Auth (mit S14)
- DNS-A-Record fuer Subdomain (mit S14)
- systemd-Service-Unit, die `docker compose up -d` beim Boot startet (post-V1)
- Backup-Skript fuer SQLite + Compose-State (post-V1)

## Disaster Recovery: Neuer VPS-Host

Vorausgesetzt der Provider liefert einen frischen Debian-VPS mit
funktionierender Netzwerkanbindung und du hast Console-/SSH-Zugang als
`root`. Bootstrap-Reihenfolge:

1. **System-Updates und Docker installieren**

   ```bash
   apt update && apt -y full-upgrade
   apt -y install ca-certificates curl gnupg
   install -m 0755 -d /etc/apt/keyrings
   curl -fsSL https://download.docker.com/linux/debian/gpg \
       | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
   echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/docker.gpg] \
         https://download.docker.com/linux/debian bookworm stable" \
       > /etc/apt/sources.list.d/docker.list
   apt update
   apt -y install docker-ce docker-ce-cli containerd.io docker-compose-plugin
   ```

2. **Workstation-Pubkey fuer root einspielen** (per Provider-Console oder via Password-Auth):

   ```bash
   mkdir -p /root/.ssh && chmod 700 /root/.ssh
   echo "<id_rsa.pub Inhalt>" >> /root/.ssh/authorized_keys
   chmod 600 /root/.ssh/authorized_keys
   ```

3. **deploy-User anlegen** (siehe "deploy-User auf VPS anlegen" oben)

4. **VPS-side Github-Key + ssh-config** (siehe "SSH-Setup: VPS → GitHub")

5. **Repo klonen, Image bauen, testen** (siehe oben)

6. **VPN, age.key, Caddy, Authentik** entsprechend Roadmap-Stand zum
   Zeitpunkt der Wiederherstellung wieder aufsetzen — alle Konfigurationen
   sind im Repo nachvollziehbar, lediglich Secrets muessen aus dem
   Off-Repo-Backup eingespielt werden (insbesondere `/etc/inventory/age.key`).

## Anhang: Schluessel-Rotation

### Workstation-VPS-Key rotieren

```bash
# alten Key sichern, neuen erzeugen
mv ~/.ssh/id_ed25519_vps{,.old}
mv ~/.ssh/id_ed25519_vps.pub{,.old}
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_vps -N "" -C "deploy@vps"

# neuen Pubkey einspielen (noch mit dem alten Key authentifiziert)
cat ~/.ssh/id_ed25519_vps.pub | ssh vps \
  'tee -a ~/.ssh/authorized_keys >/dev/null'

# pruefen, alten Eintrag entfernen
ssh vps 'sed -i "/$(cat ~/.ssh/id_ed25519_vps.old.pub | cut -d" " -f2 | head -c40)/d" \
  ~/.ssh/authorized_keys'

# alte Files loeschen
rm ~/.ssh/id_ed25519_vps.old*
```

### VPS-Github-Key rotieren

```bash
# neuen Key auf VPS
ssh vps 'ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_github_haAutomation.new \
  -N "" -C "deploy@vps-haAutomation"'

# neuen Pubkey als Deploy-Key registrieren
PUB=$(ssh vps 'cat ~/.ssh/id_ed25519_github_haAutomation.new.pub')
gh api repos/BortDeveloper/haAutomation/keys \
  -f title="vps-deploy-$(date +%Y%m%d)" -f key="$PUB" -F read_only=true

# umschalten
ssh vps '
  mv ~/.ssh/id_ed25519_github_haAutomation{,.old}
  mv ~/.ssh/id_ed25519_github_haAutomation.new ~/.ssh/id_ed25519_github_haAutomation
  mv ~/.ssh/id_ed25519_github_haAutomation.new.pub ~/.ssh/id_ed25519_github_haAutomation.pub
  ssh -T git@github.com  # smoke test
'

# alten Deploy-Key am Repo loeschen (gh api)
gh api repos/BortDeveloper/haAutomation/keys --jq '.[] | select(.title=="vps-deploy") | .id' \
  | xargs -I{} gh api -X DELETE repos/BortDeveloper/haAutomation/keys/{}
```
