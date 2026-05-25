# Cockpit-Verzahnung ha-automation

Dieses Verzeichnis enthält die **projekt-lokalen** Cockpit-Artefakte
für ha-automation. Spec, Standards und ADRs liegen weiterhin
zentral im Cockpit-Repo (`stack-master`).

## Verortung (G9: Allgemein vs. spezifisch)

- **Projekt-lokal** (hier in diesem Repo):
  - `docs/cockpit/kpi-baseline.md` — gemessene KPI-Werte für
    ha-automation (Iteration-1-Stand: noch nicht erhoben).
  - `docs/cockpit/iteration-1-backlog.md` — Iteration-1-Status
    inkl. H.1-STOPP, freigegebener Branches und Vorschau Iter 2.
  - `audit-log/` — Audit-Reports aller Experten zu ha-automation
    (Initial-Audit 2026-05-16: architect, docs, security, sre).
  - `reports/` (entsteht in Iteration 2) — KPI-Sidecar-Reports
    (`kpi-<n>-<UTC>.md`).

- **Cockpit-zentral** (in `stack-master`):
  - `shared/standards/cockpit-kpis.md` — KPI-Spec,
    Mess-Vorschriften, Schwellwerte.
  - `shared/standards/agent-responsibility-matrix.md` —
    Owner-/Co-Reviewer-Matrix.
  - `shared/architecture-decisions/` — alle Cockpit-ADRs, darunter
    ADR-0004 (Edge-Secret-Backup) als primärer Iter-2-Auftrag.
  - `config/quality-gates.yaml` — Gate-Definitionen.

## Iterations-Stand

Iteration 1: **H.1-STOPP aktiv** (Details: `iteration-1-backlog.md`).
