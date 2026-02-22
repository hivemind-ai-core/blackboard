use crate::core::errors::BBResult;
use crate::core::models::artifact::Artifact;
use crate::core::models::reference::Reference;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde_json::Value as JsonValue;

pub fn upsert_artifact(conn: &mut Connection, artifact: &Artifact) -> BBResult<i64> {
    let refs_json = serde_json::to_string(&artifact.refs)?;

    conn.execute(
        "INSERT INTO artifacts (path, produced_by, description, version, refs, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(path) DO UPDATE SET
             produced_by = excluded.produced_by,
             description = excluded.description,
             version = excluded.version,
             refs = excluded.refs,
             created_at = excluded.created_at",
        params![
            artifact.path,
            artifact.produced_by,
            artifact.description,
            artifact.version,
            refs_json,
            artifact.created_at.to_rfc3339()
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn get_artifact_by_path(conn: &mut Connection, path: &str) -> BBResult<Option<Artifact>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, produced_by, description, version, refs, created_at
         FROM artifacts WHERE path = ?1",
    )?;

    let mut rows = stmt.query(params![path])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row_to_artifact(row)?))
    } else {
        Ok(None)
    }
}

pub fn list_artifacts(
    conn: &mut Connection,
    produced_by: Option<&str>,
    ref_where: Option<&str>,
    ref_what: Option<&str>,
    ref_ref: Option<&str>,
    limit: usize,
) -> BBResult<Vec<Artifact>> {
    let limit = limit.min(100);

    let mut sql = String::from(
        "SELECT DISTINCT a.id, a.path, a.produced_by, a.description, a.version, a.refs, a.created_at
         FROM artifacts a WHERE 1=1"
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(producer) = produced_by {
        sql.push_str(" AND a.produced_by = ?");
        params.push(Box::new(producer.to_string()));
    }

    // Reference filtering
    if let (Some(where_), Some(what), Some(ref_val)) = (ref_where, ref_what, ref_ref) {
        sql.push_str(
            " AND EXISTS (
            SELECT 1 FROM json_each(a.refs) 
            WHERE json_extract(value, '$.where') = ? 
              AND json_extract(value, '$.what') = ?
              AND json_extract(value, '$.ref') = ?)",
        );
        params.push(Box::new(where_.to_string()));
        params.push(Box::new(what.to_string()));
        // Try to parse as number, otherwise use as string
        if let Ok(num) = ref_val.parse::<i64>() {
            params.push(Box::new(num));
        } else {
            params.push(Box::new(ref_val.to_string()));
        }
    }

    sql.push_str(" ORDER BY a.created_at DESC LIMIT ?");
    params.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let artifacts = stmt
        .query_map(&param_refs[..], row_to_artifact)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(artifacts)
}

pub fn find_artifacts_by_ref(
    conn: &mut Connection,
    where_: &str,
    what: &str,
    ref_: &JsonValue,
) -> BBResult<Vec<Artifact>> {
    // Cast both sides to text for consistent comparison
    let ref_param = match ref_ {
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => s.clone(),
        _ => ref_.to_string(),
    };

    let mut stmt = conn.prepare(
        "SELECT a.id, a.path, a.produced_by, a.description, a.version, a.refs, a.created_at
         FROM artifacts a
         WHERE EXISTS (
             SELECT 1 FROM json_each(a.refs)
             WHERE json_extract(value, '$.where') = ?1
               AND json_extract(value, '$.what') = ?2
               AND CAST(json_extract(value, '$.ref') AS TEXT) = CAST(?3 AS TEXT)
         )
         ORDER BY a.created_at DESC",
    )?;

    let artifacts = stmt
        .query_map(params![where_, what, ref_param], row_to_artifact)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(artifacts)
}

pub fn clear_artifacts(conn: &mut Connection) -> BBResult<usize> {
    let count = conn.execute("DELETE FROM artifacts", [])?;
    Ok(count)
}

fn row_to_artifact(row: &rusqlite::Row) -> Result<Artifact, rusqlite::Error> {
    let refs_json: String = row.get(5)?;
    let created_at_str: String = row.get(6)?;

    let refs: Vec<Reference> = serde_json::from_str(&refs_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(Artifact {
        id: row.get(0)?,
        path: row.get(1)?,
        produced_by: row.get(2)?,
        description: row.get(3)?,
        version: row.get(4)?,
        refs,
        created_at: DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::reference::Reference;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn create_test_artifact(path: &str) -> Artifact {
        Artifact {
            id: 0,
            path: path.to_string(),
            produced_by: "agent-1".to_string(),
            description: "test artifact".to_string(),
            version: Some("v1.0".to_string()),
            refs: vec![],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_upsert_artifact() {
        let mut conn = setup();
        let artifact = create_test_artifact("src/main.rs");

        let id = upsert_artifact(&mut conn, &artifact).unwrap();
        assert!(id > 0);

        let retrieved = get_artifact_by_path(&mut conn, "src/main.rs")
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.path, "src/main.rs");
        assert_eq!(retrieved.produced_by, "agent-1");
    }

    #[test]
    fn test_upsert_updates_existing() {
        let mut conn = setup();
        let artifact = create_test_artifact("src/main.rs");

        upsert_artifact(&mut conn, &artifact).unwrap();

        let mut updated = artifact.clone();
        updated.description = "updated description".to_string();
        updated.produced_by = "agent-2".to_string();
        upsert_artifact(&mut conn, &updated).unwrap();

        let retrieved = get_artifact_by_path(&mut conn, "src/main.rs")
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.description, "updated description");
        assert_eq!(retrieved.produced_by, "agent-2");
    }

    #[test]
    fn test_list_artifacts_by_producer() {
        let mut conn = setup();

        let artifact1 = create_test_artifact("src/main.rs");
        upsert_artifact(&mut conn, &artifact1).unwrap();

        let mut artifact2 = create_test_artifact("src/lib.rs");
        artifact2.produced_by = "agent-2".to_string();
        upsert_artifact(&mut conn, &artifact2).unwrap();

        let results = list_artifacts(&mut conn, Some("agent-1"), None, None, None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/main.rs");
    }

    #[test]
    fn test_find_artifacts_by_ref() {
        let mut conn = setup();

        let mut artifact = create_test_artifact("src/main.rs");
        artifact.refs = vec![Reference {
            where_: "tt".to_string(),
            what: "task".to_string(),
            ref_: serde_json::json!(13),
        }];
        upsert_artifact(&mut conn, &artifact).unwrap();

        let results =
            find_artifacts_by_ref(&mut conn, "tt", "task", &serde_json::json!(13)).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_clear_artifacts() {
        let mut conn = setup();

        let artifact = create_test_artifact("src/main.rs");
        upsert_artifact(&mut conn, &artifact).unwrap();

        let cleared = clear_artifacts(&mut conn).unwrap();
        assert_eq!(cleared, 1);

        let retrieved = get_artifact_by_path(&mut conn, "src/main.rs").unwrap();
        assert!(retrieved.is_none());
    }
}
