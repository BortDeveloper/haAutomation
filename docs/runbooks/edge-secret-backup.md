# Runbook: Edge-Secret-Backup & Restore (Cockpit-ADR-0004)

**Zweck:** Verschlüsseltes, wiederherstellbares Backup aller Edge-Secrets
(Zigbee, Home Assistant, Mosquitto, Node-RED, CCU) mit
Multi-Recipient-Eskrow (Ziel: 3 age-Recipients) und gedrilltem Restore-Pfad.

> **INTERIM (ADR-0004 Nachtrag 2, 2026-07-27):** Die Pipeline läuft
> übergangsweise mit **2 Recipients** (Edge-Host + Backup-Operator).
> Der DR-Hardware-Token-Recipient wird bis **2026-08-15** nachgerüstet
> (YubiKey-Konsens besteht, nur verschoben). Bis dahin ist der Drill ein
> **2-Pfad-Drill** (P1 Edge, P2 Operator); P3 (DR-Token) wird nach dem
> Token-Setup nachgeholt — erst dann greift die volle
> H.1-Deaktivierungs-Voraussetzung nach ADR-0004 §6.

**Spec:** `stack-master/shared/architecture-decisions/0004-edge-secret-backup.md`
**Quality-Gates:** G2.2 (keine Klartext-Secrets), G2.6 (Backup getestet, Restore-Beleg)

---

## 1. Geltungsbereich (Edge-Klasse)

| Secret | Quelle am Edge | Ablage im Repo |
|---|---|---|
| Zigbee-Network-Key + Coordinator-Backup | Z2M-Datenverzeichnis: `coordinator_backup.json` | `edge-secrets/zigbee2mqtt/coordinator_backup.json.enc` |
| Z2M-Addon-Konfiguration (inkl. `advanced.network_key`, `frontend.auth_token`) | Z2M `configuration.yaml` | `edge-secrets/zigbee2mqtt/configuration.yaml.enc` |
| HA `secrets.yaml` | HAOS `/config/secrets.yaml` | `edge-secrets/homeassistant/secrets.yaml.enc` |
| HA Long-Lived Access Tokens / Supervisor-Token | Export als `tokens.json` | `edge-secrets/homeassistant/tokens.json.enc` |
| Node-RED Credentials-Store | Addon-Datenverzeichnis: `flows_cred.json` | `edge-secrets/nodered/flows_cred.json.enc` |
| Mosquitto-Credentials + ACL | `mosquitto/passwd`, `mosquitto/acl` | `edge-secrets/mosquitto/passwd.enc`, `acl.enc` |
| CCU/RaspberryMatic-Sicherung | Sicherungs-Archiv `.tar.gz` | `edge-secrets/ccu/<name>.tar.gz.enc` |

## 2. Voraussetzungen

- [ ] Die age-Recipients sind erzeugt und eingetragen (Interim: 2, siehe
      Kasten oben) — **Operator-Akt**, Schritt-für-Schritt:
      [`operator-checklist-adr-0004.md`](operator-checklist-adr-0004.md).
      Ohne sie bricht `encrypt.sh` fail-closed ab.
- [ ] `sops` und `age` sind auf dem Arbeitshost installiert
      (sops >= 3.9 empfohlen; `encrypt.sh` hat einen Fallback für ältere
      Versionen).
- [ ] gitleaks-pre-commit-Hook aktiv: `scripts/install-hooks.sh`
      (fail-closed; Stop-Gap bis zur CI-Instanz, ADR-0004 §5 / S19).

## 3. Backup-Pfad

Für jede Datei der Edge-Klasse:

```sh
# 1) Klartext vom Edge holen (Beispiel Coordinator-Backup, via SSH/Samba
#    in ein Verzeichnis AUSSERHALB des Repos, z. B. /tmp):
scp <edge-host>:/path/to/coordinator_backup.json /tmp/

# 2) Verschlüsseln (prüft Recipients, verifiziert das Chiffrat sofort):
scripts/edge-secrets/encrypt.sh /tmp/coordinator_backup.json \
    edge-secrets/zigbee2mqtt/coordinator_backup.json.enc

# 3) Gesamt-Verifikation + Commit:
scripts/edge-secrets/verify.sh
git add edge-secrets/zigbee2mqtt/coordinator_backup.json.enc
git commit   # pre-commit-Hook läuft gitleaks

# 4) Klartext sicher löschen:
shred -u /tmp/coordinator_backup.json
```

Regeln:

- Klartext **niemals** in den Working Tree legen (`.gitignore` sperrt die
  Pfade zusätzlich; `decrypt.sh` verweigert Repo-Ziele).
- Nach jeder Änderung am Edge (neuer Token, Passwort-Wechsel, Re-Pairing)
  das betroffene Backup erneuern.
- **Erst-Backup-Beleg** (ADR-0004 Nachtrag 1, Auflage b): auch das
  Erst-Backup wird als destilliertes Protokoll im Beleg-Format nach §6
  dokumentiert (ohne die Drill-spezifischen Felder RTO/Recipient-Pfade) —
  es ist der Materialschluss des irreversiblen Mesh-Key-Risikos und muss
  nachvollziehbar sein.

## 4. Off-Site (zweite Schicht)

Ziel laut ADR-0004 §2: die sops-Chiffrate zusätzlich in das Cockpit-weite
`restic-s3`-Backup aufnehmen (Defense-in-Depth: age-Schicht + restic-Schicht).

**Status 2026-07-25:** Das restic-s3-Target ist Gegenstand des Folge-ADR
„Backup-Target-Wahl" (ADR-0004 §Folgeentscheidungen) und noch nicht
provisioniert. Bis dahin gilt: Repo-Remote (verschlüsseltes Chiffrat) ist
die einzige Off-Site-Kopie — dokumentierte Lücke, kein G2.6-Vollpass.

