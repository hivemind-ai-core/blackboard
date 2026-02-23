use crate::cli::output::{OutputFormat, OutputFormatter};
use crate::core::errors::BBResult;
use crate::core::operations::reference::find_references;
use crate::db::connection::with_connection;
use crate::util::ref_::parse_ref;
use std::path::Path;

pub fn find(project_dir: &Path, ref_str: &str, format: OutputFormat) -> BBResult<()> {
    let reference = parse_ref(ref_str)?;

    with_connection(project_dir, |conn| {
        let results = find_references(conn, &reference.where_, &reference.what, &reference.ref_)?;

        let formatter = OutputFormatter::new(format);
        print!("{}", formatter.format_ref_results(&results));

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::init;
    use crate::cli::commands::{artifact, message};
    use crate::core::models::message::Priority;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let temp = TempDir::new().unwrap();
        init::run(temp.path()).unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "").unwrap();

        temp
    }

    #[test]
    fn test_find_refs() {
        let temp = setup();

        // Post message with ref
        message::post(
            temp.path(),
            "agent-1",
            "Message about task 13",
            vec![],
            Priority::Normal,
            None,
            vec!["tt:task:13".to_string()],
        )
        .unwrap();

        // Add artifact with same ref
        artifact::add(
            temp.path(),
            "src/main.rs",
            "agent-1",
            "File for task 13",
            None,
            vec!["tt:task:13".to_string()],
        )
        .unwrap();

        // Find refs
        find(temp.path(), "tt:task:13", OutputFormat::Human).unwrap();
    }
}
