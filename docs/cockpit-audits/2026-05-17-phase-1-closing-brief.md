# Architekturvorgaben: Phase-1-Abschluss `ha-automation`

**Status**: Briefing-Dokument für Iteration 2
**Erstellt**: 2026-05-17 (Orchevpsr)
**Adressat**: `ha-automation-dev`, mit Review durch `architect` + `security` + `sre`
**Quellen-Konsolidierung**: ADR-0004, `shared/audit-log/2026-05-16-ha-automation-{architect,security,sre,docs}.md`, `config/quality-gates.yaml`, `shared/target-state.md`

> Dieses Dokument bündelt die *harten* Architekturvorgaben, deren Erfüllung
> notwendig ist, damit `ha-automation` formal von Phase 1 (AUDIT) nach
> Phase 2 (HARDENING) fortgeschrieben werden kann. Es ist kein ADR – die
> ADRs sind referenziert. Es ersetzt nicht die Audit-Reports – diese sind
> Quelle der Vorgaben.

## 1. Status Quo (warum Phase 1 noch nicht abgeschlossen ist)

| Gate | Stand | Notiz |
|---|---|---|
| G1.1 (Audits pro Experte) | ✅ | 4 Reports vom 2026-05-16 |
| G1.2 (CRITICAL dokumentiert + priorisiert) | ✅ | F-02 / R-Z2M-KEY → ADR-0004 |
| G1.3 (README ≤ 14 d) | ✅ | |
| G1.4 (Architektur-Skizze) | ✅ | `docs/architecture.md` |
| G1.5 (Bias-Reflexion) | ✅ | in allen 4 Reports |
| **H.1 (STOPP, Cross-Phase)** | 🟥 **AKTIV** | Aufhebung erst nach geprüfter ADR-0004-Umsetzung |

**Kernaussage**: G1.x sind erfüllt, **H.1 ist der einzige offene Blocker**.
Solange H.1 aktiv ist, darf weder Phase 2 starten noch eine
Phasenfortschreibung dokumentiert werden (verankert in
`config/quality-gates.yaml` H.1).

## 2. Verbindliche Architekturvorgaben (ADR-0004)

Diese sechs Blöcke sind die formalen Pflicht-Bestandteile. Reihenfolge =
Umsetzungsreihenfolge.

### 2.1 Krypto-Layer (sops/age)

- `.sops.yaml` (im Repo bereits angelegt) **Edge-Scope erweitern** um die
  path_regex aus ADR-0004 §1: `coordinator_backup.json`, `zigbee2mqtt/*.yaml`,
  `*secrets.yaml`, `flows_cred.json`, `mosquitto/{passwd,acl}`, `*tokens.json`.
- **Keine Klartext-Variante** dieser Pfade im Repo. Allowlist-Pattern in
  `.gitignore` analog zum bestehenden `*.env`/`!*.env.example`/`!*.enc`.

### 2.2 Multi-Recipient-Eskrow (n = 3)

Genau drei unabhängige age-Recipients pro sops-Datei der Edge-Klasse:

1. **Edge-Host-Recipient** – Privatkey auf HAOS (`/etc/inventory/age.key`,
   `0400 root:root`).
2. **Backup-Operator-Recipient** – Privatkey auf zentralem `vps-stack`-Host
   (`/etc/backup/age.key`).
3. **DR-Hardware-Token-Recipient** – Offline, YubiKey/PIV. **Physisch getrennt**
   vom Edge aufbewahrt (Tresor anderer Brandabschnitt oder Bankschließfach –
   BSI CON.3.A4 Off-Site-Prinzip).

Die age-**Public**-Keys werden unter
`projects/ha-automation/edge-secrets/recipients/` versioniert (Dateiname:
`<host-oder-träger>-<YYYY-MM-DD>.pub`).

> **2-Recipient-Variante ist verworfen** (ADR-0004 §Alternatives Considered G).
> Reduktion auf 2 Recipients = Re-Open des ADR via User-Eskalation, nicht
> einseitig.

### 2.3 Backup-Target (Off-Site)

- **Primärziel**: gemeinsames `restic-s3` (`config/cockpit.yaml`
  §shared_services.backup) — defense-in-depth zweischichtig:
  sops/age + restic-Repo-Verschlüsselung.
- **Bucket-Härtung** (Anforderungen, finale Provider-Wahl per **Folge-ADR**):
  SSE aktiv (KMS bevorzugt), Versioning + Object-Lock (Compliance-Mode),
  MFA-Delete, separates IAM-Principal Put/List/Get-only, Public-Access
  kategorisch verboten, EU-Sitz oder dokumentierter Switching-Pfad
  (EU Data Act Kap. VI Art. 23–31).
