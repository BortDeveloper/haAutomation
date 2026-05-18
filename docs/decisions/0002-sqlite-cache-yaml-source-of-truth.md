# ADR-0002: SQLite als Cache, YAML als Source of Truth

- **Status:** accepted
- **Datum:** 2026-05-17 (retroaktiv dokumentiert)
- **Bezug:** Architekt-Audit R6, FR-2

## Kontext

Die Inventardaten (Geräte, Firmware-Stände, Software-Versionen) brauchen
zweierlei: schnellen, indizierten Zugriff für die Web-UI **und** eine
git-nachvollziehbare, diffbare Historie — das Projekt ist als „Single Source
of Truth" mit GitOps-Anspruch (NFR-7) angelegt.

## Entscheidung

Duale Datenhaltung mit klarer Rangordnung:

- **YAML-Snapshots im Repo (`inventory/yaml/<source>.yaml`) sind die Source of
  Truth.** Sie werden nach jedem Sync deterministisch (nach `source_id`)
  sortiert neu geschrieben; bei Diff erfolgt `git commit`/`git push`.
- **SQLite (`/var/lib/inventory/inventory.db`) ist Cache und Index.** Die
  Web-UI liest ausschließlich aus SQLite; die DB ist jederzeit aus den
  YAML-Dateien reproduzierbar.
- Manuelle Daten (`manual.yaml`) sind ebenfalls Source of Truth und werden von
  Schema-Migrations **nie** angefasst.

## Folgen

**Positiv**

- Vollständige git-Historie aller Geräte-/Firmware-Änderungen, ohne separate
  Zeitreihen-DB.
- Restore = YAML auschecken, DB neu aufbauen. Backups der `.db` sind dadurch
  unkritisch (Cache).

**Negativ / Kosten**

- Doppelte Datenhaltung — die Konsistenz zwischen YAML und SQLite muss die App
  garantieren (Upsert + deterministischer Export in einem Lauf).
- Schema-Migrations betreffen nur SQLite; YAML hat kein Schema-Versioning.
