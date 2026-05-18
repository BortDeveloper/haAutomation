use crate::types::Device;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

/// Liste aller bekannten Migrations, geordnet nach Version.
/// Hinzufuegen: neue SQL-Datei unter migrations/, hier einen Eintrag anhaengen.
const MIGRATIONS: &[(i32, &str, &str)] = &[(
    1,
    "0001_init",
    include_str!("../migrations/0001_init.sql"),
)];

/// Oeffnet (oder legt an) die SQLite-Datei und setzt sinnvolle Pragmas.
pub fn open(path: impl AsRef<Path>) -> Result<Connection> {
    let path = path.as_ref();
    let conn = Connection::open(path)
        .with_context(|| format!("oeffnen von {}", path.display()))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON; \
         PRAGMA journal_mode = WAL; \
         PRAGMA synchronous = NORMAL;",
    )?;
    Ok(conn)
}

/// Wendet alle noch nicht aufgespielten Migrations an. Idempotent.
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    for (version, name, sql) in MIGRATIONS {
        let already: bool = conn
            .query_row(
                "SELECT 1 FROM schema_migrations WHERE version = ?1",
                params![version],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if already {
            continue;
        }
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)
            .with_context(|| format!("migration {} fehlgeschlagen", name))?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            params![version, name],
        )?;
        tx.commit()?;
    }
    Ok(())
}

/// Fuegt Geraete ein oder aktualisiert sie (Natural Key: source + source_id).
/// Aktualisiert last_seen und active=1 bei jedem Aufruf.
pub fn upsert_devices(conn: &Connection, devices: &[Device]) -> Result<usize> {
    let tx = conn.unchecked_transaction()?;
    let mut count = 0usize;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO devices (source, source_id, name, manufacturer, model, kind, room, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)
             ON CONFLICT(source, source_id) DO UPDATE SET
                name = excluded.name,
                manufacturer = excluded.manufacturer,
                model = excluded.model,
                kind = excluded.kind,
                room = excluded.room,
                last_seen = CURRENT_TIMESTAMP,
                active = 1",
        )?;
        for d in devices {
            stmt.execute(params![
                d.source, d.source_id, d.name, d.manufacturer, d.model, d.kind, d.room
            ])?;
            count += 1;
        }
    }
    tx.commit()?;
    Ok(count)
}

/// Fuegt nur dann einen neuen firmware_snapshot ein, wenn sich der Firmware-Stand
/// gegenueber dem letzten Snapshot dieses Geraets unterscheidet (oder noch keiner
/// existiert). Gibt true zurueck, wenn ein Snapshot geschrieben wurde.
pub fn record_firmware_if_changed(
    conn: &Connection,
    source: &str,
    source_id: &str,
    firmware: &str,
) -> Result<bool> {
    let device_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM devices WHERE source = ?1 AND source_id = ?2",
            params![source, source_id],
            |r| r.get(0),
        )
        .optional()?;

    let Some(device_id) = device_id else {
        return Ok(false);
    };

    let latest: Option<String> = conn
        .query_row(
            "SELECT firmware FROM firmware_snapshot
             WHERE device_id = ?1
             ORDER BY observed_at DESC, id DESC LIMIT 1",
            params![device_id],
            |r| r.get(0),
        )
        .optional()?;

    if latest.as_deref() == Some(firmware) {
        return Ok(false);
    }

    conn.execute(
        "INSERT INTO firmware_snapshot (device_id, firmware) VALUES (?1, ?2)",
        params![device_id, firmware],
    )?;
    Ok(true)
}

