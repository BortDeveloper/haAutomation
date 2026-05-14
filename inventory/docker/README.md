# inventory/docker — VPN-Sidecar-Setup

Die `inventory`-App selbst weiß **nichts** über VPN. Sie joint nur den
Netzwerk-Namespace eines Sidecar-Service mit fixem Namen `vpn`. Welcher
Provider dahintersteht, bestimmt das gewählte Overlay zur Compose-Laufzeit.

## Dateien

| Datei | Zweck |
|---|---|
| `docker-compose.yml` | Base: `inventory` + `caddy`. Erwartet einen Service `vpn` aus dem Overlay. |
| `docker-compose.vpn.tailscale.yml` | Overlay: `vpn` = Tailscale-Client |
| `docker-compose.vpn.netbird.yml` | Overlay: `vpn` = NetBird-Client (SaaS oder self-hosted) |
| `docker-compose.vpn.wireguard.yml` | Overlay: `vpn` = WireGuard, mit `vpn-init` der sops-entschlüsselt |
| `justfile` | Dispatcher: `just up <provider>`, `just down <provider>`, `just ping-home …` |
| `Caddyfile` | (kommt in S14) Reverse-Proxy + Authentik forward_auth |

## Secrets-Flow

Alle Secrets liegen sops+age-verschlüsselt unter `inventory/secrets/`. Der
age-Key liegt **nur auf dem VPS-Host** unter `/etc/inventory/age.key`
(chmod 400, root:root) — niemals im Repo.

| Provider | Quelle | Wohin |
|---|---|---|
| Tailscale | `secrets/vpn.tailscale.env.enc` | justfile → `/run/inventory/vpn.env` (tmpfs) → `--env-file` |
| NetBird | `secrets/vpn.netbird.env.enc` | analog |
| WireGuard | `secrets/vpn.wireguard/wg0.conf.enc` | `vpn-init` schreibt nach Named Volume `wg-conf` |

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
