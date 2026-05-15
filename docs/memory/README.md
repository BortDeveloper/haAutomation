# Memory-Snapshot

Persistenter Projekt-Kontext, den Claude Code session-uebergreifend liest.
Die Live-Variante liegt auf der Workstation unter

```
%USERPROFILE%\.claude\projects\C--Users-guebr-nr\memory\
```

Dieser Ordner hier ist ein **Snapshot zum Backup und zur Nachvollziehbarkeit**.
Aenderungen werden manuell zurueckgespiegelt — die Live-Files bleiben die
Quelle der Wahrheit fuer Claudes Recall.

## Dateien

| Datei | Typ | Inhalt |
|---|---|---|
| `MEMORY.md` | Index | One-Liner pro Memory-Eintrag, von Claude immer mitgeladen |
| `project_hasite.md` | project | Designentscheidungen, Tech-Stack, Status, Out-of-Scope |
| `project_hasite_home.md` | project | Physische Umgebung: HA / CCU / NR / Z2M / VPN / Authentik |
| `reference_hasite_repo.md` | reference | Repo-URL, SSH-Setup auf Workstation + VPS, Schluessel-Inventar |
| `user_role.md` | user | GitHub-Identitaet, Service-Landschaft, Code-Praeferenzen |

## Snapshot pflegen

Nach Aenderungen an den Live-Files:

```powershell
$src = "$env:USERPROFILE\.claude\projects\C--Users-guebr-nr\memory"
$dst = "C:\Users\guebr\nr\haAutomation\docs\memory"
Copy-Item -Recurse -Force "$src\*.md" $dst
```

Dann PR mit dem Diff.

## Was hier NICHT reingehoert

- Private Schluessel (age, ssh) — bleiben auf den Hosts
- Long-Lived Access Tokens (HA, CCU-credentials) — sops-encrypted unter `inventory/secrets/`
- Klartext-Passwoerter

Public-Key-Fingerprints, Hostnamen und Konfigurationspfade duerfen drin sein —
das Repo ist privat.
