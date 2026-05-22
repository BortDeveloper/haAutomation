---
source: shared/audit-log/2026-05-22-upstream-openccu-ccu-jack-code-audit.md
derived: 2026-05-22
commit-audited: bc00d1d
---

# ccu-jack-Adoption — projektspezifisches Audit-Derivat

Dieses Dokument ist ein projektspezifischer Auszug aus einem Cockpit-Audit
der ccu-jack-Codebasis. Es enthält nur Punkte, die für die
haAutomation-Implementation handlungsrelevant sind. Vollständige
Code-Belege siehe Cockpit-Audit-Quelle.

## 1. Quelle und Geltungsbereich

- **Cockpit-Audit:** `shared/audit-log/2026-05-22-upstream-openccu-ccu-jack-code-audit.md`
- **Stand:** 2026-05-22
- **Auditierter ccu-jack-Commit:** `bc00d1d`
- **Geltung für haAutomation:** HA-Setup + NodeRed + CCU-Brücke (HmIP-RF
  und HmIPW über DRAP)

Alle hier referenzierten Fakten sind redundant zur Cockpit-Quelle
festgehalten, damit dieses Dokument auch ohne Cockpit-Zugriff (z. B. auf
dem Test-Raspberry) eigenständig lesbar bleibt.

---

## 2. Show-Stopper-Verifikation: DRAP-Test (vor Adoption pflichtig)

**Status:** offen — muss vor jeder weiteren ccu-jack-Adoption-Entscheidung
abgeschlossen sein.

**Hardware:** DRAP-bestückte Test-CCU mit mindestens einem HmIPW-Aktor
(z. B. HmIPW-DRBL-4) im Inventar.

**Vorgehen:**

```bash
LOG_LEVEL=DEBUG ./ccu-jack
# parallel in zweitem Terminal:
journalctl -u ccu-jack -f | grep "Call of method newDevices received"
```

**Auswertung:**

- **Hypothese H-A (hochwahrscheinlich):** alle Interface-IDs werden
  aufgelistet, HmIPW-DRBL-4-Aktoren erscheinen unter `HmIP-RF` und werden
  mit-callbackt. → Adoption kann weiter geplant werden.
- **Hypothese H-B (Adoption kippt):** HmIPW-Aktoren erscheinen unter
  separater Interface-ID, die ccu-jack nicht abonniert. → Adoption-Plan
  ist nicht tragfähig, NodeRed-on-CCU bleibt strukturell notwendig.

**Aufwand:** ~5 min. Show-Stopper — kein weiterer Architektur-Schritt
sinnvoll, solange das Ergebnis nicht vorliegt.

---

## 3. Topic-Schema (was haAutomation gegenüber ccu-jack kennen muss)

Alle Topics sind in ccu-jack hartkodiert. Es gibt keinen Konfig-Hebel,
um Präfixe oder Strukturen umzubiegen.

| Topic                                | Richtung           | QoS | Retain | Payload                                       |
| ------------------------------------ | ------------------ | --- | ------ | --------------------------------------------- |
| `device/status/<addr>/<ch>/<vk>`     | ccu-jack → HA      | 1   | ja*    | `{"ts": <ms>, "v": <value>, "s": <state>}`    |
| `device/set/<addr>/<ch>/<vk>`        | HA → ccu-jack      | 2   | nein   | `{"v": <value>}` reicht                       |
| `sysvar/status/<ISE-ID>`             | ccu-jack → HA      | 2   | ja     | wie oben                                      |
| `sysvar/set/<ISE-ID>`                | HA → ccu-jack      | 2   | nein   | wie oben                                      |
| `program/{status,set,get}/<ISE-ID>`  | bidirektional      | 2   | ja     | wie oben                                      |

*Retain ist **false** ausschließlich für Taster-`PRESS_*`-Events und
`INSTALL_TEST`.

**Konsequenz für haAutomation:** Topic-Aliasse (Etagen/Räume) müssen
außerhalb von ccu-jack abgebildet werden — entweder in NodeRed oder in
einem dedizierten Bridge-Service (siehe Abschnitt 4).

---

## 4. Architektur-Lücke: HA-MQTT-Discovery fehlt nativ

ccu-jack publiziert **keine** `homeassistant/<component>/...`-
Discovery-Payloads. Geräte erscheinen in Home Assistant nicht automatisch.

### Optionen (keine Vorentscheidung)

**Option a — Externer Bridge-Service `kaistraube/ccujack_homeassistant`**

- Vorteil: existiert, dokumentiert, keine Eigenentwicklung
- Risiko: Wartungsstand prüfen (letzter Commit, Issue-Backlog,
  Discovery-Schema-Aktualität gegen aktuelle HA-Versionen)

