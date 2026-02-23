use crate::core::errors::{BBError, BBResult};
use rusqlite::Connection;
use std::path::Path;

pub fn with_connection<F, T>(project_dir: &Path, f: F) -> BBResult<T>
where
    F: FnOnce(&mut Connection) -> BBResult<T>,
{
    let bb_dir = project_dir.join(".bb");
    if !bb_dir.exists() {
        return Err(BBError::NotInitialized);
    }

    let db_path = bb_dir.join("blackboard.db");
    let mut conn = Connection::open(&db_path)?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;",
    )?;

    let result = f(&mut conn)?;

    conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE)")?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_with_connection_creates_db() {
        let temp = TempDir::new().unwrap();
        let bb_dir = temp.path().join(".bb");
        fs::create_dir(&bb_dir).unwrap();

        let result = with_connection(temp.path(), |conn| {
            let version: String =
                conn.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;
            Ok(version)
        });

        assert!(result.is_ok());
        assert!(bb_dir.join("blackboard.db").exists());
    }

    #[test]
    fn test_connection_pragmas() {
        let temp = TempDir::new().unwrap();
        let bb_dir = temp.path().join(".bb");
        fs::create_dir(&bb_dir).unwrap();

        with_connection(temp.path(), |conn| {
            let journal_mode: String =
                conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
            assert!(journal_mode.to_lowercase().contains("wal"));

            let foreign_keys: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
            assert_eq!(foreign_keys, 1);

            Ok(())
        })
        .unwrap();
    }
}
