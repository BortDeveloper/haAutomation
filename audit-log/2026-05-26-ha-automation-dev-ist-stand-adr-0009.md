# ha-automation-dev — Ist-Stand-Audit ADR-0009 (Strato-Mode + IdP-Integration)

**Datum**: 2026-05-26
**Owner**: `ha-automation-dev`
**Auftrag**: Task 9.R-1 (Iter-2-Vorlauf) — read-only Verifikations-Audit
vor Implementierungs-Task 9.3 (`docker-compose.*.yml` umstellen, authgate
raus, Caddy raus, Traefik-Labels auf `chain-vpn-authentik`).
**Rechtsgrundlage**: ADR-0009 AKZEPTIERT 2026-05-26 (Variante B —
OIDC gegen Cockpit-Authentik); Verifizierungs-Hinweis R-3 aus
`shared/audit-log/2026-05-26-architect-adr-0008-0009-akzeptanz.md`.

## 0. Naming-Drift (Vor-Bemerkung, wichtig für Implementierungs-Task)

ADR-0009 und der Akzeptanz-Audit referenzieren durchgängig die Datei
**`docker-compose.strato.yml`**. Im Repo existiert diese Datei
**nicht** unter diesem Namen. Die produktive VPS-Variante heißt:

- `inventory/docker/docker-compose.vps.yml` (84 Zeilen, Stand 2026-05-22)

Belege im Repo:

- `inventory/docker/README.md` Zeile 23 listet `docker-compose.vps.yml`
  als "VPS-Variante" (Traefik host-weit, Authentik Forward-Auth,
  Host-VPN).
- `.claude/memory/project_haautomation_public_ready.md` Zeile 23–24
  protokolliert einen historischen Rename (Trace eines früheren
  `strato`-Namens, der nicht wirksam wurde — aktueller Name ist
  `.vps.yml`).
- Die Audit-Logs vom 2026-05-16 (`architect.md`, `sre.md`,
  `security.md`) zitieren noch durchgehend `docker-compose.strato.yml`
  — das ist Audit-Drift gegen den heutigen Repo-Stand.

**Empfehlung für 9.3**: Implementierungs-Task auf
`docker-compose.vps.yml` umformulieren, NICHT eine neue
`docker-compose.strato.yml` anlegen. Alternativ: Repo-Rename
`vps.yml` → `strato.yml` zur Sprach-Konvergenz mit ADR-0009 —
das wäre aber ein separater Task und nicht Inhalt von 9.3.

Dieser Audit verwendet im Folgenden die Wendung "VPS-Mode"
synonym zu "Strato-Mode (ADR-Sprache)" und meint
`docker-compose.vps.yml`.

---

## 1. `docker-compose.vps.yml` Ist-Stand (== "Strato-Mode")

**Pfad**: `inventory/docker/docker-compose.vps.yml`

**Services**: nur **`inventory`** ist definiert (Zeile 27–80). Es gibt
- **kein `authgate`** im VPS-Compose (Zeile-Suche bestätigt: einziger Service).
- **kein `caddy`** im VPS-Compose.
- **kein VPN-Sidecar** im VPS-Compose.

Das ist gegenüber dem Stand, den ADR-0009 als IST annimmt, bereits
**weiter fortgeschritten als erwartet**: ADR-0009 §1 und der
architect-Akzeptanz-Audit gehen davon aus, dass authgate noch im
Strato-Compose entfernt werden muss. Im Repo ist authgate dort
*bereits nicht vorhanden*.

**Traefik-Labels auf `inventory`** (Zeile 70–79):

```yaml
- "traefik.enable=true"
- "traefik.docker.network=traefik"
- "traefik.http.routers.inventory.rule=Host(`${INVENTORY_DOMAIN:?...}`)"
- "traefik.http.routers.inventory.entrypoints=websecure"
- "traefik.http.routers.inventory.tls=true"
- "traefik.http.routers.inventory.tls.certresolver=letsencrypt"
- "traefik.http.routers.inventory.middlewares=chain-vpn@file"
- "traefik.http.routers.inventory.service=inventory"
- "traefik.http.services.inventory.loadbalancer.server.port=8080"
```

**Mittelware-Zustand**: `chain-vpn@file` (Zeile 77) — Stand
**vor** ADR-0009-Akzeptanz. ADR-0009 §Decision Variante B verlangt
**`chain-vpn-authentik`** (oder neue Chain). Das ist eine **1-Zeilen-
Differenz**.

