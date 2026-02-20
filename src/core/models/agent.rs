use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub current_task: String,
    pub progress: u8,
    pub status: AgentStatus,
    pub blockers: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Agent {
    pub fn new(id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            current_task: String::new(),
            progress: 0,
            status: AgentStatus::Idle,
            blockers: None,
            last_seen: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Planning,
    Coding,
    Testing,
    Reviewing,
    Blocked,
    Offline,
}

impl AgentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Planning => "planning",
            Self::Coding => "coding",
            Self::Testing => "testing",
            Self::Reviewing => "reviewing",
            Self::Blocked => "blocked",
            Self::Offline => "offline",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "idle" => Self::Idle,
            "planning" => Self::Planning,
            "coding" => Self::Coding,
            "testing" => Self::Testing,
            "reviewing" => Self::Reviewing,
            "blocked" => Self::Blocked,
            "offline" => Self::Offline,
            _ => Self::Idle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Liveness {
    Active,
    Stale,
    Offline,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_new() {
        let agent = Agent::new("test-agent");
        assert_eq!(agent.id, "test-agent");
        assert_eq!(agent.current_task, "");
        assert_eq!(agent.progress, 0);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert!(agent.blockers.is_none());
    }

    #[test]
    fn test_agent_status_as_str() {
        assert_eq!(AgentStatus::Idle.as_str(), "idle");
        assert_eq!(AgentStatus::Planning.as_str(), "planning");
        assert_eq!(AgentStatus::Coding.as_str(), "coding");
        assert_eq!(AgentStatus::Testing.as_str(), "testing");
        assert_eq!(AgentStatus::Reviewing.as_str(), "reviewing");
        assert_eq!(AgentStatus::Blocked.as_str(), "blocked");
        assert_eq!(AgentStatus::Offline.as_str(), "offline");
    }

    #[test]
    fn test_agent_status_from_str() {
        assert_eq!(AgentStatus::from_str("idle"), AgentStatus::Idle);
        assert_eq!(AgentStatus::from_str("planning"), AgentStatus::Planning);
        assert_eq!(AgentStatus::from_str("coding"), AgentStatus::Coding);
        assert_eq!(AgentStatus::from_str("testing"), AgentStatus::Testing);
        assert_eq!(AgentStatus::from_str("reviewing"), AgentStatus::Reviewing);
        assert_eq!(AgentStatus::from_str("blocked"), AgentStatus::Blocked);
        assert_eq!(AgentStatus::from_str("offline"), AgentStatus::Offline);
    }

    #[test]
    fn test_agent_status_case_insensitive() {
        assert_eq!(AgentStatus::from_str("IDLE"), AgentStatus::Idle);
        assert_eq!(AgentStatus::from_str("Coding"), AgentStatus::Coding);
    }

    #[test]
    fn test_agent_status_default() {
        assert_eq!(AgentStatus::from_str("unknown"), AgentStatus::Idle);
    }

    #[test]
    fn test_agent_status_serialization() {
        let status = AgentStatus::Coding;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"coding\"");
    }

    #[test]
    fn test_agent_status_deserialization() {
        let status: AgentStatus = serde_json::from_str("\"blocked\"").unwrap();
        assert_eq!(status, AgentStatus::Blocked);
    }

    #[test]
    fn test_liveness_enum() {
        assert_eq!(Liveness::Active, Liveness::Active);
        assert_ne!(Liveness::Active, Liveness::Stale);
    }

    #[test]
    fn test_agent_serialization() {
        let agent = Agent::new("test");
        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains("\"id\":\"test\""));
        assert!(json.contains("\"status\":\"idle\""));
    }
}
