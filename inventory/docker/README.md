# inventory/docker — VPN-Sidecar-Setup

Die `inventory`-App selbst weiß **nichts** über VPN. Sie joint nur den
Netzwerk-Namespace eines Sidecar-Service mit fixem Namen `vpn`. Welcher
Provider dahintersteht, bestimmt das gewählte Overlay zur Compose-Laufzeit.

## Zwei Deployment-Modi

| Modus | Wer macht TLS? | Wer macht Auth? | Wer macht VPN? | Compose |
|---|---|---|---|---|
| **Standalone** (Default) | Caddy (im Compose) | `authgate`-Sidecar | VPN-Sidecar (Tailscale/NetBird/WG) | `docker-compose.yml` + Overlay |
| **VPS-Stack** | Traefik (host-weit) | Authentik (Forward-Auth) | host-weites VPN (z. B. Headscale) | `docker-compose.vps.yml` |

Beide Modi nutzen **dasselbe Image** und denselben Header-Vertrag
(`X-Authentik-Username`). Die App ist unverändert.

## Dateien

| Datei | Zweck |
|---|---|
| `Dockerfile` | Multi-stage Build, baut beide Binaries (`inventory` + `authgate`). HEALTHCHECK auf `/health`. |
| `docker-compose.yml` | **Standalone**-Base: `inventory` + `authgate` + `caddy`. Erwartet einen Service `vpn` aus dem Overlay. |
| `docker-compose.vps.yml` | **VPS-Variante**: nur `inventory`, an externes `traefik`-Netz; Hardening (read_only, cap_drop:ALL, Limits). |
| `docker-compose.vpn.tailscale.yml` | Overlay: `vpn` = Tailscale-Client |
| `docker-compose.vpn.netbird.yml` | Overlay: `vpn` = NetBird-Client (SaaS oder self-hosted) |
| `docker-compose.vpn.wireguard.yml` | Overlay: `vpn` = WireGuard, mit `vpn-init` der sops-entschlüsselt |
| `justfile` | Dispatcher: `just up <provider>`, `just down <provider>`, `just ping-home …` |
| `Caddyfile` | Reverse-Proxy + `forward_auth` gegen das `authgate`-Sidecar |

## Secrets-Flow

Alle Secrets liegen sops+age-verschlüsselt unter `inventory/secrets/`. Der
age-Key liegt **nur auf dem VPS-Host** unter `/etc/inventory/age.key`
(chmod 400, root:root) — niemals im Repo.

| Provider | Quelle | Wohin |
|---|---|---|
| Tailscale | `secrets/vpn.tailscale.env.enc` | justfile → `/run/inventory/vpn.env` (tmpfs) → `--env-file` |
| NetBird | `secrets/vpn.netbird.env.enc` | analog |
| WireGuard | `secrets/vpn.wireguard/wg0.conf.enc` | `vpn-init` schreibt nach Named Volume `wg-conf` |

## Auth-Sidecar `authgate`

Solange kein externes SSO (Authentik) bereitsteht, uebernimmt das Sidecar
`authgate` die Authentifizierung. Caddy ruft es per `forward_auth`; bei
Erfolg reicht es den Header `X-Authentik-Username` ans Backend durch.

**Einrichtung** (vor dem ersten `just up`):

```bash
cp ../secrets/authgate.env.example ../secrets/authgate.env

# Session-Secret erzeugen
docker compose run --rm authgate gensecret
# -> Ausgabe als AUTHGATE_SESSION_SECRET in authgate.env eintragen

# Benutzer anlegen (Passwort via Env, damit es nicht im Verlauf landet)
docker compose run --rm -e AUTHGATE_PW='geheim' authgate hashpw admin
# -> ausgegebene Zeile als AUTHGATE_USERS in authgate.env eintragen
```

`authgate.env` ist gitignored. Produktiv: mit sops verschluesseln
(`authgate.env.enc`) und vor `just up` entschluesseln — analog zum
vpn-Secrets-Muster.

Wechsel zu Authentik spaeter: im `Caddyfile` nur das `forward_auth`-Ziel
umbiegen — Inventory und Header-Vertrag bleiben unveraendert.

## Provider wechseln

```bash
just down tailscale
just up netbird
```

Volumes bleiben pro Provider getrennt — kein State-Konflikt.

## Test-Gate (S13 aus dem Plan)

```bash
# 1. Start
just up tailscale

# 2. Healthcheck — beide Container müssen 'healthy' werden
docker compose ps    # vpn + inventory + caddy

# 3. Aus dem App-Container heraus auf HA pingen
just ping-home tailscale 192.168.x.x:8123
# erwartet: http=200 time=<irgendwas>s

# 4. Stoppen
just down tailscale
```

Bei jedem der drei Provider muss Schritt 3 erfolgreich sein — dann ist die
VPN-Abstraktion bewiesen.

## VPS-Variante (`docker-compose.vps.yml`)

Fuer Hosts, auf denen bereits **Traefik + Authentik + ein host-weites VPN**
durch das [vps-stack](https://github.com/BortDeveloper/ansible-vps-stack)-Playbook
laufen. Caddy und `authgate` entfallen — deren Aufgaben uebernehmen Traefik
und Authentik.

**Voraussetzungen:**

- externes Docker-Netz `traefik` existiert
- Traefik kennt das Middleware `chain-vpn@file` (Authentik + IP-Allowlist)
- Certresolver `letsencrypt` ist konfiguriert

**`.env`:**

```
# Gepinnt per Digest — niemals nur ein Tag, sonst kein reproduzierbarer Deploy.
INVENTORY_IMAGE=ghcr.io/bortdeveloper/inventory@sha256:<digest>
INVENTORY_DOMAIN=inventory.example.com
```

**Start:**

```bash
docker compose -f docker-compose.vps.yml --env-file .env up -d
```

Hardening: `read_only: true`, `cap_drop: [ALL]`, `no-new-privileges`, tmpfs
fuer `/tmp`, Memory/CPU/PIDs-Limits. Vor jedem Tag-Push das Image lokal
scannen:

```bash
docker build -f docker/Dockerfile -t inventory:dev ..
../../scripts/trivy-scan.sh inventory:dev
```

> Die produktive Distribution (Image-Push nach GHCR-private, Pinning via
> Digest in `versions.yml` des vps-Repos) ist Sache der Brueckenpakete
> C (Ansible-Rolle) und D (Build-Workflow) — siehe `project_session_handoff_…`
> in der Memory.
