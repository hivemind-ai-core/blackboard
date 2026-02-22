use crate::core::errors::{BBError, BBResult};
use crate::core::models::message::{Message, Priority};
use crate::core::models::reference::Reference;
use crate::core::validation::limits::MAX_REFS_PER_ENTITY;
use crate::core::validation::limits::{validate_message_content, validate_tags};
use crate::db::queries::message as message_queries;
use chrono::{DateTime, Utc};
use rusqlite::Connection;

pub fn post_message(
    conn: &mut Connection,
    from_agent: &str,
    content: &str,
    tags: Vec<String>,
    priority: Priority,
    in_reply_to: Option<i64>,
    refs: Vec<Reference>,
) -> BBResult<Message> {
    validate_message_content(content)?;
    validate_tags(&tags)?;

    if refs.len() > MAX_REFS_PER_ENTITY {
        return Err(BBError::InvalidInput(format!(
            "too many refs (max {MAX_REFS_PER_ENTITY})"
        )));
    }

    // Verify in_reply_to exists if provided
    if let Some(reply_to) = in_reply_to {
        if message_queries::get_message(conn, reply_to)?.is_none() {
            return Err(BBError::NotFound(format!("message {reply_to} not found")));
        }
    }

    let message = Message {
        id: 0,
        from_agent: from_agent.to_string(),
        content: content.to_string(),
        tags,
        priority,
        in_reply_to,
        refs,
        created_at: Utc::now(),
    };

    let id = message_queries::insert_message(conn, &message)?;

    // Return the message with the new ID
    let mut result = message;
    result.id = id;
    Ok(result)
}

pub fn get_message_thread(conn: &mut Connection, id: i64) -> BBResult<Vec<Message>> {
    // First get the original message
    let mut messages = Vec::new();

    if let Some(msg) = message_queries::get_message(conn, id)? {
        messages.push(msg);

        // Get replies (capped at 50 total in the query)
        let replies = message_queries::get_message_replies(conn, id)?;
        messages.extend(replies);

        Ok(messages)
    } else {
        Err(BBError::NotFound(format!("message {id} not found")))
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
    message_queries::list_messages(
        conn, since, tags, from_agent, priority, ref_where, ref_what, ref_ref, limit,
    )
}

pub fn delete_messages_before(conn: &mut Connection, before: DateTime<Utc>) -> BBResult<usize> {
    message_queries::delete_messages_before(conn, before)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_post_message() {
        let mut conn = setup();

        let msg = post_message(
            &mut conn,
            "agent-1",
            "Hello world",
            vec!["greeting".to_string()],
            Priority::Normal,
            None,
            vec![],
        )
        .unwrap();

        assert!(msg.id > 0);
        assert_eq!(msg.from_agent, "agent-1");
        assert_eq!(msg.content, "Hello world");
        assert_eq!(msg.tags, vec!["greeting"]);
    }

    #[test]
    fn test_post_message_empty_content_fails() {
        let mut conn = setup();

        let result = post_message(
            &mut conn,
            "agent-1",
            "",
            vec![],
            Priority::Normal,
            None,
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_post_message_with_reply() {
        let mut conn = setup();

        let parent = post_message(
            &mut conn,
            "agent-1",
            "Parent message",
            vec![],
            Priority::Normal,
            None,
            vec![],
        )
        .unwrap();

        let reply = post_message(
            &mut conn,
            "agent-2",
            "Reply message",
            vec![],
            Priority::Normal,
            Some(parent.id),
            vec![],
        )
        .unwrap();

        assert_eq!(reply.in_reply_to, Some(parent.id));
    }

    #[test]
    fn test_post_message_reply_to_nonexistent_fails() {
        let mut conn = setup();

        let result = post_message(
            &mut conn,
            "agent-1",
            "Reply",
            vec![],
            Priority::Normal,
            Some(9999),
            vec![],
        );

        assert!(matches!(result, Err(BBError::NotFound(_))));
    }

    #[test]
    fn test_get_message_thread() {
        let mut conn = setup();

        let parent = post_message(
            &mut conn,
            "agent-1",
            "Parent",
            vec![],
            Priority::Normal,
            None,
            vec![],
        )
        .unwrap();

        let reply = post_message(
            &mut conn,
            "agent-2",
            "Reply",
            vec![],
            Priority::Normal,
            Some(parent.id),
            vec![],
        )
        .unwrap();

        let thread = get_message_thread(&mut conn, parent.id).unwrap();
        assert_eq!(thread.len(), 2);
        assert_eq!(thread[0].id, parent.id);
        assert_eq!(thread[1].id, reply.id);
    }

    #[test]
    fn test_list_messages_by_tag() {
        let mut conn = setup();

        post_message(
            &mut conn,
            "agent-1",
            "Message 1",
            vec!["decision".to_string()],
            Priority::Normal,
            None,
            vec![],
        )
        .unwrap();

        post_message(
            &mut conn,
            "agent-2",
            "Message 2",
            vec!["question".to_string()],
            Priority::Normal,
            None,
            vec![],
        )
        .unwrap();

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
        assert_eq!(results[0].content, "Message 1");
    }

    #[test]
    fn test_delete_messages_before() {
        let mut conn = setup();

        // Post old message
        let old_msg = Message {
            id: 0,
            from_agent: "agent-1".to_string(),
            content: "Old message".to_string(),
            tags: vec![],
            priority: Priority::Normal,
            in_reply_to: None,
            refs: vec![],
            created_at: Utc::now() - chrono::Duration::days(10),
        };
        message_queries::insert_message(&mut conn, &old_msg).unwrap();

        let cutoff = Utc::now() - chrono::Duration::days(5);
        let deleted = delete_messages_before(&mut conn, cutoff).unwrap();

        assert_eq!(deleted, 1);
    }
}