**Netz-Zustand**: externes Netz `traefik` (Zeile 81–83) wird benutzt —
identisch zur vps-stack-Konvention (`docs/ARCHITECTURE.md` §4).

**Hardening** (Zeile 52–65): `read_only: true`, `cap_drop: ALL`,
`no-new-privileges: true`, `mem_limit: 256m`, `pids_limit: 128` —
unverändert, kein Touch nötig.

**Environment**: `AUTH_HEADER: X-Authentik-Username` (Zeile 45) ist
bereits **korrekt für Authentik gesetzt**. Header-Vertrag ist nicht
authgate-spezifisch — er ist identisch (Liskov-Beleg, ADR-0004
Projekt).

---

## 2. `forward_auth`-Konfiguration

**Ergebnis**: `forward_auth` ist im VPS-Mode **nicht im Compose**
sichtbar. Es wird über die Traefik-Middleware-Chain `chain-vpn@file`
(File-Provider von vps-stack) eingebracht.

**Standalone-Mode** (`docker-compose.yml`): `forward_auth` ist im
`Caddyfile` (Zeile 27–30) definiert:

```caddyfile
forward_auth authgate:9000 {
    uri /auth/verify
    copy_headers X-Authentik-Username
}
```

→ Im Standalone-Mode ist das Ziel **authgate:9000**.
→ Im VPS-Mode ist das Ziel **die Authentik-Outpost-Middleware**, die
  in der vps-stack-Traefik-File-Provider-Konfig `chain-vpn` definiert
  ist (`ansible-vps-stack/docs/ARCHITECTURE.md` Zeile 149 listet
  `chain-vpn-authentik` als VPN+SSO-Chain).

**Header-Vertrag**: durchgängig `X-Authentik-Username`. App liest
ihn in `inventory/src/auth.rs` Zeile 11–12 (Default-Fallback). Kein
App-Code-Change nötig.

**Kritischer Punkt**: `chain-vpn@file` ≠ `chain-vpn-authentik@file`.
Es muss geprüft werden, welche der zwei Chains am Live-Traefik
existiert und was sie liefert:

- `chain-vpn` könnte rein die IP-Allowlist (100.64.0.0/10) sein,
  ohne Authentik-Forward-Auth.
- `chain-vpn-authentik` ist die kombinierte VPN+SSO-Chain (laut
  vps-stack-ARCHITECTURE.md Zeile 96 und 136).

→ Im aktuellen `inventory`-Routing verwendet das VPS-Compose
**`chain-vpn@file`** (also vermutlich VPN-IP-only, ohne SSO).
Wenn das stimmt, ist das **ein eigener Audit-Befund**: das aktuelle
Live-Routing für `inventory` läuft *möglicherweise* schon heute ohne
Authentik (nur VPN-Allowlist). Verifikation am Live-VPS nötig
(Block-Punkt — siehe §7).

---

## 3. S14 Authentik-Application-Anlage (vps-stack-Repo, read-only)

**Pfad**: `~/projects/ansible-vps-stack/scripts/setup-authentik-providers.sh`

**Ergebnis**: **Keine** Application `inventory`, `ha-automation`,
`inventory.<domain>` oder ähnlich vorhanden. Das `SERVICES`-Array
(Zeile 46–66) listet:

```
nextcloud, paperless, grafana, directus, forgejo, wikijs, netbird
nodered, ollama, whisper, transcribe, speak, vosk, obsidian, traefik,
codeserver, n8n
```

→ **S14 ist nicht umgesetzt**. Task 9.1 (vps-stack-dev,
Application-Anlage + Vault-Secret `vault_inventory_oidc_client_secret`)
ist *Voraussetzung* für 9.3 — ohne Authentik-Application auf der
vps-stack-Seite kann die Traefik-Middleware-Chain in 9.3 nicht
produktiv schalten (Outpost würde 500/no-provider liefern).

**Status `mfa_required_apps`** (`inventory/group_vars/all/vars.yml`
Zeile 333–339): `inventory` ist **nicht** auf der Liste. Task 9.2
(Eintrag hinzufügen) ist offen.

**Bewertung**: Diese beiden Beobachtungen liegen formal außerhalb
des Audit-Scopes (vps-stack-Repo, nicht ha-automation), sind aber für
die Reihenfolge in §7 entscheidend.

---

## 4. Header-Vertrag aktueller Stand

**App-Seite** (`inventory/src/auth.rs` Zeile 11–12):

