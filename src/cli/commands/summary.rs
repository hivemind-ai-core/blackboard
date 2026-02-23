use crate::cli::output::{OutputFormat, OutputFormatter, SummaryData};
use crate::core::errors::BBResult;
use crate::core::models::agent::AgentStatus;
use crate::core::models::message::Priority;
use crate::core::operations::agent as agent_ops;
use crate::core::operations::artifact as artifact_ops;
use crate::core::operations::message as message_ops;
use crate::db::connection::with_connection;
use chrono::Utc;
use std::path::Path;

pub fn summary(project_dir: &Path, format: OutputFormat) -> BBResult<()> {
    let data = with_connection(project_dir, |conn| {
        // Get all agents (includes liveness side-effect)
        let agents = agent_ops::get_all_agents_with_liveness(conn)?;

        // Separate blocked agents
        let blocked_agents: Vec<_> = agents
            .iter()
            .filter(|a| a.status == AgentStatus::Blocked)
            .cloned()
            .collect();

        // Recent messages (last 30 minutes)
        let recent_since = Utc::now() - chrono::Duration::minutes(30);
        let recent_messages = message_ops::list_messages(
            conn,
            Some(recent_since),
            &[],
            None,
            None,
            None,
            None,
            None,
            20,
        )?;

        // High priority messages
        let high_priority_messages = message_ops::list_messages(
            conn,
            None,
            &[],
            None,
            Some(Priority::High),
            None,
            None,
            None,
            10,
        )?;

        // Recent artifacts (last hour)
        let artifact_since = Utc::now() - chrono::Duration::hours(1);
        let recent_artifacts = artifact_ops::list_artifacts(conn, None, None, None, None, 20)?;
        let recent_artifacts: Vec<_> = recent_artifacts
            .into_iter()
            .filter(|a| a.created_at >= artifact_since)
            .collect();

        Ok(SummaryData {
            active_agents: agents,
            blocked_agents,
            recent_messages,
            high_priority_messages,
            recent_artifacts,
        })
    })?;

    let formatter = OutputFormatter::new(format);
    print!("{}", formatter.format_summary(&data));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::init;
    use crate::cli::commands::message;
    use crate::core::models::message::Priority;
    use tempfile::TempDir;

    #[test]
    fn test_summary() {
        let temp = TempDir::new().unwrap();
        init::run(temp.path()).unwrap();

        message::post(
            temp.path(),
            "agent-1",
            "Test message",
            vec![],
            Priority::Normal,
            None,
            vec![],
        )
        .unwrap();

        summary(temp.path(), OutputFormat::Human).unwrap();
    }
}
