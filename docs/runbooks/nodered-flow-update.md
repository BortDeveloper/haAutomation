# Runbook: Node-RED-Flow kontrolliert ändern

**Zweck:** Einen bestehenden Node-RED-Flow ändern, ohne den Produktiv-Betrieb
zu gefährden. Optimiert für Solo-Setup ohne Two-Person-Rule, mit Test-HA als
Sicherheits-Stage.

**Geltungsbereich:** Änderungen an Flows im Node-RED-Add-on (Produktiv-HA),
ausgelöst entweder durch
[Flow-Analyse mit Claude](nodered-flow-analysis-with-claude.md) oder durch
ad-hoc-Feature-Wunsch. Erfasst **nicht** Add-on-Updates selbst (separate
Disziplin).

**Anker:** ISO/IEC 27001:2022 **A.8.32** (Change Management) — Änderungen an
operativen Systemen mit dokumentiertem Pre-State, Test, Rollback-Pfad.

> ⚠️ **Kernregel:** Vor *jeder* Flow-Änderung in Produktion gibt es einen
> committeten Pre-Snapshot (`inventory sync nodered` + Git-Tag). Ohne den ist
> der Rollback-Pfad nicht garantiert reproduzierbar.

## Voraussetzungen

- [ ] Test-HA + Test-Node-RED-Add-on stehen (siehe
      [docs/test-umgebung-real-hardware.md](../test-umgebung-real-hardware.md)).
- [ ] `inventory`-Binary gebaut, `smoke-test.sh` läuft grün gegen **beide**
      HAs (Test und Produktion) — separate `local/`-env-Dateien.
- [ ] Repo ist clean. Eine Branch ist ok, aber kein offener Dirty-Tree.
- [ ] Optional, empfohlen: vorausgehende Analyse-Session mit Audit-Log-Eintrag
      (siehe [Flow-Analyse-Runbook](nodered-flow-analysis-with-claude.md)
      Schritt 5).

## Schritt 1 — Pre-Snapshot anlegen

1. **HA-Backup** auf der Produktiv-HA:
   - UI: Einstellungen → System → Backups → „Backup erstellen", Name
     `pre-nodered-change-YYYY-MM-DD`.
   - Backup enthält die Add-on-Daten inkl. `flows.json` als ZIP.
   - Backup auf NAS / USB ziehen (nicht nur auf der HA-SD lassen).

2. **Git-Snapshot** des aktuellen Sync-Stands:
   ```sh
   cd inventory
   INVENTORY_DB=./inventory.db INVENTORY_YAML_DIR=./yaml \
     ./target/release/inventory sync nodered
   cd ..
   git add yaml/nodered.yaml
   git commit -m "snapshot(nodered): pre-change YYYY-MM-DD"
   git tag nodered-pre-YYYY-MM-DD
   ```
   Falls `git status` nach dem Sync leer ist (kein Diff): aktueller Repo-Stand
   ist schon der Pre-Snapshot, nur den Tag setzen.

## Schritt 2 — Änderung in der Test-Umgebung umsetzen

1. **Test-HA hochfahren** (RPi-2 aus `docs/test-umgebung-real-hardware.md`).
2. Falls die Änderung gegen einen anderen Flow-Stand passieren soll als das,
   was die Test-HA aktuell hat: Produktiv-`flows.json` aus dem HA-Backup in
   die Test-Node-RED-Add-on-Konfig kopieren (Pfad
   `/addon_configs/<slug>/flows.json`, via Samba-Share oder SSH-Add-on).
3. **Node-RED-Editor** in der Test-HA öffnen (Sidebar → Node-RED → über
   Supervisor-Ingress).
4. Änderung umsetzen. Empfehlung: Pro Change-Topic **ein** Commit-würdiger
   Diff. Keine 5-Refactors-in-1-Session, der Diff im YAML wäre nicht
   nachvollziehbar.
5. **Deploy** im Test-Editor (Button rechts oben). Wenn der Test-Aktor (z. B.
   HM-Schaltsteckdose) wie erwartet reagiert: Schritt 3.

## Schritt 3 — Diff verifizieren

Sync gegen die Test-HA, dann gegen den Pre-Snapshot diffen:

```sh
cd inventory
# gegen Test-HA syncen (eigene env mit HA_URL=test-ha + NODERED_INGRESS_PATH)
./smoke-test.sh
diff yaml/nodered.yaml local/yaml/nodered.yaml | less
```

Erwartung:
- Diff zeigt **genau** die geplante Änderung, nicht mehr und nicht weniger.
- Maskierte Felder bleiben `***masked***` — auftauchen von Klartext bedeutet
  ein neues Custom-Node-Schema, das die Sanitizer-Heuristik nicht greift
  (→ vor dem Produktiv-Deploy in `inventory/src/sync/nodered.rs`
  `is_sensitive_key` ergänzen oder Allow-List nachpflegen).
- Sortier-Reihenfolge (`id`-alphabetisch) bleibt stabil — Diff zeigt
  inhaltliche Änderung, keine Reordering-Noise.

