use crate::core::errors::BBResult;
use crate::core::models::artifact::Artifact;
use crate::core::models::message::Message;
use crate::db::queries::artifact as artifact_queries;
use crate::db::queries::message as message_queries;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceResults {
    pub messages: Vec<Message>,
    pub artifacts: Vec<Artifact>,
}

pub fn find_references(
    conn: &mut Connection,
    where_: &str,
    what: &str,
    ref_: &JsonValue,
) -> BBResult<ReferenceResults> {
    let messages = message_queries::find_messages_by_ref(conn, where_, what, ref_)?;
    let artifacts = artifact_queries::find_artifacts_by_ref(conn, where_, what, ref_)?;

    Ok(ReferenceResults {
        messages,
        artifacts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::artifact::Artifact;
    use crate::core::models::message::{Message, Priority};
    use crate::core::models::reference::Reference;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;
    use serde_json::json;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_find_references() {
        let mut conn = setup();

        // Create message with ref
        let msg = Message {
            id: 0,
            from_agent: "agent-1".to_string(),
            content: "Test message".to_string(),
            tags: vec![],
            priority: Priority::Normal,
            in_reply_to: None,
            refs: vec![Reference {
                where_: "tt".to_string(),
                what: "task".to_string(),
                ref_: serde_json::json!(13),
            }],
            created_at: chrono::Utc::now(),
        };
        crate::db::queries::message::insert_message(&mut conn, &msg).unwrap();

        // Create artifact with same ref
        let artifact = Artifact {
            id: 0,
            path: "src/main.rs".to_string(),
            produced_by: "agent-1".to_string(),
            description: "Main file".to_string(),
            version: None,
            refs: vec![Reference {
                where_: "tt".to_string(),
                what: "task".to_string(),
                ref_: serde_json::json!(13),
            }],
            created_at: chrono::Utc::now(),
        };
        crate::db::queries::artifact::upsert_artifact(&mut conn, &artifact).unwrap();

        let results = find_references(&mut conn, "tt", "task", &json!(13)).unwrap();

        assert_eq!(results.messages.len(), 1);
        assert_eq!(results.artifacts.len(), 1);
    }

    #[test]
    fn test_find_references_no_matches() {
        let mut conn = setup();

        let results = find_references(&mut conn, "tt", "task", &json!(999)).unwrap();

        assert!(results.messages.is_empty());
        assert!(results.artifacts.is_empty());
    }
}
