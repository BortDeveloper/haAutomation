use anyhow::{Context, Result};
use rusqlite::{params, Connection};
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
