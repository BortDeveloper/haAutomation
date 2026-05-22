---
name: haAutomation — akzeptierte Klar-Namen-Restspuren
description: Drei Commits auf public main tragen den Klar-Namen "<operator>"; User-Entscheid 2026-05-19: akzeptieren, kein Re-Scrub.
type: project
---
**Stand 2026-05-19**: `BortDeveloper/haAutomation` ist public, History
war heute Mittag scrubbed (HEAD `36d606bce8`, alle Authors Anonymous).
**Danach** hat der User via GitHub-Web-UI drei Commits angelegt, die
den Klar-Namen "<operator>" als Git-Author tragen:

- `c10bd6328d` — Add GitHub Actions workflow for rust-clippy analysis
- `5c6e305305` — Add GitHub Actions workflow for Rust project
- `f12e5907eb` — Add construction warning to README

**User-Entscheidung 2026-05-19**: **Variante A** — akzeptieren.
Begründung: GitHub-Username `BortDeveloper` ist eh public und über
das Profil mit dem Klar-Namen verknüpft. Re-Scrub wäre destruktiv
gewesen, Force-Push auf bereits-public-indexiertes Repo kann externe
Caches (Software Heritage, Sourcegraph etc.) nicht löschen.

**Schutz-nach-vorn**: GitHub-noreply-Email-Konvention für künftige
Commits ist in der Cockpit-Memory `feedback_template_local_separation.md`
verankert:
- `git config --global user.name "BortDeveloper"`
- `git config --global user.email "31363351+BortDeveloper@users.noreply.github.com"`
- GitHub-Settings → Emails: „Keep my email addresses private" +
  „Block command line pushes that expose my email"

**Backup-Mirror** vom 2026-05-19 Vormittag bleibt unter
`/root/repo-backups/haAutomation-pre-scrub-2026-05-19.git` (Rollback-
Option für Notfall, ohne Klar-Namen).
