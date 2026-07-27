# 2-Pfad-Restore-Drill Edge-Secrets (G2.6, Klasse (a)) — 2026-07-27

**Projekt:** ha-automation
**Quality Gate:** G2.6 (Backup nur mit Drill-Beleg nach Standard)
**Standard-Bezug:** `stack-master:shared/standards/wiederkehrende-verifikation.md`
§2 Klasse (a), Beleg-Format §3, Terminierung §4; BSI IT-Grundschutz
CON.3; Google SRE Book Kap. 26 („the proof of a backup is a restore");
Cockpit-ADR-0004 §6 i. V. m. Nachtrag 2 (Interim: 2-Pfad-Drill).
**Ausführender:** Operator (User); Protokoll destilliert durch
Projekt-Agent. Beleg projektlokal geführt (Standard §3:
Projekt-Audit-Log nach G9), Querverweis aus stack-master folgt per
Gate-Check.
**Datum (UTC):** 2026-07-27
**Geprüfter Stand:** `main` @ `171f304` (Runbook
`docs/runbooks/edge-secret-backup.md` + 3 Chiffrate unter
`edge-secrets/`).

## Vorbereitung

sops **v3.10.2** auf beiden Zielhosts checksum-verifiziert installiert
(Binary-Download + SHA-256-Abgleich gegen Release-Checksums), da die
Zielhosts das Repo und damit `decrypt.sh` nicht tragen.

## Kommando-Kette und Ergebnis (je Recipient-Pfad ein Prüfpunkt)

Je Pfad: Chiffrate auf den Zielhost geholt, `SOPS_AGE_KEY_FILE`
gesetzt, alle 3 Chiffrate mit `sops decrypt` (explizite
`--input-type`/`--output-type`-Flags, s. Abweichung c) nach außerhalb
jedes Repos entschlüsselt, `sha256sum` gebildet, Klartexte per
`shred -u` vernichtet.

| Prüfpunkt | Recipient-Pfad | Ergebnis |
| --- | --- | --- |
| **P1** Edge-Host | HAOS, `SOPS_AGE_KEY_FILE=/etc/inventory/age.key` — 3/3 Chiffrate entschlüsselt, Hashes s. u. | **PASS** |
| **P2** Backup-Operator | VPS, `SOPS_AGE_KEY_FILE=/etc/backup/age.key` — 3/3 Chiffrate entschlüsselt, **alle drei Hashes identisch zu P1** | **PASS** |
| **P3** DR-Hardware-Token | AUSSTEHEND bis Token-Nachrüstung (F-59, Frist 2026-08-15) — per ADR-0004 Nachtrag 2 nicht Teil des Interim-Drills | **OFFEN** (kein FAIL) |

Vollständigkeits-Hashes (SHA-256, P1 ≡ P2 kreuzverglichen):

| Datei | SHA-256 |
| --- | --- |
| `secrets.yaml` | `857af2b9ab281e3739c89dcd84e2ac07cb8e97e1ee67ef4e421f83b27bff0942` |
| `flows_cred.json` | `80ca0f9b12346fa8bbfacb90cb36d5a106ef0e4635ba4a4e6d10f2f8c45c43f9` |
| `ccu3-3.85.7.20251129-2026-07-27-1335.sbk` | `7301fbf5ba649b58e8c6105d2aaf69a444c10b925bdf791379fd80ac3a3c3e81` |

**Gesamtergebnis: 2/2 gedrillte Pfade PASS (BESTANDEN im
Interim-Umfang); P3 offen per F-59.**

## Abweichungsliste (Methodik, ehrlich ausgewiesen)

- **(a) Keine Vorab-Referenz-Hashes:** beim Backup-Lauf wurden keine
  Klartext-Referenz-Hashes notiert. Ersatz-Kriterium dieses Drills:
  die encrypt-Zeit-Sofortverifikation von `encrypt.sh`
  (Roundtrip-Check beim Verschlüsseln) **plus** der
  P1≡P2-Kreuzvergleich aller drei Hashes über zwei unabhängige
  Recipient-Keys. Künftige Läufe: Referenz-Hash direkt beim
  Backup-Lauf festhalten.
- **(b) RTO nicht präzise gemessen:** Größenordnung wenige Minuten je
  Pfad **inklusive** sops-Binary-Install. Künftige Läufe: Zeitstempel
  Start/Ende je Pfad notieren.
- **(c) Typ-Flags außerhalb des Repos nötig:** auf den Zielhosten
  (ohne Repo, ohne `decrypt.sh`) brauchte `sops decrypt` explizite
  `--input-type`/`--output-type`-Flags (`json` bzw. `yaml`), weil die
  Typ-Erkennung an der `.enc`-Endung scheitert. `decrypt.sh` kodiert
  das, ist am Zielhost aber nicht vorhanden — als Hinweis in Runbook
  §5 aufgenommen (gleicher Commit).
- **(d) P3 ausstehend:** der DR-Token-Pfad wird nach dem Token-Setup
  (F-59, 2026-08-15) nachgeholt; erst mit vollzogenem P3 greift die
  volle H.1-Deaktivierungs-Voraussetzung nach ADR-0004 §6.

## Terminierung (Standard §4)

- Folgetermin Klasse (a) nach Runbook-Frequenz (vierteljährlich):
  **+3 Monate → fällig 2026-10-27**.
- Zusätzlich ereignisgetrieben: **P3-Nachhol-Drill** unmittelbar nach
  DR-Token-Setup (F-59, Frist 2026-08-15).
- Beide Einträge mit Owner und Verfalls-Folge im
  `stack-master:shared/milestones/follow-up-register.md` sind
  **Orchestrator-Akt** (dieses Projekt schreibt nicht in stack-master)
  — referenziert im ha-automation-G2-Gate-Check 2026-07-27.
