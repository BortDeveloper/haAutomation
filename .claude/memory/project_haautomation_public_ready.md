---
name: haAutomation public-fähig (2026-05-19)
description: Working-Tree + Git-History des haAutomation-Repos vollständig anonymisiert; Public-Stellung nur noch User-Klick auf GitHub-Visibility.
type: project
---
**Stand 2026-05-19 (nachmittags)**: `BortDeveloper/haAutomation` ist
**PUBLIC**. Visibility-Wechsel erfolgt durch User. Repo ist weltweit
lesbar; GitHub-Actions-Quota gilt unlimited für öffentliche Repos.

Pre-Public-Verifikation (2026-05-19) clean:
- HEAD `36d606bce8`, Author `Anonymous User <user@example.org>`
- 4 Branches alle gescrubbed
- 0 Issues, 0 offene PRs, 0 Webhooks, 0 Tags
- Beschreibung neutral umgestellt vor Public-Schaltung
- README-Sweep ohne identifizierende Strings

**Wie es gemacht wurde**:
- **PR #3** (gemerged 2026-05-18) — erste Welle: Real-Werte (Hostname,
  Domain, Mail, Ortsname, Subnet, Raum-Namen, Device-IDs) im Working-Tree
  durch Platzhalter ersetzt; `docs/memory/` (mit-eingechecktes Tool-Memory)
  und `docs/cockpit-audits/` (Iter-0-Redundanz mit stack-master) gelöscht.
- **PR #4** (gemerged 2026-05-19) — VPS-Provider-Codename + Hosting-Provider-
  Hinweise raus; Compose-File-Rename `docker-compose.vps.yml` →
  `docker-compose.vps.yml`; Cross-Repo-Link auf `ansible-vps-stack`
  entfernt (kann nach dessen Public-Setzung wieder rein).
- **Git-History-Scrub** (2026-05-19) — `git-filter-repo` mit
  `--replace-text` (Blob) + `--replace-message` (Commit-Messages) +
  `--mailmap` (Author-Identitäten) + `--path-rename` (historische
  identifizierende Pfade). Neuer HEAD: `36d606bce8`. Backup unter
  `/root/repo-backups/haAutomation-pre-scrub-2026-05-19.git`.

**Memory-Migration** (parallel zu PR #3): `docs/memory/` ist jetzt
in `~/.claude/projects/ha-automation/memory/` (7 Dateien). Im Repo
gibt es kein Tool-Memory mehr — entspricht der state-of-the-art
Template/Local-Trennung (siehe `feedback_template_local_separation.md`).

**Rollback-Befehl** (falls je nötig):
```
cd /root/repo-backups/haAutomation-pre-scrub-2026-05-19.git
git push --mirror --force https://github.com/BortDeveloper/haAutomation
```

**CI-Pipeline live (2026-05-19 18:44 UTC)**:
- PR #6 `fccc71a1` — URL-Encoding-Bug-Fix in `inventory/src/bin/authgate.rs`
  Z. 574 (`urlencode()` matched `/` als unreserved → jetzt nach RFC 3986
  §2.3 korrekt nur a-zA-Z0-9-._~). Vorbestehender Bug, der ohne CI
  unentdeckt blieb.
- PR #5 `b70181b8` — Smoke-CI mit `cargo build --workspace --locked` +
  `cargo test --workspace --locked`, working-directory `inventory/`,
  SHA-pinned Actions (KPI-2), sops v3.13.1 + age via apt (für
  secrets-Tests). Alter `rust.yml` (scheiterte 13s wegen fehlendem
  inventory-Workdir) durch hardened `ci.yml` ersetzt.
- Erster CI-Run auf main: 52/52 Tests grün ✅.
- Aktive Workflows: `CI Smoke Build` + `rust-clippy analyze`.
- Identity-Schutz: alle heutigen Commits unter
  `Anonymous User <31363351+BortDeveloper@users.noreply.github.com>`
  (GitHub-noreply-Format, Squatting-frei).
- Repo-Setting-Hinweis: `has_pull_requests` war kurz auf `false`
  gerutscht (Effekt vom Public-Switch?), wieder auf `true` gesetzt
  via `gh api -X PATCH`.

**Was noch offen ist**:
- Lokale Klone (User-Workstation, `<operator-control-host>`,
  `inventory`-Tailscale-Host) müssen resynced werden — alte SHAs
  sind invalidiert nach Force-Push.
- Visibility-Wechsel auf Public ist User-Aktion in GitHub-UI.
- Sister-Repo `BortDeveloper/ansible-vps-stack` ist **noch nicht**
  durch denselben Prozess gegangen. Wenn es auch public werden soll,
  identische Drei-Phasen-Operation (Working-Tree-PRs + History-Scrub)
  nötig.
