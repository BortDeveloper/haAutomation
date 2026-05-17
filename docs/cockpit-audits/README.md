# Cockpit-Audits & Architekturvorgaben

Dieses Verzeichnis enthält **externe Bewertungen und Architekturvorgaben**,
die im übergeordneten Governance-Framework
(`stack-master`, Multi-Project-Multi-Agent-Cockpit) erstellt wurden und
`ha-automation` als teilnehmendes Projekt betreffen.

> **Quelle der Originale**: `stack-master/shared/audit-log/` bzw.
> `stack-master/shared/milestones/`. Die Dateien hier sind Kopien zum
> Stichtag der jeweiligen Iteration und werden bei neuen Audits ergänzt,
> nicht überschrieben. Maßgeblich für die laufende Bearbeitung bleibt die
> jeweils neueste Fassung.

## Inhalt

### Iteration 0 (2026-05-16) — Initial-Audit

Vier parallele Expertenprüfungen des IST-Stands von `ha-automation`:

| Datei | Experte | Fokus |
|---|---|---|
| [`2026-05-16-architect.md`](2026-05-16-architect.md) | architect | Gesamtarchitektur, ADR-Lücken, Cockpit-Konvergenz, Stack-Drift `cockpit.yaml` |
| [`2026-05-16-security.md`](2026-05-16-security.md) | security | Secrets, TLS/Auth, Container-Hardening, CVE-Bewusstsein, Zigbee-Network-Key (CRITICAL) |
| [`2026-05-16-sre.md`](2026-05-16-sre.md) | sre | Observability, Backup/Restore, Idempotenz, Deployment-Robustheit, Zigbee-Backup-Lücke (CRITICAL) |
| [`2026-05-16-docs.md`](2026-05-16-docs.md) | docs | README-Struktur, Architecture Diagrams, Runbooks, Onboarding |

Jeder Report enthält Findings, Recommendations, externe-Standard-Belege
(BSI, NIST, OWASP, CIS, ISO/IEC 27001, EU Data Act, EU KI-VO) und eine
Bias-Reflexion (KIC-EIN-2 / EU KI-VO Art. 14).

### Iteration 2 (2026-05-17) — Phase-1-Closing-Brief

| Datei | Adressat |
|---|---|
| [`2026-05-17-phase-1-closing-brief.md`](2026-05-17-phase-1-closing-brief.md) | `ha-automation-dev`, Review durch architect/security/sre |

Konsolidiert die Architekturvorgaben, die für den **formalen Abschluss
von Phase 1 (AUDIT)** in diesem Projekt erfüllt sein müssen. Bezugs-ADR:
[ADR-0004 Verschlüsseltes Edge-Secret-Backup mit Schlüssel-Eskrow](#).

## Wie sind diese Dokumente zu nutzen?

1. **Audit-Reports** sind Stichtags-Belege. Sie ändern sich nicht
   rückwirkend; spätere Korrekturen erscheinen in neuen Reports mit
   neuem Datum.
2. **Phase-Briefs** sind Arbeitsdokumente. Sie listen Abnahmekriterien
   und werden im Cockpit-Repo gepflegt; die hier abgelegte Kopie ist
   der Stand zum Übergabe-Zeitpunkt.
3. **Findings-Schweregrade** (CRITICAL/HIGH/MEDIUM/LOW) folgen der
   Cockpit-Konvention aus `stack-master/config/quality-gates.yaml`.
   CRITICAL stoppt das Projekt (H.1 STOPP-Taste), HIGH blockiert die
   nächste Phase, MEDIUM/LOW sind Backlog.
4. **ADR-Referenzen** (z.B. „ADR-0004") verweisen auf
   `stack-master/shared/architecture-decisions/`. Diese sind die
   normativen Architekturentscheidungen.

## Kontakt zur Cockpit-Ebene

Rückfragen, Widerspruch oder Re-Open-Anträge zu Findings/Vorgaben gehen
über das `stack-master`-Repo (Issues oder PRs auf
`shared/architecture-decisions/`). Das Cockpit ist Single Source of
Truth für phasenübergreifende Architekturentscheidungen — Änderungen
hier in `projects/ha-automation/docs/cockpit-audits/` allein haben
keinen normativen Effekt.