- **3-2-1-Regel**: Sekundär-Off-Site (NAS oder zweiter Provider) wird im
  Folge-ADR konkretisiert. **Kein Phase-1-Blocker**, aber bis Phase-2-Gate G2.6
  vorgesehen.

### 2.4 CI-Guardrail / pre-commit-Hook

- **Werkzeug-Wahl: `gitleaks`** (ADR-0004 §5 festgeschrieben). Konsistenz
  zu vps-stack.
- **Custom-Regeln**: Marker-Pattern für unverschlüsselte Zigbee-Backups
  (`"network_key":`, `"trust_center_link_key":`, `coordinator_ieee:`).
- **Stop-Gap-Verortung**: pre-commit-Hook am Entwicklerplatz – `ha-automation`
  hat aktuell **keine eigene CI-Pipeline** (vermerkt als S19 im SRE-Audit).
- **Pflicht-Migration auf CI** sobald S19 umgesetzt ist (Phase 2). Pre-commit-
  only ist explizit **nicht** der Zielzustand.

### 2.5 Restore-Drill (G2.6-Vorlauf, ADR-0004 §6)

- Restore eines vollständigen Snapshot-Sets in eine **frische, nicht-produktive**
  Umgebung.
- sops-Dekryption über **jeden der drei Recipient-Pfade** durchspielen
  (nicht nur Standard-Pfad).
- Beleg in `shared/audit-log/YYYY-MM-DD-ha-automation-restore-drill.md`:
  Datum, Operator, Recipient, gemessene RTO, Vollständigkeits-Hash.
- Frequenz: vierteljährlich + nach jeder Strukturänderung des Backup-Pfads.

### 2.6 Re-Provisioning-Runbook

- Pfad: `projects/ha-automation/docs/runbooks/coordinator-replacement.md`.
- Pflichtschritt im Runbook: **„Zigbee-Network-Key aus sops-Backup
  wiederherstellen *vor* Coordinator-Neuanbindung"** (verhindert
  Mesh-Re-Pairing-Katastrophe).

## 3. Zusatz-Vorgaben aus Architekt-Audit (HIGH-Findings)

Diese sind formal **nicht** durch G1.x gefordert, aber als **Architektur-Lücke
bei Phase-1-Abschluss** dokumentiert. Erledigung in Iteration 2 zwingend
geplant:

| Ref | Vorgabe | Status |
|---|---|---|
| **R1** | Stack-Drift in `config/cockpit.yaml` auflösen: `tech_stack` ist `[rust, sqlite, docker, caddy, sops-age]` (Inventory-Backend), nicht `[home-assistant, mqtt, zigbee]`. Plus erklärender Kommentar „observiert HA/CCU/Hue/Shelly via VPN". | **User-Aktion** (Schreibrecht `cockpit.yaml`); architect liefert ADR-Entwurf falls gewünscht |
| **R2** | SSO-Wahl: **Aufgelöst durch ADR-0003** (Authentik gewinnt). `authgate` bleibt explizit als dokumentierter Übergang bis S14. | ✅ |
| **R3** | `.sops.yaml`-TODOs schließen (echte VPS-Host-age-Pubkey, mind. eine `.env.enc` als Beispiel). | Teil von §2.1 + §2.2 |
| **R4** | Pinned Image-Tags für alle Compose-Overlays (Tailscale, NetBird, WireGuard, Caddy, sops). Digest-Pin analog `docker-compose.vps.yml`. | **Phase-2-Gate G2.4**, aber Vorlauf in Iteration 2 |

R5–R10 sind MEDIUM/LOW und gehören in den Phase-2-Backlog (kein
Phase-1-Closing-Hindernis).

## 4. Abhängigkeiten und Reihenfolge

```text
   ┌──────────────────────────────────────────────────────┐
   │ 0. USER  – DR-Hardware-Token beschaffen (YubiKey o.ä.)│
   │           Tresor-Standort festlegen, Public-Key       │
   │           generieren                                  │
   └─────────────────────────┬────────────────────────────┘
                             │
                             ▼
   ┌──────────────────────────────────────────────────────┐
   │ 1. ha-automation-dev – .sops.yaml mit 3 Recipients +  │
   │    Edge-Pfade (§2.1, §2.2)                            │
   └─────────────────────────┬────────────────────────────┘
                             ▼
   ┌──────────────────────────────────────────────────────┐
   │ 2. ha-automation-dev – Re-Encryption aller            │
   │    bestehenden .enc-Files gegen die 3 Recipients      │
   └─────────────────────────┬────────────────────────────┘
                             ▼
   ┌──────────────────────────────────────────────────────┐
   │ 3. ha-automation-dev – gitleaks pre-commit-Hook +     │
   │    .gitleaks.toml Custom-Regeln (§2.4)                │
   └─────────────────────────┬────────────────────────────┘
                             ▼
   ┌──────────────────────────────────────────────────────┐
   │ 4. ha-automation-dev – coordinator-replacement.md     │
   │    Runbook (§2.6)                                     │
   └─────────────────────────┬────────────────────────────┘
                             ▼
   ┌──────────────────────────────────────────────────────┐
   │ 5. ha-automation-dev + sre – Restore-Drill,           │
   │    Audit-Log-Eintrag (§2.5)                           │
   └─────────────────────────┬────────────────────────────┘
                             ▼
   ┌──────────────────────────────────────────────────────┐
   │ 6. architect – Verifikations-Review                   │
   │    (Schreibt shared/audit-log/-Bestätigung)           │
   └─────────────────────────┬────────────────────────────┘
                             ▼
   ┌──────────────────────────────────────────────────────┐
   │ 7. orchevpsr – H.1 deaktivieren in current.md,     │
   │    Phase-1 → Phase-2 fortschreiben                    │
   └──────────────────────────────────────────────────────┘
```

