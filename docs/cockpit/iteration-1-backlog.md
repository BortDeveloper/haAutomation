# ha-automation — Iteration-1-Backlog (Cockpit)

**Stand**: 2026-05-25 (Iteration 1, H.1-STOPP aktiv)
**Migriert aus**: `stack-master/shared/milestones/iteration-1-tasks.md` (Sektion ha-automation, Zeilen 71–73) als Teil der G9-Migration (Allgemein/Spezifisch-Trennung).
**Bezug**: `stack-master/shared/milestones/2026-05-25-general-vs-specific-migration.md`

> Erstanlage 2026-05-25 (G9-Migration Phase C-ha).
> Quelle: `stack-master/shared/milestones/iteration-1-tasks.md` (ha-automation-Sektion).
> Mess-Vorschrift, Schwellwerte, Owner-Matrix:
> `stack-master/shared/standards/cockpit-kpis.md`.
> Cockpit-Hochlevel-Steuerung (projektübergreifender Sprint-Plan,
> Sunset-Choreografie) bleibt im Orchestrator-Backlog von stack-master.

---

## Status Iteration 1

**H.1-STOPP aktiv.**

Cross-Phase-Gate H.1 ist für ha-automation **aktiviert**, bis ADR-0004
(Edge-Secret-Backup) implementiert ist.

**Begründung**: ADR-0004 ist **AKZEPTIERT**, aber die Implementierung
hängt an externer Hardware-Token-Beschaffung. Implementierungs-Scope:
sops/age mit 3 Recipients, DR-Hardware-Token, Restore-Drill,
gitleaks-Hook.

**Spec-Referenz ADR**: `stack-master/shared/architecture-decisions/0004-edge-secret-backup.md`

### Ausgeschlossen während H.1-STOPP

- (a) Maßnahmen, die neue Secret-Klassen einführen oder das
      Secret-Backup-Substrat ändern.
- (b) Breit angelegte HARDENING-Maßnahmen, die ADR-0004-Substanz
      vorgreifen.

### Erlaubt während H.1-STOPP

- Andere Feature- oder Bugfix-Arbeit, sofern keine neuen
  Secret-Klassen eingeführt werden.
- Konkret freigegeben in Iteration 1: **`fix/authgate-location-url-encoding`** (einzige Ausnahme im Sprint).

### PR-Hinweis

- **`feat/nodered-integration`**: existierender PR — **„do NOT merge"**
  solange H.1-STOPP aktiv ist (führt potentiell neue Secret-Klassen ein,
  fällt unter Ausschluss (a)).

---

## Vorschau Iteration 2

Nach Eingang der DR-Hardware-Token und ADR-0004-Implementierungs-Start:

1. **ADR-0004-Implementierung**
   - sops/age-Setup mit 3 Recipients (Edge-Host, Maintainer-Hauptkey,
     DR-Hardware-Token)
   - DR-Hardware-Token-Setup via `ykman` (nur nach explizitem
     User-Konsens, Hardware-Interaktion)
   - Restore-Drill dokumentieren (nicht nur Backup-Pfad testen)
   - gitleaks-Hook in pre-commit verankern (Repo trägt bereits
     `.gitleaks.toml`)
   - Abschluss-Audit durch `security` + `sre` → H.1-Deaktivierung
     durch Orchestrator

2. **KPI-Baseline-Erstmessung (alle 8 KPIs)**
   - Werte werden in `docs/cockpit/kpi-baseline.md` gepflegt
     (Wert / Datum / Erheber / Sidecar-Pfad).
   - Sidecar-Reports landen in `reports/kpi-<n>-<UTC>.md`
     (Konvention analog vps-stack; Verzeichnis wird mit erster
     Erhebung erstmals angelegt).
   - Baseline-Phase-Regel (ADR-0005-FE-3): effektive Severity bei
     Ersterhebung MEDIUM-gecappt.

