use crate::core::models::reference::Reference;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub id: i64,
    pub path: String,
    pub produced_by: String,
    pub description: String,
    pub version: Option<String>,
    pub refs: Vec<Reference>,
    pub created_at: DateTime<Utc>,
}

impl Artifact {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_new() {
        let artifact = Artifact {
            id: 0,
            path: "src/main.rs".to_string(),
            produced_by: "agent-1".to_string(),
            description: "Main entry point".to_string(),
            version: None,
            refs: vec![],
            created_at: Utc::now(),
        };
        assert_eq!(artifact.path, "src/main.rs");
        assert_eq!(artifact.produced_by, "agent-1");
        assert_eq!(artifact.description, "Main entry point");
        assert!(artifact.version.is_none());
        assert!(artifact.refs.is_empty());
    }

    #[test]
    fn test_artifact_serialization() {
        let artifact = Artifact {
            id: 0,
            path: "src/main.rs".to_string(),
            produced_by: "agent-1".to_string(),
            description: "Main entry".to_string(),
            version: None,
            refs: vec![],
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&artifact).unwrap();
        assert!(json.contains("\"path\":\"src/main.rs\""));
        assert!(json.contains("\"produced_by\":\"agent-1\""));
    }
}
