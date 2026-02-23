use crate::cli::output::{OutputFormat, OutputFormatter};
use crate::core::errors::BBResult;
use crate::core::models::agent::AgentStatus;
use crate::core::operations::agent as agent_ops;
use crate::core::operations::classify_liveness;
use crate::db::connection::with_connection;
use std::collections::HashMap;
use std::path::Path;

pub fn status(project_dir: &Path, format: OutputFormat) -> BBResult<()> {
    with_connection(project_dir, |conn| {
        let agents = agent_ops::get_all_agents_with_liveness(conn)?;

        let mut liveness_map = HashMap::new();
        for agent in &agents {
            let liveness = classify_liveness(agent.last_seen);
            liveness_map.insert(agent.id.clone(), liveness);
        }

        let formatter = OutputFormatter::new(format);
        print!("{}", formatter.format_agents(&agents, &liveness_map));

        Ok(())
    })
}

pub fn status_set(
    project_dir: &Path,
    agent_id: &str,
    task: &str,
    progress: Option<u8>,
    status: Option<AgentStatus>,
    blockers: Option<&str>,
) -> BBResult<()> {
    with_connection(project_dir, |conn| {
        let agent =
            agent_ops::update_agent_status(conn, agent_id, Some(task), progress, status, blockers)?;

        println!("Updated status for {}: {}", agent.id, agent.status.as_str());
        if !agent.current_task.is_empty() {
            println!("  Task: {}", agent.current_task);
        }
        if agent.progress > 0 {
            println!("  Progress: {}%", agent.progress);
        }
        if let Some(blocks) = &agent.blockers {
            println!("  Blockers: {blocks}");
        }

        Ok(())
    })
}

pub fn status_get(project_dir: &Path, agent_id: &str, format: OutputFormat) -> BBResult<()> {
    with_connection(project_dir, |conn| {
        let agent = agent_ops::get_agent(conn, agent_id)?.ok_or_else(|| {
            crate::core::errors::BBError::NotFound(format!("agent '{agent_id}' not found"))
        })?;

        let liveness = classify_liveness(agent.last_seen);
        let mut liveness_map = HashMap::new();
        liveness_map.insert(agent.id.clone(), liveness);

        let formatter = OutputFormatter::new(format);
        print!("{}", formatter.format_agents(&[agent], &liveness_map));

        Ok(())
    })
}

pub fn status_clear(project_dir: &Path, agent_id: &str) -> BBResult<()> {
    with_connection(project_dir, |conn| {
        let agent = agent_ops::clear_agent_status(conn, agent_id)?;
        println!("Cleared status for {}", agent.id);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::init;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let temp = TempDir::new().unwrap();
        init::run(temp.path()).unwrap();
        temp
    }

    #[test]
    fn test_status_set_and_get() {
        let temp = setup();

        // Set status
        status_set(
            temp.path(),
            "test-agent",
            "working on feature",
            Some(50),
            Some(AgentStatus::Coding),
            None,
        )
        .unwrap();

        // Get status
        status_get(temp.path(), "test-agent", OutputFormat::Human).unwrap();
    }

    #[test]
    fn test_status_clear() {
        let temp = setup();

        // Set status
        status_set(
            temp.path(),
            "test-agent",
            "working",
            Some(50),
            Some(AgentStatus::Coding),
            None,
        )
        .unwrap();

        // Clear status
        status_clear(temp.path(), "test-agent").unwrap();

        // Verify it's cleared
        let result = with_connection(temp.path(), |conn| agent_ops::get_agent(conn, "test-agent"));
        assert!(result.is_ok());
    }
}
