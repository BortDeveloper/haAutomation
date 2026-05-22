# haAutomation Memory-Snapshot

Persistente, **projektspezifische Erkenntnisse** zum haAutomation-Projekt
(HA-Setup, NodeRed-Flows, Homematic-Brücke, Inventory-Backend). Wird
session-übergreifend von Claude Code gelesen.

## Snapshot vs. Live-Variante

| Schicht | Pfad | Rolle |
|---|---|---|
| **Live** (project-spezifisch) | `~/.claude/projects/ha-automation/memory/` | Quelle der Wahrheit für Claude-Recall im haAutomation-Projekt |
| **Live** (Cockpit-Bezug) | `~/.claude/projects/-root-projects-stack-master/memory/` | Cockpit-Sicht auf haAutomation (Public-Ready-Status, Erprobungs-Phase, MQTT-Architektur) |
| **Snapshot** (hier) | `<repo>/.claude/memory/` | Versionierter Stand für git-Sicherung + Wiederherstellung auf neuer Maschine |

## Scope

**Hier**: alles für Verständnis und Betrieb des haAutomation-Projekts:
- Projekt-Scope + Heim-Topologie (anonymisiert)
- Repo-/SSH-/Deploy-Key-Setup
- Inventory-Build-Server-Setup
- Cockpit-Bezug (Public-Ready-Status, Erprobungs-Phase)
- HA-Smarthome-Ziel-Architektur (MQTT-zentrisch, ccu-jack-Adoption)

**Nicht hier**:
- Generelle Cockpit-/Orchestrierungs-Erkenntnisse → `stack-master/.claude/memory/`
- ansible-vps-stack-Spezifika → `ansible-vps-stack/.claude/memory/`

## PII-Konvention (verbindlich — Repo ist PUBLIC!)

Vor jedem Snapshot-Update folgende Klar-Werte durch Platzhalter ersetzen:

| Klar-Wert | Platzhalter |
|---|---|
| `ansible.guebraun.org` | `<operator-control-host>` |
| `paperless.guebraun.org` | `<operator-paperless-host>` |
| `guebraun@gmail.com`, `guenther@guebraun.org` | `<operator@example.org>` |
| `Bortfeld` (Ortsname) | `<smarthome-location>` |
| `Guenther Braun`, `Guenther Braeunlich`, `guebr` | `<operator>` / `<operator-user>` |
| `100.127.56.21` (Tailscale-IP) | `<inventory-tailscale-ip>` |
| `C:\Users\guebr\`, `/mnt/c/Users/guebr/` | `<windows-userprofile>\`, `<wsl-userprofile>` |
| `Strato` (Hosting-Provider-Name) | `<vps-provider>` |
| SSH-Key-Fingerprints | `<ssh-key-fingerprint>` |
| `BortDeveloper` (GitHub-Username) | bleibt — öffentlich |
| `BortDeveloper/haAutomation` (Repo-Refs) | bleibt — öffentlich |
| `originSessionId:`-Zeilen (UUIDs) | entfernen |

**Strikter Maßstab**: haAutomation ist PUBLIC auf GitHub. Jede Memory hier
ist weltweit lesbar — vor commit grep über die Konvention oben:
```bash
grep -rEnI "guebraun|Guenther|Bortfeld|100\.127\.|guebr@" .claude/memory/
```

## Restore auf neuer Maschine

Wird vom zentralen `stack-master/scripts/cockpit-setup-on-new-machine.sh`
mit erledigt — kopiert dieses `.claude/memory/`-Verzeichnis nach
`~/.claude/projects/ha-automation/memory/`.

## Historie

Diese Memory-Sammlung wurde am 2026-05-22 aus zwei Quellen konsolidiert:

1. **Cockpit-Memory** (`~/.claude/projects/-root-projects-stack-master/memory/`) — die 4 haAutomation-bezogenen Project-Memories
2. **Historische haBortfeld-Memory** (`~/.claude/projects/ha-automation/memory/`) — 5 Snapshots aus der Phase vor dem GitHub-Repo-Rename `haBortfeld` → `haAutomation` (PR #3+#4, filter-repo 2026-05-19)

Alle Einträge wurden im Zuge der Konsolidierung PII-gescrubbt nach der
Konvention oben. Datei-Namen wurden auf das neue `haautomation`-Schema
umgestellt (z. B. `project_habortfeld.md` → `project_haautomation_scope.md`).
