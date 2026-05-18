# ADR-0004: `authgate` als Übergangs-SSO, Zielzustand Authentik

- **Status:** accepted
- **Datum:** 2026-05-17 (retroaktiv dokumentiert)
- **Bezug:** Architekt-Audit R6 + R2, Cockpit-ADR-0003

## Kontext

Die Inventory-Web-UI darf nicht ungeschützt im Netz stehen. Die Zielarchitektur
sieht einen vorgelagerten Reverse Proxy mit `forward_auth` vor, der bei
erfolgreicher Authentifizierung den Header `X-Authentik-Username` setzt; die
App parst nur diesen Header und enthält keine OIDC-Logik.

Solange am Deploy-Host **kein externes SSO** bereitsteht, ist dieser
`forward_auth`-Slot leer. Zusätzlich bestand eine offene Frage (Architekt-R2):
Cockpit-Soll ist ein zentrales IdP (`target-state.md`), das Repo plante
Authentik.

## Entscheidung

- Der `forward_auth`-Slot wird übergangsweise durch ein **eigenes Sidecar
  `authgate`** gefüllt (`src/bin/authgate.rs`, Binary aus demselben Crate):
  Login-Formular + zustandsloses, HMAC-SHA256-signiertes Session-Cookie,
  PBKDF2-HMAC-SHA256 (600.000 Runden), fail-closed bei leerer Benutzerliste.
- **SSO-Provider-Wahl (R2): Authentik.** Aufgelöst durch **Cockpit-ADR-0003** —
  die bereits betriebsfertige Authentik-Instanz gewinnt; `authgate` bleibt der
  explizit dokumentierte Übergang bis Roadmap-Schritt **S14**.
- Der Header-Vertrag `X-Authentik-Username` ist für `authgate` und das spätere
  Authentik **identisch**. Der Wechsel ist ein Einzeiler im Caddyfile
  (`forward_auth`-Ziel); die `inventory`-App bleibt unverändert.

## Folgen

**Positiv**

- Die UI ist von Tag 1 an authentifiziert, ohne auf das Authentik-Setup zu warten.
- Stabiler Vertrag → der IdP-Wechsel berührt nur den Proxy.

**Negativ / Kosten**

- Temporär existiert ein zweites Identity-Subsystem parallel zum Zielzustand.
  Bewusst befristet bis S14; danach wird `authgate` außer Betrieb genommen.
