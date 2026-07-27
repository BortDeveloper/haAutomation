# Erst-Backup-Beleg Edge-Secrets (ADR-0004, F-11-Kern) — 2026-07-27

**Projekt:** ha-automation
**Quality-Gate-Bezug:** G2.2 (keine Klartext-Secrets), G2.6
(Backup-Substanz); Cockpit-ADR-0004 Nachtrag 1 Auflage b
(Erst-Backup-Beleg-Pflicht) und Nachtrag 2 (Interim-2-Recipient-Modus).
**Standard-Bezug:** `stack-master:shared/standards/wiederkehrende-verifikation.md`
§3 (Beleg-Format; gemäß Runbook §3 ohne die Drill-Felder
RTO/Recipient-Pfade — dafür siehe
[`2026-07-27-restore-drill-2-pfad.md`](2026-07-27-restore-drill-2-pfad.md)).
**Ausführender:** Operator (User); Protokoll destilliert durch
Projekt-Agent.
**Datum (UTC):** 2026-07-27
**Geprüfter Stand:** Pipeline/Runbook `main`; Chiffrate-Commits
`77a5eb8` und `171f304`.

## Kommando-Kette und Ergebnis

Je Datei wörtlich nach Runbook
`docs/runbooks/edge-secret-backup.md` §3 (Klartext via SSH/Samba nach
außerhalb des Repos → `scripts/edge-secrets/encrypt.sh` →
`scripts/edge-secrets/verify.sh` → Commit mit gitleaks-pre-commit-Hook
→ `shred -u`):

| Prüfpunkt | Gegenstand | Ergebnis |
| --- | --- | --- |
| **E1** HA `secrets.yaml` | `edge-secrets/homeassistant/secrets.yaml.enc` (Commit `77a5eb8`) | **PASS** |
| **E2** Node-RED `flows_cred.json` | `edge-secrets/nodered/flows_cred.json.enc` (Commit `77a5eb8`) | **PASS** |
| **E3** CCU3-Systemsicherung | `edge-secrets/ccu/ccu3-3.85.7.20251129-2026-07-27-1335.sbk.enc` (Commit `171f304`; CCU-sops-Regel im selben Commit um die `.sbk`-Endung erweitert) | **PASS** |
| **V** Gesamt-Verifikation | `scripts/edge-secrets/verify.sh` → „Chiffrate: 3, Pubkeys: 2/2" | **PASS** |
| **G** Leak-Guard | gitleaks (pre-commit, fail-closed) je Commit: „no leaks found" | **PASS** |
| **S** Klartext-Hygiene | alle Klartexte nach Verschlüsselung per `shred -u` vernichtet | **PASS** |

**Gesamtergebnis: 6/6 PASS.** 3 Chiffrate versioniert, keine
Klartext-Secrets im Repo.

## Recipient-Lage (Interim, ADR-0004 Nachtrag 2)

2 age-Recipients aktiv (`edge-secrets/recipients/edge-host-2026-07-27.pub`,
`backup-operator-2026-07-27.pub`), `encrypt.sh`-Guard `==2` per
Nachtrag 2 (`a149003`). Der dritte Recipient (DR-Hardware-Token) ist
**F-59** mit Frist **2026-08-15**; bis dahin gilt der
Interim-2-Recipient-Modus, keine volle H.1-Deaktivierung.

## Abweichungsliste — Scope-Realität vs. Runbook-Soll §1

Vor Ort am 2026-07-27 festgestellt; Runbook §1 wurde im selben Commit
wie dieser Beleg an die Realität angepasst:

- **A1 — Kein Zigbee auf dem Edge-Host:** kein Z2M-/deCONZ-Add-on
  installiert, kein ZHA-Eintrag in `core.config_entries`. Beide
  Z2M-Zeilen des bisherigen Soll-Scopes sind **Real-N/A**; das im
  ADR-0004 als irreversibel geführte Mesh-Key-Material existiert auf
  diesem Host nicht.
- **A2 — Matter-Server installiert, aber ungenutzt:** kein
  `matter`-Eintrag in `config_entries`, `addon_configs` leer → **N/A**;
  **LOW-Empfehlung:** Add-on deinstallieren erwägen
  (Angriffsflächen-/Scope-Hygiene).
- **A3 — HA-Tokens:** Long-Lived-/Supervisor-Tokens sind reissuable
  (kein irreversibles Material) → **N/A mit Begründung** statt
  `tokens.json`-Export; nach einem Restore werden Tokens neu
  ausgestellt.
- **A4 — Mosquitto passwd/ACL:** **N/A per Querverweis** — die Logins
  liegen in den Add-on-Optionen (über den HA-Backup-Restore-Pfad
  abgedeckt; real bewiesen im mqtt-broker-addon-Drill 2026-07-27) und
  in ccu2mqtt `.local/secrets`; ACL-SSOT ist das ccu2mqtt-Repo
  (ADR-0017).
- **A5 — CCU3-Sicherungsformat:** die CCU3 liefert `.sbk`, nicht
  `.tar.gz`. sops-Regel bereits in `171f304` gefixt; Runbook-Tabelle
  im selben Commit wie dieser Beleg nachgezogen.

## Fazit

Der Erst-Backup-Materialschluss nach ADR-0004 (Nachtrag 1, Auflage b)
ist für den **realen** Edge-Scope (3 Secret-Bestände) vollzogen und
belegt. Die N/A-Einstufungen A1–A4 sind Beleg-Bestandteil im Sinne von
Standard §6 („n/a nur mit Begründung"). Restore-Nachweis: siehe
Drill-Beleg gleichen Datums.
