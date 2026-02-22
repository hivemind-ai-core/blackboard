use chrono::Utc;
use crate::core::errors::BBResult;
use crate::core::models::message::Priority;
use crate::core::operations::message as message_ops;
use crate::core::operations::agent as agent_ops;
use crate::core::validation::duration::validate_duration;
use crate::cli::output::{OutputFormatter, OutputFormat};
use crate::db::connection::with_connection;
use crate::util::ref_::parse_ref;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub fn log(
    project_dir: &Path,
    since: Option<&str>,
    tags: Vec<String>,
    from_agent: Option<&str>,
    priority: Option<Priority>,
    ref_where: Option<&str>,
    ref_what: Option<&str>,
    ref_ref: Option<&str>,
    limit: usize,
    format: OutputFormat,
) -> BBResult<()> {
    let since_dt = if let Some(s) = since {
        let duration = validate_duration(s)?;
        Some(Utc::now() - duration)
    } else {
        None
    };

    with_connection(project_dir, |conn| {
        // Touch agent on read
        if let Some(agent) = from_agent {
            let _ = agent_ops::touch_agent(conn, agent);
        }

        let messages = message_ops::list_messages(
            conn, since_dt, &tags, from_agent, priority,
            ref_where, ref_what, ref_ref, limit
        )?;

        let formatter = OutputFormatter::new(format);
        print!("{}", formatter.format_messages(&messages));
        
        Ok(())
    })
}

pub fn post(
    project_dir: &Path,
    from_agent: &str,
    content: &str,
    tags: Vec<String>,
    priority: Priority,
    reply_to: Option<i64>,
    refs: Vec<String>,
) -> BBResult<()> {
    let parsed_refs: Result<Vec<_>, _> = refs.iter()
        .map(|r| parse_ref(r))
        .collect();
    let parsed_refs = parsed_refs?;

    with_connection(project_dir, |conn| {
        let message = message_ops::post_message(
            conn, from_agent, content, tags, priority, reply_to, parsed_refs
        )?;

        println!("Posted message #{} from {}", message.id, message.from_agent);
        
        Ok(())
    })
}

pub fn show_message(project_dir: &Path, id: i64, format: OutputFormat) -> BBResult<()> {
    with_connection(project_dir, |conn| {
        let messages = message_ops::get_message_thread(conn, id)?;

        let formatter = OutputFormatter::new(format);
        print!("{}", formatter.format_message_thread(&messages));
        
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::cli::commands::init;

    fn setup() -> TempDir {
        let temp = TempDir::new().unwrap();
        init::run(temp.path()).unwrap();
        temp
    }

    #[test]
    fn test_post_and_log() {
        let temp = setup();
        
        post(
            temp.path(),
            "test-agent",
            "Hello world",
            vec!["greeting".to_string()],
            Priority::Normal,
            None,
            vec![],
        ).unwrap();

        log(temp.path(), None, vec![], None, None, None, None, None, 10, OutputFormat::Human).unwrap();
    }

    #[test]
    fn test_post_with_ref() {
        let temp = setup();
        
        post(
            temp.path(),
            "test-agent",
            "Hello with ref",
            vec![],
            Priority::Normal,
            None,
            vec!["tt:task:13".to_string()],
        ).unwrap();

        log(temp.path(), None, vec![], None, None, Some("tt"), Some("task"), Some("13"), 10, OutputFormat::Human).unwrap();
    }
}
