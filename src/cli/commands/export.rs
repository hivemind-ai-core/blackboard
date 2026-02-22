use chrono::Utc;
use serde::Serialize;
use crate::core::errors::BBResult;
use crate::core::operations::agent as agent_ops;
use crate::core::operations::message as message_ops;
use crate::core::operations::artifact as artifact_ops;
use crate::db::connection::with_connection;
use std::path::Path;

#[derive(Serialize)]
struct ExportData {
    exported_at: String,
    project_dir: String,
    agents: Vec<crate::core::models::agent::Agent>,
    messages: Vec<crate::core::models::message::Message>,
    artifacts: Vec<crate::core::models::artifact::Artifact>,
}

pub fn export(project_dir: &Path) -> BBResult<()> {
    let (agents, messages, artifacts) = with_connection(project_dir, |conn| {
        let agents = agent_ops::get_all_agents_with_liveness(conn)?;
        let messages = message_ops::list_messages(conn, None, &[], None, None, None, None, None, 10000)?;
        let artifacts = artifact_ops::list_artifacts(conn, None, None, None, None, 10000)?;
        
        Ok((agents, messages, artifacts))
    })?;

    let data = ExportData {
        exported_at: Utc::now().to_rfc3339(),
        project_dir: project_dir.to_string_lossy().to_string(),
        agents,
        messages,
        artifacts,
    };

    let json = serde_json::to_string_pretty(&data)?;
    println!("{json}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::cli::commands::init;
    use crate::cli::commands::message;
    use crate::core::models::message::Priority;

    #[test]
    fn test_export() {
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
        ).unwrap();

        export(temp.path()).unwrap();
    }
}