```rust
let header_name = std::env::var("AUTH_HEADER")
    .unwrap_or_else(|_| "X-Authentik-Username".to_string());
```

→ App liest `X-Authentik-Username` (default), konfigurierbar via
`AUTH_HEADER`-env.

**VPS-Compose** (Zeile 45): `AUTH_HEADER: X-Authentik-Username`
→ explizit gesetzt.

**Standalone-Compose** (Zeile 28): `AUTH_HEADER: X-Authentik-Username`
→ identisch.

**authgate-Setter** (`inventory/src/bin/authgate.rs` Zeile 98):

```rust
let header = env_or("AUTHGATE_HEADER", "X-Authentik-Username");
```

→ authgate setzt denselben Header wie Authentik-Outpost erwarten
würde. **Drop-in-Tausch ist mechanisch trivial**.

**Caddy-Copy** (`inventory/docker/Caddyfile` Zeile 29):
`copy_headers X-Authentik-Username` → Caddy reicht den Header
durch.

**Audit-Bestätigung**: Header-Vertrag ist **konsistent über alle
Layer**. Kein Drift, kein Code-Change nötig — passt zu ADR-0009
§Decision Variante B (Liskov auf Deployment-Ebene).

---

## 5. Projekt-ADR-0004 (`authgate-Übergangs-SSO`) — Sunset-Status

**Pfad**: `docs/decisions/0004-authgate-uebergangs-sso.md`

**Status laut Datei** (Zeile 3): `accepted` (Datum 2026-05-17).

**Sunset-Vermerk** (Zeile 26–27 und Zeile 41–42):

> "`authgate` bleibt der explizit dokumentierte Übergang bis
> Roadmap-Schritt **S14**."
>
> "Temporär existiert ein zweites Identity-Subsystem parallel zum
> Zielzustand. Bewusst befristet bis S14; danach wird `authgate`
> außer Betrieb genommen."

**Sunset-Bedingung**: **Roadmap-S14** (`docs/roadmap.md` Zeile 51):

> "S14 | Caddy + Authentik Forward-Auth (löst `authgate` ab),
> Subdomain mit Let's-Encrypt | anonymer Request → Login-Redirect;
> nach Login UI sichtbar; `X-Authentik-Username` korrekt; `curl`
> ohne Cookie → 401"

Status S14 in `roadmap.md` Zeile 51: **offen** (kein ✓-Marker).

**ADR-0009-Folge**: Task 9.5 verlangt Status-Wechsel der Projekt-
ADR-0004 auf **ABGELÖST im VPS-Mode**, in Standalone bleibt es
AKZEPTIERT. Dieser Status-Wechsel ist *nicht* Teil von 9.R-1, aber
Folge-Task nach 9.3-Merge.

**Bewertung**: Sunset-Vermerk ist klar formuliert. ADR-0009-
Variante-B-Implementation **ist** die S14-Realisierung. Mit
ADR-0009-Akzeptanz und 9.3-Merge ist S14 mechanisch erfüllt.

---

## 6. Standalone- vs. VPS-Mode (Cross-Cuts)

**Datei-Struktur**:

```
inventory/docker/
├── docker-compose.yml                     ← Standalone-Mode (Caddy + authgate + vpn-sidecar)
├── docker-compose.vps.yml                 ← VPS-Mode (== "Strato-Mode" laut ADR-0009)
├── docker-compose.vpn.tailscale.yml       ← Overlay für Standalone
├── docker-compose.vpn.netbird.yml         ← Overlay für Standalone
├── docker-compose.vpn.wireguard.yml       ← Overlay für Standalone (mit sops-Init)
├── Caddyfile                              ← nur Standalone-relevant
└── README.md                              ← dokumentiert beide Modi
```

**Standalone-Mode** (`docker-compose.yml`):
- enthält `inventory` (Zeile 11–46), `authgate` (Zeile 52–92),
  `caddy` (Zeile 94–127)
- nutzt `network_mode: service:vpn` für inventory → braucht ein
  VPN-Overlay (justfile-dispatched)
- bleibt unverändert nach ADR-0009 (Variante B betrifft NUR VPS-Mode)

**VPS-Mode** (`docker-compose.vps.yml`):
- nur `inventory`, hängt am externen `traefik`-Netz
- Authentik-Anbindung passiert in **vps-stack-Traefik-File-Provider**,
  nicht hier im Compose