**Option b — NodeRed-Subflow**

- Vorteil: NodeRed bleibt im Setup, kein zusätzlicher Prozess
- Vorteil: niedrigste Einstiegshürde, schnell iterierbar
- Risiko: NodeRed wird damit zur Discovery-Pflichtkomponente —
  Single-Point-of-Failure-Erweiterung

**Option c — Eigener Bridge-Service (~200 LOC Go oder Python)**

- Vorteil: kann gleichzeitig Cockpit-Topic-Schema (Etagen/Aliasse)
  abbilden und Discovery-Payloads erzeugen
- Vorteil: deterministisches, getestetes Mapping; reproduzierbar
- Risiko: Eigenentwicklung + langfristige Wartungspflicht
- Empfehlung: kandidat für **Iter-3-Aufgabe**, wenn DRAP-Test (Abschnitt
  2) Hypothese H-A bestätigt und Topic-Volumen klar ist

---

## 5. Auth-Härtung am Deploy (Pflicht vor Erst-Inbetriebnahme)

| Befund                                                  | Code-Stelle                                  | Härtung                                                                                                                  |
| ------------------------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Default-Config `Users: {}` = anonym                     | `dist/ccu/addon/ccu-jack-default.cfg:42`     | Vor erstem Start einen User mit bcrypt-Hash anlegen; bcrypt-Cost ≥ 12 (Default `MinCost` = 4 ist 2026 zu niedrig)        |
| RBAC-Dead-Code (kein AuthZ-Enforcement)                 | `rtcfg/model.go:161-183` (nie aufgerufen)    | Authentifizierte User = Vollzugriff; bis Authentik-Phase-3 mit dedizierter ccu-jack-Instanz pro Trust-Zone arbeiten      |
| TLS-Bridge-Client `InsecureSkipVerify` ohne Warning     | `mqtt/mbridge.go:117`                        | **Niemals** `cfg.Insecure = true` in Prod-Config; CA-Pinning via `CACertFile` ist der saubere Pfad                       |
| Auto-Cert mit 10-Jahres-Gültigkeit                      | `main.go:193`                                | In closed-Network tolerabel; für externe Erreichbarkeit eigene CA mit ≤ 27 Monaten Laufzeit (NIST SP 800-57)             |
| Reverse-Proxy-Unaware                                   | `httpauth.go:54`                             | Bei Proxy-Front: Client-IP-Verlust in Logs akzeptieren, oder ccu-jack direkt mit TLS exponieren                          |

---

## 6. NodeRed-Pfad bleibt vorerst (Risiko-Vermeidung)

`node-red-contrib-ccu-jack` ist als existierender Adapter dokumentiert
(deterministische Topics, JSON-Payload).

**Empfehlung:** NodeRed-on-CCU **nicht** wegnehmen, solange

1. die DRAP-Verifikation (Abschnitt 2) nicht durchgelaufen ist, **und**
2. die HA-Discovery-Bridge (Abschnitt 4) nicht steht und produktiv läuft.

Ein vorzeitiger NodeRed-Rückbau erzeugt eine Lücke, die mit ccu-jack
allein nicht zu schließen ist.

---

## 7. Authentik-Integrationspfad (ADR-0003-Bezug, Phase-3, nicht jetzt)

Nur als Vormerkung — **nicht** vorab umsetzen:

- **HTTP:** Authentik-Outpost vor ccu-jack mit
  Basic-Auth-Header-Durchreichung
- **MQTT:** externer Mosquitto mit `mosquitto-go-auth`-Plugin + OIDC,
  ccu-jack betrieben als Bridge-Client
- **Architektur-Grenze:** ccu-jack-Auth ist **nicht** via Plugin
  tauschbar (go-mqtt-Registry-Architektur). Externalisierung geht nur
  per Proxy-davor, nicht per In-Process-Hook.

---

## 8. Offene Fragen an die Hardware (Verifikation während DRAP-Test)

- Reagiert ein DRAP-Aktor auf
  `device/set/<adresse>/<ch>/LEVEL` mit Payload `{"v": 0.5}`
  (= Rollladen 50 %)?
- Erscheinen HmIPW-Sensoren (Tür/Fenster) mit `STATE`-Datenpunkt unter
  `device/status/...`?
- Wie verhält sich ccu-jack bei einem CCU-Reboot — werden
  Watchdog-Re-Registrierungen erkennbar im Log? Welche Log-Zeile ist
  der eindeutige Indikator?

Antworten sind Voraussetzung für die Architekturentscheidung in
Iter-3 (Bridge-Service Option c) und für den Rückbau-Plan von
NodeRed-on-CCU.