## 5. Restore-Pfad

Gleichwertige Wege (je Recipient; DR-Token erst nach Nachrüstung,
Interim-Kasten oben):

| Recipient | Privatkey | Vorgehen |
|---|---|---|
| Edge-Host | HAOS `/etc/inventory/age.key` | `decrypt.sh` findet den Key automatisch |
| Backup-Operator | VPS-Host `/etc/backup/age.key` | `decrypt.sh` findet den Key automatisch |
| DR-Hardware-Token *(ab Nachrüstung, bis 2026-08-15)* | YubiKey (offline verwahrt) | `age-plugin-yubikey` im PATH; `SOPS_AGE_KEY_FILE` auf die Token-Identity-Datei setzen |

```sh
# Klartext direkt weiterverarbeiten (bevorzugt, nichts landet auf Platte):
scripts/edge-secrets/decrypt.sh edge-secrets/homeassistant/secrets.yaml.enc | less

# Oder in eine Datei AUSSERHALB des Repos:
scripts/edge-secrets/decrypt.sh \
    edge-secrets/zigbee2mqtt/coordinator_backup.json.enc \
    /tmp/coordinator_backup.json
# ... verwenden, dann:
shred -u /tmp/coordinator_backup.json
```

Für den Coordinator-Tausch gilt zusätzlich das Runbook
[`coordinator-replacement.md`](coordinator-replacement.md) — Network-Key
**vor** der Erstverbindung des neuen Coordinators wiederherstellen.

## 6. Restore-Drill (G2.6-Pflicht)

- **Frequenz:** vierteljährlich; zusätzlich nach jeder Strukturänderung
  des Backup-Pfads.
- **Umfang:** vollständiges Snapshot-Set in eine frische (nicht produktive)
  Umgebung wiederherstellen, sops-Dekryption mit **jedem** eingetragenen
  Recipient-Pfad, nicht nur dem Standard-Pfad.
  **Interim (Nachtrag 2, 2026-07-27):** bis zur DR-Token-Nachrüstung ist
  das ein **2-Pfad-Drill** — P1 Edge-Host, P2 Backup-Operator. P3
  (DR-Token) wird nach dem Token-Setup (Frist 2026-08-15) nachgeholt;
  erst mit vollzogenem P3 greift die volle H.1-Deaktivierungs-
  Voraussetzung nach ADR-0004 §6.
- **Beleg** (ohne ihn ist G2.6 NICHT erfüllt): destilliertes Protokoll im
  Format des Cockpit-Standards
  `stack-master/shared/standards/wiederkehrende-verifikation.md` §3, als
  `stack-master/shared/audit-log/YYYY-MM-DD-ha-automation-restore-drill.md`
  mit:
  - Datum (UTC) und Ausführendem (Operator),
  - geprüftem Commit-Stand (Runbook + `edge-secrets/`-Chiffrate:
    Commit-Hash),
  - Kommando-Kette (was wurde in welcher Reihenfolge ausgeführt),
  - Ergebnis **PASS/FAIL je Prüfpunkt** — je Recipient-Pfad
    (Edge / Operator / DR-Token) ein eigener Prüfpunkt,
  - **Abweichungsliste** (gefundene Drift ist ein dokumentiertes Finding,
    kein Schönheitsfehler),
  - gemessene RTO und Vollständigkeits-Hash (z. B. `sha256sum` über die
    entschlüsselten Dateien, verglichen gegen den Hash beim Backup-Lauf).
- **Zählt NICHT als Beleg** (Standard §3): Doku-Claims ohne Protokoll,
  Erinnerungs-Aussagen sowie **Roh-Output-Dumps** — Restore-Rohausgaben
  können Pfade, Hostnamen und Credentials tragen und gehören weder ins
  Repo noch ins Audit-Log.
- **Terminierung** (Standard §4): nach vollzogenem Drill sofort den
  Folgetermin (+3 Monate, siehe Frequenz oben) mit Owner und
  Verfalls-Folge im `stack-master`-Follow-up-Register eintragen lassen
  (Eintrag erfolgt durch den Orchestrator, nicht durch dieses Projekt).

## 7. Rotation

- **Jährlich (Q1, Kalender-Anker im SRE-Drill):** alle drei age-Recipients
  rotieren — neue Keys erzeugen, Pubkeys unter `edge-secrets/recipients/`
  tauschen, beide `.sops.yaml` aktualisieren, `sops updatekeys` auf alle
  `.enc`-Dateien, Drill auf allen drei Pfaden. (NIST SP 800-57 §5.3.6.)
- **Ereignisbasiert sofort:** Verlust eines Recipient (Token/Host-Key),
  Personalwechsel, Host-Kompromittierung; **Zigbee-Network-Key gesondert**
  nach Verlust eines Mesh-Geräts ohne Wiederbeschaffung (danach
  Mesh-Re-Pairing nach Key-Rotation).
- Rotation der Recipients ≠ Rotation der Inhalte: die Inhalte (z. B. der
  Network-Key selbst) rotieren nur ereignisbasiert.

## 8. Bezug

- Cockpit-ADR-0004 §1 (Krypto-Layer), §2 (Off-Site), §3 (Eskrow),
  §4 (Rotation), §5 (Guardrail), §6 (Drill)
- `edge-secrets/README.md`, `edge-secrets/.sops.yaml`
- Audits 2026-05-16: F-02 (SRE, CRITICAL), R-Z2M-KEY (security, CRITICAL),
  R-SOPS-RECIPIENTS-TODO (MEDIUM)