/// Liefert alle Geraete, sortiert nach (source, source_id) fuer Determinismus.
pub fn list_devices(conn: &Connection) -> Result<Vec<Device>> {
    let mut stmt = conn.prepare(
        "SELECT source, source_id, name, manufacturer, model, kind, room
         FROM devices
         ORDER BY source, source_id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Device {
            source: r.get(0)?,
            source_id: r.get(1)?,
            name: r.get(2)?,
            manufacturer: r.get(3)?,
            model: r.get(4)?,
            kind: r.get(5)?,
            room: r.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml_io;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    fn fresh() -> Connection {
        Connection::open(":memory:").unwrap()
    }

    #[test]
    fn migrate_creates_all_tables() {
        let conn = fresh();
        migrate(&conn).unwrap();

        let expected = [
            "devices",
            "firmware_snapshot",
            "software",
            "manual_meta",
            "schema_migrations",
        ];
        for table in expected {
            let found: Option<String> = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                    params![table],
                    |r| r.get(0),
                )
                .ok();
            assert_eq!(found.as_deref(), Some(table), "Tabelle {} fehlt", table);
        }
    }

    #[test]
    fn migrate_creates_firmware_index() {
        let conn = fresh();
        migrate(&conn).unwrap();
        let idx: Option<String> = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='index' AND name='firmware_snapshot_device_idx'",
                [],
                |r| r.get(0),
            )
            .ok();
        assert!(idx.is_some(), "Index firmware_snapshot_device_idx fehlt");
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = fresh();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, MIGRATIONS.len() as i64);
    }

    fn sorted(mut v: Vec<Device>) -> Vec<Device> {
        v.sort_by(|a, b| (&a.source, &a.source_id).cmp(&(&b.source, &b.source_id)));
        v
    }

    #[test]
    fn roundtrip_devices_from_fixture() {
        let conn = fresh();
        migrate(&conn).unwrap();

        let input = yaml_io::load_devices(fixture("devices.yaml")).unwrap();
        assert_eq!(input.len(), 3, "Fixture muss 3 Geraete liefern");

        let n = upsert_devices(&conn, &input).unwrap();
        assert_eq!(n, 3);

        let read = list_devices(&conn).unwrap();
        assert_eq!(read.len(), 3);
        assert_eq!(sorted(input), sorted(read));
    }

    #[test]
    fn upsert_is_idempotent_no_duplicates() {
        let conn = fresh();
        migrate(&conn).unwrap();
        let input = yaml_io::load_devices(fixture("devices.yaml")).unwrap();
        upsert_devices(&conn, &input).unwrap();
        upsert_devices(&conn, &input).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM devices", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 3, "doppelter Lauf darf keine Duplikate erzeugen");
    }

    fn make_test_device(source: &str, id: &str) -> Device {
        Device {
            source: source.into(),
            source_id: id.into(),
            name: "Test".into(),
            manufacturer: None,
            model: None,
            kind: None,
            room: None,
        }
    }

    #[test]
    fn firmware_first_call_inserts() {
        let conn = fresh();
        migrate(&conn).unwrap();
        upsert_devices(&conn, &[make_test_device("ccu", "ABC123")]).unwrap();

        let inserted = record_firmware_if_changed(&conn, "ccu", "ABC123", "1.0").unwrap();
        assert!(inserted);
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM firmware_snapshot", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn firmware_unchanged_does_not_insert() {
        let conn = fresh();
        migrate(&conn).unwrap();
        upsert_devices(&conn, &[make_test_device("ccu", "ABC123")]).unwrap();

        record_firmware_if_changed(&conn, "ccu", "ABC123", "1.0").unwrap();
        let inserted = record_firmware_if_changed(&conn, "ccu", "ABC123", "1.0").unwrap();
        assert!(!inserted);
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM firmware_snapshot", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn firmware_change_inserts_new_snapshot() {
        let conn = fresh();
        migrate(&conn).unwrap();
        upsert_devices(&conn, &[make_test_device("ccu", "ABC123")]).unwrap();

        record_firmware_if_changed(&conn, "ccu", "ABC123", "1.0").unwrap();
        let inserted = record_firmware_if_changed(&conn, "ccu", "ABC123", "1.1").unwrap();
        assert!(inserted);
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM firmware_snapshot", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn firmware_unknown_device_is_noop() {
        let conn = fresh();
        migrate(&conn).unwrap();
        let inserted = record_firmware_if_changed(&conn, "ccu", "NOPE", "1.0").unwrap();
        assert!(!inserted);
    }

    #[test]
    fn upsert_updates_existing_row() {
        let conn = fresh();
        migrate(&conn).unwrap();
        let mut input = yaml_io::load_devices(fixture("devices.yaml")).unwrap();
        upsert_devices(&conn, &input).unwrap();

        // Aenderung simulieren: Raum umbenennen
        input[0].room = Some("Room A renamed".into());
        upsert_devices(&conn, &input).unwrap();

        let read = list_devices(&conn).unwrap();
        let updated = read.iter().find(|d| d.source_id == "light.room_a_ceiling").unwrap();
        assert_eq!(updated.room.as_deref(), Some("Room A renamed"));
    }
}
