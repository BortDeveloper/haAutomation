---
name: HA-Smarthome MQTT-Architektur (2026-05-22)
description: Reale gewachsene Architektur des User-Smarthomes — HA + NodeRed + separate OpenCCU mit HmIPW-DRAP; MQTT als zentrale Abstraktions-Schicht; ccu-jack-Adoption als Ziel.
type: project
---
**Stand 2026-05-22** — Erkannt aus User-NodeRed-Flow-Auszug + Konversation.

## Topologie heute

```
HmIP/HmIPW/HM-Geräte — **HmIPW-Support ist Muss** (User-Bestätigung
2026-05-22), HmIPW-DRAP-Gerät zwingt separate OpenCCU-Hardware. Damit:
ccu-jack-HmIPW-Kompatibilität ist KO-Kriterium für die Adoption.
        │
        ▼
OpenCCU (separater Raspberry, Tailnet 100.79.140.88)
 ├─ ReGaHss (XML-RPC intern, Ports 2047 BinRPC / 2048 XML-RPC)
 ├─ NodeRed-Instanz AUF der CCU (veraltet, ungepflegt — der Schmerzpunkt)
 │    └─ node-red-contrib-ccu via XML-RPC
 ├─ XML-API-Addon (heutiger ha-automation-Pfad, soll weg)
 └─ ccu-connection-Config in NodeRed enthält Inline-Klartext-Credentials
    (CRITICAL-Befund 2026-05-22; siehe „Lessons" unten)
        │
        ▼ MQTT TLS Client-Cert
HomeAssistant (separater Host) — hostet Mosquitto-Broker auf
   `homeassistant.fritz.box:8883`, Client-ID `ccu-bort`
 ├─ NodeRed-in-HA (Prozesslogik, soll bleiben + gestärkt)
 ├─ HA-Integrationen für non-HM-Devices → publishen auf MQTT (gleiche Abstraktion)
 └─ HA-`homematic`-Core-Integration: aktiv in geringem Umfang
    (User-Bestätigung 2026-05-22), soll abgebaut werden — keine
    weiteren MQTT-Konsumenten außer HA / NodeRed / ha-automation
        │
        ▼ (geplant)
ha-automation (Inventory-Tool, Rust) — soll auf MQTT-Subscribe wechseln
```

## MQTT-Topic-Schema (gewachsen, inkonsistent — Migrations-Knackpunkt)

### Set-Pfad (NodeRed → CCU)

```
<typ>/<etage>/<gerät>/<aktion>/<sub>
```

| Element | Werte gesehen |
|---|---|
| **typ** | `licht`, `dim`, `st` (Steckdose), `schuetz` |
| **etage** | `dg`, `og`, `eg`, `ke` (Keller), `gr` (Garten/außen) |
| **gerät** | `flureingang`, `kuechemitte`, `weihnachtsbaum`, `treppe1`–`treppe4`, ..., lowercase, geräte-logischer Name |
| **aktion** | `set` (write), `status` (read) |
| **sub** | `state` (bool), `level` (0–1 float für Dimmer), `color` (0–196 für RGB-Hue), `bwm`, `timer` |

Beispiele: `licht/eg/flureingang/set/state`, `dim/og/duschergb/set/color`,
`schuetz/gr/wintergarten/set/state`.

### Get-Pfad (CCU → MQTT) — anderes Schema!

`ccu-rpc-event`-Nodes publishen mit Template `${deviceName}/${datapoint}`,
z. B. `bwm_eg_flur/motion`. Unterstriche werden im Subflow `Name2Topic`
zu `/` gewandelt + lowercase. **Konsequenz**: Set und Get folgen
**unterschiedlichen Schemata** — historisch gewachsen.

### Channel-Index-Spreizung

Mehrkanal-HmIP-Geräte (z. B. `DIM_OG_BadSpiegelDuo` mit Channels 1–6,
`DIM_GR_Nord` 2–4) werden im Set-Pfad über **einen** logischen
Topic angesprochen und in NodeRed via Subflow auf mehrere
`ccu-set-value`-Nodes (verschiedene `channelIndex`) gefächert.
ccu-jack-Schema kennt das anders (`/status/<addr>/<dp>`-Pattern) —
das ist der **Hauptaufwand der ccu-jack-Migration**.

## Ziel-Architektur (Orchestrator-Empfehlung 2026-05-22, vor Deliberation)

Pivot weg von XML-API → hin zu **ccu-jack** (`mdzio/ccu-jack`, GPL-3.0,
Go-Binary, aktiv gepflegt — letzter Push 2025-12-08):

