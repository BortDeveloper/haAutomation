# ha-automation — KPI-Baseline (Cockpit Iteration 1)

**Stand**: 2026-05-25 (noch nicht erhoben — Phase 1 für ha-automation aktiv, H.1-STOPP wegen ADR-0004)
**Migriert aus**: `stack-master/shared/standards/cockpit-kpis.md` (Baseline-Tabellen-Zeilen) als Teil der G9-Migration (Allgemein/Spezifisch-Trennung).
**Spec-Quelle (Mess-Vorschriften)**: `stack-master/shared/standards/cockpit-kpis.md` (Sektion „KPI-<n>")
**Owner-/Co-Reviewer-Matrix**: `stack-master/shared/standards/cockpit-kpis.md` oder `stack-master/shared/standards/agent-responsibility-matrix.md`
**Erhebungs-Vorschau**: Iteration 2 nach ADR-0004-Implementierung.

Werte werden hier (im Projekt-Repo) gepflegt; Spec, Schema und Severity-Schwellen bleiben in stack-master.

---

> Erstanlage 2026-05-25 (G9-Migration Phase C-ha).
> Quelle des Gerüsts: `stack-master/shared/milestones/migration-export-ha-automation-2026-05-25.md`
> Sektion 1. Mess-Vorschrift, Schwellwerte, Owner-Matrix:
> `stack-master/shared/standards/cockpit-kpis.md`.
> Iteration-1-Stand: keine KPI-Erhebung; Erstmessung in Iteration 2
> nach ADR-0004-Implementierung.

**Baseline-Phase-Regel**: Solange noch keine Werte vorliegen, gilt
ADR-0005 Folgeentscheidung 3 (measure-only): effektive Severity wird
bei Ersterhebung auf MEDIUM gecappt. Zusätzlich gilt bis 2026-05-31
ein **Sparmodus-Cap** für KPI-1, KPI-2 und KPI-6.

---

## KPI-1 — Security CI strictness ratio

- **Wert**: _noch nicht erhoben_
- **Datum**: —
- **Erheber**: — (Owner: `security`; Co-Reviewer: `architect`, `sre`)
- **Severity (spec → effektiv)**: — → bei Erhebung MEDIUM (Baseline-Cap ADR-0005-FE-3, zusätzlich Sparmodus-Cap bis 2026-05-31)
- **Spec-Referenz**: `stack-master/shared/standards/cockpit-kpis.md` Sektion KPI-1 (inkl. Spec-Patch v2: Job-granular + Transport-Filter)
- **Quell-Sidecar**: — (Iteration 2: `reports/kpi-1-<UTC>.md`)

## KPI-2 — Action pinning score

- **Wert**: _noch nicht erhoben_
- **Datum**: —
- **Erheber**: — (Owner: `security`; Co-Reviewer: `architect`)
- **Severity (spec → effektiv)**: — → bei Erhebung MEDIUM (Baseline-Cap ADR-0005-FE-3, zusätzlich Sparmodus-Cap bis 2026-05-31)
- **Spec-Referenz**: `stack-master/shared/standards/cockpit-kpis.md` Sektion KPI-2
- **Quell-Sidecar**: — (Iteration 2: `reports/kpi-2-<UTC>.md`)

## KPI-3 — Role contract completeness

- **Wert**: _noch nicht erhoben_
- **Datum**: —
- **Erheber**: — (Owner: `architect`; Co-Reviewer: `docs`)
- **Severity (spec → effektiv)**: — → bei Erhebung MEDIUM (Baseline-Cap ADR-0005-FE-3)
- **applies_when**: `tech_stack` enthält `ansible` — für ha-automation
  zu prüfen im Rahmen der ersten Erhebung; entfällt ohne
  Severity-Wirkung, falls Projekt kein Ansible einsetzt.
- **Spec-Referenz**: `stack-master/shared/standards/cockpit-kpis.md` Sektion KPI-3
- **Quell-Sidecar**: — (Iteration 2: `reports/kpi-3-<UTC>.md`, falls applies_when erfüllt)

## KPI-4 — Test coverage breadth

- **Wert**: _noch nicht erhoben_
- **Datum**: —
- **Erheber**: — (Owner: `sre`; Co-Reviewer: `architect`)
- **Severity (spec → effektiv)**: — → bei Erhebung MEDIUM (Trend-Gate ohne Vor-Iteration ist Baseline-MEDIUM, ADR-0005-FE-3)
- **Spec-Referenz**: `stack-master/shared/standards/cockpit-kpis.md` Sektion KPI-4
- **Quell-Sidecar**: — (Iteration 2: `reports/kpi-4-<UTC>.md`)

## KPI-5 — Exception debt index

- **Wert**: _noch nicht erhoben_
- **Datum**: —
- **Erheber**: — (Owner: `architect`; Co-Reviewer: `sre`)
- **Severity (spec → effektiv)**: — → bei Erhebung MEDIUM (kein Trend-Vergleich vor Baseline-Setzung; ADR-0005-FE-3)
- **Spec-Referenz**: `stack-master/shared/standards/cockpit-kpis.md` Sektion KPI-5
- **Quell-Sidecar**: — (Iteration 2: `reports/kpi-5-<UTC>.md`)

## KPI-6 — Supply-chain integrity score

- **Wert**: _noch nicht erhoben_
- **Datum**: —
- **Erheber**: — (Owner: `security`; Co-Reviewer: `sre`)
- **Severity (spec → effektiv)**: — → bei Erhebung MEDIUM (Baseline-Cap ADR-0005-FE-3, zusätzlich Sparmodus-Cap bis 2026-05-31)
- **Spec-Referenz**: `stack-master/shared/standards/cockpit-kpis.md` Sektion KPI-6 (inkl. Revisions-Hinweis 2026-05-25 zur Service-Einheit-Zählung)
- **Quell-Sidecar**: — (Iteration 2: `reports/kpi-6-<UTC>.md`)

## KPI-7 — Secrets exposure control

- **Wert**: _noch nicht erhoben_
- **Datum**: —
- **Erheber**: — (Owner: `security`; Co-Reviewer: `compliance`, `sre`)
- **Severity (spec → effektiv)**: — → bei Erhebung MEDIUM (Baseline-Cap ADR-0005-FE-3)
- **Spec-Referenz**: `stack-master/shared/standards/cockpit-kpis.md` Sektion KPI-7
- **Quell-Sidecar**: — (Iteration 2: `reports/kpi-7-<UTC>.md`)

## KPI-8 — Data-protection governance maturity (AI services)

- **Wert**: _noch nicht erhoben_ — **KI-Komponente in Klärung**
- **Datum**: —
- **Erheber**: — (Owner: `compliance`; Co-Reviewer: `security`, `architect`)
- **Severity (spec → effektiv)**: hängt von `applies_when`-Prüfung ab.
  Falls KI-Komponente bestätigt: bei Erhebung MEDIUM (Baseline-Cap
  ADR-0005-FE-3). Falls keine KI-Komponente: KPI entfällt für
  ha-automation ohne Severity-Wirkung, Gate G3.9 entfällt analog
  (identisch zu G3.8-`applies_when`).
- **Spec-Referenz**: `stack-master/shared/standards/cockpit-kpis.md` Sektion KPI-8
- **Quell-Sidecar**: — (Iteration 2: `reports/kpi-8-<UTC>.md`, falls applies_when erfüllt)
- **Klärungsbedarf**: ha-automation-dev klärt in Iteration 2
  (Anlass: ADR-0004-Implementierung) ob Node-RED-Flows,
  Inferenz-Komponenten oder automatisierte Entscheidungen unter den
  `applies_when`-Bedingungen von KPI-8 / G3.9 fallen. Klärung wird in
  Folge-Audit-Report oder Folge-ADR dokumentiert. Cross-Referenz:
  siehe `docs/cockpit/iteration-1-backlog.md` Sektion „Vorschau
  Iteration 2".
