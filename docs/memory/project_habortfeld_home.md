---
name: project-habortfeld-home
description: "Physische Umgebung des haBortfeld-Setups — HA-Installation, CCU-Modell, Node-RED-Topologie, Z2M, VPN-Backbone."
metadata: 
  node_type: memory
  type: project
  originSessionId: f0b8800a-334e-4fb1-8ba0-6d7fd3505322
---

Smart-Home-Setup in Bortfeld, Stand 2026-05-14:

- **Home Assistant**: laeuft auf **Home Assistant OS (HAOS)**. Supervisor verfuegbar → Sync-Pfade nutzen Supervisor-API wo moeglich; Snapshot-Backups via `hassio backups`. Long-Lived Access Token noch zu erstellen, sobald S10 ansteht.
- **Homematic**: **RaspberryMatic** auf eigenem Host. XML-API-Addon installierbar / wahrscheinlich schon da — liefert `devicelist.cgi`, `statelist.cgi`, `programlist.cgi`. Backup via `xmlapi` oder regulaerer CCU-Sicherung.
- **Node-RED**: als **HA-Addon** unter Supervisor (kein eigener Host). Flows unter `/addon_configs/<addon-slug>/flows.json`. Admin-API von extern via Supervisor-Ingress, Long-Lived-Token oder Supervisor-Token.
- **Zigbee**: **Zigbee2MQTT als HA-Addon**, MQTT-Broker ist `core-mosquitto` (ebenfalls Addon). Topic-Prefix `zigbee2mqtt`. Devices und Firmware via Topic `zigbee2mqtt/bridge/devices` (retained).
- **VPN-Backbone Strato → Heimnetz**: initialer Deploy laeuft auf **Tailscale** (Free-Tier). NetBird (SaaS/self-hosted) und WireGuard sind als Compose-Overlays vorbereitet, aber nicht aktiviert.
- **Authentik-Instanz**: existiert bereits (URL noch zu erfragen), wird fuer Inventory-UI als weitere Application + Outpost-Provider konfiguriert — kein Neu-Setup.
- **Domain**: bestehende Domain mit zu vergebender Subdomain fuer Inventory-UI. A-Record auf Strato-IP. Konkretes Subdomain-Label steht noch aus (kommt mit S14).
- **Sekundaerer Server fuer Inventory-Hosting**: Strato-VPS mit Tag `ansible-strato-stack`. Mit Heim-Netz ueber VPN verbunden. SSH-Auth zu diesem Server via deploy-key `ansible-strato-stack`, der lokal als `id_rsa` (RSA-4096) unter `C:\Users\guebr\.ssh\` liegt.

**Why hier festgehalten:** Diese Setup-Fakten beschreiben den realen Zustand der Umgebung — sind aus dem Repo allein NICHT ableitbar (z.B. dass Z2M ueberhaupt im Einsatz ist) und entscheiden ueber Sync-Pfade, Auth-Mechanik und Backup-Strategie.

**How to apply:** Bei Fragen zu Endpoints/Pfaden ohne erneutes Nachfragen diese Werte annehmen. Aber: kurz die *aktuelle* Erreichbarkeit/Existenz pruefen (z.B. mit `gh`/`curl`), bevor man auf Basis dieser Fakten Code ausfuehrt — Setups veraltern.