**Parallel zur Sequenz** (architect-Tasks ohne Blocker auf User):

- ADR-Entwurf **Folge-ADR Backup-Target-Wahl** (S3-Provider-Bewertungs-Matrix).
- ADR-Entwurf **R1 Stack-Drift cockpit.yaml** (User-Schreibrecht; architect
  liefert Vorschlagstext).
- Repo-internes ADR-Verzeichnis `projects/ha-automation/docs/decisions/`
  anlegen (R6), retroaktive ADRs aus den 6 erkannten Architekturentscheidungen.

## 5. Abnahmekriterien (Phase-1-Closing-Gate)

Phase 1 für `ha-automation` gilt als **abgeschlossen**, wenn alle folgenden
Punkte nachweisbar sind:

- [ ] G1.1–G1.5 ✅ (bereits erfüllt – Stand 2026-05-17)
- [ ] `projects/ha-automation/edge-secrets/recipients/` enthält **3
      Public-Keys** (Edge-Host, Operator, DR-Token), versioniert.
- [ ] `.sops.yaml` enthält **keinen `# TODO`-Eintrag** mehr; alle Edge-Pfade
      aus §2.1 sind als `creation_rules` mit allen 3 Recipients verknüpft.
- [ ] Mindestens eine `.enc`-Datei pro Edge-Pfad-Klasse ist im Repo
      eingecheckt und enthält den sops-Header.
- [ ] `gitleaks`-Hook ist konfiguriert (`.gitleaks.toml` mit Custom-Regeln);
      Probelauf liefert `PASS`.
- [ ] `projects/ha-automation/docs/runbooks/coordinator-replacement.md`
      existiert mit Network-Key-Restore-Schritt vor Coordinator-Neuanbindung.
- [ ] **Erfolgreicher Restore-Drill** dokumentiert in
      `shared/audit-log/YYYY-MM-DD-ha-automation-restore-drill.md` mit allen
      3 Recipient-Pfaden durchgespielt.
- [ ] `architect`-Verifikations-Review-Eintrag in `shared/audit-log/` mit
      explizitem „H.1 darf deaktiviert werden".
- [ ] `shared/milestones/current.md` aktualisiert: Phase-Wechsel
      `ha-automation` → Phase 2.

## 6. Nicht-Phase-1-Vorgaben (zur Klarstellung)

Folgendes ist **Phase 2** (HARDENING) für `ha-automation` und nicht
Voraussetzung für Phase-1-Closing — wird hier nur gelistet, damit
ha-automation-dev kein Scope-Drift vorgaukelt:

- Container-Hardening Standalone-Compose (`cap_drop: ALL`, `read_only`, …)
- Pinned Image-Tags **alle** Overlays (G2.4)
- Metrics-Endpoint + JSON-Lines-Logs (G3.5-Vorlauf)
- Negativ-ADR EU KI-VO + Mapping-ADR EU Data Act
- Lizenz-Inkonsistenz auflösen
- Sekundär-Off-Site Backup für 3-2-1 (G2.6-vollständig)

## 7. Eskalations-Punkte für User

- **User-Aktion erforderlich**: DR-Hardware-Token beschaffen (Schritt 0).
  Ohne diese Hardware kann der gesamte Pfad nicht starten — externe
  Beschaffung war der ausschlaggebende Grund, ha-automation in Iteration 2
  zu verschieben.
- **User-Schreibrecht erforderlich**: `config/cockpit.yaml`-Korrektur
  (R1 Stack-Drift). Architect liefert Patch-Vorschlag, User entscheidet.
- **User-Entscheidung anstehend**: Folge-ADR Backup-Target-Wahl
  (S3-Provider) — bevor §2.3 Bucket-Härtung implementierbar ist.
