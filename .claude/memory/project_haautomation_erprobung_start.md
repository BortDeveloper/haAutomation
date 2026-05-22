---
name: haAutomation Erprobungs-Phase ab 2026-05-19
description: Tool ist public + CI-hardened + Branch-Protection aktiv; User startet Basis-Funktionalitäts-Erprobung; Folge-ADRs (Crate-Rename, KPI-9/10) bewusst pausiert.
type: project
---
**Stand 2026-05-19 abends**: User-Pivot von Iteration auf Erprobung.

## Was als Erprobungs-Basis dient

- `BortDeveloper/haAutomation` ist **public**, gerade frisch durchgehärtet:
  - PR #5 Smoke-CI gemerged, alle 7 Checks grün auf main
  - PR #6 URL-Encoding-Bugfix gemerged
  - PR #7 Security-Hardening gemerged (8 Commits: security.yml, deny.toml,
    audit.toml, dependabot.yml, SECURITY.md, rust-toolchain.toml 1.91,
    Disclaimer DE+EN, Third-Party-Notices)
  - Repo-Settings: Secret-Scanning + Push-Protection + Vulnerability-
    Alerts + Branch-Protection mit 6 Required-Checks aktiv
- Memory `reference_inventory_server.md`: Build-Host ist Tailscale
  `inventory` (<inventory-tailscale-ip>), Workstation baut nicht
- Toolchain laut Repo: rust 1.91 stable
- Konvention Template/Local-Trennung: `.local/env.template` → `.local/.env`
  (gitignored) für konkrete Werte (SERVER, DOMAIN, HA-Token, CCU-URL, etc.)

## Was bewusst pausiert ist (Folge-Iteration)

- **Crate-Rename-Deliberation**: Skelett vorhanden unter
  `shared/deliberations/2026-05-19-haAutomation-crate-rename.md` (draft).
  Erste Anwendung des ADR-0010-Formats — nicht aktiv weitergeführt,
  wartet auf Erprobungs-Erkenntnisse (z. B. ob der publizierte
  Tolnay-`inventory` real in Build-Logs reibt oder nur theoretisch).
- **KPI-9 License Compliance Score + KPI-10 Trademark Hygiene**:
  Nicht gestartet. Folge-ADR nach Erprobung sinnvoll.
- **cargo-deny multiple-versions Warnings**: sichtbar (hashbrown 2×,
  windows-sys 3×, syn 1.x vs 2.x), kein Build-Blocker — bewusst auf
  `"warn"` belassen.
- **Klar-Namen-Restspuren** auf public main: 3 Commits aus
  pre-Email-Privacy-Phase tragen „<operator>" — User-Entscheid
  „akzeptieren" (siehe Memory `project_haautomation_known_releaks.md`).

## Erwartete Erprobungs-Schritte (User auf inventory-Host)

1. Toolchain einmalig: build-essential, libssl-dev, libsqlite3-dev,
   sops, age, rustup install 1.91
2. `git clone https://github.com/BortDeveloper/haAutomation`
3. `cp .local/env.template .local/.env` + reale Werte eintragen
4. `cargo build --release --locked` im `inventory/`-Verzeichnis
5. `cargo test --workspace --locked` — sollte 52/52 grün laufen
6. Erster Sync-Lauf via `target/release/inventory sync ha|ccu|hue|shelly`
7. Web-UI via authgate + serve-Modus

## Eskalations-Triggers während Erprobung

- Build-Failure → Memory-Hinweis (1.91 reicht für edition2024) + ggf.
  apt-Deps nachziehen
- Test-Failure neu (52 sind nominell grün) → Folge-PR
- Crate-Name-Verwirrung in Logs sichtbar → Crate-Rename-Deliberation
  vorziehen
- HA/CCU/Hue/Shelly-API-Errors → spezifisch debuggen, keine
  Tool-Frage

## Erprobungs-Stand 2026-05-20

Auf Tailscale-Inventory-Host (separate IP), CCU `100.79.140.88`:

- **Shelly**: ✅ läuft. 14 Shellys per mDNS gefunden + upserted.
- **CCU**: 🟡 Token-in-URL-Workaround. PR #16 Basic-Auth **funktioniert
  nicht** auf User-CCU — XML-API-Addon hat eigene Token-Auth
  (`session.tcl` prüft `?sid=` unabhängig vom lighttpd-Auth). F-
  Empfehlung aus Deliberation 2026-05-20-subagent-briefing-drift war
  strukturell zu optimistisch.
  Verifizierter Workflow:
  ```bash
  # Token registrieren (einmalig)
  ssh root@<ccu> 'echo "source /www/addons/xmlapi/session.tcl; puts [register_token inventory-sync]" | tclsh'
  # Token persistiert in /etc/config/addons/xmlapi/token.list
  
  # Sync per curl-Workaround (sync ccu --sid noch nicht impl.)
  export CCU_SID='<16-char-token>'
  curl -s "$CCU_URL/addons/xmlapi/devicelist.cgi?sid=$CCU_SID" > ccu.xml
  ```
- **HA**: 🟡 URL-Doppel-Präfix-Bug im User-`local/token`-Script.
  `HA_URL=http://...` wird im Script nochmal mit `http://` prefixiert.
  Fix: `HA_URL` ohne Schema setzen ODER `--url`-CLI direkt. Code OK.
- **Hue**: ✅ Optional (PR #13). `unset HUE_CONFIG` skip; sonst
  `hue.yml` mit Bridge-Liste anlegen.

## Offene Folge-Arbeiten

- `sync ccu --sid` Folge-PR blockt durch Subagent-Refuse-Loop —
  siehe Deliberation `2026-05-20-subagent-briefing-drift.md`
  + Memory `feedback_subagent_briefing_drift_lessons.md`. Blockt
  bis User-Aktion: Briefing-Update auf `ha-automation-dev.md` (A)
  ODER explizit Override.
- HA-URL-Script-Fix beim User selbst (Operator-Wartung)
- F-Re-Evaluation: falls je CCU-Addon-Patch (`session.tcl` um
  lighttpd-Auth-Header-Check erweitern) gewünscht, könnte F
  reanimiert werden — Aufwand mittel

## Token-Handling-Hinweise

- CCU-Token (16-char alphanum) in `/etc/config/addons/xmlapi/token.list`
  persistent — überlebt Reboots
- Revoke: `ssh root@<ccu> 'echo "source /www/addons/xmlapi/session.tcl;
  puts [revoke_token <token>]" | tclsh'`
- Token nicht via CLI-argv (bash-history) — immer ENV `CCU_SID`
- Tailnet-only-Verkehr; CCU nicht öffentlich erreichbar
