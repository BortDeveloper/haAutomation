# Operator-Pflicht-Run-Beleg — 2026-07-27

**Projekt:** ha-automation
**Quality Gate:** G2.17 (Operator-Pflicht-Run vor Phase-2-Pass, dauerhaft)
**Standard-Bezug:** NIST SP 800-204C §3.2 (build-time verification +
actual run integrity); SLSA Build L2+; ISO/IEC 27001:2022 A.8.16

## Kontext

Der Run erfolgt nach vollzogenem Erst-Backup (F-11-Kern, Commits
`77a5eb8`/`171f304`) und bestandenem 2-Pfad-Restore-Drill — Belege
gleichen Datums in diesem Verzeichnis.

## Pflichtfelder (G2.17-Evidence-Spec)

| Feld | Wert |
| --- | --- |
| Run-ID | 30263851204 |
| Workflow | `.github/workflows/ci.yml` (Name „CI Smoke Build") |
| Trigger | `workflow_dispatch` gegen `main` @ `171f304`, ausgelöst vom Operator (User) per `gh workflow run ci.yml -R BortDeveloper/haAutomation -r main` |
| Trigger-UTC | 2026-07-27T11:56:19Z |
| Ende-UTC | 2026-07-27T11:57:05Z |
| Conclusion | completed / success |

## Referenz Verifikations-Klassen (Standard §5, G2.17-Pflicht)

Gemäß `wiederkehrende-verifikation.md` §6, Zeile ha-automation:

- **(a) Restore-Drill:** vollzogen 2026-07-27 als 2-Pfad-Drill
  (Interim per ADR-0004 Nachtrag 2), Beleg
  [`2026-07-27-restore-drill-2-pfad.md`](2026-07-27-restore-drill-2-pfad.md);
  **P3 (DR-Token) offen** bis F-59 (2026-08-15).
- **(b) Negativtest HA-Login/API-Token-Zugriffspfad:** **OFFEN** —
  nicht vollzogen, im Gate-Check als offener Punkt auszuweisen.
- **(c) Runbook-Rehearsal:** vollzogen — das Runbook
  `docs/runbooks/edge-secret-backup.md` wurde beim Erst-Backup und
  Drill wörtlich exekutiert; gefundene Drift (Scope-Tabelle §1,
  `.sbk`-Format, Typ-Flags) dokumentiert und korrigiert.

## Konsumierender Phase-2-Gate-Check (Verweis)

- stack-master `shared/milestones/current.md` —
  ha-automation-G2-Gate-Check 2026-07-27 (Orchestrator, folgt).

## Fazit

Der Operator-Pflicht-Run auf `main` ist mit Run-ID, Trigger und
Conclusion belegt. Die Gate-FORMAL-Erklärung „bestanden" obliegt dem
Orchestrator nach Experten-Review, nicht diesem Agenten.
