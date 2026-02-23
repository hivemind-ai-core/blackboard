use crate::core::errors::BBResult;
use crate::db::connection::with_connection;
use crate::db::schema::init_schema;
use crate::util::discovery::is_initialized;
use std::fs;
use std::path::Path;

pub fn run(project_dir: &Path) -> BBResult<()> {
    if is_initialized(project_dir) {
        println!(
            "Blackboard already initialized at {}",
            project_dir.display()
        );
        return Ok(());
    }

    let bb_dir = project_dir.join(".bb");

    // Check if .bb directory exists but DB is missing/corrupt
    if bb_dir.exists() {
        if !bb_dir.join("blackboard.db").exists() {
            eprintln!("Warning: .bb/ exists but database is missing. Recreating...");
        }
    } else {
        fs::create_dir(&bb_dir)?;
    }

    // Create gitignore
    fs::write(bb_dir.join(".gitignore"), "*\n")?;

    // Initialize database schema
    with_connection(project_dir, |conn| init_schema(conn))?;

    println!("Initialized blackboard at {}/.bb/", project_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_bb_directory() {
        let temp = TempDir::new().unwrap();

        run(temp.path()).unwrap();

        assert!(temp.path().join(".bb").exists());
        assert!(temp.path().join(".bb/blackboard.db").exists());
        assert!(temp.path().join(".bb/.gitignore").exists());
    }

    #[test]
    fn test_init_creates_gitignore_with_star() {
        let temp = TempDir::new().unwrap();

        run(temp.path()).unwrap();

        let gitignore = fs::read_to_string(temp.path().join(".bb/.gitignore")).unwrap();
        assert_eq!(gitignore, "*\n");
    }

    #[test]
    fn test_init_already_initialized() {
        let temp = TempDir::new().unwrap();

        // First init
        run(temp.path()).unwrap();

        // Second init should succeed (no-op)
        run(temp.path()).unwrap();

        assert!(temp.path().join(".bb").exists());
    }
}
