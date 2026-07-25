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

## Status (2026-07-25)

**Noch keine Keys hinterlegt** (R-SOPS-RECIPIENTS-TODO). Die Erzeugung ist
ein **Operator-Akt** mit Host-/Hardware-Zugriff — Schritt-für-Schritt:
[`docs/runbooks/operator-checklist-adr-0004.md`](../../docs/runbooks/operator-checklist-adr-0004.md).
Bis dahin ist die Pipeline bewusst fail-closed
(`scripts/edge-secrets/encrypt.sh` bricht ab). Keine Platzhalter-Keys.

## Workflow nach Bereitstellung

1. Auf jedem Host/Token den Key erzeugen, **nur den Public-Teil** als
   `<träger>-<YYYY-MM-DD>.pub` hierher committen.
2. Die drei `age1...`-Werte in **beiden** sops-Konfigurationen unter **allen**
   `creation_rules` eintragen: `edge-secrets/.sops.yaml` (Edge-Klasse,
   maßgeblich) und `home-inventory/secrets/.sops.yaml` (Inventory-Betriebssecrets).
3. `sops updatekeys` auf alle `.enc`-Dateien anwenden.
4. `scripts/edge-secrets/verify.sh` muss `OK` melden.
5. PR mit dem sichtbaren Diff.

Eine Reduktion auf weniger als drei Recipients ist nur über ein Re-Open von
Cockpit-ADR-0004 zulässig, nicht einseitig.
