use crate::core::errors::BBError;
use crate::core::errors::BBResult;
use crate::core::models::artifact::Artifact;
use crate::core::models::reference::Reference;
use crate::core::validation::limits::{
    MAX_REFS_PER_ENTITY, validate_artifact_description, validate_artifact_path, validate_version,
};
use crate::db::queries::artifact as artifact_queries;
use rusqlite::Connection;
use std::path::Path;

pub fn register_artifact(
    conn: &mut Connection,
    path: &str,
    produced_by: &str,
    description: &str,
    version: Option<&str>,
    refs: Vec<Reference>,
    project_root: &Path,
) -> BBResult<Artifact> {
    // Validate path and check it doesn't escape project
    validate_artifact_path(path, project_root)?;
    validate_artifact_description(description)?;

    if let Some(ver) = version {
        validate_version(ver)?;
    }

    if refs.len() > MAX_REFS_PER_ENTITY {
        return Err(BBError::InvalidInput(format!(
            "too many refs (max {MAX_REFS_PER_ENTITY})"
        )));
    }

    let artifact = Artifact {
        id: 0,
        path: path.to_string(),
        produced_by: produced_by.to_string(),
        description: description.to_string(),
        version: version.map(|v| v.to_string()),
        refs,
        created_at: chrono::Utc::now(),
    };

    artifact_queries::upsert_artifact(conn, &artifact)?;

    // Return the artifact (get it to get the ID)
    artifact_queries::get_artifact_by_path(conn, path)?
        .ok_or_else(|| BBError::NotFound(format!("artifact {path} not found after upsert")))
}

pub fn get_artifact(conn: &mut Connection, path: &str) -> BBResult<Option<Artifact>> {
    artifact_queries::get_artifact_by_path(conn, path)
}

pub fn list_artifacts(
    conn: &mut Connection,
    produced_by: Option<&str>,
    ref_where: Option<&str>,
    ref_what: Option<&str>,
    ref_ref: Option<&str>,
    limit: usize,
) -> BBResult<Vec<Artifact>> {
    artifact_queries::list_artifacts(conn, produced_by, ref_where, ref_what, ref_ref, limit)
}

pub fn clear_artifacts(conn: &mut Connection) -> BBResult<usize> {
    artifact_queries::clear_artifacts(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (Connection, TempDir) {
        let temp = TempDir::new().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        (conn, temp)
    }

    #[test]
    fn test_register_artifact() {
        let (mut conn, temp) = setup();

        // Create a file in the temp dir
        let file_path = temp.path().join("src/main.rs");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, "fn main() {}").unwrap();

        let artifact = register_artifact(
            &mut conn,
            "src/main.rs",
            "agent-1",
            "Main entry point",
            Some("v1.0.0"),
            vec![],
            temp.path(),
        )
        .unwrap();

        assert!(artifact.id > 0);
        assert_eq!(artifact.path, "src/main.rs");
        assert_eq!(artifact.produced_by, "agent-1");
        assert_eq!(artifact.description, "Main entry point");
        assert_eq!(artifact.version, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_register_artifact_upserts() {
        let (mut conn, temp) = setup();

        // Create a file
        let file_path = temp.path().join("src/main.rs");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, "fn main() {}").unwrap();

        // Register first time
        register_artifact(
            &mut conn,
            "src/main.rs",
            "agent-1",
            "First description",
            Some("v1.0.0"),
            vec![],
            temp.path(),
        )
        .unwrap();

        // Register again with different data
        let artifact = register_artifact(
            &mut conn,
            "src/main.rs",
            "agent-2",
            "Updated description",
            Some("v2.0.0"),
            vec![],
            temp.path(),
        )
        .unwrap();

        assert_eq!(artifact.produced_by, "agent-2");
        assert_eq!(artifact.description, "Updated description");
        assert_eq!(artifact.version, Some("v2.0.0".to_string()));
    }

    #[test]
    fn test_register_artifact_traversal_fails() {
        let (mut conn, temp) = setup();

        let result = register_artifact(
            &mut conn,
            "../etc/passwd",
            "agent-1",
            "Malicious",
            None,
            vec![],
            temp.path(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_list_artifacts_by_producer() {
        let (mut conn, temp) = setup();

        // Create files
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "").unwrap();
        fs::write(temp.path().join("src/lib.rs"), "").unwrap();

        register_artifact(
            &mut conn,
            "src/main.rs",
            "agent-1",
            "Main",
            None,
            vec![],
            temp.path(),
        )
        .unwrap();

        register_artifact(
            &mut conn,
            "src/lib.rs",
            "agent-2",
            "Lib",
            None,
            vec![],
            temp.path(),
        )
        .unwrap();

        let results = list_artifacts(&mut conn, Some("agent-1"), None, None, None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/main.rs");
    }

    #[test]
    fn test_clear_artifacts() {
        let (mut conn, temp) = setup();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "").unwrap();

        register_artifact(
            &mut conn,
            "src/main.rs",
            "agent-1",
            "Main",
            None,
            vec![],
            temp.path(),
        )
        .unwrap();

        let cleared = clear_artifacts(&mut conn).unwrap();
        assert_eq!(cleared, 1);

        let remaining = list_artifacts(&mut conn, None, None, None, None, 10).unwrap();
        assert!(remaining.is_empty());
    }
}
