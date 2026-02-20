use crate::core::errors::BBResult;
use crate::core::models::agent::{Agent, AgentStatus};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

pub fn upsert_agent(conn: &mut Connection, agent: &Agent) -> BBResult<()> {
    conn.execute(
        "INSERT INTO agents (id, current_task, progress, status, blockers, last_seen, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
             current_task = excluded.current_task,
             progress = excluded.progress,
             status = excluded.status,
             blockers = excluded.blockers,
             last_seen = excluded.last_seen,
             updated_at = excluded.updated_at",
        params![
            agent.id,
            agent.current_task,
            agent.progress,
            agent.status.as_str(),
            agent.blockers,
            agent.last_seen.to_rfc3339(),
            agent.updated_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn get_agent(conn: &mut Connection, id: &str) -> BBResult<Option<Agent>> {
    let mut stmt = conn.prepare(
        "SELECT id, current_task, progress, status, blockers, last_seen, updated_at
         FROM agents WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row_to_agent(row)?))
    } else {
        Ok(None)
    }
}

pub fn get_all_agents(conn: &mut Connection) -> BBResult<Vec<Agent>> {
    let mut stmt = conn.prepare(
        "SELECT id, current_task, progress, status, blockers, last_seen, updated_at
         FROM agents ORDER BY last_seen DESC",
    )?;

    let agents = stmt
        .query_map([], |row| row_to_agent(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(agents)
}

pub fn delete_offline_agents(conn: &mut Connection) -> BBResult<usize> {
    let count = conn.execute("DELETE FROM agents WHERE status = 'offline'", [])?;
    Ok(count)
}

pub fn update_offline_status(conn: &mut Connection, stale_minutes: i64) -> BBResult<usize> {
    let count = conn.execute(
        "UPDATE agents 
         SET status = 'offline' 
         WHERE status != 'offline' 
           AND datetime(last_seen) < datetime('now', ?1 || ' minutes')",
        params![format!("-{}", stale_minutes)],
    )?;
    Ok(count)
}

fn row_to_agent(row: &rusqlite::Row) -> Result<Agent, rusqlite::Error> {
    let last_seen_str: String = row.get(5)?;
    let updated_at_str: String = row.get(6)?;

    Ok(Agent {
        id: row.get(0)?,
        current_task: row.get(1)?,
        progress: row.get(2)?,
        status: AgentStatus::from_str(&row.get::<_, String>(3)?),
        blockers: row.get(4)?,
        last_seen: DateTime::parse_from_rfc3339(&last_seen_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
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
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup() -> (Connection, Agent) {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let agent = Agent {
            id: "test-agent".to_string(),
            current_task: "testing".to_string(),
            progress: 50,
            status: AgentStatus::Coding,
            blockers: None,
            last_seen: Utc::now(),
            updated_at: Utc::now(),
        };

        (conn, agent)
    }

    #[test]
    fn test_upsert_agent_insert() {
        let (mut conn, agent) = setup();

        upsert_agent(&mut conn, &agent).unwrap();

        let retrieved = get_agent(&mut conn, "test-agent").unwrap().unwrap();
        assert_eq!(retrieved.id, "test-agent");
        assert_eq!(retrieved.current_task, "testing");
        assert_eq!(retrieved.progress, 50);
    }

    #[test]
    fn test_get_agent_not_found() {
        let (mut conn, _) = setup();

        let result = get_agent(&mut conn, "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_upsert_agent_update() {
        let (mut conn, mut agent) = setup();

        upsert_agent(&mut conn, &agent).unwrap();

        agent.current_task = "updated task".to_string();
        agent.progress = 75;
        upsert_agent(&mut conn, &agent).unwrap();

        let retrieved = get_agent(&mut conn, "test-agent").unwrap().unwrap();
        assert_eq!(retrieved.current_task, "updated task");
        assert_eq!(retrieved.progress, 75);
    }

    #[test]
    fn test_get_all_agents() {
        let (mut conn, agent1) = setup();

        upsert_agent(&mut conn, &agent1).unwrap();

        let agent2 = Agent {
            id: "test-agent-2".to_string(),
            current_task: "other task".to_string(),
            progress: 25,
            status: AgentStatus::Planning,
            blockers: None,
            last_seen: Utc::now(),
            updated_at: Utc::now(),
        };
        upsert_agent(&mut conn, &agent2).unwrap();

        let agents = get_all_agents(&mut conn).unwrap();
        assert_eq!(agents.len(), 2);
    }
}
