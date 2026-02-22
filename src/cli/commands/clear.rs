use crate::core::errors::BBResult;
use crate::core::operations::{
    agent as agent_ops, artifact as artifact_ops, message as message_ops,
};
use crate::core::validation::duration::validate_duration;
use crate::db::connection::with_connection;
use chrono::Utc;
use std::io::{self, Write};
use std::path::Path;

pub fn clear(
    project_dir: &Path,
    messages_before: Option<&str>,
    reset_offline: bool,
    artifacts: bool,
    confirm: bool,
) -> BBResult<()> {
    let mut counts = ClearCounts::default();
    let mut actions = Vec::new();

    // Calculate what will be cleared
    if let Some(before) = messages_before {
        let duration = validate_duration(before)?;
        let cutoff = Utc::now() - duration;

        with_connection(project_dir, |conn| {
            // Count messages to be deleted
            let msgs = message_ops::list_messages(
                conn,
                Some(cutoff),
                &[],
                None,
                None,
                None,
                None,
                None,
                10000,
            )?;
            counts.messages = msgs.len();
            Ok(())
        })?;

        if counts.messages > 0 {
            actions.push(format!(
                "Delete {count} messages before {before}",
                count = counts.messages,
                before = before
            ));
        }
    }

    if reset_offline {
        with_connection(project_dir, |conn| {
            let agents = agent_ops::get_all_agents_with_liveness(conn)?;
            counts.offline_agents = agents
                .iter()
                .filter(|a| a.status == crate::core::models::agent::AgentStatus::Offline)
                .count();
            Ok(())
        })?;

        if counts.offline_agents > 0 {
            actions.push(format!("Delete {} offline agents", counts.offline_agents));
        }
    }

    if artifacts {
        with_connection(project_dir, |conn| {
            let arts = artifact_ops::list_artifacts(conn, None, None, None, None, 10000)?;
            counts.artifacts = arts.len();
            Ok(())
        })?;

        if counts.artifacts > 0 {
            actions.push(format!("Clear {} artifacts", counts.artifacts));
        }
    }

    // If no actions, print counts and exit
    if actions.is_empty() {
        println!("Nothing to clear.");
        return Ok(());
    }

    // Print what will be done
    println!("The following actions will be performed:");
    for action in &actions {
        println!("  • {action}");
    }

    // Confirm or prompt
    let proceed = if confirm {
        true
    } else {
        print!("Proceed? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim().to_lowercase();
        trimmed == "y" || trimmed == "yes"
    };

    if !proceed {
        println!("Aborted.");
        return Ok(());
    }

    // Perform actions
    if let Some(before) = messages_before {
        let duration = validate_duration(before)?;
        let cutoff = Utc::now() - duration;

        with_connection(project_dir, |conn| {
            let deleted = message_ops::delete_messages_before(conn, cutoff)?;
            println!("Deleted {deleted} messages");
            Ok(())
        })?;
    }

    if reset_offline {
        with_connection(project_dir, |conn| {
            let deleted = agent_ops::delete_offline_agents(conn)?;
            println!("Deleted {deleted} offline agents");
            Ok(())
        })?;
    }

    if artifacts {
        with_connection(project_dir, |conn| {
            let cleared = artifact_ops::clear_artifacts(conn)?;
            println!("Cleared {cleared} artifacts");
            Ok(())
        })?;
    }

    Ok(())
}

#[derive(Default)]
struct ClearCounts {
    messages: usize,
    offline_agents: usize,
    artifacts: usize,
}
