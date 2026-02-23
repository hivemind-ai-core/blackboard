use crate::core::errors::BBResult;
use rusqlite::Connection;

pub const SCHEMA_SQL: &str = include_str!("../../migrations/001_initial.sql");

pub fn run_migrations(conn: &Connection) -> BBResult<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_run_migrations() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations
        run_migrations(&conn).unwrap();

        // Check that schema_version was created
        let version: i64 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);

        // Check that agents table exists
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agents'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Check that messages table exists
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='messages'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Check that artifacts table exists
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='artifacts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
