---
name: user-role
description: "GitHub-Identitaet, Mail, betriebene Services. Hilft beim Einordnen von Aktionen mit Aussenwirkung (PRs, Deploys)."
metadata: 
  node_type: memory
  type: user
  originSessionId: f0b8800a-334e-4fb1-8ba0-6d7fd3505322
---

GitHub-Identitaet: **BortDeveloper** (Mail: <operator@example.org>). Solo-Betreiber, kein Team.

Betreibt:
- privates Smart Home in <smarthome-location> (HA + Homematic + Node-RED + Zigbee2MQTT), Refactoring + Inventarisierung laufen als Projekt **ha<smarthome-location>** (siehe [[project-habortfeld]])
- VPS mit Tag `ansible-vps-stack` (Ansible-managed) als sekundaere Compute-Plattform, ueber VPN ans Heimnetz angebunden — hostet u.a. das Inventory-Backend

Praeferenzen:
- Code lesbar gegenueber idiomatisch — bevorzugt synchronen Rust-Code ohne tokio und mit minimalen Frameworks, klares "ich will das verstehen koennen"
- Doku auf Deutsch, Code/Identifier auf Englisch
- Bevorzugt explizite Konfigurationsdateien gegenueber Magie (z.B. drei separate Compose-Overlays statt Template-Eval)
- Will Inkrement-Schritte mit reproduzierbarem Test-Gate vor "weiter"
