# Runbook: Node-RED-Flows mit Claude analysieren

**Zweck:** Strukturierte Analyse eines Node-RED-Flow-Snapshots durch eine
externe KI (Anthropic Claude), z. B. zur Smell-Detection, zur Identifikation
von Refactor-Kandidaten oder zur Beurteilung von Architektur-Drifts gegenüber
[ADR-0009](../decisions/0009-nodered-sync-source.md).

**Geltungsbereich:** Flow-Snapshots, die durch `home-inventory sync nodered` nach
`home-inventory/yaml/nodered.yaml` geschrieben wurden. Erfasst **nicht** Live-Flows
am laufenden Node-RED-Add-on.

> ⚠️ **Kernregel:** Es wird **ausschliesslich** der von
> [ADR-0009](../decisions/0009-nodered-sync-source.md) maskierte
> `home-inventory/yaml/nodered.yaml` an Claude übergeben — **nie** die
> Roh-API-Antwort. Klartext-Credentials in Custom-Node-Schemata, deren
> Schlüsselnamen die Maskierungs-Heuristik nicht greift (Backlog: Allow-List),
> müssen vor dem Upload **manuell** redacted werden.

## Rechtsanker

- **EU-KI-VO**: Claude ist ein General-Purpose-AI-Tool gemäss
  [ADR-0007](../decisions/0007-eu-ki-vo-nicht-anwendbar.md). Die Verwendung
  zur Analyse eigener Repo-Artefakte ist **kein** High-Risk-AI-System (Annex
  III), kein Inverkehrbringen, kein Substantial Modification. Keine
  zusätzliche Pflicht durch Art. 6 ff.
- **DSGVO**: Personenbezogene Daten sind in Flow-JSONs nicht zu erwarten;
  Friendly-Names von Geräten/Räumen können als pseudonym gelten. Falls
  Klartext-Personennamen im Flow auftauchen (z. B. in `tab.info` als
  Maintainer-Hinweis), vor Upload redacten.
- **Vertraulichkeit Anthropic-Verarbeitung**: Eingaben über Claude.ai werden
  laut Anthropic-Privacy-Policy nicht standardmässig für Modell-Training
  verwendet, *können* aber in Cache-/Trust-Safety-Pipelines erscheinen. Für
  diesen Solo-Betrieb akzeptiert; bei Mehr-Personen-Setup separate Bewertung.

## Voraussetzungen

- [ ] `inventory`-Binary gebaut: `cargo build --release --bin home-inventory`
- [ ] `local/test-setup.env` vorhanden, `HA_URL`/`HA_TOKEN`/`NODERED_INGRESS_PATH` gesetzt
      (siehe `home-inventory/test-setup.env.example`).
- [ ] Test-HA + Node-RED-Add-on erreichbar (Smoke-Test `home-inventory/smoke-test.sh` läuft grün).
- [ ] Repo ist clean (`git status` leer) — ein Auto-Sync soll nur den
      Flow-Diff zeigen, keine Reste vorheriger Arbeit.
- [ ] Zugriff auf Claude (Web `claude.ai` oder Claude Code CLI).

## Schritt 1 — Aktuellen Flow-Snapshot ziehen

```sh
cd home-inventory
./smoke-test.sh
```

Erwartetes Ergebnis: Schritt 4/4 (`Node-RED Sync`) endet mit
`Node-RED sync ok: N flows/nodes, sanitized, yaml: ./local/yaml/nodered.yaml`.

> 💡 Für die Analyse die committed `home-inventory/yaml/nodered.yaml`-Version
> nutzen, nicht `local/yaml/nodered.yaml` (Sandbox-Variante). Dafür ohne
> `local/`-Override:
> ```sh
> INVENTORY_DB=./inventory.db INVENTORY_YAML_DIR=./yaml \
>   ./target/release/home-inventory sync nodered
> ```
> und prüfen, dass `git diff yaml/nodered.yaml` nur erwartete Änderungen zeigt.

## Schritt 2 — Manuelle Redact-Vorprüfung

Heuristik-basierte Maskierung der `sync nodered`-Pipeline greift Schlüssel mit
`(?i)password|secret|token|credential|api[_-]?key`. **Nicht** abgedeckt:

- Custom-Node-Schemata mit Klartext-Auth in `payload`/`url`/`headers`
  (z. B. ein HTTP-Request-Node mit Bearer-Token in `headers.Authorization`).
- Klartext-Personenangaben in `tab.info` / `comment`-Nodes.

```sh
# Visueller Quick-Check vor dem Upload
grep -Ei 'bearer |basic |://[^/]*:[^/]*@|api[._-]?key|secret|password|token' \
  yaml/nodered.yaml
```

Jeden Treffer entweder bestätigen (`***masked***` ist drin → ok) oder vor dem
Upload manuell redacten.

## Schritt 3 — Claude-Session vorbereiten

**Prompt-Template** (in Claude einfügen, dann YAML anhängen):