Wenn der Diff Drift enthält, der nicht von der Änderung kommt: **STOPP**. Drift
heißt, dass die Test-HA in einem anderen Zustand war als angenommen. Schritt 2
neu aufrollen.

## Schritt 4 — Cargo-Tests (falls Sanitizer-Erweiterung)

Wurde in Schritt 3 ein neues Custom-Node-Schema entdeckt UND der Sanitizer
erweitert: vor dem Produktiv-Deploy die Tests grün halten:

```sh
cargo test --bin inventory sync::nodered
```

Erwartet: 13/13 Tests grün. Wenn ein Test fehlschlägt, ist der Sanitizer in
einem Zustand, in dem Klartext-Credentials in `yaml/nodered.yaml` landen
können → **kein Produktiv-Deploy** bevor das gefixt ist.

## Schritt 5 — Produktiv-Deploy

Zwei Wege, je nach Vertrauen in den Test-Lauf:

**Weg A — Editor-Deploy (Standard):** Änderung im Produktiv-Node-RED-Editor
nachbauen (Test-Editor offen lassen, parallel im zweiten Browser-Tab). Vorteil:
visuelle Verifikation. Nachteil: manuell, fehleranfällig bei vielen Änderungen.

**Weg B — API-Push (für umfangreiche Änderungen):**

1. `flows.json` aus dem Test-Container exportieren (UI: Menü → Export → alle
   Flows → Download-Datei).
2. Mit Bedacht auf Produktiv pushen:
   ```sh
   curl -X POST "${HA_URL}/${NODERED_INGRESS_PATH}/flows" \
     -H "Authorization: Bearer ${HA_TOKEN}" \
     -H "Content-Type: application/json" \
     -H "Node-RED-Deployment-Type: full" \
     --data @flows-from-test.json
   ```
3. Node-RED restartet die Flows automatisch — kurze Unterbrechung (<5 s).

Weg B ist **mächtiger und gefährlicher**: ein POST überschreibt **alle** Flows,
nicht nur den geänderten. Vorbedingung: Schritt 1 (Pre-Snapshot) ist sicher.

## Schritt 6 — Post-Snapshot + Commit

```sh
cd inventory
# gegen Produktiv-HA
INVENTORY_DB=./inventory.db INVENTORY_YAML_DIR=./yaml \
  ./target/release/inventory sync nodered
cd ..
git add yaml/nodered.yaml
git commit -m "feat(nodered): <kurze Aenderungsbeschreibung>

Anker: <Audit-Log-Eintrag>, Test-Lauf: gruen (siehe Schritt 3).
Pre-Snapshot-Tag: nodered-pre-YYYY-MM-DD."
```

Optional Tag setzen: `git tag nodered-post-YYYY-MM-DD-<topic>`.

## Rollback-Pfad

Wenn der Produktiv-Deploy ein Problem zeigt (Flow-Fehler, falsche
Schalt-Logik, Geräte hängen):

**Schnell-Rollback (Minuten):**

1. In der Produktiv-Node-RED-UI: Menü → Deploy → „Restore deployed flows"
   (macht das **vorletzte** Deploy rückgängig, nur 1 Schritt zurück).

**Sicherer Rollback aus Pre-Snapshot:**

1. Pre-Snapshot-Tag auschecken:
   ```sh
   git show nodered-pre-YYYY-MM-DD:yaml/nodered.yaml > /tmp/pre.yaml
   ```
2. Die YAML ist sanitisiert — sie ist **kein** Re-Import-Format. Für echten
   Rollback aus dem HA-Backup restoren:
   - HA-UI → Einstellungen → System → Backups → Backup `pre-nodered-change-…`
     → „Wiederherstellen" → nur „Add-on Node-RED" auswählen.
   - HA bootet das Add-on neu, alte `flows.json` ist aktiv.

**Kompletter HA-Restore** (nur als allerletzte Option, weil ALLE HA-States
auf den Backup-Zeitpunkt zurückgesetzt werden): HA-Backup voll restore.

## Wenn der Test-Deploy fehlschlägt

- Test-HA in einen sauberen Snapshot zurücksetzen (Test-HA hat eigenen
  Backup-Zyklus, siehe Hardware-Doku §12.2).
- Pre-Snapshot der Produktiv-`flows.json` erneut auf Test-HA spielen
  (Schritt 2.2).
- Änderung in kleinere Sub-Steps zerlegen — die meisten Test-Deploy-Fehler
  kommen aus einer „zu grossen Change" pro Iteration.

## Bezug

- [Flow-Analyse mit Claude](nodered-flow-analysis-with-claude.md) — typischer
  Vorgänger zu diesem Runbook
- [ADR-0009](../decisions/0009-nodered-sync-source.md) — Sync-Source +
  Maskierungs-Heuristik
- [docs/test-umgebung-real-hardware.md](../test-umgebung-real-hardware.md) — Aufbau Test-HA
- `inventory/smoke-test.sh` — Verifikations-Smoke
- `inventory/src/sync/nodered.rs` — `sanitize()` und `is_sensitive_key`
- ISO/IEC 27001:2022 A.8.32 — Change Management
