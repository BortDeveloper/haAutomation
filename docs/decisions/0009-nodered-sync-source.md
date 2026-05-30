# ADR-0009: Node-RED als Sync-Source im Rust-Backend (via HA-Supervisor-Ingress)

- **Status:** accepted
- **Datum:** 2026-05-29
- **Bezug:** User-Auftrag „erstes Test-Setup" (2026-05-29), Iteration-1-Backlog,
  Audit-Derivat `docs/cockpit/ccu-jack-adoption-audit-derivat.md`
- **Ergänzt:** [ADR-0001](0001-rust-synchroner-minimal-stack.md) (Sync-Pattern),
  [ADR-0002](0002-sqlite-cache-yaml-source-of-truth.md) (Persistenz),
  [ADR-0008](0008-eu-data-act-mapping.md) (Datenzugang vernetzter Produkte)

## Kontext

Das Inventory-Backend deckt heute vier Sync-Sources ab: HA, CCU, Hue, Shelly
(`inventory/src/sync/{ha,ccu,hue,shelly}.rs`). **Node-RED** — bei diesem User
der zentrale Logik-Layer (HA-Add-on plus historisch eine zweite Node-RED-
Instanz auf der CCU, die als „Schmerzpunkt" markiert ist) — ist nicht erfasst.
Damit fehlt:

1. **Sichtbarkeit** auf die Flow-Inventarisierung (welche Flows existieren,
   welche Nodes, welche externen Bindings/Credentials).
2. **Versionierung** der Flows als Git-First-Artefakt analog zu `ccu.yaml` /
   `ha.yaml`.
3. **Diff-Basis** für die spätere KI-gestützte Flow-Analyse (separates
   Runbook, siehe `docs/runbooks/nodered-flow-analysis-with-claude.md`).

Node-RED bietet eine **Admin-API** (`GET /flows`, `POST /flows`) — historisch
ohne Auth, ab Node-RED 1.x mit `adminAuth`-Token oder Bearer. Im
HA-Add-on-Setup ist Node-RED standardmäßig hinter dem
**HA-Supervisor-Ingress** erreichbar (Sidebar-Klick in der HA-UI), während der
Add-on-eigene Port `1880` per Default **nicht** nach außen exponiert wird.

### Zwei mögliche Schnittstellen

| Variante | Pfad | Auth | Bewertung |
|---|---|---|---|
| **A) Direkt auf `:1880`** | `http://<ha-host>:1880/flows` | Node-RED-eigener `adminAuth`-Bearer | erfordert Port-Exposition + zweites Credential-Set; widerspricht der Default-Add-on-Härtung |
| **B) Über HA-Supervisor-Ingress** | `{HA_URL}/api/hassio_ingress/<session>/flows` (oder vergleichbarer Proxy-Pfad) | HA-Long-Lived-Access-Token (LLAT, Bearer) | konsistent mit `inventory sync ha`; nutzt bestehende Token-Pflege; keine zusätzliche Exposition |

## Entscheidung

**Variante B**: Das Inventory-Backend greift auf die Node-RED-Admin-API
**ausschließlich über den HA-Supervisor-Ingress** zu, authentifiziert mit dem
**HA-Long-Lived-Access-Token**, der bereits für `inventory sync ha` existiert.

Konkret:

