use crate::core::errors::BBResult;
use crate::core::models::message::{Message, Priority};
use crate::core::models::reference::Reference;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde_json::Value as JsonValue;

pub fn insert_message(conn: &mut Connection, message: &Message) -> BBResult<i64> {
    let tags_json = serde_json::to_string(&message.tags)?;
    let refs_json = serde_json::to_string(&message.refs)?;

    conn.execute(
        "INSERT INTO messages (from_agent, content, tags, priority, in_reply_to, refs, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            message.from_agent,
            message.content,
            tags_json,
            message.priority.as_str(),
            message.in_reply_to,
            refs_json,
            message.created_at.to_rfc3339()
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn get_message(conn: &mut Connection, id: i64) -> BBResult<Option<Message>> {
    let mut stmt = conn.prepare(
        "SELECT id, from_agent, content, tags, priority, in_reply_to, refs, created_at
         FROM messages WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row_to_message(row)?))
    } else {
        Ok(None)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn list_messages(
    conn: &mut Connection,
    since: Option<DateTime<Utc>>,
    tags: &[String],
    from_agent: Option<&str>,
    priority: Option<Priority>,
    ref_where: Option<&str>,
    ref_what: Option<&str>,
    ref_ref: Option<&str>,
    limit: usize,
) -> BBResult<Vec<Message>> {
    let limit = limit.min(100);

    let mut sql = String::from(
        "SELECT DISTINCT m.id, m.from_agent, m.content, m.tags, m.priority, m.in_reply_to, m.refs, m.created_at
         FROM messages m WHERE 1=1"
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(since) = since {
        sql.push_str(" AND m.created_at >= ?");
        params.push(Box::new(since.to_rfc3339()));
    }

    if let Some(from) = from_agent {
        sql.push_str(" AND m.from_agent = ?");
        params.push(Box::new(from.to_string()));
    }

    if let Some(p) = priority {
        sql.push_str(
            " AND (CASE m.priority 
            WHEN 'critical' THEN 4 
            WHEN 'high' THEN 3 
            WHEN 'normal' THEN 2 
            WHEN 'low' THEN 1 
            END) >= ?",
        );
        params.push(Box::new(p.level() as i64));
    }

    if !tags.is_empty() {
        sql.push_str(
            " AND EXISTS (
            SELECT 1 FROM json_each(m.tags) 
            WHERE value IN (",
        );
        sql.push_str(&tags.iter().map(|_| "?").collect::<Vec<_>>().join(", "));
        sql.push_str("))");
        for tag in tags {
            params.push(Box::new(tag.clone()));
        }
    }

    // Reference filtering
    if let (Some(where_), Some(what), Some(ref_val)) = (ref_where, ref_what, ref_ref) {
        sql.push_str(
            " AND EXISTS (
            SELECT 1 FROM json_each(m.refs) 
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

    sql.push_str(" ORDER BY m.created_at DESC LIMIT ?");
    params.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let messages = stmt
        .query_map(&param_refs[..], row_to_message)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

pub fn find_messages_by_ref(
    conn: &mut Connection,
    where_: &str,
    what: &str,
    ref_: &JsonValue,
) -> BBResult<Vec<Message>> {
    // Cast both sides to text for consistent comparison
    let ref_param = match ref_ {
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => s.clone(),
        _ => ref_.to_string(),
    };

    let mut stmt = conn.prepare(
        "SELECT m.id, m.from_agent, m.content, m.tags, m.priority, m.in_reply_to, m.refs, m.created_at
         FROM messages m
         WHERE EXISTS (
             SELECT 1 FROM json_each(m.refs)
             WHERE json_extract(value, '$.where') = ?1
               AND json_extract(value, '$.what') = ?2
               AND CAST(json_extract(value, '$.ref') AS TEXT) = CAST(?3 AS TEXT)
         )
         ORDER BY m.created_at DESC"
    )?;

    let messages = stmt
        .query_map(params![where_, what, ref_param], row_to_message)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

pub fn get_message_replies(conn: &mut Connection, message_id: i64) -> BBResult<Vec<Message>> {
    let mut stmt = conn.prepare(
        "SELECT id, from_agent, content, tags, priority, in_reply_to, refs, created_at
         FROM messages
         WHERE in_reply_to = ?1
         ORDER BY created_at ASC
         LIMIT 50",
    )?;

    let messages = stmt
        .query_map(params![message_id], row_to_message)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

pub fn delete_messages_before(conn: &mut Connection, before: DateTime<Utc>) -> BBResult<usize> {
    let count = conn.execute(
        "DELETE FROM messages WHERE created_at < ?1",
        params![before.to_rfc3339()],
    )?;
    Ok(count)
}

fn row_to_message(row: &rusqlite::Row) -> Result<Message, rusqlite::Error> {
    let tags_json: String = row.get(3)?;
    let refs_json: String = row.get(6)?;
    let created_at_str: String = row.get(7)?;

    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let refs: Vec<Reference> = serde_json::from_str(&refs_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(Message {
        id: row.get(0)?,
        from_agent: row.get(1)?,
        content: row.get(2)?,
        tags,
        priority: Priority::parse(&row.get::<_, String>(4)?),
        in_reply_to: row.get(5)?,
        refs,
        created_at: DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    7,
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

    fn create_test_message(id: &str) -> Message {
        Message {
            id: 0,
            from_agent: id.to_string(),
            content: "test content".to_string(),
            tags: vec!["test".to_string()],
            priority: Priority::Normal,
            in_reply_to: None,
            refs: vec![],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_insert_and_get_message() {
        let mut conn = setup();
        let message = create_test_message("agent-1");

        let id = insert_message(&mut conn, &message).unwrap();
        assert!(id > 0);

        let retrieved = get_message(&mut conn, id).unwrap().unwrap();
        assert_eq!(retrieved.from_agent, "agent-1");
        assert_eq!(retrieved.content, "test content");
    }

    #[test]
    fn test_list_messages_with_tags() {
        let mut conn = setup();

        let mut msg1 = create_test_message("agent-1");
        msg1.tags = vec!["decision".to_string(), "important".to_string()];
        insert_message(&mut conn, &msg1).unwrap();

        let mut msg2 = create_test_message("agent-2");
        msg2.tags = vec!["question".to_string()];
        insert_message(&mut conn, &msg2).unwrap();

        let results = list_messages(
            &mut conn,
            None,
            &["decision".to_string()],
            None,
            None,
            None,
            None,
            None,
            10,
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].tags.contains(&"decision".to_string()));
    }

    #[test]
    fn test_find_messages_by_ref() {
        let mut conn = setup();

        let mut msg = create_test_message("agent-1");
        msg.refs = vec![Reference {
            where_: "tt".to_string(),
            what: "task".to_string(),
            ref_: serde_json::json!(13),
        }];
        insert_message(&mut conn, &msg).unwrap();

        let results =
            find_messages_by_ref(&mut conn, "tt", "task", &serde_json::json!(13)).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_delete_messages_before() {
        let mut conn = setup();

        let mut msg = create_test_message("agent-1");
        msg.created_at = Utc::now() - chrono::Duration::days(10);
        insert_message(&mut conn, &msg).unwrap();

        let cutoff = Utc::now() - chrono::Duration::days(5);
        let deleted = delete_messages_before(&mut conn, cutoff).unwrap();

        assert_eq!(deleted, 1);
    }
}