**ADR-0009-Scope-Bestätigung**: Variante B betrifft ausschließlich
`docker-compose.vps.yml`. `docker-compose.yml` bleibt authgate-basiert,
weil dort kein Cockpit-Authentik erreichbar ist (Standalone =
Smoke/Test laut ADR-0009 §3).

---

## 7. Delta-Tabelle Ist vs. Soll (ADR-0009 Variante B, nur VPS-Mode)

| # | Element | IST (heute, Repo-Stand) | SOLL (ADR-0009 Variante B) | Δ in Task 9.3 |
|---|---|---|---|---|
| 1 | authgate-Service in `docker-compose.vps.yml` | **nicht vorhanden** | nicht vorhanden | **keine Änderung** |
| 2 | caddy-Service in `docker-compose.vps.yml` | nicht vorhanden | nicht vorhanden | keine Änderung |
| 3 | Traefik-Middleware-Label | `chain-vpn@file` (Zeile 77) | `chain-vpn-authentik@file` (oder neue Chain) | **1-Zeilen-Change** |
| 4 | `AUTH_HEADER`-env | `X-Authentik-Username` (Zeile 45) | `X-Authentik-Username` | keine Änderung |
| 5 | App-Code Header-Lese-Logik | `auth.rs` liest `X-Authentik-Username` | identisch | keine Änderung |
| 6 | Authentik-Application `inventory.<domain>` (vps-stack) | **nicht vorhanden** | Application + Provider angelegt | **Task 9.1** (vps-stack-dev), Voraussetzung |
| 7 | `mfa_required_apps`-Eintrag `inventory` (vps-stack) | **nicht vorhanden** | enthalten | **Task 9.2** (vps-stack-dev), Voraussetzung |
| 8 | Projekt-ADR-0004 Status (VPS-Mode) | `accepted` | `ABGELÖST` (für VPS-Mode) | **Task 9.5** (Folge nach 9.3-Merge) |
| 9 | `haAutomation/docs/architecture.md` Userpfad | erwähnt `authgate` als aktiven Auth-Provider | Userpfad ohne authgate für VPS-Mode | **Task 9.4** |

**Kurz**: 9.3 selbst ist im VPS-Compose nur ein **1-Zeilen-Change**
(Middleware-Label), weil authgate und Caddy dort schon nicht
existieren. Die eigentliche Arbeit liegt in den Vorgelagerten
(9.1, 9.2) und den Doku-Folge-Tasks (9.4, 9.5).

---

## 8. Kopplungs-Hinweis (zu Task 8.1, vps-stack Bridge-Refactor)

**ADR-0008 Variante B** (vps-stack Network-Policy, AKZEPTIERT) führt
drei Bridges (`edge-net`, `app-net`, `data-net`) ein und definiert
per-Service-Membership. `ha-automation`/`inventory` ist explizit
als Mitglied von `app-net` benannt
(`shared/audit-log/2026-05-26-architect-adr-0008-0009-akzeptanz.md`
Zeile 76–80).

**Aktuell** verwendet `docker-compose.vps.yml` Zeile 33–34 das externe
Netz `traefik` (Singleton). Task 8.1 wird vermutlich:
- entweder das `traefik`-Netz in `edge-net` umbenennen, oder
- `inventory` zusätzlich an `app-net` hängen.

**Konsequenz**: Wenn 9.3 (Middleware-Label-Change) **vor** 8.1
(Bridge-Refactor) merged, muss `docker-compose.vps.yml` in 8.1
*noch einmal* angefasst werden (Netz-Block + service `networks`-Liste).
Wenn 8.1 **vor** 9.3 merged, ist 9.3 ein **isolierter 1-Zeilen-PR**
ohne Touch am Netz-Block.

**Empfehlung**: 8.1 vor 9.3 (deckungsgleich mit Architect-Empfehlung
im Akzeptanz-Audit Zeile 76–80). Wenn dies nicht möglich ist,
9.3 in zwei PRs splitten (PR-A: Label-Change; PR-B: Netz-Anpassung
nach 8.1).

---

## 9. Risiken

### R-1 HA-Service-Ausfall während 9.3-Umstellung (MEDIUM)

Wenn 9.3 live geschaltet wird, ohne dass 9.1 (Authentik-Application
`inventory`) auf der vps-stack-Seite produktiv ist, liefert die
Traefik-Middleware-Chain einen 500 / no-provider, und `inventory.
<domain>` ist nicht erreichbar bis Authentik-Application angelegt ist.

