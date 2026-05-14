# Strato-Host Setup

Operative Dokumentation des Strato-VPS, auf dem das Inventory-Backend
laeuft. Enthaelt vollstaendige Bootstrap-Anleitung zur Neuaufsetzung.

## Server-Kontext

| Eigenschaft | Wert |
|---|---|
| Hostname (oeffentlich) | `paperless.guebraun.org` |
| Rolle im Repo | "ansible-strato-stack" (managed von einem separaten Ansible-Repo gleichen Namens) |
| Betriebssystem | Debian 12 (Kernel 6.1.x) |
| Docker-Engine | 29.x |
| VPN-Backbone zum Heimnetz | Tailscale (initial), Overlays fuer NetBird / WireGuard liegen vor |
| Hostkey Ed25519 | `SHA256:y2llW7O1YaO/xOfRJMhONCoqRo7mwOf2rOi+rWqm0uA` |

## Benutzer- und Rechte-Modell

Bewusste Trennung in zwei Linux-Accounts:

| Account | Zweck | Hat sudo? | Mitglied in `docker`? | Login mit |
|---|---|---|---|---|
| `root` | Bootstrap, System-Pakete, neue User, Routing-Konfig | n/a | ja | `~/.ssh/id_rsa` (Workstation) |
| `deploy` | Tagesbetrieb: `git pull`, `docker build`, `docker compose up/down` | nein | **ja** (994) | `~/.ssh/id_ed25519_strato` (Workstation) |

**Warum getrennt?** Mit `deploy` kann der gesamte Container-Lifecycle ohne
Root-Privilegien laufen. Systemaenderungen erfordern explizit den Wechsel
auf `root` und sind dadurch sichtbar. Die deploy-Keys haben keinen
Schreibzugriff auf `/etc` oder die Docker-Konfiguration.

## Schluessel-Inventar

Im Projekt entstehen mehrere SSH-Keys mit jeweils klarem Scope. Diese Tabelle
ist die Quelle der Wahrheit darueber, "welcher Key macht was":

| Privatkey | Wo | Scope | Erlaubt |
|---|---|---|---|
| `~/.ssh/id_rsa` (Workstation) | Win | Workstation-Identitaet | Login `root@strato`; Deploy-Key fuer `ansible-strato-stack`-Repo |
| `~/.ssh/id_ed25519_strato` (Workstation) | Win | Strato-Login-Identitaet | Login `deploy@strato` |
| `~/.ssh/id_ed25519_haBortfeld` (Workstation) | Win | haBortfeld-Git-Identitaet | **Schreiben** auf `BortDeveloper/haBortfeld` |
| `~/.ssh/id_ed25519_github_haBortfeld` (Strato) | Lin | Strato-Git-Identitaet | **Lesen** von `BortDeveloper/haBortfeld` |

Jeder Key hat genau eine Aufgabe. Rotationen betreffen jeweils nur einen
Server / ein Repo.

## SSH-Setup: Workstation → Strato

### Lokal: Key + Config

```bash
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_strato -N "" -C "deploy@strato"
```

In `~/.ssh/config` anhaengen:

```
Host strato
    HostName paperless.guebraun.org
    User deploy
    IdentityFile ~/.ssh/id_ed25519_strato
    IdentitiesOnly yes
```

### Host-Key verifizieren

```bash
ssh-keyscan -t ed25519 paperless.guebraun.org
# Fingerprint via:
ssh-keygen -lf <(ssh-keyscan -t ed25519 paperless.guebraun.org 2>/dev/null)
```

Erwartet: `SHA256:y2llW7O1YaO/xOfRJMhONCoqRo7mwOf2rOi+rWqm0uA`.
Dann `>> ~/.ssh/known_hosts`.

### deploy-User auf Strato anlegen (als root)

```bash
useradd -m -s /bin/bash -G docker deploy
install -d -m700 -o deploy -g deploy /home/deploy/.ssh
echo "<inhalt von id_ed25519_strato.pub>" > /home/deploy/.ssh/authorized_keys
chmod 600 /home/deploy/.ssh/authorized_keys
chown deploy:deploy /home/deploy/.ssh/authorized_keys
```

Pruefen:

```bash
ssh strato 'whoami && groups && docker ps'
```

Erwartet: `deploy`, Groups enthaelt `docker`, `docker ps` listet bestehende
Container.

## SSH-Setup: Strato → GitHub

Strato pullt **read-only** aus `haBortfeld`. Schreibender Sync-Pfad
(git_publish.rs in S12) bekommt spaeter einen separaten Write-Key oder
nutzt ein PAT.

### Strato-seitig: Key + ssh-Config + known_hosts

```bash
ssh strato bash <<'BASH'
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_github_haBortfeld -N "" \
  -C "deploy@strato-haBortfeld"
ssh-keyscan -t ed25519 github.com >> ~/.ssh/known_hosts

cat > ~/.ssh/config <<EOF
Host github.com
    IdentityFile ~/.ssh/id_ed25519_github_haBortfeld
    IdentitiesOnly yes
EOF
chmod 600 ~/.ssh/config
BASH
```

### Pubkey als Deploy-Key registrieren

Von der Workstation aus (gh-CLI als `BortDeveloper` authentifiziert):

```bash
PUB=$(ssh strato 'cat ~/.ssh/id_ed25519_github_haBortfeld.pub')
gh api repos/BortDeveloper/haBortfeld/keys \
  -f title="strato-deploy" \
  -f key="$PUB" \
  -F read_only=true
```

### Pruefen

