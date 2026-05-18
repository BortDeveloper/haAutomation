# ADR-0003: VPN-Provider-Abstraktion über `service:vpn`

- **Status:** accepted
- **Datum:** 2026-05-17 (retroaktiv dokumentiert)
- **Bezug:** Architekt-Audit R6 + Praise, 12-Factor App (Faktor IV)

## Kontext

Das Backend läuft auf einem VPS-VPS und inspiziert das Heimnetz über VPN.
Welcher VPN-Anbieter genutzt wird (Tailscale, NetBird, WireGuard) ist eine
Betriebsentscheidung, die sich ändern kann. Provider-spezifischer Code im
Backend würde jeden Wechsel zu einem Code-Change machen.

## Entscheidung

Der VPN-Provider wird **nicht** im App-Code abgebildet, sondern als
austauschbares Deployment-Artefakt:

- Es gibt genau einen kanonischen Vertrag: ein Compose-Service mit dem **festen
  Namen `vpn`**. Die `inventory`-App joint dessen Netzwerk-Namespace
  (`network_mode: service:vpn`) und kennt selbst kein VPN.
- Jeder Provider liefert ein eigenes Compose-Overlay
  (`docker-compose.vpn.<provider>.yml`), das den `vpn`-Service definiert.
- Initialer Deploy: **Tailscale**. NetBird und WireGuard sind vorbereitet,
  aber nicht aktiv.
- Bewusst **drei separate Overlay-Dateien** statt einer Template-Evaluation —
  explizite Konfiguration vor Magie.

## Folgen

**Positiv**

- Die App ist vollständig VPN-agnostisch und damit ohne Code-Change testbar.
- Provider-Wechsel = Austausch eines Overlays beim `docker compose`-Aufruf.

**Negativ / Kosten**

- Drei Overlay-Dateien müssen parallel gepflegt werden (z.B. Image-Pinning,
  Healthchecks). Bewusst akzeptiert.
- Jeder VPN-Sidecar braucht erhöhte Capabilities (`NET_ADMIN` u.a.); die
  Härtung erfolgt pro Overlay (`cap_drop: ALL` + gezieltes `cap_add`).
