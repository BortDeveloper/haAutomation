# Anforderungen

## Kontext

Drei Systeme tragen heute Smart-Home-Logik:

- **Home Assistant** auf eigenem Server, mit Node-RED Add-on
- **Homematic CCU** auf separatem Server
- Punkt-zu-Punkt-Integrationen dazwischen (Systemvariablen, MQTT, HA-Sensoren)

Logik ist ueber alle drei verteilt, teils redundant, kaum dokumentiert. Eine
Aenderung erfordert Reverse-Engineering durch alle Schichten. Ziel ist eine
einmalige saubere Restrukturierung mit klarer Verantwortlichkeit pro System
und einem maschinell gepflegten Inventar als Sicherheitsnetz.

## Ziele

1. **Sichtbarkeit** — vollstaendiges Inventar aller Geraete, Firmware-Staende
   und Software-/Addon-Versionen ueber alle drei Systeme, automatisch aktuell
2. **Ordnung** — fuer jede automatisierte Funktion ist eindeutig, in welchem
   System sie lebt (Ownership-Regeln, siehe [architecture.md](architecture.md))
3. **Wartbarkeit** — Konfiguration im Git, Aenderungen via PR, jederzeit
   reproduzierbar und auditierbar
4. **Sicherheit** — Credentials verschluesselt at rest, OIDC-faehiges Auth
   vor jedem Web-Endpoint

## Funktionale Anforderungen

| ID | Anforderung |
|---|---|
| FR-1 | Periodische Inventarisierung aller Geraete aus HA (`/api/states`), Homematic CCU (XML-API), Philips-Hue-Bridges (REST v1, multi-bridge), Shellys (HTTP Gen1+Gen2, mDNS-Discovery) und perspektivisch Zigbee2MQTT (`bridge/devices`) sowie Node-RED (`/flows` + `npm list`). |
| FR-2 | Single Source of Truth fuer das Inventar sind versionierte YAML-Dateien im Repo plus eine daraus generierte SQLite-DB. |
| FR-3 | Web-UI zum Durchsuchen, Filtern und Facettieren des Inventars, lesend. Schreibender Pfad: PR auf YAML. |
| FR-4 | Firmware-Aenderungen werden als zeitgestempelte Snapshots in der DB festgehalten; ein Diff fuehrt zu einem Auto-Commit + Push. |
| FR-5 | Manuelle Metadaten (Anschaffungsdatum, Standort, Garantie, Handbuch-URL) leben in `inventory/manual.yaml` und werden mit dem auto-erfassten Inventar verknuepft. |
| FR-6 | Ownership-Regeln (CCU vs. HA vs. Node-RED) sind dokumentiert und werden bei der Migration konsequent angewandt. |
| FR-7 | Refaktorierung bestehender Automationen pro Domaene (Licht, Heizung, Anwesenheit, …) jeweils als eigener PR mit Vorher-/Nachher-Mapping. |
| FR-8 | Backups: CCU-Programme und Node-RED-Flows werden naechtlich als Snapshot ins Repo committet. |

## Nicht-funktionale Anforderungen

| ID | Anforderung |
|---|---|
| NFR-1 | Ausschliesslich freie / Open-Source-Komponenten. |
| NFR-2 | Code lesbar fuer den Eigentuemer — keine komplexen Frameworks. Konkret: synchroner Rust-Code, kein tokio, kein Web-Framework jenseits von `tiny_http`, keine ORMs. |
| NFR-3 | Backend in **Rust**, Datenbank **SQLite**. |
| NFR-4 | VPN-Provider austauschbar zur Compose-Laufzeit: Tailscale, NetBird (SaaS oder self-hosted), WireGuard. Die App selbst kennt kein VPN. |
| NFR-5 | OIDC-Readiness ist Pflicht. Auth-Logik laeuft in einem vorgelagerten Authentik-Outpost via Caddy `forward_auth`; die App liest nur einen vertrauenswuerdigen Header. |
| NFR-6 | Secrets at rest verschluesselt mit `sops` + `age`. Der age-Privatkey liegt **nur** auf dem VPS (`/etc/inventory/age.key`, chmod 400, root:root) und niemals im Repo. |
| NFR-7 | GitOps: ausser dem age-Privatkey ist der gesamte System-Zustand im Repo nachvollziehbar. |
| NFR-8 | Inkrementelle Lieferung. Jeder Schritt aus der Roadmap ist eigenstaendig testbar und rollback-faehig. |
| NFR-9 | Image-Groesse fuer Inventory-Container < 30 MB. Cold-Start des Containers < 3 s. |
| NFR-10 | Subdomain unter `*.example.org` o.ae., TLS via Caddy / Let's Encrypt. |

## Explizit ausserhalb des Scope (V1)

- Schreibender Pfad aus dem Inventory-UI zurueck in HA/CCU (z.B. Geraet konfigurieren)
- Mobile-optimierte UI oder native App
- Eigene Regel-Engine im Inventory-Backend
- weitere Cloud-Provider neben dem aktuellen VPS
- Mehrbenutzer-Rollen jenseits "authenticated / not" (kommt erst, wenn OIDC reale Gruppen liefert)

## Festlegungen zur Umgebung

| Punkt | Wert | Konsequenz fuer die Implementierung |
|---|---|---|
| HA-Installation | Home Assistant OS (HAOS) | Supervisor verfuegbar → NR-Sync und Backups via Supervisor-API |
| CCU | RaspberryMatic | XML-API-Addon installierbar, Standard-Pfade fuer S11 |
| Node-RED | als HA-Addon | Flows-Pfad `/addon_configs/<slug>`, Admin-API via Supervisor-Ingress |
| Zigbee | Z2M als HA-Addon, Broker `core-mosquitto`, Prefix `zigbee2mqtt` | Sync (S16) liest direkt vom MQTT-Topic `zigbee2mqtt/bridge/devices` |
| Philips Hue | mehrere Bridges, REST-API v1 | Multi-Bridge-Sync mit YAML-Config (`sync hue --config`); liefert Firmware |
| Shellys | 20+ Geraete (Gen1+Gen2 gemischt) | mDNS-Discovery + Per-Device-HTTP-Fetch; Gen2-RPC bevorzugt, Gen1-Fallback |
| VPN initial | Tailscale | S13a = erste echte Deploy-Stufe; S13b/c bleiben Alternativen |
| Authentik | bestehende Instanz | S14 = nur neue Application + Outpost-Provider, kein Aufsetzen |
| Domain | bestehend, neue Subdomain | konkrete Subdomain wird in S14 festgelegt, A-Record auf die VPS-IP |

## Verbleibende Detailfragen (jeweils erst bei dem Step relevant)

- HA Long-Lived Access Token + Supervisor-Token erzeugen — fuer S10
- RaspberryMatic XML-API installieren falls nicht da — fuer S11
- Authentik-URL und Admin-Zugang — fuer S14
- Konkrete Subdomain — fuer S14
