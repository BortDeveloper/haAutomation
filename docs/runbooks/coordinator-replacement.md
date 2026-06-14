# Runbook: Zigbee-Coordinator austauschen / neu aufsetzen

**Zweck:** Ersatz eines defekten oder zu wechselnden Zigbee-Coordinators
(USB-Stick) **ohne** Neuanlernen aller Geräte.

**Geltungsbereich:** Zigbee2MQTT als HA-Addon, MQTT-Broker `core-mosquitto`.

> ⚠️ **Kernregel:** Der Zigbee-Network-Key (samt PAN-ID und Extended PAN-ID)
> muss aus dem sops-Backup **wiederhergestellt werden, BEVOR** der neue
> Coordinator das erste Mal mit dem Netz verbunden wird. Wird er mit einem
> frischen, zufälligen Network-Key gestartet, bildet er ein **neues**
> Zigbee-Netz — alle 20+ Geräte müssten einzeln neu angelernt werden
> (Mesh-Re-Pairing-Katastrophe).

## Voraussetzungen

- [ ] Das verschlüsselte Coordinator-Backup liegt vor:
      `coordinator_backup.json.enc` (sops, Edge-Klasse — siehe
      `home-inventory/secrets/.sops.yaml`).
- [ ] age-Privatkey eines der drei Recipients ist verfügbar
      (Edge-Host, Backup-Operator oder DR-Hardware-Token —
      `edge-secrets/recipients/README.md`).
- [ ] Ersatz-Coordinator ist beschafft und **stromlos / nicht eingesteckt**.
- [ ] Zugriff auf HA Supervisor (Addon `Zigbee2MQTT`).

## Schritt 1 — Zigbee2MQTT stoppen

1. HA → Einstellungen → Add-ons → **Zigbee2MQTT** → **Stoppen**.
2. Warten bis der Addon-Status `stopped` ist. Solange das Addon läuft, darf
   der Coordinator nicht getauscht werden.

## Schritt 2 — Network-Key aus dem sops-Backup wiederherstellen  ⚠️ PFLICHT

**Dieser Schritt steht VOR dem Einstecken des neuen Coordinators.**

1. Backup entschlüsseln (auf einem Host mit verfügbarem age-Key):

   ```sh
   sops -d coordinator_backup.json.enc > /tmp/coordinator_backup.json
   ```

2. Die drei Identitätswerte aus dem Backup ablesen und festhalten:
   `network_key`, `pan_id`, `ext_pan_id` (im JSON unter `network_key` bzw.
   den Coordinator-Parametern).
3. In der Zigbee2MQTT-Addon-Konfiguration (`configuration.yaml`) sicherstellen,
   dass `advanced:` exakt diese Werte trägt:

   ```yaml
   advanced:
     network_key: [<aus Backup>]
     pan_id: <aus Backup>
     ext_pan_id: [<aus Backup>]
   ```

4. Alternativ — falls die Z2M-Version den Restore unterstützt: die
   entschlüsselte `coordinator_backup.json` als `coordinator_backup.json` in
   das Z2M-Datenverzeichnis legen, damit Z2M sie beim Start auf den neuen
   Coordinator zurückschreibt.
5. Klartextkopie wieder entfernen:

   ```sh
   shred -u /tmp/coordinator_backup.json   # bzw. rm
   ```

## Schritt 3 — Coordinator physisch tauschen

1. Alten Coordinator abziehen.
2. Neuen Coordinator einstecken. Den Gerätepfad prüfen
   (`/dev/serial/by-id/...`) und in der Z2M-Konfiguration `serial.port`
   anpassen, falls er sich geändert hat.

## Schritt 4 — Zigbee2MQTT starten und verifizieren

1. Addon **Zigbee2MQTT** starten.
2. Im Addon-Log prüfen: der Coordinator meldet sich mit der **erwarteten
   PAN-ID** — nicht mit einer neuen.
3. In der Z2M-Weboberfläche (Map) prüfen, dass die bekannten Geräte ohne
   Neuanlernen wieder erscheinen (Router-Geräte zuerst, batteriebetriebene
   Sensoren melden sich beim nächsten Aufwachen).
4. Eine Geräteaktion testen (z.B. ein Licht schalten).

## Schritt 5 — Frisches Backup ziehen

Nach erfolgreichem Tausch ein neues `coordinator_backup.json` erzeugen,
gegen die drei Recipients verschlüsseln und committen:

```sh
sops -e coordinator_backup.json > coordinator_backup.json.enc
git add coordinator_backup.json.enc && git commit -m "chore: coordinator-backup nach Austausch"
```

## Wenn doch ein neues Netz entstanden ist

Erscheint nach Schritt 4 eine **neue PAN-ID** und sind die Geräte weg:
Addon stoppen, Schritt 2 erneut sorgfältig durchführen (Werte exakt prüfen),
Coordinator neu starten. Erst wenn der Restore nachweislich fehlschlägt,
bleibt das Neuanlernen aller Geräte als letzte Option.

## Bezug

- Closing-Brief §2.6 (Re-Provisioning-Runbook, Pflichtschritt Network-Key-Restore)
- Security-Audit Finding R-Z2M-KEY (CRITICAL) — gepflegt extern
- `home-inventory/secrets/.sops.yaml`, `edge-secrets/recipients/README.md`
