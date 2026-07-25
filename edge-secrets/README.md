# edge-secrets — verschlüsseltes Edge-Secret-Backup (Cockpit-ADR-0004)

Dieses Verzeichnis hält die **sops/age-verschlüsselten** Backups der
Edge-Secrets (Zigbee, Home Assistant, Mosquitto, Node-RED, CCU). Hier liegt
**ausschließlich Chiffrat** (`*.enc`) plus Konfiguration und Public Keys —
niemals Klartext, niemals private Schlüssel.

## Verzeichnis-Layout

```text
edge-secrets/
├── .sops.yaml          # creation_rules der Edge-Klasse (SSOT)
├── recipients/         # age PUBLIC Keys der 3 Eskrow-Recipients
├── zigbee2mqtt/        # coordinator_backup.json.enc, configuration.yaml.enc
├── homeassistant/      # secrets.yaml.enc, tokens.json.enc (HA LLAT)
├── nodered/            # flows_cred.json.enc
├── mosquitto/          # passwd.enc, acl.enc
└── ccu/                # RaspberryMatic-Sicherungs-Archiv (*.tar.gz.enc)
```

Die Komponenten-Verzeichnisse entstehen mit dem ersten Backup-Lauf
(Git trackt keine leeren Verzeichnisse).

## Regeln (verbindlich)

1. **Nur `.enc`-Dateien** werden hier committet. Die Klartext-Pfade der
   Edge-Klasse sind in `.gitignore` gesperrt (deny-then-allow-Pattern).
2. **Genau 3 age-Recipients** pro Datei (Edge-Host, Backup-Operator,
   DR-Hardware-Token) — Multi-Recipient-Eskrow, ADR-0004 §3.
3. Verschlüsseln/Entschlüsseln/Prüfen ausschließlich über die Scripts:
   - `scripts/edge-secrets/encrypt.sh` — Klartext → Chiffrat (fail-closed
     ohne eingetragene Recipients)
   - `scripts/edge-secrets/decrypt.sh` — Restore-Pfad (verweigert
     Klartext-Ziele innerhalb des Repos)
   - `scripts/edge-secrets/verify.sh` — Chiffrat-Disziplin prüfen, ohne
     private Schlüssel lauffähig
4. **gitleaks-Guardrail**: pre-commit-Hook aktivieren mit
   `scripts/install-hooks.sh` (fail-closed; `.gitleaks.toml` trägt die
   Marker-Regeln der Edge-Klasse).

## Doku

- Runbook Backup/Restore/Drill: `docs/runbooks/edge-secret-backup.md`
- Operator-Checkliste Schlüssel-Erzeugung: `docs/runbooks/operator-checklist-adr-0004.md`
- Coordinator-Tausch: `docs/runbooks/coordinator-replacement.md`
- Spec: `stack-master/shared/architecture-decisions/0004-edge-secret-backup.md`
