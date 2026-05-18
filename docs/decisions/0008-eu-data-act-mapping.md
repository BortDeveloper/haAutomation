# ADR-0008: EU Data Act — Anwendbarkeit + Artikel-Mapping

- **Status:** accepted
- **Datum:** 2026-05-17
- **Bezug:** Architekt-Audit R7, `shared/standards/eu-data-act.md` § ADR-Pflicht

## Kontext

`ha-automation` sammelt Daten **vernetzter Produkte** aus dem Heimnetz (Home
Assistant, Homematic CCU, Hue, Shelly; später Zigbee2MQTT/Node-RED): Geräte-
identität, Firmware- und Software-Stände. Die Verordnung (EU) 2023/2854
(Data Act) ist seit dem **12.09.2025** anwendbar und war im Repo bisher nicht
referenziert.

## Entscheidung

Der Data Act ist **anwendbar**. Mapping der relevanten Kapitel:

| Norm | Relevanz für `ha-automation` | Bewertung |
|---|---|---|
| **Art. 3–7** — Datenzugang vernetzter Produkte | Der Heimnetz-Betreiber ist hier zugleich Dateninhaber **und** Nutzer (Solo-Betrieb, kein B2B/B2C-Drittzugang). | Zugangs-/Bereitstellungspflichten praktisch erfüllt: die Inventardaten liegen als versioniertes YAML offen im Repo. |
| **Art. 13** — missbräuchliche Vertragsklauseln | Kein Vertrag mit Dritten über die Daten. | nicht einschlägig |
| **Art. 23–31** — Wechsel zwischen Datenverarbeitungsdiensten (Cloud-Switching) | Relevant für das geplante Off-Site-Backup (`restic` → S3). | **Auflage an den Folge-ADR Backup-Target:** EU-Sitz des Providers **oder** dokumentierter Switching-Pfad. |

## Folgen

- Die Data-Act-Anwendbarkeit ist dokumentiert; die `eu-data-act.md`-ADR-Pflicht
  ist erfüllt.
- **Bindende Vorgabe für den Folge-ADR Backup-Target** (Closing-Brief §2.3):
  Die S3-Provider-Wahl muss Art. 23–31 erfüllen (EU-Sitz oder Switching-Pfad).
  Dieser Folge-ADR ist eine ausstehende **User-Entscheidung**.
