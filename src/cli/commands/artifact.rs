use crate::cli::output::{OutputFormat, OutputFormatter};
use crate::core::errors::BBResult;
use crate::core::operations::agent as agent_ops;
use crate::core::operations::artifact as artifact_ops;
use crate::db::connection::with_connection;
use crate::util::ref_::parse_ref;
use std::path::Path;

pub fn list(
    project_dir: &Path,
    produced_by: Option<&str>,
    ref_where: Option<&str>,
    ref_what: Option<&str>,
    ref_ref: Option<&str>,
    limit: usize,
    format: OutputFormat,
) -> BBResult<()> {
    with_connection(project_dir, |conn| {
        // Touch agent on read
        if let Some(agent) = produced_by {
            let _ = agent_ops::touch_agent(conn, agent);
        }

        let artifacts =
            artifact_ops::list_artifacts(conn, produced_by, ref_where, ref_what, ref_ref, limit)?;

        let formatter = OutputFormatter::new(format);
        print!("{}", formatter.format_artifacts(&artifacts));

        Ok(())
    })
}

pub fn add(
    project_dir: &Path,
    path: &str,
    produced_by: &str,
    description: &str,
    version: Option<&str>,
    refs: Vec<String>,
) -> BBResult<()> {
    let parsed_refs: Result<Vec<_>, _> = refs.iter().map(|r| parse_ref(r)).collect();
    let parsed_refs = parsed_refs?;

    with_connection(project_dir, |conn| {
        let artifact = artifact_ops::register_artifact(
            conn,
            path,
            produced_by,
            description,
            version,
            parsed_refs,
            project_dir,
        )?;

        println!(
            "Registered artifact: {} (ID: {})",
            artifact.path, artifact.id
        );

        Ok(())
    })
}

pub fn show(project_dir: &Path, path: &str, format: OutputFormat) -> BBResult<()> {
    with_connection(project_dir, |conn| {
        let artifact = artifact_ops::get_artifact(conn, path)?.ok_or_else(|| {
            crate::core::errors::BBError::NotFound(format!("artifact '{path}' not found"))
        })?;

        let formatter = OutputFormatter::new(format);
        print!("{}", formatter.format_artifacts(&[artifact]));

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::init;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let temp = TempDir::new().unwrap();
        init::run(temp.path()).unwrap();

        // Create a test file
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();

        temp
    }

    #[test]
    fn test_add_and_list() {
        let temp = setup();

        add(
            temp.path(),
            "src/main.rs",
            "test-agent",
            "Main entry point",
            Some("v1.0.0"),
            vec![],
        )
        .unwrap();

        list(temp.path(), None, None, None, None, 10, OutputFormat::Human).unwrap();
    }

    #[test]
    fn test_show() {
        let temp = setup();

        add(
            temp.path(),
            "src/main.rs",
            "test-agent",
            "Main entry point",
            None,
            vec![],
        )
        .unwrap();

        show(temp.path(), "src/main.rs", OutputFormat::Human).unwrap();
    }
}
