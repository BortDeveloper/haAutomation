# Test-Umgebung: Reale Hardware

> **Status**: ENTWURF 2026-05-27 (Erstfassung)
> **Zielgruppe**: Cockpit-Maintainer + Folge-Entwickler, die die
> Hardware-Test-Umgebung fuer das `haAutomation`-Repo (Rust-Inventory-
> Backend) neu aufbauen muessen.
> **Scope**: physische Test-Datenquellen — CCU, HomeMatic-Aktoren,
> Shelly-Geraete, Home Assistant, MQTT-Broker. **Nicht** im Scope:
> der Inventory-Backend-Code selbst (vgl. `inventory/` im Repo).
> **Verbindlichkeits-Anker**:
> - BSI IT-Grundschutz **IND.2.1** (IoT-Geraete — Sicherer Betrieb)
>   fuer die Netzwerk-Trennungs-Empfehlung (siehe Sektion 8).
> - ISO/IEC 27001:2022 **A.7.13** (Equipment maintenance) fuer
>   Backup-/Snapshot-Strategie (siehe Sektion 11).
> - `docs/getting-started.md` als logischer Vorgaenger
>   (Software-Sicht); diese Datei ergaenzt die **Hardware-Sicht**.

## Inhaltsverzeichnis

1. [Zweck und Abgrenzung](#1-zweck-und-abgrenzung)
2. [Komponenten-Uebersicht](#2-komponenten-uebersicht)
3. [Hardware-Stueckliste](#3-hardware-stueckliste)
4. [RaspberryMatic-Setup (CCU-Ersatz)](#4-raspberrymatic-setup-ccu-ersatz)
5. [HomeMatic-Aktoren anlernen](#5-homematic-aktoren-anlernen)
6. [Shelly-Setup](#6-shelly-setup)
7. [MQTT-Broker (Mosquitto)](#7-mqtt-broker-mosquitto)
8. [Home Assistant Setup](#8-home-assistant-setup)
9. [Netzwerk-Empfehlungen](#9-netzwerk-empfehlungen)
10. [Verifikations-Checkliste](#10-verifikations-checkliste)
11. [Integration mit dem Rust-Inventory-Backend](#11-integration-mit-dem-rust-inventory-backend)
12. [Reset / Teardown](#12-reset--teardown)
13. [Offene Punkte / TODO](#13-offene-punkte--todo)

---

## 1. Zweck und Abgrenzung

### 1.1 Was diese Anleitung leistet

Sie beschreibt den **kompletten physischen Aufbau** einer Test-
Umgebung, in der das Rust-Inventory-Backend gegen **echte
Geraetequellen** synchronisieren kann:

- **CCU (HomeMatic-Zentrale)** als XML-RPC-/XML-API-Quelle
  (`inventory sync ccu`)
- **Home Assistant** als REST-API-Quelle (`inventory sync ha`)
- **Shelly-Geraete** als HTTP-Quelle Gen1/Gen2 plus mDNS-Discovery
  (`inventory sync shelly`)
- **MQTT-Broker** als Telemetrie-Bus (vorbereitend; aktuell
  ueberwiegend von Home Assistant verbraucht, nicht direkt vom
  Inventory-Backend)

### 1.2 Was diese Anleitung nicht leistet

- Sie ist **kein Beschaffungs-Auftrag**. Die Stueckliste (Sektion 3)
  nennt Bezugsquellen und Preis-Groessenordnungen 2026, aber jede
  Position muss **vor Kauf** verifiziert werden (Lieferbarkeit,
  Firmware-Stand, Preis-Drift).
- Sie ist **kein Security-Audit** der Test-Umgebung. Findings, die
  beim Aufbau auffallen, werden in Sektion 13 als TODO notiert und
  in eine spaetere Security-Sprint-Iteration ueberfuehrt.
- Sie modifiziert **nicht** den Rust-Backend-Code unter
  `inventory/`. Nur Daten-Bereitstellung.

### 1.3 Cockpit-Kontext (warum diese Datei jetzt entsteht)

Das `haAutomation`-Repo ist laut `config/cockpit.yaml` als
`tech_stack: [home-assistant, mqtt, zigbee]` getaggt — das ist
**Drift**. Die Repo-Realitaet ist ein **Rust-Inventory-Backend**,
das HA/CCU/Hue/Shelly via VPN observiert. Die Drift-Klaerung ist
ein Iter-2-Architekten-Task (vgl. `shared/milestones/current.md`).

Bis die Drift formell aufgeloest ist, gilt Repo-Realitaet als
Quelle der Wahrheit. Diese Anleitung **bestaetigt** die Repo-
Realitaet: die Test-Umgebung erzeugt die Datenquellen, **aus
denen** das Backend zieht — nicht eine HA-eigene Konfig-Pipeline.

H.1-STOPP (ADR-0004 Edge-Secret-Backup) ist weiterhin aktiv. Diese
Anleitung **fuehrt keine neuen Secret-Klassen ein** und greift
ADR-0004-Substanz nicht vor. Wo Secrets entstehen (HA Long-Lived
Token, MQTT-Credentials, CCU XML-API-Auth), wird in Sektion 11 auf
den spaeteren `secrets/`-sops-Pfad verwiesen, **ohne** den Pfad
selbst zu aendern.

---

## 2. Komponenten-Uebersicht

```text
                              +----------------------------+
                              |   VPS (im Repo: vps-stack) |
                              |                            |
                              |   inventory (Rust)         |
                              |   - sync ccu               |
                              |   - sync ha                |
                              |   - sync hue               |
                              |   - sync shelly            |
                              +-------------+--------------+
                                            |
                                            | (WireGuard / VPN)
                                            |
+-------------------------------------------v------------------------------------+
|                              Test-Netz (Heim-LAN)                              |
|                                                                                |
|   +----------------------+         +----------------------+                    |
|   |   Raspberry Pi 4     |         |   Raspberry Pi 4     |                    |
|   |   RaspberryMatic     |         |   Home Assistant     |                    |
|   |                      |         |   (HassOS oder       |                    |
|   |   - HM-MOD-RPI-PCB   |         |    Docker-Container) |                    |
|   |   - XML-API-Addon    |         |                      |                    |
|   |   - HmIP-Access-Pt.  |         |   - REST-API :8123   |                    |
|   |                      |         |   - HACS (optional)  |                    |
|   |   XML-RPC :2010/:2001|         |   - MQTT-Add-On      |                    |
|   |   XML-API :80        |         |     Mosquitto :1883  |                    |
|   +----+----------+------+         +------+-------+-------+                    |
|        |          |                       |       |                            |
|        | 868 MHz  | 868 MHz               |       | 802.11 / Wired             |
|        | (HM Funk)| (HmIP)                |       |                            |
|        |          |                       |       |                            |
|        v          v                       |       |                            |
|   +----+----+ +---+------+                |       |                            |
|   | HM-LC-* | | HmIP-PSM |                |       |                            |
|   | (Funk-  | | (Schalt- |                |       |                            |
|   |  Aktor) |  | aktor)  |                |       |                            |
|   +---------+ +----------+                |       |                            |
|                                           |       |                            |
|   +---------------------+                 |       | MQTT                       |
|   | Shelly Plus 1/2PM   |<--Wi-Fi-------->|       |  (Topic: shellies/+/...)   |
|   | Shelly H&T          |<--Wi-Fi---------+-------+                            |
|   +---------------------+                                                      |
|                                                                                |
|   +---------------------+                                                      |
|   | Wi-Fi Access Point  | (idealerweise eigene IoT-SSID + VLAN; siehe Sek. 9) |
|   +---------------------+                                                      |
+--------------------------------------------------------------------------------+
```

### 2.1 Daten-Fluss (knapp)

| Pfad | Protokoll | Konsument                                     |
| ---- | --------- | --------------------------------------------- |
| CCU XML-API (`/addons/xmlapi/devicelist.cgi`) | HTTP        | `inventory sync ccu`                |
| CCU XML-RPC (:2010 HmIP, :2001 BidCos) | XML-RPC     | (perspektivisch, nicht aktiv genutzt) |
| Home Assistant `/api/states`           | HTTPS REST  | `inventory sync ha`                  |
| Shelly Gen1 `/status`, `/settings`     | HTTP        | `inventory sync shelly`              |
| Shelly Gen2 `/rpc/Shelly.GetStatus`    | HTTP RPC    | `inventory sync shelly`              |
| Shelly Telemetry (Gen1+Gen2)           | MQTT        | Home Assistant (UI) — nicht Backend  |
| Mosquitto                              | MQTT 3.1.1  | HA-MQTT-Integration                  |

Quelle der Pfade: `README.md` Sektion „Sync sources" und
`docs/architecture.md`.

### 2.2 Trennung Repo-Welt vs. Test-Welt

| Welt              | Lebt wo                     | Aufgabe                          |
| ----------------- | --------------------------- | -------------------------------- |
| **Backend-Code**  | `inventory/` im Repo        | Rust-Logik, CI, Test-Fixtures    |
| **Test-Fixtures** | `inventory/fixtures/*.xml`  | Offline-Snapshots (CCU XML)      |
| **Test-Umgebung** | physisch im Heim-LAN        | erzeugt **echte** Datenquellen   |

Die Test-Umgebung ist die **Ergaenzung zu** den Fixtures, nicht ihr
Ersatz. Fixtures bleiben fuer CI-deterministische Unit-Tests. Reale
Hardware ist fuer **Integrations-Smoke**, **Firmware-Drift-Tests**
und **Sync-Idempotenz-Verifikation**.

---

## 3. Hardware-Stueckliste

> **Disclaimer**: Preise und Bezugsquellen Stand **2026-05**.
> Vor Kauf verifizieren. „minimal-set" zielt auf ≤500 € Gesamt; das
> „komfortabel"-Set ergaenzt Geraete-Typen, die die Sync-Pfade
> breiter abdecken.

### 3.1 Minimal-Set (≈ 480–500 €)

| Pos | Artikel                                  | Bezug             | Preis (≈)  | Zweck                                  |
| --- | ---------------------------------------- | ----------------- | ---------- | -------------------------------------- |
| 1   | Raspberry Pi 4 Model B, 4 GB             | reichelt, BerryBase, Pi-Shop.ch | 75 € | RaspberryMatic-Host (CCU) |
| 2   | Raspberry Pi 4 Model B, 4 GB             | reichelt, BerryBase             | 75 € | Home-Assistant-Host (HAOS) |
| 3   | Netzteil USB-C 5 V/3 A (RPi-zertifiziert)| 2x                | 2 × 13 €   | Strom RPi                              |
| 4   | microSD-Karte 32 GB A1/A2, Class 10      | 2x (SanDisk Extreme, Samsung Pro Endurance) | 2 × 12 € | Boot-Medium (Endurance bevorzugt) |
| 5   | HM-MOD-RPI-PCB (HomeMatic-Funkmodul)     | ELV, eq-3-Shop    | 35 €       | 868-MHz-Funk fuer CCU                  |
| 6   | HmIP-PSM (Schalt-Steckdose, HmIP)        | eq-3, ELV         | 50 €       | HmIP-Test-Aktor (Schalten + Messen)    |
| 7   | HM-LC-Sw1-PI-3 oder HM-LC-Sw1-FM (Funk-Schaltaktor) | ELV (legacy) | 35–45 € | HomeMatic-Funk-Test-Aktor          |
| 8   | Shelly Plus 1 (Gen2)                     | shelly.com, Reichelt | 18 €    | Shelly-Gen2-HTTP-Test                  |
| 9   | Shelly Plus 1PM (Gen2)                   | shelly.com        | 22 €       | Shelly-Gen2-mit-Powermessung-Test      |
| 10  | Shelly H&T (Gen1)                        | shelly.com        | 35 €       | Shelly-Gen1-HTTP-Test (Sensor)         |
| 11  | RJ45-Kabel 2x, Netzwerk-Switch (5-Port)  | beliebig          | 30 €       | LAN-Anbindung beider RPis              |
| 12  | Verteilersteckdose + Sicherungsklemmen   | Baumarkt          | 25 €       | Stromzufuehrung der Aktoren            |

**Summe Minimal-Set**: ≈ 462 €

### 3.2 Komfortabel-Set (zusaetzlich ≈ 280 €)

| Pos | Artikel                                  | Bezug             | Preis (≈)  | Zweck                                  |
| --- | ---------------------------------------- | ----------------- | ---------- | -------------------------------------- |
| C1  | HmIP-BROLL (Rollladen-Aktor)             | eq-3              | 70 €       | HmIP-Rollladen-Test (Dimm-/Stop-Werte) |
| C2  | HmIP-eTRV-2 (Heizkoerper-Thermostat)     | eq-3              | 60 €       | HmIP-Thermostat-Test (set_point usw.)  |
| C3  | HM-LC-Dim1T-FM (Funk-Dimmer)             | ELV               | 65 €       | HM-Funk-Dimmer-Test                    |
| C4  | Shelly Plus 2PM (Gen2, 2-Kanal)          | shelly.com        | 28 €       | Multi-Kanal-Shelly                     |
| C5  | Shelly i4 (Gen2, Eingangs-Modul)         | shelly.com        | 19 €       | Input-Only-Geraet (Edge-Case)          |
| C6  | Hue Bridge V2 (gebraucht ok)             | ebay, kleinanzeigen | 30 €     | Hue-v1-REST-Test (perspektivisch)      |
| C7  | Hue White Bulb E27 (1 Stueck)            | mediamarkt, amazon | 12 €      | Hue-Test-Geraet                        |

**Summe Komfortabel-Set additiv**: ≈ 284 €

### 3.3 Hinweise zur Bezugsquelle

- **eq-3 / ELV** sind 2026 die einzigen ernsthaften Quellen fuer
  HomeMatic-Funk (alte Linie) und HmIP. Drittanbieter-Klone existieren
  praktisch nicht.
- **shelly.com** liefert direkt aus EU, Reichelt + Conrad fuehren
  ebenfalls.
- **Raspberry Pi 4 4 GB** ist 2026 stabil verfuegbar; **Pi 5** geht
  auch, RaspberryMatic-Image-Support fuer Pi 5 ist seit 2024 stable
  (`verifizieren vor Kauf`: Release-Tag im RaspberryMatic-GitHub-
  Repository auf Pi-5-Eintrag).
- **HM-MOD-RPI-PCB** ist die einzige praktikable Funk-Loesung fuer
  RaspberryMatic. Alternative „RPI-RF-MOD" funktioniert auch, ist
  aber teurer und braucht eigenes Gehaeuse — fuer Test-Umgebung
  ueberdimensioniert.

---

## 4. RaspberryMatic-Setup (CCU-Ersatz)

### 4.1 Varianten-Entscheidung

Der User-Auftrag nennt drei Optionen:

| Variante         | Stand 2026 | Eignung Test-Umgebung |
| ---------------- | ---------- | --------------------- |
| **RaspberryMatic** | aktiv gepflegt, woechentliche Releases, grosse Community | **EMPFEHLUNG** |
| **pivCCU3**      | aktiv, aber Debian-Host-basiert; mehr Konfig-Aufwand | nicht erste Wahl |
| **OCCU-Pure**    | eq-3-Original-OCCU-Repo seit Jahren wenig Bewegung | nicht empfohlen |

**Empfehlung**: **RaspberryMatic**. Begruendung:

1. Aktive Pflege (mehrere Releases pro Jahr), grosse Community,
   gute Backup-/Restore-Werkzeuge.
2. SD-Karten-Image „all-in-one" — Boot-Medium ist sofort lauffaehig,
   keine separate Debian-Installation.
3. XML-API-Add-on (`CCU-Jack` und/oder klassisches `XML-API`-Add-on)
   ist im RaspberryMatic-Repo direkt installierbar.
4. Repo-`fixtures/ccu_devicelist.xml` ist gegen das klassische
   `XML-API`-Format gebaut — RaspberryMatic emuliert das nativ.

### 4.2 SD-Karten-Image bereitstellen

1. Aktuelle Release-Page besuchen:
   `https://github.com/jens-maus/RaspberryMatic/releases`
   (Tag-Namen folgen `<jahr>.<monat>.<patch>`, z. B. `3.79.x`).
2. Image-Datei fuer RPi 4 herunterladen, z. B.:
   `RaspberryMatic-3.79.x.20260201-rpi4.zip`.
3. **Signatur verifizieren** (RaspberryMatic-Releases sind
   SHA256-summiert und teilweise GPG-signiert):
   ```bash
   sha256sum -c RaspberryMatic-3.79.x.20260201-rpi4.zip.sha256
   ```
   Falls eine `.asc`-Datei beiliegt:
   ```bash
   gpg --verify RaspberryMatic-3.79.x.20260201-rpi4.zip.asc \
     RaspberryMatic-3.79.x.20260201-rpi4.zip
   ```
4. Image auspacken und mit **Raspberry Pi Imager** oder
   **BalenaEtcher** auf die microSD schreiben. Custom-Settings
   im Imager (Hostname, WLAN) bei RaspberryMatic **nicht**
   benutzen — das Image bringt eigenes Erst-Konfig-Flow mit.

### 4.3 Hardware-Aufbau

1. **RPi ausschalten** (Netzteil abziehen, nicht heisses Aufstecken).
2. **HM-MOD-RPI-PCB auf die GPIO-Header stecken** (40-Pin-Header,
   Modul sitzt parallel ueber der Platine). Korrekte Orientierung:
   Antenne zeigt vom RPi weg; Beschriftung „HM-MOD-RPI-PCB" lesbar
   nach oben.
3. **microSD einsetzen.**
4. **Ethernet-Kabel** anschliessen (WLAN geht auch, ist fuer
   Funk-Stoerungs-Vermeidung aber nicht ideal — siehe Sektion 9).
5. **Netzteil anschliessen.** Geraet bootet.

### 4.4 Erste-Boot-Sequenz

1. Nach ca. 90 s ist die Web-UI erreichbar. IP-Adresse aus dem
   Router-DHCP-Log lesen (Hostname: `homematic-ccu3` per Default).
2. Browser oeffnen: `https://<ip>/` — Selbstsigniertes Zertifikat
   akzeptieren (Test-Umgebung).
3. **Erst-Setup-Assistent** durchlaufen:
   - Sprache, Zeitzone, NTP-Server (Default: `pool.ntp.org`).
   - **Statisches DHCP** im Router setzen (Reservierung auf
     MAC-Adresse), **keine statische IP im RaspberryMatic** —
     RaspberryMatic-Image-Updates ueberschreiben sonst manchmal
     statische Configs.
   - **Admin-Passwort** setzen (mind. 16 Zeichen, Wuerfelpasswort
     bevorzugt — BSI ORP.4.A8: starke Authentisierung).
4. **Funkmodul aktivieren**: Systemsteuerung → „Funk-Konfiguration"
   → das BidCos-RF-Modul und das HmIP-RF-Modul sollten beide unter
   „HM-MOD-RPI-PCB" erscheinen, Status „aktiv".
5. **XML-API-Add-on installieren**:
   - Systemsteuerung → „Zusatzsoftware" (Addons).
   - „XML-API" Version ≥ 1.20 installieren (im RaspberryMatic-
     Addon-Store direkt waehlbar).
   - Nach Installation: Test-URL `http://<ip>/addons/xmlapi/devicelist.cgi`
     in Browser → liefert XML mit Geraeteliste (anfangs leer, bis
     Aktoren angelernt sind).

### 4.5 Sicherheits-Haertung (Minimal-Niveau Test-Umgebung)

- **SSH abschalten**, wenn nicht aktiv genutzt: Systemsteuerung →
  „Sicherheit" → „Sicherheits-Modus" auf „Sicher" (aktiviert HTTPS-
  Pflicht, deaktiviert telnet/rsh-Bypass).
- **Admin-Passwort** nicht im Klartext im Repo speichern. Spaeter
  in `inventory/secrets/` per sops verschluesselt (Anker: ADR-0004,
  noch nicht implementiert — vorerst lokal im Passwort-Manager).
- **Backup-Strategie**: Systemsteuerung → „Sicherheit" → „Backup"
  → woechentlicher Auto-Export auf USB-Stick **oder** auf
  HA-NAS-Share. RaspberryMatic-Backup ist `tar.gz`-Archiv inkl.
  Geraete-Datenbank — Restore ist ein einzelner Upload-Schritt.

Anker: **ISO/IEC 27001:2022 A.7.13** (Equipment maintenance) —
Hardware-bezogene Wiederherstellbarkeit ist Pflicht. Test-Backups
bekommen denselben Backup-Pfad wie spaetere produktive Stuecke; das
spart spaeter den Wechsel.

### 4.6 Wichtige RaspberryMatic-Pfade fuer das Inventory-Backend

| Endpunkt                                  | Methode  | Verwendet von               |
| ----------------------------------------- | -------- | --------------------------- |
| `http://<ccu>/addons/xmlapi/devicelist.cgi` | GET    | `inventory sync ccu`        |
| `http://<ccu>/addons/xmlapi/statelist.cgi` | GET    | (Future: Zustandsabfragen)  |
| `http://<ccu>:2010/` (HmIP XML-RPC)       | XML-RPC | (perspektivisch)            |
| `http://<ccu>:2001/` (BidCos XML-RPC)     | XML-RPC | (perspektivisch)            |

Der Backend-Sync nutzt aktuell ausschliesslich die XML-API
(`devicelist.cgi`) — siehe `inventory/fixtures/ccu_devicelist.xml`
als Format-Referenz.

---

## 5. HomeMatic-Aktoren anlernen

### 5.1 Anlern-Prinzipien

HomeMatic kennt zwei inkompatible Funk-Welten:

- **HM-Funk (BidCos-RF)**: Aeltere Geraete, `HM-LC-*`, `HM-RC-*`,
  ohne Internet-Pflicht. Anlern-Modus: Geraet 5–30 s in Anlern-
  Modus, CCU im „Anlern-Modus" parallel.
- **HmIP**: Neuere Geraete, `HmIP-*`. Anlern-Modus ueber QR-Code-
  Scan (oder manuell ueber Geraete-Seriennummer + KEY).

Beide Welten sind in RaspberryMatic gleichzeitig betreibbar, brauchen
aber je einen aktiven RF-Modul-Eintrag (siehe 4.4 Schritt 4).

### 5.2 HM-Funk-Aktor anlernen (Beispiel HM-LC-Sw1-PI-3)

1. **CCU**: Systemsteuerung → „Geraete anlernen" → „BidCos-RF" →
   „Anlern-Modus aktivieren" (60 s Fenster).
2. **Aktor**: Stromversorgung herstellen. Bei den meisten Funk-Aktoren
   reicht das **Einstecken/Anschliessen** — sie sind im ersten Boot
   automatisch im Anlern-Modus fuer 30–60 s.
3. **CCU**: nach ein paar Sekunden erscheint der Aktor in der Liste
   „Neue Geraete". Namen vergeben (z. B. `Test-Schaltaktor-1`),
   „Fertig" druecken.
4. **Verifikation**: Aktor erscheint unter „Geraete" mit Status
   „erreichbar". Schaltbefehl `Ein/Aus` ueber die Web-UI testen.

**Reset eines HM-Funk-Aktors**: 4 s drueckhalten der internen
Anlern-Taste, dann nochmals 4 s — LED blinkt langsam → schnell →
aus. Geraet ist auf Werkseinstellungen zurueck.

### 5.3 HmIP-Aktor anlernen (Beispiel HmIP-PSM)

1. **Geraet** an Steckdose stecken. Blaue LED blinkt (Anlern-Bereit).
2. **CCU**: „Geraete anlernen" → „HmIP" → „mit Seriennummer" oder
   „mit QR-Code".
3. **Seriennummer** + **HmIP-KEY** vom Geraete-Aufkleber abtippen
   (oder QR-Code mit Handy lesen und ueber Web-UI eintippen).
4. CCU verbindet sich, LED am Geraet wird gruen.
5. Namen vergeben, „Fertig".

**Reset eines HmIP-Aktors**: 4 s Taste drueckhalten bis orange LED,
dann erneut 4 s druecken bis gruen-blinkend. Geraet ist zurueck.

### 5.4 Multi-Anlern-Reihenfolge

Wenn mehrere Aktoren gleichzeitig angelernt werden:

1. **Erst HM-Funk-Geraete** (langsamer Funk-Stack, braucht
   ungestoertes 60-s-Fenster).
2. **Dann HmIP-Geraete** (schneller, robuster gegen Stoerung).
3. **Pro Anlernvorgang nur 1 Geraet** in den Anlern-Modus bringen —
   sonst landen mehrere Aktoren als „unbekannt" in der CCU-Liste
   und muessen einzeln zugeordnet werden.

### 5.5 Test-Daten-Set: Minimum + Komfort

Nach dem Anlernen sollte mindestens vorhanden sein:

- **Minimum**: 1 HM-Funk-Schaltaktor + 1 HmIP-Schaltaktor = 2 CCU-
  Geraete. Diese reichen, um den Sync-Pfad `inventory sync ccu`
  end-to-end zu pruefen (XML-API liefert dann 2 `<device>`-Eintraege).
- **Komfort**: + 1 HmIP-Rollladen + 1 HmIP-Heizkoerper-Thermostat
  + 1 HM-Funk-Dimmer = 5 Geraete. Diese decken die wichtigsten
  Channel-Typen ab (`SHUTTER_TRANSMITTER`, `CLIMATECONTROL_*`,
  `DIMMER`).

Die `inventory/fixtures/ccu_devicelist.xml`-Datei kann nach
erfolgreichem Anlern-Vorgang aktualisiert werden:

```bash
curl -o inventory/fixtures/ccu_devicelist.xml \
  "http://<ccu-ip>/addons/xmlapi/devicelist.cgi"
```

(Achtung: dies ist eine Fixture-Aktualisierung — separater
Backend-Commit, gehoert nicht in diese Hardware-Anleitung. Nur
Hinweis, dass die Datei real-getrieben aktualisierbar ist.)

---

## 6. Shelly-Setup

### 6.1 Shelly-Geraete-Generationen

| Generation | Geraete-Beispiele                         | API                    |
| ---------- | ----------------------------------------- | ---------------------- |
| **Gen1**   | Shelly 1, Shelly 1PM, Shelly H&T          | HTTP REST `/status`    |
| **Gen2 / Plus** | Shelly Plus 1, Plus 1PM, Plus 2PM, Plus i4 | JSON-RPC `/rpc/Shelly.GetStatus` |
| **Gen3 / Pro**  | Shelly Pro 1/2/3/4 (Hutschiene)        | JSON-RPC (Gen2-API-kompatibel) |

Das Backend (`inventory sync shelly`) deckt **Gen1 + Gen2** ab.
Gen3 ist API-kompatibel zu Gen2 und funktioniert ohne Code-Aenderung;
ein expliziter Test ist trotzdem sinnvoll.

### 6.2 Erste Wi-Fi-Verbindung

1. **Geraet anschliessen** (Schaltaktoren brauchen 230 V; H&T laeuft
   auf Batterie und „weckt" nur bei Schwellenwert-Ueberschreitung).
2. Geraet oeffnet eigenen Wi-Fi-Access-Point: SSID `shellyplus1-XXXXXX`
   oder `shelly1pm-XXXXXX` (Gen1).
3. Mit Handy oder Laptop in dieses WLAN einbuchen.
4. Browser `http://192.168.33.1/` (Gen1) bzw. `http://192.168.33.1/`
   (Gen2 ebenfalls).
5. **In das eigene Test-WLAN konfigurieren** (siehe Sektion 9 zur
   IoT-SSID-Empfehlung).
6. **Statisches DHCP** im Router setzen (Reservierung auf MAC) —
   Backend-Sync ist auf stabile IPs oder mDNS angewiesen.

### 6.3 OTA-Update

**Vor MQTT-Konfig**: jedes Geraet einmal OTA-aktualisieren.

- **Gen1**: Web-UI → „Settings" → „Firmware Update" → „Update".
- **Gen2**: Web-UI → „Settings" → „Device" → „Firmware Update".

Gen2-Firmware sollte ≥ `1.4.x` (2026 stable) sein. Gen1-Firmware
ist meist ≥ `1.14.x`. **Vor Kauf neuer Gen1-Hardware**: pruefen,
ob das Modell noch Firmware-Updates erhaelt — Shelly hat 2025
einige Gen1-Modelle in den „Maintenance-Only"-Modus geschoben.

### 6.4 MQTT-Konfig (Gen2-Beispiel)

1. Web-UI → „Settings" → „Networking" → „MQTT".
2. **Enable MQTT**: ja.
3. **Server**: `<mqtt-broker-ip>:1883` (siehe Sektion 7 fuer Broker).
4. **Username/Password**: aus dem MQTT-Broker (Sektion 7.3).
5. **Topic-Prefix**: Default `shellyplus1-XXXXXX` — fuer Test-Umgebung
   ok; fuer Produktion siehe Sektion 13 TODO „Topic-Schema
   konsolidieren".
6. **RPC-over-MQTT**: aktivieren (erlaubt HA, RPC-Calls ueber MQTT
   statt HTTP zu schicken — entlastet bei vielen Geraeten).
7. **Save & Reboot**.

Gen1-Geraete haben weniger Optionen, das Prinzip ist identisch.

### 6.5 Backend-relevante HTTP-Endpunkte

| Endpunkt                          | Gen | Verwendet von             |
| --------------------------------- | --- | ------------------------- |
| `GET /status`                     | 1   | `inventory sync shelly`   |
| `GET /settings`                   | 1   | `inventory sync shelly`   |
| `POST /rpc/Shelly.GetStatus`      | 2   | `inventory sync shelly`   |
| `POST /rpc/Shelly.GetDeviceInfo`  | 2   | `inventory sync shelly`   |

Das Backend macht **mDNS-Discovery** ueber das LAN, wenn
`inventory sync shelly --discover-seconds 10` aufgerufen wird —
d. h. Shelly-Geraete muessen im **selben Layer-2-Segment** wie der
VPN-Endpoint sein. Bei VLAN-Trennung (Sektion 9): mDNS-Reflector
auf dem Switch oder Avahi-Bridge konfigurieren, sonst keine
Discovery.

---

## 7. MQTT-Broker (Mosquitto)

### 7.1 Variante: HA-Add-on vs. eigener Container

Empfehlung **fuer die Test-Umgebung**: **HA-Add-on „Mosquitto
broker"**.

- Vorteile: Auto-Konfig, integriertes Anlegen von HA-User per
  HA-User-Login, automatisches TLS-Zertifikat ueber HA-CA.
- Nachteile: Broker stirbt mit HA (single point of failure) —
  fuer Produktion separater Container; fuer Test-Umgebung ok.

Eigener Mosquitto-Container ist sinnvoll, **wenn** der Broker ueber
mehrere HA-Instanzen oder ohne HA laufen soll. Konfig dann in
`docker-compose.yml` mit Volume-Mount fuer `/etc/mosquitto/conf.d/`
und `/mosquitto/data/`.

### 7.2 Setup als HA-Add-on

1. **HA Web-UI** → „Einstellungen" → „Add-ons" → „Add-on-Store".
2. „Mosquitto broker" suchen, „Installieren".
3. **Konfig** (Default ist meist ausreichend):
   ```yaml
   logins: []
   require_certificate: false
   anonymous: false
   ```
4. **Start** → Add-on laeuft auf Port `1883` (MQTT) + `8883`
   (MQTT-TLS, falls aktiviert).
5. **Integration aktivieren**: HA → „Einstellungen" → „Geraete &
   Dienste" → „MQTT" → „Konfigurieren" → Broker `core-mosquitto`,
   Port `1883`. HA legt automatisch einen User „homeassistant" an.

### 7.3 User fuer Shelly-Geraete

Shelly-Geraete brauchen einen eigenen MQTT-User (nicht den HA-User
wiederverwenden — ASVS V4.0.3 V14.2: trenne Service-Accounts).

1. **HA Web-UI** → „Einstellungen" → „Personen" → „Benutzer" →
   „Benutzer hinzufuegen": `shelly-test`, starkes Passwort.
2. Diesen User dem MQTT-Broker bekannt machen — beim HA-Add-on
   geschieht das **automatisch** (jeder HA-User darf MQTT
   benutzen). Bei externem Mosquitto: `mosquitto_passwd`
   manuell.
3. In jedem Shelly: Username/Passwort eintragen (Sektion 6.4).

### 7.4 ACL-Empfehlung (knapp)

Fuer Test-Umgebung reicht der HA-User-Default. Fuer Produktion:

```text
# /etc/mosquitto/acls.conf (externer Broker)
user homeassistant
topic readwrite #

user shelly-test
topic readwrite shellies/#
topic readwrite shellyplus1-+/+
topic readwrite shellyplus1pm-+/+
topic readwrite shelly1pm-+/+
```

Mit dieser ACL kann der `shelly-test`-User nur sein eigenes
Topic-Praefix lesen/schreiben — kein Lateral-Movement ueber
HA-Topics moeglich.

---

## 8. Home Assistant Setup

### 8.1 Varianten-Entscheidung

| Variante         | Eignung Test-Umgebung | Begruendung                              |
| ---------------- | --------------------- | ---------------------------------------- |
| **HassOS** (Image auf RPi) | **EMPFEHLUNG** | Minimal-Overhead, Auto-Update, Add-ons direkt verfuegbar, Backup-Snapshot ein Button |
| **HA Container** (Docker) | OK, wenn ein Docker-Host vorhanden ist | Kein Add-on-Store (Add-ons nur in Supervisor) — MQTT-Broker dann separat |
| **HA Supervised** (Debian + Supervisor) | nicht empfohlen | Hoher Konfig-Aufwand, Supervisor unterstuetzt nur wenige Debian-Setups offiziell |

**Empfehlung**: **HassOS auf einem zweiten RPi 4**. Begruendung:

1. Schnellster Aufbau (Image flashen, 10 min bis Onboarding).
2. **Mosquitto-Add-on** und **HACS** sind ohne Container-Friemelei
   sofort installierbar.
3. **Auto-Backup** zur SD-Karte und/oder auf NAS-Share ist
   integriert (ISO/IEC 27001:2022 A.7.13 — Equipment maintenance).
4. **Snapshot vor Test-Lauf** ist ein Klick — Reset/Teardown
   (Sektion 12) wird damit trivial.

### 8.2 SD-Karten-Image bereitstellen

1. Release-Page: `https://www.home-assistant.io/installation/raspberrypi`
2. Aktuelles `haos_rpi4-64-*.img.xz` herunterladen (2026:
   `haos_rpi4-64-12.x.img.xz` oder neuer).
3. **SHA256 verifizieren** (Hash steht neben dem Download).
4. **Raspberry Pi Imager**: „Other specific-purpose OS" → „Home
   Assistant OS" → automatisches Download + Flash.

### 8.3 Erst-Setup

1. SD einsetzen, Ethernet anschliessen, Strom an.
2. Nach 3–5 min: Browser auf `http://homeassistant.local:8123`
   oder `http://<ip>:8123`.
3. **Onboarding**:
   - User anlegen (starkes Passwort; in Test-Umgebung ok als
     Wuerfelpasswort, fuer Produktion siehe TODO Sektion 13).
   - Standort, Zeitzone (wichtig fuer Sonnen-Trigger).
   - „Geraetesuche" wird einige Shellys per mDNS automatisch
     finden — **bewusst auf „Spaeter" klicken**, wir machen die
     Integrationen kontrolliert (8.5).
4. **Backup-Snapshot sofort nach Onboarding**:
   „Einstellungen" → „System" → „Backups" → „Backup erstellen" →
   Name `baseline-after-onboarding`. Das ist der spaetere
   Restore-Punkt (Sektion 12).

### 8.4 HACS installieren (optional, fuer HmIP-Local-Integration)

HmIP-Geraete koennen ueber drei Wege in HA landen:

1. **HomeMatic (XML-RPC) Integration** (eingebaut) — spricht direkt
   mit RaspberryMatic auf Ports 2010/2001.
2. **HomeMatic-IP-Cloud Integration** (eingebaut) — geht ueber
   eq-3-Cloud. **Privacy-relevant**, siehe Sektion 13 TODO.
3. **HomeMatic-IP-Local Integration via HACS** — geht direkt ueber
   den HmIP-Access-Point oder die CCU **ohne Cloud**. Bevorzugt
   gegenueber Variante 2.

HACS-Installation (fuer Variante 3):

1. SSH ins HA (Add-on „SSH & Web Terminal" installieren, dann
   einloggen).
2. ```bash
   wget -O - https://get.hacs.xyz | bash -
   ```
3. HA neustarten („Einstellungen" → „System" → „Neustart").
4. „Einstellungen" → „Geraete & Dienste" → „Integration hinzufuegen"
   → „HACS" → GitHub-OAuth durchklicken.
5. In HACS → „Integrationen" → „homematicip-local" installieren.

### 8.5 Integrationen aktivieren (kontrolliert)

In dieser Reihenfolge:

1. **HomeMatic** (XML-RPC, eingebaut):
   - „Geraete & Dienste" → „Integration hinzufuegen" → „HomeMatic".
   - „CCU2/CCU3"-Variante.
   - Host: `<raspberrymatic-ip>`, Port: `2001` (BidCos-RF) + `2010`
     (HmIP).
   - Username/Passwort: der RaspberryMatic-Admin (aus 4.4).
   - **Verifikation**: HM-Funk-Aktor erscheint unter „Geraete".
2. **HomeMatic-IP-Local** (via HACS, falls 8.4 installiert):
   - „Integration hinzufuegen" → „homematicip-local".
   - „CCU"-Modus, Host: `<raspberrymatic-ip>`.
   - **Verifikation**: HmIP-PSM erscheint unter „Geraete".
3. **MQTT**:
   - „Integration hinzufuegen" → „MQTT".
   - Broker `core-mosquitto`, Port `1883`, User aus HA-User-DB.
   - **Verifikation**: Shellys mit MQTT-aktiv erscheinen unter
     „MQTT-Geraete".
4. **Shelly Native** (eingebaut, fuer Gen2-Geraete sehr stabil):
   - „Integration hinzufuegen" → „Shelly".
   - Mit Discovery: alle Gen2-Geraete auf demselben LAN tauchen auf,
     einzeln bestaetigen.
   - **Verifikation**: Schalt-Toggle in HA-UI → Geraet reagiert.

### 8.6 Long-Lived Access Token fuer das Backend

Der Rust-Backend-Sync `inventory sync ha` braucht einen
Long-Lived-Token:

1. HA-UI → User-Profil (links unten Avatar) → ganz unten „Long-Lived
   Access Tokens" → „Token erstellen".
2. Name: `inventory-backend-test`.
3. Token **einmalig anzeigen** und **sofort** im Passwort-Manager
   ablegen. Spaeter in `inventory/secrets/` per sops verschluesselt
   (ADR-0004, noch nicht implementiert).
4. **Niemals** den Token in einem Repo-Commit ablegen — gitleaks-
   Hook (ADR-0004, geplant) wuerde das blocken.

Backend-Config-Beispiel (Repo-Pfad `inventory/test-setup.env.example`
ist die Soll-Vorlage):

```ini
INVENTORY_HA_URL=https://homeassistant.example.local:8123
INVENTORY_HA_TOKEN=<long-lived-token>
INVENTORY_CCU_URL=http://<raspberrymatic-ip>
INVENTORY_SHELLY_DISCOVER_SECONDS=10
```

---

## 9. Netzwerk-Empfehlungen

### 9.1 Pflicht-Niveau (Test-Umgebung)

- **Statisches DHCP** (Router-Reservierung) fuer:
  - RaspberryMatic-CCU
  - Home-Assistant-RPi
  - alle Shelly-Geraete (sonst Backend-Sync-Drift bei IP-Wechsel)
- **mDNS funktioniert** im LAN — meistens out-of-the-box, bei
  manchen Routern (FritzBox: ok; UniFi: separat aktivieren).

### 9.2 Empfohlen (BSI IT-Grundschutz IND.2.1)

**BSI IT-Grundschutz IND.2.1.A1 + A2** verlangen Netzwerk-Trennung
fuer IoT-Geraete. Praktisch:

- **Eigene IoT-SSID** (z. B. `iot-test`) mit eigenem VLAN (z. B.
  `vlan10`).
- **Firewall-Regel**: IoT-VLAN darf **nicht** ins Main-VLAN
  initiieren; Main-VLAN darf in IoT (fuer HA-Zugriff auf Shellys);
  IoT darf raus ins Internet **nur** fuer Firmware-Updates (besser:
  gar nicht — Update-Routing ueber Proxy / nur manuell).
- **mDNS-Reflector** zwischen VLANs (`avahi-reflector` oder
  UniFi-Builtin), damit `inventory sync shelly --discover` ueber
  die VLAN-Grenze hinweg funktioniert.

**Hinweis-Charakter**: VLAN-Setup ist fuer das Funktionieren der
Test-Umgebung **nicht zwingend**. Fuer Cockpit-Konformitaet (G3-
Phase) wird es Pflicht.

### 9.3 VPN-Anbindung zum VPS

Der Inventory-Backend-Container laeuft auf einem **Public VPS**
(siehe `docs/vps-setup.md`) und greift via VPN-Sidecar in das Heim-
LAN. Test-Umgebung muss:

- WireGuard-Endpoint im Heim-LAN bereitstellen (Router oder
  separater RPi).
- `AllowedIPs` so setzen, dass CCU, HA, Shellys im Sub-Netz
  liegen (z. B. `10.0.0.0/24`).
- Reverse-DNS oder Hosts-Datei pflegen, damit das Backend mit
  Hostnamen statt IPs arbeiten kann (`homeassistant.example.local`
  → `10.0.0.10`).

Details: `docs/vps-setup.md` Sektion „VPN sidecar".

---

## 10. Verifikations-Checkliste

Pro Komponente ein Ein-Zeilen-Smoke-Test. „Geht" heisst: die
Test-Umgebung steht fuer die naechste Sprint-Iteration.

| # | Komponente             | Smoke-Test                                                                                  |
| - | ---------------------- | ------------------------------------------------------------------------------------------- |
| 1 | RaspberryMatic         | `curl http://<ccu>/addons/xmlapi/devicelist.cgi` liefert XML mit ≥ 1 `<device>`-Eintrag     |
| 2 | HM-Funk-Aktor          | RaspberryMatic-Web-UI → Geraet schaltbar (Ein/Aus klickbar, LED am Aktor reagiert)         |
| 3 | HmIP-Aktor             | RaspberryMatic-Web-UI → Geraet erreichbar, Schalten OK                                      |
| 4 | Home Assistant         | `http://<ha>:8123` → Login → Dashboard zeigt mindestens 1 Geraet aus Integration            |
| 5 | HA HomeMatic-Integr.   | HA-UI → „Geraete" → CCU-Aktor sichtbar, Schalt-Toggle funktioniert                          |
| 6 | Shelly Gen2 (HTTP)     | `curl -X POST http://<shelly>/rpc/Shelly.GetStatus` liefert JSON mit `sys.uptime` ≥ 0       |
| 7 | Shelly Gen1 (HTTP)     | `curl http://<shelly>/status` liefert JSON mit `uptime` ≥ 0                                 |
| 8 | Shelly H&T (Sensor)    | HA-UI → Sensor zeigt aktuelle Temperatur **oder** zuletzt gemessenen Wert (Batterie!)       |
| 9 | MQTT-Broker            | `mosquitto_sub -h <broker> -t '#' -v -u <user> -P <pass>` zeigt Shelly-Nachrichten          |
| 10| HA REST-API            | `curl -H "Authorization: Bearer <TOKEN>" https://<ha>:8123/api/states` liefert JSON-Array   |
| 11| VPN-Tunnel             | Von VPS aus: `curl http://<ccu>/addons/xmlapi/devicelist.cgi` funktioniert wie aus Heim-LAN |
| 12| Backend-Sync CCU       | `inventory sync ccu --url http://<ccu>` → Exit-Code 0, SQLite hat ≥ 1 Device                |
| 13| Backend-Sync HA        | `inventory sync ha --url https://<ha>:8123 --token <TOKEN>` → Exit-Code 0                   |
| 14| Backend-Sync Shelly    | `inventory sync shelly --discover-seconds 10` → Exit-Code 0, ≥ 1 Shelly im YAML             |

Wenn 1–14 gruen: Hardware-Test-Umgebung ist produktiv.

---

## 11. Integration mit dem Rust-Inventory-Backend

### 11.1 Datenfluss in das Backend

Das Backend ruft je Sync-Subkommando die zugehoerige Quelle ab und
schreibt nach **SQLite** + **YAML**. Pro Source ein YAML.

| Sub-Kommando             | Endpunkt                                          | YAML-Datei                 |
| ------------------------ | ------------------------------------------------- | -------------------------- |
| `inventory sync ccu`     | `http://<ccu>/addons/xmlapi/devicelist.cgi`       | `inventory/yaml/ccu.yaml`  |
| `inventory sync ha`      | `https://<ha>:8123/api/states`                    | `inventory/yaml/ha.yaml`   |
| `inventory sync hue`     | `http://<hue-bridge>/api/<key>/lights` + sensors  | `inventory/yaml/hue.yaml`  |
| `inventory sync shelly`  | mDNS + Gen1/Gen2 HTTP                             | `inventory/yaml/shelly.yaml` |

Idempotenz-Anker: Natural Key `(source, source_id)`. Mehrfach-
Ausfuehrung darf das YAML byte-identisch lassen, solange sich
nichts geaendert hat — sonst entstehen Sync-Commits ohne Substanz.

### 11.2 Konkretes Soll-Test-Datenset

Nach erfolgreichem Aufbau **sollte** der Backend-Sync gegen die
Test-Umgebung folgendes ergeben (Minimum-Set):

| Source | Geraete (Mindestzahl) | Beispiel-Eintraege                                |
| ------ | --------------------- | ------------------------------------------------- |
| ccu    | 2                     | `HM-LC-Sw1-PI-3` (Funk), `HmIP-PSM` (HmIP)        |
| ha     | 1                     | mindestens 1 HA-Automation oder Helper-Entitaet   |
| shelly | 1                     | `Shelly Plus 1` (Gen2) oder `Shelly 1PM` (Gen1)   |

Das ergibt 4 Eintraege im aggregierten Inventar — genug fuer eine
Sync-Idempotenz-Pruefung (zweimal `inventory sync …` ausfuehren,
zweiter Lauf muss `git status` clean lassen).

**Komfort-Set** ergibt ≥ 8 Geraete, deckt alle Channel-Typen
(Switch, Dimmer, Rollladen, Heizungsventil, Sensor) ab.

### 11.3 Secrets-Schnittstelle

Die Test-Umgebung erzeugt drei Secret-Klassen:

| Secret                | Verwendet von        | Ablage (Soll)                                              |
| --------------------- | -------------------- | ---------------------------------------------------------- |
| RaspberryMatic-Admin  | (perspektivisch CCU-Auth, aktuell ungenutzt von Backend) | `inventory/secrets/ccu.sops.yaml` (ADR-0004 noch offen) |
| HA Long-Lived Token   | `inventory sync ha`  | `inventory/secrets/ha.sops.yaml`                           |
| MQTT-User `shelly-test` Passwort | Shelly-Geraete + HA  | nicht ins Backend; Shelly-eigene Persistenz                |

**Wichtig**: ADR-0004 (sops/age) ist als H.1-STOPP **offen**. Bis
dahin Test-Secrets lokal im Passwort-Manager, **nie** im Git-
Klartext. gitleaks-Hook (geplant) wird sonst alle Pushes blocken.

### 11.4 Aufruf-Beispiele

Mit `inventory/test-setup.env.example` als Vorlage (Pfad existiert
im Repo):

```bash
# CCU-Sync (aus VPS heraus via VPN)
inventory sync ccu --url "http://10.0.0.20"

# HA-Sync
inventory sync ha \
  --url "https://homeassistant.example.local:8123" \
  --token "${INVENTORY_HA_TOKEN}"

# Shelly-Sync mit mDNS
inventory sync shelly --discover-seconds 15

# Shelly-Sync gegen feste IPs (falls mDNS ueber VPN nicht geht)
inventory sync shelly \
  --ip 10.0.0.50 \
  --ip 10.0.0.51 \
  --ip 10.0.0.52
```

Nach erfolgreichem Sync:

```bash
cat inventory/yaml/ccu.yaml inventory/yaml/ha.yaml inventory/yaml/shelly.yaml
sqlite3 inventory.db 'SELECT source, source_id, model FROM devices;'
```

---

## 12. Reset / Teardown

### 12.1 Vor Test-Lauf: Snapshot anlegen

1. **RaspberryMatic**: Systemsteuerung → „Sicherheit" → „Backup
   erstellen" → Datei `raspberrymatic-baseline-YYYYMMDD.sbk` lokal
   speichern.
2. **Home Assistant**: „Einstellungen" → „System" → „Backups" →
   „Backup erstellen" → Name `ha-baseline-YYYYMMDD`. Standard ist
   „Full"; „Partial" reicht aber meist (HA-Config + Add-ons).
3. **SD-Karten-Image** (optional, harte Variante): RPi
   herunterfahren, SD entnehmen, mit `dd` oder „Raspberry Pi
   Imager" Backup-to-Image → Datei `<host>-fullsd-YYYYMMDD.img`.

Beide RaspberryMatic-/HA-Backup-Dateien gehen auf **denselben
USB-Stick oder NAS-Share** wie spaetere produktive Backups —
ein konsistenter Backup-Pfad spart spaeter den Wechsel.

ISO/IEC 27001:2022 A.8.13 (Information Backup) ist hier
sekundaerer Anker; A.7.13 (Equipment maintenance) bleibt primaer.

### 12.2 Nach Test-Lauf: Restore-Pfad

Reihenfolge (umgekehrte Aufbau-Reihenfolge):

1. **Home Assistant**: „Einstellungen" → „System" → „Backups" →
   gewuenschten Snapshot waehlen → „Wiederherstellen". HA neustartet
   automatisch.
2. **RaspberryMatic**: Systemsteuerung → „Sicherheit" → „Backup
   wiederherstellen" → Datei hochladen. CCU neustartet.
3. **Shelly-Geraete**: nur wenn waehrend des Tests neue Config gesetzt
   wurde — per Web-UI „Settings" → „Reset to factory defaults"
   oder via API:
   - Gen2: `curl -X POST http://<shelly>/rpc/Shelly.FactoryReset`
   - Gen1: Reset-Taster ≥ 10 s druecken
4. **Backend-State** (lokal): `inventory.db` loeschen, `inventory/yaml/*.yaml`
   ueber `git checkout` zuruecksetzen.

### 12.3 Voll-Rueckbau

Falls die Test-Umgebung komplett abgebaut wird:

1. Snapshots wegspielen + an Cockpit-Stelle dokumentieren.
2. SD-Karten:
   - RaspberryMatic-SD: `dd if=/dev/zero of=/dev/sdX bs=1M count=100`
     (loescht Bootloader; SD wieder normal verwendbar).
   - HA-SD: ebenso.
3. Geraete-Reset (Sektion 5.2 / 5.3 Reset-Sequenzen, Shelly
   Factory-Reset).
4. Wi-Fi-Credentials aus Shellys (im factory-Reset enthalten).
5. CCU-XML-API-Token / HA-Long-Lived-Token im Passwort-Manager
   loeschen.
6. Hardware physisch verstauen, Stueckliste-Status aktualisieren.

---

## 13. Offene Punkte / TODO

Diese Punkte fielen beim Schreiben auf, gehoeren aber **nicht** in
diese Hardware-Anleitung und werden in spaetere Sprints geschoben.

### 13.1 Security-/Privacy-relevant

- **HomeMatic-IP-Cloud-Konto ist Privacy-Risiko**. Wenn Variante 2
  (Sektion 8.5) statt Variante 3 (HACS-HomeMatic-IP-Local) gewaehlt
  wird, fliessen Geraete-Events ueber eq-3-Cloud-Server. Fuer
  Test-Umgebung OK, **nicht** fuer Produktion. Action: in einem
  spaeteren Audit „Cloud-Integration vermeiden" als HIGH-Backlog
  notieren.
- **Shelly OTA aus dem Internet** ist Default. Bei IoT-VLAN-
  Sperrung (Sektion 9.2) muss OTA-Pfad manuell freigegeben oder
  lokal gespiegelt werden. Default-Empfehlung: Updates nur
  manuell, nicht Auto.
- **MQTT in der Test-Umgebung lauscht auf `1883` plain TCP**.
  Fuer Produktion MQTT-TLS (`8883`) Pflicht — ASVS V9.1.
- **Selbstsignierte HA-Zertifikate** (Default in HassOS) brauchen
  spaeter eine eigene CA oder ein Let's-Encrypt-via-DNS-Setup.
  Iter-2+.

### 13.2 Backend-/Inventory-relevant

- **MQTT-Topic-Schema konsolidieren**: Shelly-Default-Topics sind
  Geraete-individuell (`shellyplus1-XXXXXX/…`). Spaeter ein
  einheitliches Praefix (`home/<room>/<device>/…`) waere ein
  Refactor — separater Backend-Task, **nicht** Hardware-Setup.
- **mDNS ueber VLAN**: wenn VLAN-Trennung kommt (9.2), braucht
  `inventory sync shelly --discover` einen mDNS-Reflector. Soll
  in `docs/architecture.md` als Constraint dokumentiert werden.
- **Fixture-Refresh-Workflow**: ein Skript, das `ccu_devicelist.xml`
  aus der echten Test-Umgebung pullt und mit dem Repo-Fixture
  diffed, waere fuer Drift-Erkennung praktisch. Separater
  Backend-Task.

### 13.3 Cockpit-relevant

- **Stack-Drift in `config/cockpit.yaml`**: `tech_stack:
  [home-assistant, mqtt, zigbee]` ist nicht repraesentativ. Iter-2-
  Architekten-Task (vgl. `shared/milestones/current.md`).
- **ADR-0004 (Edge-Secret-Backup)**: muss spaetestens vor dem
  Phase-2-Gate-Pass implementiert sein. Bis dahin Secrets lokal
  im Passwort-Manager.

### 13.4 Hardware-Beschaffung

- **HM-LC-* (HomeMatic-Funk-Linie)** wird von eq-3 nicht mehr
  beworben; aktive Verfuegbarkeit nur noch im ELV-Lagerbestand.
  Vor Kauf eines HM-Funk-Test-Aktors bei ELV pruefen, ob das
  Wunsch-Modell noch aktiv lieferbar ist. HmIP ist die laufende
  Linie.
- **Raspberry Pi 5**: RaspberryMatic-Support fuer Pi 5 ist seit
  ~2024 vorhanden, aber **vor Kauf** das aktuelle Release-Tag
  pruefen. HassOS-Support fuer Pi 5 ist seit `haos_rpi5-64-11.x`
  stable.
- **Shelly Pro (Hutschienen-Geraete)**: identische API zu Plus-
  Gen2, aber teurer und fuer eine Test-Umgebung nicht noetig.

---

## Quellen-Verzeichnis

### Verbindliche externe Standards

- **BSI IT-Grundschutz IND.2.1** (Allgemeine ICS-Komponente —
  insbesondere A1/A2 zu Netzwerktrennung).
- **ISO/IEC 27001:2022 A.7.13** (Equipment maintenance) und
  **A.8.13** (Information backup).
- **OWASP ASVS v4.0.3 V9.1** (Client Communications Security —
  TLS-Pflicht).
- **OWASP ASVS v4.0.3 V14.2** (Configuration — getrennte
  Service-Accounts).

### Externe Projekt-Quellen (vor Verwendung verifizieren)

- RaspberryMatic Releases: `https://github.com/jens-maus/RaspberryMatic/releases`
- Home Assistant OS Installation: `https://www.home-assistant.io/installation/raspberrypi`
- Shelly Firmware: `https://shelly-api-docs.shelly.cloud/` (Gen1)
  und `https://shelly-api-docs.shelly.cloud/gen2/` (Gen2)
- eq-3 Produktkatalog HomeMatic / HmIP: `https://www.eq-3.de/`

### Repo-interne Bezuege

- `README.md` — Komponenten + Sync sources
- `docs/architecture.md` — Datenfluss
- `docs/getting-started.md` — Software-Sicht (logischer Vorgaenger
  zu dieser Hardware-Sicht)
- `docs/vps-setup.md` — VPN-Endpoint
- `inventory/fixtures/ccu_devicelist.xml` — XML-API-Format-
  Referenz
- `inventory/test-setup.env.example` — ENV-Vorlage fuer Backend-
  Sync

---

> **Hinweis zur Datei-Geschichte**: Diese Datei wird **direkt auf
> `main` committet** (Cockpit-Workflow-Disziplin R-1 — Doku-
> Direktpfad gilt fuer `docs/`-Updates im Projekt-Repo). Sie ist
> ein deklarativer Doku-Patch, kein Code. Falls Branch-Protection
> auf `main` Admin-Bypass erfordert: dokumentiert in dieser Datei-
> historie via `git log`.
