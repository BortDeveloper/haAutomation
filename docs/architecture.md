# Architektur

## Komponenten und Datenfluss

```
+----------------------- Public VPS --------------------------+
|                                                              |
|   +-------+    +----------+    +------------+               |
|   | caddy |--->| inventory|--->|  SQLite    |               |
|   |       |    |  (Rust)  |    |  + YAML    |               |
|   +---^---+    +----^-----+    +------------+               |
|       |             |                |                       |
|       |             | (network_mode: service:vpn)            |
|       |             |                |                       |
|       |        +----v----+           |  git push             |
|       |        |   vpn   |        +--v---------+             |
|       |        | sidecar |        |  inventory |             |
|       |        +----+----+        |   repo     |             |
|       |             |             +------------+             |
| auth via forward_   |                                         |
| auth -> Authentik   |                                         |
+---------------------|---------------------------------------+
                      |
                  VPN tunnel
                      |
+---------------------v---------------------------------------+
|                  Heim-Netzwerk                              |
|                                                              |
|   Home Assistant     Homematic CCU       Node-RED            |
|   :8123 (REST)       /addons/xmlapi      :1880 / Supervisor  |
|                                                              |
|   Hue-Bridges        Shellys (20+)       Zigbee2MQTT         |
|   :80 v1 REST        :80 Gen1/Gen2 HTTP  MQTT (Phase 5)      |
|                      (per mDNS discovered)                   |
+--------------------------------------------------------------+
```

### Lesepfad

1. Cron im `inventory`-Container triggert Sync-Subkommandos pro Source
2. App ruft per HTTP/mDNS gegen HA, CCU, Hue-Bridges, Shellys (und kuenftig
   Z2M/NR) ab — alle Verbindungen laufen ueber den `vpn`-Netnamespace
3. Sync mapped auf das kanonische `Device` und upserted via Natural Key
   (source + source_id) idempotent in SQLite (`/var/lib/inventory/inventory.db`)
4. Bei vorhandener Firmware-Info: `firmware_snapshot`-Eintrag, nur wenn der
   Stand sich seit dem letzten Snapshot geaendert hat
5. Pro Source wird `inventory/yaml/<source>.yaml` deterministisch
   (source_id-sortiert) neu geschrieben
6. Bei YAML-Diff: `git add yaml/<source>.yaml`, `git commit -m "auto-sync <source>"`,
   `git push` (optional via `--publish`)
7. Web-UI liest aus SQLite

### Userpfad

1. Browser → `inventory.<domain>` → Caddy
2. Caddy `forward_auth` → Auth-Provider
3. Bei Erfolg: Caddy reicht Request mit Header `X-Authentik-Username` an Inventory durch
4. Inventory rendert HTML/JSON aus DB

**Auth-Provider:** Solange kein externes SSO bereitsteht, ist der Provider das
eigene Sidecar `authgate` (Binary im selben Crate, `src/bin/authgate.rs`) —
Login-Formular + zustandsloses, HMAC-signiertes Session-Cookie. Der spaetere
Wechsel zu Authentik (S14) aendert nur das `forward_auth`-Ziel im Caddyfile;
die Header-Schnittstelle (`X-Authentik-Username`) bleibt identisch, Inventory
selbst bleibt unveraendert.

## Ownership-Regeln

Die wichtigste Designentscheidung. Jede automatisierte Funktion lebt in
**genau einer** Schicht.

| Schicht | Verantwortlich fuer | Beispiel |
|---|---|---|
| Homematic CCU | Direktverknuepfungen, latenzkritisch, auch bei HA-Ausfall funktional | Taster -> Licht, Wind-Sensor -> Rollladen hoch |
| Home Assistant | Geraete-State, Entities, UI/Dashboard, Szenen, Helper, einfache 1:1-Automationen | Bewegung -> Licht an, Sonnenuntergang -> Szene |
| Node-RED | Verzweigungen, Timer, externe APIs, Notifications, Zustandsmaschinen | "Wenn niemand da, aber Tuer offen seit 5min, dann push" |

**Regeln:**
- Doppelte Logik (gleicher Trigger in zwei Schichten) ist immer ein Bug
- CCU-Programme sind eine Ausnahme, kein Default — nur wenn HA/NR-Ausfall nicht toleriert wird
- Node-RED ruft HA-Services auf, nicht umgekehrt
- HA-Automationen mit mehr als einer Bedingung **plus** mehr als einer Aktion gehoeren nach Node-RED

## Trust Boundaries

```
[ Internet ]
     |
     | HTTPS, Let's Encrypt
     v
[ Caddy ]   <-- public boundary; rate-limit, TLS termination
     |
     | forward_auth
     v
[ authgate Sidecar ]   (Behelf; spaeter: Authentik Outpost)
     |
     | nach erfolgreichem Login: setzt X-Authentik-Username header
     v
[ Inventory App ]   <-- trusts ONLY requests that arrived via Caddy
     |
     | filesystem + outbound HTTP via VPN
     v
[ SQLite ]   [ HA / CCU / NR ]
```

- Inventory akzeptiert **keine** direkten Verbindungen — nur via Caddy (in der Praxis: Caddy bindet 80/443 auf 0.0.0.0, der Rest ist im internen Docker-Netz)
- Inventory glaubt dem Header `X-Authentik-Username` nur, wenn die Connection vom Caddy-Container kommt — dies wird ueber Docker-Netzwerk-Trennung erzwungen, nicht im App-Code
- VPN-Sidecar hat keine eingehenden Ports veroeffentlicht

## Secrets-Architektur

```
inventory/secrets/                          (im Repo, verschluesselt)
├── .sops.yaml                              listet age-recipients (pubkeys)
├── common.env.enc                          HA_TOKEN, CCU_USER, CCU_PASS, …
├── vpn.tailscale.env.enc                   TS_AUTHKEY
├── vpn.netbird.env.enc                     NB_SETUP_KEY, NB_MANAGEMENT_URL
└── vpn.wireguard/wg0.conf.enc              komplette WG-Config

/etc/inventory/age.key                      (NUR auf dem VPS, chmod 400)
```

- Entschluesselung passiert beim Container-Start, das Klartext-Material landet auf tmpfs (`/run/inventory/…`), nie auf Disk
- Rotation: neuen Pubkey in `.sops.yaml`, `sops updatekeys` auf alle `.enc`, alten Key entfernen — alles sichtbar im PR
- Verlust des age-Privatkey = Unbrauchbarkeit aller Secrets, **kein** Backup auf demselben Host. Empfehlung: Pubkey-Backup in Passwortmanager.

## Build- und Deploy-Pfad

```
Entwickler-Laptop                          VPS-Host
+-------------------+                      +-------------------+
| cargo build       |                      |                   |
| cargo test        |                      |                   |
| docker build      |                      |                   |
| git push          | --(via SSH)-->       | git pull          |
+-------------------+                      | just up <provider>|
                                            +-------------------+
```

Kein CI in V1 (kommt spaeter). Deployment ist erstmal manuell via `git pull`
plus `just up`. Image wird auf dem VPS gebaut, nicht aus einer
Registry gezogen — vermeidet Registry-Komplexitaet.

## Migrations-Architektur fuer Daten

- SQLite-Schema in `inventory/migrations/NNN_name.sql`, eines pro Schema-Version
- App fuehrt fehlende Migrations beim Start aus, idempotent
- Manuelle Daten (`manual.yaml`) sind **nie** von Migrations betroffen — sie sind das Source-of-Truth, DB ist nur Cache