- **Neues Modul** `inventory/src/sync/nodered.rs` analog zu `ha.rs`.
- **Subkommando** `inventory sync nodered` parallel zu `sync ha|ccu|hue|shelly`.
- **Endpoint-Konstruktion**: `{HA_URL}/<ingress-path>/flows` — die exakte
  Path-Form (Session-Token vs. statischer Reverse-Proxy-Route) wird bei der
  Implementierung (Task #3 im Cockpit-Backlog) gegen die laufende Test-HA
  verifiziert. Beide Varianten sind via HA-LLAT autorisiert; die Entscheidung
  hat keinen architektonischen Hebel und gehört nicht in dieses ADR.
- **Persistenz Iteration 1**: **YAML-Snapshot only** unter
  `inventory/yaml/nodered.yaml` als Source of Truth (gemäß
  [ADR-0002](0002-sqlite-cache-yaml-source-of-truth.md)). **Keine SQLite-
  Tabellen** in dieser Iteration. Begründung: das vorhandene `devices`-Schema
  (source, source_id, name, manufacturer, model, kind, room) modelliert
  Geräte, nicht Flow-Strukturen. Eine Quetschung von Nodes in `devices`
  würde Semantik verlieren; saubere `nodered_flow`/`nodered_node`-Tabellen
  erfordern eine Schema-Migration und Erweiterung von `db.rs`/`yaml_io.rs`
  auf nicht-`Device`-Typen — Scope für **Iteration 2**, sobald
  Cross-Source-Joins (z. B. „welche HA-Entities werden von welchem Flow
  referenziert") tatsächlich gefordert sind.
- **Persistenz Iteration 2** (späteres Folge-ADR): SQLite-Tabellen
  `nodered_flow` (tab + globale Configs) und `nodered_node` (Nodes pro
  Flow, mit FK auf Flow). Natural Key `(source='nodered', source_id=node.id)`.
- **Idempotenz**: Deterministische Serialisierung des Flow-Arrays vor dem
  YAML-Write (Sortierung nach `id`, Pretty-Print mit fester Indentation).
  Mehrfach-Sync ohne Flow-Änderung lässt `git status` clean — analog zu
  `write_devices_for_source` in `yaml_io.rs`.
- **Secrets-Pfad**: HA-LLAT wird **nicht** dupliziert. Es bleibt in
  `local/test-setup.env` als `HA_TOKEN` (Test) bzw. `inventory/secrets/ha.sops.yaml`
  (Produktion, abhängig von ADR-0004 — H.1-STOPP weiterhin aktiv).
- **Inline-Credential-Maskierung**: Flow-JSON enthält bei diesem User
  dokumentiert **Inline-Klartext-Credentials** in CCU-Connection-Nodes
  (CRITICAL-Befund 2026-05-22, Memory `project_ha_smarthome_mqtt_architecture`).
  Das Modul **maskiert** vor dem YAML-Schreiben rekursiv jeden Wert in
  Feldern, deren Schlüssel `(?i)password|secret|token|credential|api[_-]?key`
  matcht — ersatzweise `"***masked***"`. Die SQLite-Rohdaten bleiben unmaskiert
  (lokal, nicht im Repo); die YAML-Version ist die committed Source of Truth.

## Folgen

**Positiv**

- **Eine Token-Klasse** für HA + Node-RED (Bedienkomfort + Audit-Vereinfachung).
- **Keine zusätzliche Port-Exposition** — Node-RED bleibt hinter dem
  HA-Ingress, BSI IT-Grundschutz `APP.3.2.A1` (Sichere Konfiguration eines
  Webservers — Minimal-Exposition) bleibt erfüllt.
- **Konsistenz** mit dem bestehenden Sync-Pattern (`ha.rs` ist 200 Zeilen, das
  neue Modul wird in derselben Größenordnung liegen).
- **Diff-fähige Basis** für die KI-Flow-Analyse: das committed
  `yaml/nodered.yaml` ist die einzige Quelle, die Claude (oder ein anderer
  externer Analyse-Pfad) zu sehen bekommt — kein Direktzugriff auf laufende
  Flows.

**Negativ / Kosten**

- **Path-Verifikation bei Implementierung erforderlich**: Welche Path-Form
  HA für Ingress-Forwarding anbietet, ist add-on- und HA-Version-abhängig.
  Das Risiko ist klein (zwei realistische Varianten, beide via LLAT), kostet
  aber 1-2 h Manual-Probing gegen die Test-HA.
- **Kopplung an HA-Verfügbarkeit**: Wenn HA down ist, ist auch der Node-RED-
  Sync-Pfad zu. Bei Variante A wäre `:1880` unabhängig erreichbar gewesen.
  Bewertung: akzeptabel — Node-RED-Add-on hängt sowieso am HA-Supervisor-
  Lifecycle.
- **Maskierung ist Best-Effort**: Regex-basierte Schlüssel-Erkennung greift
  konventionelle Felder ab, **nicht** Custom-Node-Schemata mit ungewöhnlichen
  Namen. Folge-Task (separat): Allow-List für bekannte sichere Felder + Audit-
  Mode (`inventory sync nodered --audit` listet alle nicht-gematchten Felder
  zur Review).

## Abgrenzung — was dieses ADR nicht entscheidet

- **Hardware-Setup**: ist in `docs/test-umgebung-real-hardware.md` festgelegt;
  Node-RED-Add-on-Setup wird dort ergänzt (Backlog-Task #7).
- **KI-Analyse-Prozess**: separater Runbook unter `docs/runbooks/nodered-flow-analysis-with-claude.md` (Backlog-Task #5).
- **Flow-Update-Prozess** (POST `/flows` + Rollback): separater Runbook
  `docs/runbooks/nodered-flow-update.md` (Backlog-Task #6). Dieses ADR
  beschränkt sich bewusst auf den **Read-Pfad** (`GET /flows`). Schreib-Pfad
  bekommt ggf. ein eigenes ADR.
- **Migration der CCU-internen Node-RED-Instanz** (der „Schmerzpunkt") ist
  ein separater Architektur-Task im Rahmen der ccu-jack-Adoption — nicht
  Gegenstand dieses ADR.
