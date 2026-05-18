# ADR-0007: EU-KI-VO — nicht anwendbar (Negativ-ADR)

- **Status:** accepted
- **Datum:** 2026-05-17
- **Bezug:** Architekt-Audit R7, `shared/standards/eu-ai-act.md` § Audit-Hinweis

## Kontext

`shared/standards/eu-ai-act.md` verlangt, dass **jedes** Cockpit-Projekt seinen
Status zur Verordnung (EU) 2024/1689 (KI-VO) dokumentiert — auch ein „nicht
betroffen" ist nur mit konkretem Artikelbezug valide. Bisher fehlte dieser
Eintrag für `ha-automation`.

## Entscheidung

`ha-automation` ist **nicht in den Anwendungsbereich der EU-KI-VO** gefasst:

- **Art. 3 Abs. 1 (Definition KI-System):** Das System liest, normalisiert,
  speichert und stellt Geräte- und Firmware-Inventardaten dar. Es enthält
  **keine Inferenz, kein trainiertes Modell, keine Ableitung** von Ausgaben aus
  Eingaben über deterministische Regeln hinaus. Damit liegt kein „KI-System"
  im Sinne der Definition vor.
- **Art. 6 + Anhang III (Hochrisiko):** Kein gelisteter Hochrisiko-Anwendungs-
  fall ist berührt.
- Roadmap-Phase 5 sieht die Migration bestehender Heim-Automationen vor. Diese
  sind regelbasiert (HA-/Node-RED-Trigger→Aktion) und fallen aller Voraussicht
  nach ebenfalls nicht unter Art. 3 Abs. 1.

## Folgen

- Status „nicht anwendbar" ist mit Artikelbezug dokumentiert; die
  `eu-ai-act.md`-Auditpflicht ist erfüllt.
- **Re-Check-Trigger:** Sobald in Phase 5 ein **lernendes** Verfahren
  hinzukommt (z.B. Anwesenheits-Prädiktion aus Verhaltensmustern) oder eine
  HA-ML-Komponente angebunden wird, ist dieses ADR neu zu bewerten. Der Trigger
  ist im Iterationsplan vermerkt (Architekt-R10).
