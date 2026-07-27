# Edge-Secret Recipients (age Public Keys)

Dieses Verzeichnis hält die **age Public Keys** der drei Recipients, gegen die
jede Datei der Edge-Klasse verschlüsselt wird (Multi-Recipient-Eskrow,
Cockpit-ADR-0004 / Closing-Brief §2.2).

Public Keys sind **nicht geheim** und gehören versioniert ins Repo. Private
Keys gehören **niemals** hierher — sie liegen auf den jeweiligen Hosts bzw.
dem Hardware-Token.

## Die drei Recipients (n = 3, verbindlich)

| # | Recipient | Privatkey liegt auf | Pubkey-Datei (Konvention) |
|---|---|---|---|
| 1 | Edge-Host | HAOS, `/etc/inventory/age.key` (0400 root:root) | `edge-host-<YYYY-MM-DD>.pub` |
| 2 | Backup-Operator | VPS-Stack-Host, `/etc/backup/age.key` | `backup-operator-<YYYY-MM-DD>.pub` |
| 3 | DR-Hardware-Token | offline YubiKey/PIV, physisch off-site verwahrt | `dr-token-<YYYY-MM-DD>.pub` |

Dateiname: `<träger>-<YYYY-MM-DD>.pub`, eine Zeile `age1...`
(beim DR-Token: `age1yubikey1...`).

## Status (2026-07-27) — Interim 2 Recipients

Hinterlegt sind **2 von 3** Recipients (ADR-0004 Nachtrag 2, 2026-07-27):

- `edge-host-2026-07-27.pub`
- `backup-operator-2026-07-27.pub`

Der DR-Hardware-Token-Recipient (`dr-token-<datum>.pub`) wird bis
**2026-08-15** nachgerüstet (YubiKey-Konsens besteht, nur verschoben) —
Schritt-für-Schritt:
[`docs/runbooks/operator-checklist-adr-0004.md`](../../docs/runbooks/operator-checklist-adr-0004.md).
Bis dahin prüfen `encrypt.sh`/`verify.sh` auf genau 2 Recipients.
Keine Platzhalter-Keys.

## Workflow nach Bereitstellung

1. Auf jedem Host/Token den Key erzeugen, **nur den Public-Teil** als
   `<träger>-<YYYY-MM-DD>.pub` hierher committen.
2. Die drei `age1...`-Werte in **beiden** sops-Konfigurationen unter **allen**
   `creation_rules` eintragen: `edge-secrets/.sops.yaml` (Edge-Klasse,
   maßgeblich) und `home-inventory/secrets/.sops.yaml` (Inventory-Betriebssecrets).
3. `sops updatekeys` auf alle `.enc`-Dateien anwenden.
4. `scripts/edge-secrets/verify.sh` muss `OK` melden.
5. PR mit dem sichtbaren Diff.

Eine Reduktion auf weniger als drei Recipients ist nur über ein Re-Open bzw.
einen Nachtrag zu Cockpit-ADR-0004 zulässig, nicht einseitig — der aktuelle
Interim-2-Modus ist durch Nachtrag 2 (2026-07-27) gedeckt.