```bash
ssh strato 'ssh -T git@github.com'
```

Erwartet: `Hi BortDeveloper/haBortfeld! You've successfully authenticated...`

## Repo klonen

```bash
ssh strato 'git clone git@github.com:BortDeveloper/haBortfeld.git ~/haBortfeld'
```

Aktualisieren spaeter:

```bash
ssh strato 'cd ~/haBortfeld && git pull'
```

## Erster Image-Build und Test

```bash
ssh strato 'cd ~/haBortfeld/inventory && \
  docker build -f docker/Dockerfile -t inventory:dev . | tail -5'
```

Pruefen:

```bash
ssh strato 'docker image inspect inventory:dev --format "{{.Size}}"' \
  | awk '{printf "%.2f MB\n", $1/1048576}'
```

Erwartet: < 30 MB (NFR-9). Aktuell ~10 MB.

End-to-End-Test:

```bash
ssh strato 'docker rm -f inv-test 2>/dev/null
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
ssh strato bash -c '
  cd ~/haBortfeld &&
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
ssh strato 'docker logs -f inventory'
```

### Image-Cleanup

```bash
ssh strato 'docker image prune -f'
```

### Build-Cache aufraeumen

```bash
ssh strato 'docker builder prune -f'
```

## Trust Boundaries

```
+--------- Workstation (Windows) ---------+
|                                          |
|  id_rsa  ----------+ -> root@strato      |
|  id_ed25519_strato + -> deploy@strato    |
|  id_ed25519_haBortfeld -> github (write) |
|                                          |
+--------------------|---------------------+
                     | SSH (ed25519 hostkey)
                     v
+--------- Strato VPS (Debian) -----------+
|                                          |
|  root          (System, sudo)            |
|  deploy        (Container-Lifecycle)     |
|  id_ed25519_github_haBortfeld            |
|                  -> github (read-only)   |
|                                          |
|  ~deploy/haBortfeld/  <-- git pull       |
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

Niemals committen: `id_rsa`, `id_ed25519_strato`, `age.key`, alle privaten
Keys auf Strato.

## Geplante Erweiterungen

Diese Punkte sind heute **noch nicht** auf dem Strato-Host vorhanden und
kommen mit den entsprechenden Steps aus der Roadmap:

- `/etc/inventory/age.key` (root:deploy 0440) — fuer sops-Decryption (mit S8)
- Tailscale-Sidecar inkl. Auth-Key (mit S13a)
- Caddy + TLS-Cert + Authentik-Forward-Auth (mit S14)
- DNS-A-Record fuer Subdomain (mit S14)
- systemd-Service-Unit, die `docker compose up -d` beim Boot startet (post-V1)
- Backup-Skript fuer SQLite + Compose-State (post-V1)

## Disaster Recovery: Neuer Strato-Host

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

3. **deploy-User anlegen** (siehe "deploy-User auf Strato anlegen" oben)

4. **Strato-side Github-Key + ssh-config** (siehe "SSH-Setup: Strato → GitHub")

5. **Repo klonen, Image bauen, testen** (siehe oben)

6. **VPN, age.key, Caddy, Authentik** entsprechend Roadmap-Stand zum
   Zeitpunkt der Wiederherstellung wieder aufsetzen — alle Konfigurationen
   sind im Repo nachvollziehbar, lediglich Secrets muessen aus dem
   Off-Repo-Backup eingespielt werden (insbesondere `/etc/inventory/age.key`).

## Anhang: Schluessel-Rotation

### Workstation-Strato-Key rotieren

```bash
# alten Key sichern, neuen erzeugen
mv ~/.ssh/id_ed25519_strato{,.old}
mv ~/.ssh/id_ed25519_strato.pub{,.old}
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_strato -N "" -C "deploy@strato"

# neuen Pubkey einspielen (noch mit dem alten Key authentifiziert)
cat ~/.ssh/id_ed25519_strato.pub | ssh strato \
  'tee -a ~/.ssh/authorized_keys >/dev/null'

# pruefen, alten Eintrag entfernen
ssh strato 'sed -i "/$(cat ~/.ssh/id_ed25519_strato.old.pub | cut -d" " -f2 | head -c40)/d" \
  ~/.ssh/authorized_keys'

# alte Files loeschen
rm ~/.ssh/id_ed25519_strato.old*
```

### Strato-Github-Key rotieren

```bash
# neuen Key auf Strato
ssh strato 'ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_github_haBortfeld.new \
  -N "" -C "deploy@strato-haBortfeld"'

# neuen Pubkey als Deploy-Key registrieren
PUB=$(ssh strato 'cat ~/.ssh/id_ed25519_github_haBortfeld.new.pub')
gh api repos/BortDeveloper/haBortfeld/keys \
  -f title="strato-deploy-$(date +%Y%m%d)" -f key="$PUB" -F read_only=true

# umschalten
ssh strato '
  mv ~/.ssh/id_ed25519_github_haBortfeld{,.old}
  mv ~/.ssh/id_ed25519_github_haBortfeld.new ~/.ssh/id_ed25519_github_haBortfeld
  mv ~/.ssh/id_ed25519_github_haBortfeld.new.pub ~/.ssh/id_ed25519_github_haBortfeld.pub
  ssh -T git@github.com  # smoke test
'

# alten Deploy-Key am Repo loeschen (gh api)
gh api repos/BortDeveloper/haBortfeld/keys --jq '.[] | select(.title=="strato-deploy") | .id' \
  | xargs -I{} gh api -X DELETE repos/BortDeveloper/haBortfeld/keys/{}
```