**Mitigation**: harte Reihenfolge-Disziplin: 9.1 + 9.2 (vps-stack-
Seite) müssen **deployed** sein, **bevor** 9.3 (ha-automation-Seite)
gemerged + deployed wird. Smoke-Test nach 9.1: `curl https://inventory.
<domain>` muss Authentik-Login-Redirect liefern (mit alter `chain-vpn`
würde es noch die App liefern bzw. 401 ohne VPN).

### R-2 Token-Session-Invalidation (LOW)

Aktuelle authgate-Sessions (HMAC-Cookies, TTL 8h laut `docker-compose.yml`
Zeile 71 `AUTHGATE_SESSION_TTL: "28800"`) sind **nicht** Authentik-
kompatibel. Bei Umstellung müssen alle Nutzer:innen einmal
neu via Authentik einloggen. Bei Solo-Maintainer-Realität trivial.

**Mitigation**: Service-Window in einer ruhigen Stunde ankündigen
(an sich selbst). Kein technischer Mitigationsbedarf.

### R-3 chain-vpn-authentik vs. chain-vpn Drift (MEDIUM)

Aktueller Live-Stand: `inventory` ist via `chain-vpn@file` geroutet
(Compose Zeile 77). Wenn `chain-vpn` heute auf dem Live-Traefik
*nur* die IP-Allowlist enthält (ohne Authentik), ist `inventory`
heute **VPN-only ohne SSO** — also offen für jeden mit VPN-Zugang.

**Verifikation am Live-VPS nötig** (Block-Punkt, siehe §10):
- ist `chain-vpn` auf dem Live-Traefik nur IP-Allowlist, oder
  inkludiert sie Authentik?
- wenn nur IP-Allowlist: VPS-Mode war bereits vor ADR-0009 NICHT
  ADR-0003-konform — das ist eine *bestehende* G3.2-Lücke, nicht
  ein Implementierungs-Risiko von 9.3.

### R-4 Naming-Drift (LOW)

ADR-0009 und alle Folge-Tasks (9.1–9.7) zitieren
`docker-compose.strato.yml`, das im Repo nicht existiert. Wenn
ha-automation-dev nicht die Datei-Realität (`docker-compose.vps.yml`)
kennt, kann 9.3 versehentlich eine neue Datei anlegen statt die
bestehende zu modifizieren.

**Mitigation**: Dieser Audit-Eintrag macht das Drift explizit
(§0). Empfehlung: Drift-Notiz in `shared/audit-log/`-Folge-Eintrag
oder Repo-Rename als separater Task.

---

## 10. Block-Punkte (offen für Live-Verifikation)

Folgende Punkte konnten **nicht aus dem Repo allein** beantwortet
werden und benötigen Live-VPS-Zugriff (außerhalb dieses read-only
Audits):

- **B-1**: Was liefert `chain-vpn@file` heute auf dem Live-Traefik?
  Reine IP-Allowlist oder bereits Authentik-Forward-Auth?
- **B-2**: Ist `inventory.<domain>` heute erreichbar und gibt es
  echte authgate-Nutzer im VPS-Mode, oder läuft VPS-Mode nur als
  "deployed aber nicht aktiv genutzt"?
- **B-3**: Sind `chain-vpn-authentik` und `chain-vpn` zwei separate
  Chains auf dem Live-Traefik (vps-stack-Seite) oder identisch?

Diese Block-Punkte sind **kein Hindernis für 9.R-1** (Audit ist
abgeschlossen) und auch **kein Hindernis für 9.3-Vorbereitung**
(Soll-Stand ist klar). Sie müssen vor dem 9.3-Deploy auf dem
Live-VPS verifiziert werden — Owner für B-1/B-3 ist eher
`vps-stack-dev`, Owner für B-2 ist User selbst.

---

## 11. Empfehlung an Orchestrator (Reihenfolge + Service-Window)

### Reihenfolge

1. **8.1** (vps-stack-dev, Bridge-Refactor) — zuerst, damit
   `docker-compose.vps.yml` in 9.3 nur einmal angefasst wird.
2. **9.1 + 9.2** (vps-stack-dev, Authentik-Application + MFA-Eintrag)
   — vor 9.3, sonst R-1 (HA-Service-Ausfall).
3. **9.3** (ha-automation-dev, Middleware-Label-Change) — als
   1-Zeilen-PR, sobald 1–2 deployed sind.
