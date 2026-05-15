---
name: project-habortfeld
description: "Designentscheidungen fuer das haBortfeld-Projekt — Inventarisierung von HA/CCU/NR + Refactoring. Stack, Constraints, OOS."
metadata: 
  node_type: memory
  type: project
  originSessionId: f0b8800a-334e-4fb1-8ba0-6d7fd3505322
---

Projekt **haBortfeld** (Repo: BortDeveloper/haBortfeld) — Single Source of Truth fuer das Smart Home in Bortfeld. Zwei Hauptziele: (1) automatische Inventarisierung aller Geraete, Firmware, Software-Versionen ueber HA, Homematic CCU, Node-RED und Z2M; (2) Refactoring der ueber alle drei Systeme verteilten Automationslogik gemaess Ownership-Regeln.

**Why:** Logik liegt heute fragmentiert in CCU-Programmen, HA-Automationen und Node-RED-Flows, oft redundant. Aenderungen erfordern Reverse-Engineering ueber alle Schichten. Inventarisierung soll als Fundament + Sicherheitsnetz dienen, bevor refaktoriert wird.

**How to apply:** Bei allen Code- und Infra-Entscheidungen pruefen, ob sie zu diesen Festlegungen passen:

- **Inventory-Backend in Rust**, synchron (`tiny_http`, `ureq`, `rusqlite`, `clap`, `serde_yaml_ng`). Kein tokio/axum/sqlx — der Eigentuemer will den Code lesen koennen. Image-Ziel <30 MB.
- **Datenhaltung:** SQLite + parallel YAML-Snapshots im Repo (YAML = source of truth, SQLite = Cache+Index).
- **VPN-Layer austauschbar:** App selbst kennt kein VPN, joint `network_mode: service:vpn`. Drei Overlays bereits angelegt (Tailscale, NetBird, WireGuard); initialer Deploy laeuft auf **Tailscale**.
- **Auth:** OIDC-Readiness Pflicht. App parsed nur `X-Authentik-Username`-Header, OIDC-Logik lebt in vorgelagertem Caddy `forward_auth`. Solange kein SSO am Deploy-Host verfuegbar ist, fuellt das eigene Sidecar `authgate` (S13d, `src/bin/authgate.rs`) diesen `forward_auth`-Slot — Login-Formular + HMAC-signiertes Session-Cookie. S14 ersetzt es durch die bestehende Authentik-Instanz; der Header-Vertrag bleibt gleich.
- **Secrets:** sops + age. age-Privatkey nur auf Strato-Host unter `/etc/inventory/age.key` (chmod 400, root:root), niemals im Repo. Entschluesselung passiert zur Container-Boot-Zeit, Klartext landet auf tmpfs (`/run/inventory/…`), nie auf Disk.
- **Ownership-Regeln fuer Refactoring** (siehe [[project-habortfeld-home]]):
  - **CCU**: nur Direktverknuepfungen, latenzkritisch, ausfallsicher (z.B. Taster→Licht)
  - **HA**: Geraete-State, UI, Szenen, 1:1-Automationen
  - **Node-RED**: alles mit Verzweigungen, Timern, externen APIs, Notifications
  - Doppelte Logik in zwei Schichten = Bug.
- **Out of Scope V1:** schreibender Pfad vom Inventory zurueck nach HA/CCU, mobile UI, Regel-Engine im Inventory, Multi-Tenant-Auth.
- **Vorgehen:** inkrementell in Steps. Jeder Step ein PR mit reproduzierbarem Test-Gate. Roadmap unter `docs/roadmap.md` im Repo. Status 2026-05-15: **Phase 1 + 2 + 3 abgeschlossen** (S1–S12b), 52 cargo-Tests gruen. Library fuer alle vier Sync-Quellen (HA, CCU, Hue multi-bridge, Shelly mDNS) fertig, YAML-Export + git-publish-Loop geschlossen. Real-System-Smoke (echte HA/CCU/Hue/Shellys) ist als gesammelter Live-Lauf am Ende von Phase 3 vorgesehen — bewusst vertagt, damit alle Quellen gleichzeitig getestet werden. Naechste Stufe: Phase 4 (S13 Compose-VPN-Stack, S13a/b/c Overlays, S13d `authgate`-Sidecar, S14 Caddy+Authentik). Deploy- und Build-Host ist der dedizierte Tailscale-Server `inventory` (siehe [[reference-inventory-server]]).
- **Brand-spezifische Erweiterungen** ueber HA-Sync hinaus: Hue (multi-bridge, v1 REST, liefert swversion fuer FW-Tracking) und Shelly (mDNS-Discovery + Gen1/Gen2 HTTP) sind eigene Sync-Module, weil die HA-API keine Firmware-Versionen exposed.
- **Sprache:** Code englisch, Doku deutsch.