3. **KPI-8 / G3.9 `applies_when`-Klärung**
   - Frage: Fallen Node-RED-Flows, Inferenz-Komponenten oder
     automatisierte Entscheidungen im ha-automation-Stack unter die
     `applies_when`-Bedingungen von KPI-8?
   - Owner-Klärung: `compliance` (Co-Reviewer: `security`, `architect`).
   - Ergebnis-Doku: Folge-Audit-Report oder Folge-ADR.

4. **ADR-0009 (ha-automation-IdP-Integration)** durch `architect`
   - Vorbereitung der Phase-3-Querschnittsintegration (SSO/IdP).
   - Anstoß durch Orchestrator, sobald Phase 2 für ha-automation
     anläuft.

5. **Stack-Drift-ADR** (Iteration-2-Slot)
   - `config/cockpit.yaml` listet Tech-Stack `[home-assistant, mqtt,
     zigbee]`; Repo-Realität ist Rust-Inventory-Backend, das
     HA/CCU/Hue/Shelly via VPN observiert.
   - ADR fixiert: Repo-Realität ist Source of Truth, cockpit.yaml
     wird durch Orchestrator nachgezogen.

---

## Nachtrag 2026-07-25 — ADR-0004 repo-seitig umgesetzt (H.1-Teilschließung)

Repo-seitige ADR-0004-Struktur ist vollständig (Ziel-Direktive G2-Pass,
Welle 1):

- `edge-secrets/.sops.yaml` (Edge-Klasse-creation_rules, SSOT) +
  `edge-secrets/README.md` (Layout, Regeln)
- Scripts: `scripts/edge-secrets/{encrypt,decrypt,verify}.sh`
  (fail-closed ohne Recipients; verify läuft ohne private Schlüssel)
- Runbook `docs/runbooks/edge-secret-backup.md` (Backup/Restore/Drill/
  Rotation) + Operator-Checkliste
  `docs/runbooks/operator-checklist-adr-0004.md`
- gitleaks-pre-commit-Hook war bereits aktiv (`.gitleaks.toml`,
  `scripts/install-hooks.sh`)

**Bewusst NICHT repo-seitig lösbar (Operator-Rest, blockiert
H.1-Deaktivierung weiterhin):** Erzeugung der 3 age-Keys (Edge-Host,
Backup-Operator, DR-Hardware-Token inkl. Beschaffung/Off-Site-Standort),
Recipient-Eintragung, Erst-Backup, **Restore-Drill mit Beleg** (G2.6),
restic-Off-Site (Folge-ADR Backup-Target). Keine Dummy-Keys im Repo —
Pipeline bleibt bis zur Key-Eintragung fail-closed.
H.1-Deaktivierung bleibt Orchestrator-Akt nach Abschluss-Audit.

### Offene Operator-Tasks (außerhalb dieses Repos)

- **N-1 (aus ccu2mqtt-A1-security, G2.2-relevant): influxdb-Klartext-
  Passwort in der Live-`configuration.yaml`** der HA-Instanz.
  Repo-Befund 2026-07-25: die Datei ist in diesem Repo **nicht getrackt**
  (kein `influx`-Treffer im Working Tree; Pickaxe-Suche über die gesamte
  Git-History aller Branches negativ) — der Wert war hier **nie
  committet**, aus Repo-Sicht daher keine Rotationspflicht.
  Fix auf der Live-Instanz (Operator): Wert nach HA-Konvention in
  `/config/secrets.yaml` verschieben (`influxdb_password: ...`), in
  `configuration.yaml` per `!secret influxdb_password` referenzieren;
  `secrets.yaml` wird danach über den ADR-0004-Backup-Pfad als
  `edge-secrets/homeassistant/secrets.yaml.enc` gesichert (niemals im
  Klartext committen — `.gitignore` sperrt `**/*secrets.yaml`).
  Ob der Wert anderweitig exponiert war (Rotationsentscheid), bewertet
  der ccu2mqtt-A1-Kontext, nicht dieses Repo.

---

## Verweise

- KPI-Baseline-Werte (Iteration-2-Ziel): `docs/cockpit/kpi-baseline.md`
- Historische Audit-Reports: `audit-log/2026-05-16-*.md`
- Cockpit-Steuerung & Hochlevel-Plan: `stack-master/shared/milestones/`