```text
Du analysierst einen Node-RED-Flow-Snapshot aus dem haAutomation-Repo.
Kontext: Solo-Smart-Home, HA + Node-RED + OpenCCU + MQTT. Die Datei ist
durch `home-inventory sync nodered` aus der HA-Supervisor-Ingress-API gezogen
und gemaess ADR-0009 sanitisiert (Credentials sind ***masked***).

Bitte liefere:

1. **Struktur-Map**: Tabs, Top-Level-Flows, Subflows. Pro Flow: Anzahl
   Nodes, dominante Node-Typen, vermutete Aufgabe (1 Satz).
2. **Smell-Detection** mit Severity (high/med/low). Beispiele:
   - Inline-Credentials, die der Sanitizer uebersehen hat
   - Dead Code (nicht verbundene Nodes, deaktivierte Subflows)
   - Globale Variablen ohne klare Lifecycle
   - Doppelte Verantwortlichkeiten (z. B. dieselbe HA-Entity in 2 Flows)
   - Fehlende Error-Handler an externen Calls
3. **Refactor-Vorschlaege**: pro Smell ein konkreter Vorschlag (kein
   Code, nur Intent + Aufwand-Klasse XS/S/M/L).
4. **Architektur-Drift-Check** gegen ADR-0009: gibt es Flows, die
   direkt mit der CCU sprechen statt ueber HA (Schmerzpunkt-Pattern,
   vgl. project_ha_smarthome_mqtt_architecture.md)?

Antworte in Markdown, jede Sektion eigene Ueberschrift. Halte dich an
Beobachtungen aus der YAML — keine Spekulation ueber Code, den du nicht
siehst.
```

YAML als File-Attachment oder Code-Block anhängen. Bei Claude Code: einfach
`/include yaml/nodered.yaml` (oder Datei in den Kontext lesen lassen).

## Schritt 4 — Diskussions-Loop

1. Claudes Erstantwort durchlesen. Auf jeden HIGH-Smell **prüfen**:
   - Tatsächlich vorhanden? (Manchmal halluziniert das Modell Nodes,
     die nicht im Snapshot stehen.)
   - Severity gerechtfertigt? (Tab als „dead code" einzustufen kann
     bedeuten, dass nur die UI-Verbindung fehlt, der Flow aber per
     `link in`/`link out` aktiv ist.)
2. Klärungsfragen stellen, bis das Findings-Set stabil ist. Vorgehen:
   pro Finding eine Antwort „bestätigt" / „zurueckgewiesen mit Grund" /
   „weiter klären".
3. Iteration beenden, wenn Claude keine neuen HIGH-Findings mehr
   produziert UND die offenen MEDIUMs eingeordnet sind.

## Schritt 5 — Entscheidungsprotokoll ablegen

Pro Analyse-Session **einen** Audit-Log-Eintrag schreiben:

```text
audit-log/<YYYY-MM-DD>-nodered-flow-review.md
```

Inhalt (knapp, ein Eintrag pro Finding):

```markdown
# Node-RED Flow-Review YYYY-MM-DD

**Snapshot:** yaml/nodered.yaml @ <git-rev>
**Tool:** Claude <model-name>, Session-Dauer: ca. NN Minuten

## Findings (akzeptiert)

- **HIGH** — Flow „<name>": <kurze Beschreibung>
  - Vorschlag: <Refactor-Intent>, Aufwand: M
  - Folge-Task: <Backlog-Ref oder „nicht eingeplant">

## Findings (zurueckgewiesen)

- **MED (Claude)** — <beschreibung> → zurueckgewiesen, weil <Grund>

## Folge-Aktionen

- [ ] <konkreter Task im Iter-2-Backlog>
- [ ] <Refactor-PR <#nr>>
```

Der Eintrag ist Source-of-Truth für die Folge-Iteration. Claude-Chats sind
flüchtig — das Audit-Log ist der dauerhafte Anker.

## Wenn der Snapshot zu gross für eine Claude-Session ist

Node-RED-Flows können >100 KB werden. Wenn das Modell zu viele Nodes auf
einmal sieht, sinkt die Antwortqualität. Vorgehen:

1. **Pro Tab analysieren**: YAML aufteilen per `z`-Filter (jeder Tab hat
   eigene `z`-ID), pro Tab eine Session.
2. **Config-Nodes separat**: globale Configs (kein `z`, oben in der Datei)
   in einer eigenen Session prüfen — die sind oft der Sanitizer-Edge-Case.
3. **Subflows isoliert**: Subflow-Definitionen kommen mit eigenen Tabs;
   diese wie eigene Flows behandeln.

## Bezug

- [ADR-0007](../decisions/0007-eu-ki-vo-nicht-anwendbar.md) — EU-KI-VO-Status für GPAI-Tool-Nutzung
- [ADR-0009](../decisions/0009-nodered-sync-source.md) — Sync-Source + Maskierungsregel
- `home-inventory/src/sync/nodered.rs` — `sanitize()`-Heuristik
- `.claude/memory/project_ha_smarthome_mqtt_architecture.md` — Architektur-Schmerzpunkt-Kontext
- `docs/runbooks/nodered-flow-update.md` — Folge-Runbook für Refactor-Umsetzung