4. **9.4 + 9.5** (ha-automation-dev, Doku + Projekt-ADR-Status)
   — nach 9.3-Merge.
5. **9.6** (security, Re-Audit) — nach 9.3 produktiv.

### Service-Window

- 9.3-Deploy braucht **kein** Service-Window im klassischen Sinn,
  weil:
  - der Compose-Change selbst ist eine Label-Änderung; `docker
    compose up -d` reloads den Container in ~2–5 Sekunden.
  - authgate-Sessions sind im VPS-Mode irrelevant (authgate läuft
    dort nicht).
- Empfohlen: ruhige Stunde, weil ein einmaliger Re-Login der User
  durch Authentik anfällt (R-2). Bei Solo-Maintainer-Setup praktisch
  zu vernachlässigen.

### Verifikations-Smoke-Test nach 9.3-Deploy

- `curl -sI https://inventory.<domain>` (anonym, von außen
  außerhalb VPN) → erwartet 401 oder Authentik-Redirect.
- Browser → `https://inventory.<domain>` → Authentik-Login →
  nach Erfolg Inventory-UI sichtbar, Header `X-Authentik-Username`
  korrekt gesetzt.
- `curl --cookie ...` ohne gültige Authentik-Session → 401.

Identisch zum bestehenden S14-Gate-Wortlaut in `roadmap.md` Zeile 51.

---

## 12. Zusammenfassung (für Orchestrator-Report)

- **9.3 ist mechanisch klein**: 1-Zeilen-Middleware-Label-Change
  in `docker-compose.vps.yml`. authgate und Caddy sind im VPS-Mode
  bereits nicht vorhanden (kein "Entfernen" nötig).
- **Hauptarbeit liegt vps-stack-seitig**: Task 9.1 (Authentik-
  Application `inventory` anlegen) ist Voraussetzung und blockiert
  9.3-Deploy.
- **Naming-Drift dokumentiert**: ADR-0009 referenziert
  `docker-compose.strato.yml`, Repo hat `docker-compose.vps.yml`.
- **Header-Vertrag stabil**: `X-Authentik-Username` durch alle
  Layer konsistent (App, env, authgate, Caddy, ADR).
- **Kopplung 8.1↔9.3**: 8.1 sollte vor 9.3 mergen (sonst Doppel-
  Touch am Compose).
- **3 Block-Punkte** offen (Live-Traefik-Chain-Definition,
  produktive Nutzung VPS-Mode, chain-vpn vs. chain-vpn-authentik
  Trennung) — keine 9.R-1-Blocker, aber 9.3-Deploy-Voraussetzung.
- **Risk-Profil**: R-3 (MEDIUM, *bestehende* G3.2-Lücke falls
  `chain-vpn` heute keine Authentik enthält) verdient Beachtung
  unabhängig von 9.3.

## Quellen / Verweise (Audit-intern)

- `~/projects/haAutomation/inventory/docker/docker-compose.vps.yml`
  (Zeile 27–80, 81–87)
- `~/projects/haAutomation/inventory/docker/docker-compose.yml`
  (Zeile 11–127)
- `~/projects/haAutomation/inventory/docker/Caddyfile`
  (Zeile 27–32)
- `~/projects/haAutomation/inventory/docker/README.md`
- `~/projects/haAutomation/inventory/src/auth.rs`
  (Zeile 11–12)
- `~/projects/haAutomation/inventory/src/bin/authgate.rs`
  (Zeile 98)
- `~/projects/haAutomation/docs/architecture.md`
  (Zeile 55–66, 86–104)
- `~/projects/haAutomation/docs/decisions/0004-authgate-uebergangs-sso.md`
- `~/projects/haAutomation/docs/roadmap.md`
  (Zeile 51–54)
- `~/projects/ansible-vps-stack/scripts/setup-authentik-providers.sh`
  (Zeile 46–66, SERVICES-Array)
- `~/projects/ansible-vps-stack/inventory/group_vars/all/vars.yml`
  (Zeile 333–339, mfa_required_apps)
- `~/projects/ansible-vps-stack/docs/ARCHITECTURE.md`
  (Zeile 96, 136, 149)
- `~/projects/stack-master/shared/architecture-decisions/0009-ha-automation-idp-integration.md`
- `~/projects/stack-master/shared/audit-log/2026-05-26-architect-adr-0008-0009-akzeptanz.md`
  (Zeile 67–80, Task-Sequenz 9.R-1 ff.)
