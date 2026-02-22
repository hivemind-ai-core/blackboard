use crate::core::models::reference::Reference;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub from_agent: String,
    pub content: String,
    pub tags: Vec<String>,
    pub priority: Priority,
    pub in_reply_to: Option<i64>,
    pub refs: Vec<Reference>,
    pub created_at: DateTime<Utc>,
}

impl Message {}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "normal" => Self::Normal,
            "high" => Self::High,
            "critical" => Self::Critical,
            _ => Self::Normal,
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Normal => 1,
            Self::High => 2,
            Self::Critical => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_new() {
        let msg = Message {
            id: 0,
            from_agent: "agent-1".to_string(),
            content: "Hello world".to_string(),
            tags: vec![],
            priority: Priority::Normal,
            in_reply_to: None,
            refs: vec![],
            created_at: Utc::now(),
        };
        assert_eq!(msg.from_agent, "agent-1");
        assert_eq!(msg.content, "Hello world");
        assert!(msg.tags.is_empty());
        assert_eq!(msg.priority, Priority::Normal);
        assert!(msg.in_reply_to.is_none());
        assert!(msg.refs.is_empty());
    }

    #[test]
    fn test_priority_as_str() {
        assert_eq!(Priority::Low.as_str(), "low");
        assert_eq!(Priority::Normal.as_str(), "normal");
        assert_eq!(Priority::High.as_str(), "high");
        assert_eq!(Priority::Critical.as_str(), "critical");
    }

    #[test]
    fn test_priority_from_str() {
        assert_eq!(Priority::parse("low"), Priority::Low);
        assert_eq!(Priority::parse("normal"), Priority::Normal);
        assert_eq!(Priority::parse("high"), Priority::High);
        assert_eq!(Priority::parse("critical"), Priority::Critical);
    }

    #[test]
    fn test_priority_case_insensitive() {
        assert_eq!(Priority::parse("LOW"), Priority::Low);
        assert_eq!(Priority::parse("High"), Priority::High);
    }

    #[test]
    fn test_priority_level() {
        assert_eq!(Priority::Low.level(), 0);
        assert_eq!(Priority::Normal.level(), 1);
        assert_eq!(Priority::High.level(), 2);
        assert_eq!(Priority::Critical.level(), 3);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low < Priority::Normal);
        assert!(Priority::Normal < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }

    #[test]
    fn test_priority_serialization() {
        let priority = Priority::High;
        let json = serde_json::to_string(&priority).unwrap();
        assert_eq!(json, "\"high\"");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message {
            id: 0,
            from_agent: "agent-1".to_string(),
            content: "Hello".to_string(),
            tags: vec![],
            priority: Priority::Normal,
            in_reply_to: None,
            refs: vec![],
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"from_agent\":\"agent-1\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }
}