- ccu-jack ersetzt **sowohl** XML-API-Addon **als auch**
  NodeRed-on-CCU-Bridge
- ccu-jack publisht alle CCU-Datenpunkte als MQTT direkt gegen den
  HA-Broker
- NodeRed-in-HA subscribed auf ccu-jack-Topics statt eigene Bridge
  zu betreiben
- ha-automation `sync ccu` wird `sync mqtt` (Refactor)
- HA-`homematic`-Core-Integration: vermutlich entbehrlich (User-Klärung)
- CCU-Firewall: extern nur ccu-jack-MQTT-Port (8883)
- Eliminiert: R-1 (Token-Leak), R-3 (Expiry), R-4 (exec.cgi-RCE),
  R-5 (HTTPS-opt-in), R-6 (HA-Addon-Proxy CRITICAL-konditional)
- CCU-Login-Deliberation 2026-05-21 wird **gegenstandslos**, ebenso
  B-Bundle und Option-C-Upstream-PR

## Drei Audits 2026-05-22 (Deliberation-Sektionen 3.1/3.2/3.3)

- `shared/audit-log/2026-05-22-upstream-ccu-jack-inventory.md` —
  Codebase sauber + aktiv. **HI-2**: HmIPW-Frage offen, empirischer
  Test zwingend. **HI-3**: Topic-Schema hart, Bridge nur Prefix-Mapping
  → Option B realistischer. **HI-1**: UI schreibt Pauschal-Permission
  „all", Read-only nur per JSON-Edit.
- `shared/audit-log/2026-05-22-security-ccu-jack-threat-model.md` —
  **CRITICAL C-1**: Default `Users: {}` = Allow-All. **HIGH H-1**:
  Mosquitto-ACL pro Client pflichtig. **HIGH H-3**: OpenCCU-Firewall
  nach XML-API-Deinstall (Ports 80/443/2001/2010/9292 droppen).
  **HIGH H-5**: Klingel-/Tür-Topics personenbezogen (DSGVO Art. 5/32).
- `shared/audit-log/2026-05-22-sre-ccu-jack-ops.md` —
  **HIGH F-1**: ccu-jack-Cfg nicht im `.sbk`. **HIGH F-2**: DR-Pfad
  undokumentiert. **HIGH F-7**: M3-Drift-Detection-Lücke, ClientID-
  Konvention zwingend, M3-Cap max. 14 Tage.

## Migrations-Phasen (Skizze)

1. **M1 Staging**: Test-Raspberry + OpenCCU + ccu-jack-Install
2. **M2 Schema-Brücke**: ccu-jack-Default-Schema vs. bestehendes Schema —
   Entscheidung sauberer Cut vs. Adapter-Layer (architect)
3. **M3 Parallelbetrieb**: alte Bridge + ccu-jack parallel, NodeRed-Flows
   doppelt für 1–2 Wochen
4. **M4 Cutover**: alte NodeRed-on-CCU + XML-API deinstallieren, Firewall
   härten

## Lessons / Hygiene-Befunde

- **CRITICAL 2026-05-22 (resolved)**: NodeRed-`ccu-connection`-Config
  enthielt CCU-Admin-Passwort im Klartext im Flow-JSON. Im User-Chat
  geleakt. **User-Bestätigung 2026-05-22**: CCU-Admin-Passwort rotiert,
  Hygiene-Hinweis akzeptiert. Mit ccu-jack-Migration entfällt das
  Problem strukturell (Credentials wandern in ccu-jack-Config).
- **Set/Get-Schema-Inkonsistenz** ist häufiges Symptom von gewachsenen
  Smarthome-Setups — bei Greenfield-Re-Design einheitliches Schema
  forcieren.
- **NodeRed-on-CCU als Bridge** ist Standard-Pattern, aber Single-Point-
  of-Failure und Pflege-Last (TCL+Node.js auf Buildroot). ccu-jack
  (Go-Binary) ist die saubere Ablösung.

## Verweise

- `shared/deliberations/2026-05-21-ccu-secure-login.md` (wird durch
  ccu-jack-Adoption gegenstandslos)
- `shared/audit-log/2026-05-21-upstream-xmlapi-auth-inventory.md`
- `shared/audit-log/2026-05-22-legal-xmlapi-upstream-pr.md`
- ccu-jack: <https://github.com/mdzio/ccu-jack> (GPL-3.0, v2.12.4)
- ccu-jack ↔ HA: <https://github.com/kaistraube/ccujack_homeassistant>
- ccu-jack ↔ NodeRed: <https://github.com/ptweety/node-red-contrib-ccu-jack>
