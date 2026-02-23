use crate::core::errors::{BBError, BBResult};
use crate::db::migrations::run_migrations;
use crate::util::discovery::is_initialized;
use rusqlite::Connection;
use std::path::Path;

pub fn init_schema(conn: &Connection) -> BBResult<()> {
    run_migrations(conn)
}

#[allow(dead_code)]
pub fn ensure_initialized(project_dir: &Path) -> BBResult<()> {
    if !is_initialized(project_dir) {
        return Err(BBError::NotInitialized);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_initialized_true() {
        let temp = TempDir::new().unwrap();
        let bb_dir = temp.path().join(".bb");
        fs::create_dir(&bb_dir).unwrap();

        // Create DB
        let db_path = bb_dir.join("blackboard.db");
        let conn = Connection::open(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        drop(conn);

        assert!(ensure_initialized(temp.path()).is_ok());
    }

    #[test]
    fn test_ensure_initialized_false() {
        let temp = TempDir::new().unwrap();

        assert!(matches!(
            ensure_initialized(temp.path()),
            Err(BBError::NotInitialized)
        ));
    }

    #[test]
    fn test_init_schema() {
        let conn = Connection::open_in_memory().unwrap();

        init_schema(&conn).unwrap();

        // Verify tables were created
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"agents".to_string()));
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"artifacts".to_string()));
    }
}
