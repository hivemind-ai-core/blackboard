use crate::core::errors::{BBError, BBResult};
use std::path::{Path, PathBuf};

pub const MAX_AGENT_ID_LEN: usize = 64;
pub const MAX_TASK_LEN: usize = 256;
pub const MAX_BLOCKERS_LEN: usize = 1024;
pub const MAX_MESSAGE_CONTENT_LEN: usize = 65536;
pub const MAX_ARTIFACT_PATH_LEN: usize = 4096;
pub const MAX_ARTIFACT_DESC_LEN: usize = 1024;
pub const MAX_VERSION_LEN: usize = 64;
pub const MAX_TAG_LEN: usize = 32;
pub const MAX_TAGS_PER_MESSAGE: usize = 10;
pub const MAX_REFS_PER_ENTITY: usize = 20;

pub fn validate_agent_id(id: &str) -> BBResult<()> {
    if id.is_empty() {
        return Err(BBError::InvalidInput(
            "agent ID cannot be empty".to_string(),
        ));
    }
    if id.len() > MAX_AGENT_ID_LEN {
        return Err(BBError::InvalidInput(format!(
            "agent ID too long (max {} chars)",
            MAX_AGENT_ID_LEN
        )));
    }
    // Check for control characters
    if id.chars().any(|c| c.is_control()) {
        return Err(BBError::InvalidInput(
            "agent ID contains control characters".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_task(task: &str) -> BBResult<()> {
    if task.len() > MAX_TASK_LEN {
        return Err(BBError::InvalidInput(format!(
            "task too long (max {} chars)",
            MAX_TASK_LEN
        )));
    }
    Ok(())
}

pub fn validate_blockers(blockers: &str) -> BBResult<()> {
    if blockers.len() > MAX_BLOCKERS_LEN {
        return Err(BBError::InvalidInput(format!(
            "blockers too long (max {} chars)",
            MAX_BLOCKERS_LEN
        )));
    }
    Ok(())
}

pub fn validate_message_content(content: &str) -> BBResult<()> {
    if content.is_empty() {
        return Err(BBError::InvalidInput(
            "message content cannot be empty".to_string(),
        ));
    }
    if content.len() > MAX_MESSAGE_CONTENT_LEN {
        return Err(BBError::InvalidInput(format!(
            "message content too long (max {} chars)",
            MAX_MESSAGE_CONTENT_LEN
        )));
    }
    Ok(())
}

pub fn validate_artifact_path(path: &str, project_root: &Path) -> BBResult<PathBuf> {
    if path.is_empty() {
        return Err(BBError::InvalidInput(
            "artifact path cannot be empty".to_string(),
        ));
    }
    if path.len() > MAX_ARTIFACT_PATH_LEN {
        return Err(BBError::InvalidInput(format!(
            "artifact path too long (max {} chars)",
            MAX_ARTIFACT_PATH_LEN
        )));
    }

    // Reject absolute paths
    if Path::new(path).is_absolute() {
        return Err(BBError::PathTraversal(
            "absolute path not allowed".to_string(),
        ));
    }

    // Reject path traversal attempts
    if path.contains("..") {
        return Err(BBError::PathTraversal(
            "path traversal not allowed".to_string(),
        ));
    }

    // Validate path doesn't escape project directory
    let full_path = project_root.join(path);
    let canonical = full_path
        .canonicalize()
        .map_err(|_| BBError::InvalidInput(format!("invalid path: {}", path)))?;

    let project_canonical = project_root
        .canonicalize()
        .map_err(|_| BBError::InvalidInput("invalid project directory".to_string()))?;

    if !canonical.starts_with(&project_canonical) {
        return Err(BBError::PathTraversal(
            "path escapes project directory".to_string(),
        ));
    }

    Ok(full_path)
}

pub fn validate_artifact_description(desc: &str) -> BBResult<()> {
    if desc.len() > MAX_ARTIFACT_DESC_LEN {
        return Err(BBError::InvalidInput(format!(
            "artifact description too long (max {} chars)",
            MAX_ARTIFACT_DESC_LEN
        )));
    }
    Ok(())
}

pub fn validate_version(version: &str) -> BBResult<()> {
    if version.len() > MAX_VERSION_LEN {
        return Err(BBError::InvalidInput(format!(
            "version too long (max {} chars)",
            MAX_VERSION_LEN
        )));
    }
    Ok(())
}

pub fn validate_tags(tags: &[String]) -> BBResult<()> {
    if tags.len() > MAX_TAGS_PER_MESSAGE {
        return Err(BBError::InvalidInput(format!(
            "too many tags (max {})",
            MAX_TAGS_PER_MESSAGE
        )));
    }

    for tag in tags {
        if tag.is_empty() {
            return Err(BBError::InvalidInput("tag cannot be empty".to_string()));
        }
        if tag.len() > MAX_TAG_LEN {
            return Err(BBError::InvalidInput(format!(
                "tag too long (max {} chars): {}",
                MAX_TAG_LEN, tag
            )));
        }
        if tag.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(BBError::InvalidInput(format!(
                "tag contains invalid characters: {}",
                tag
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_agent_id_valid() {
        assert!(validate_agent_id("agent-1").is_ok());
        assert!(validate_agent_id("claude-01").is_ok());
    }

    #[test]
    fn test_validate_agent_id_empty() {
        assert!(validate_agent_id("").is_err());
    }

    #[test]
    fn test_validate_agent_id_too_long() {
        let long_id = "a".repeat(MAX_AGENT_ID_LEN + 1);
        assert!(validate_agent_id(&long_id).is_err());
    }

    #[test]
    fn test_validate_agent_id_control_chars() {
        assert!(validate_agent_id("agent\x00name").is_err());
    }

    #[test]
    fn test_validate_task_too_long() {
        let long_task = "a".repeat(MAX_TASK_LEN + 1);
        assert!(validate_task(&long_task).is_err());
    }

    #[test]
    fn test_validate_task_valid() {
        assert!(validate_task("Short task").is_ok());
    }

    #[test]
    fn test_validate_blockers_too_long() {
        let long_blockers = "a".repeat(MAX_BLOCKERS_LEN + 1);
        assert!(validate_blockers(&long_blockers).is_err());
    }

    #[test]
    fn test_validate_message_content_empty() {
        assert!(validate_message_content("").is_err());
    }

    #[test]
    fn test_validate_message_content_too_long() {
        let long_content = "a".repeat(MAX_MESSAGE_CONTENT_LEN + 1);
        assert!(validate_message_content(&long_content).is_err());
    }

    #[test]
    fn test_validate_message_content_valid() {
        assert!(validate_message_content("Hello world").is_ok());
    }

    #[test]
    fn test_validate_artifact_path_absolute() {
        let temp = TempDir::new().unwrap();
        assert!(validate_artifact_path("/etc/passwd", temp.path()).is_err());
    }

    #[test]
    fn test_validate_artifact_path_traversal() {
        let temp = TempDir::new().unwrap();
        assert!(validate_artifact_path("../etc/passwd", temp.path()).is_err());
    }

    #[test]
    fn test_validate_artifact_path_valid() {
        let temp = TempDir::new().unwrap();
        // Create the file first since canonicalize requires it to exist
        let file_path = temp.path().join("src/main.rs");
        std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        std::fs::write(&file_path, "").unwrap();
        assert!(validate_artifact_path("src/main.rs", temp.path()).is_ok());
    }

    #[test]
    fn test_validate_version_too_long() {
        let long_version = "v".repeat(MAX_VERSION_LEN + 1);
        assert!(validate_version(&long_version).is_err());
    }

    #[test]
    fn test_validate_tags_too_many() {
        let tags: Vec<String> = (0..MAX_TAGS_PER_MESSAGE + 1)
            .map(|i| format!("tag{}", i))
            .collect();
        assert!(validate_tags(&tags).is_err());
    }

    #[test]
    fn test_validate_tags_empty_tag() {
        assert!(validate_tags(&["valid".to_string(), "".to_string()]).is_err());
    }

    #[test]
    fn test_validate_tags_with_whitespace() {
        assert!(validate_tags(&["hello world".to_string()]).is_err());
    }

    #[test]
    fn test_validate_tags_valid() {
        assert!(validate_tags(&["tag1".to_string(), "tag2".to_string()]).is_ok());
    }
}
