use std::collections::HashMap;
use serde::Serialize;
use crate::core::models::agent::{Agent, Liveness};
use crate::core::models::message::Message;
use crate::core::models::artifact::Artifact;
use crate::core::operations::reference::ReferenceResults;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Human,
    Json,
}

pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn format_agents(&self, agents: &[Agent], liveness_map: &HashMap<String, Liveness>) -> String {
        match self.format {
            OutputFormat::Human => self.format_agents_human(agents, liveness_map),
            OutputFormat::Json => self.format_agents_json(agents, liveness_map),
        }
    }

    fn format_agents_human(&self, agents: &[Agent], liveness_map: &HashMap<String, Liveness>) -> String {
        if agents.is_empty() {
            return "No agents found.\n".to_string();
        }

        let mut lines = vec![
            format!("{:<20} {:<12} {:<8} {:<30} {:<20}", 
                "AGENT", "STATUS", "PROG", "TASK", "LAST SEEN")
        ];
        lines.push("-".repeat(100));

        for agent in agents {
            let liveness = liveness_map.get(&agent.id).copied().unwrap_or(Liveness::Offline);
            let status_indicator = match liveness {
                Liveness::Active => "",
                Liveness::Stale => " [STALE]",
                Liveness::Offline => "",
            };

            let task = if agent.current_task.len() > 27 {
                format!("{}...", &agent.current_task[..27])
            } else {
                agent.current_task.clone()
            };

            let status_str = format!("{}{}", agent.status.as_str(), status_indicator);
            let last_seen = format_timestamp_human(agent.last_seen);

            lines.push(format!(
                "{:<20} {:<12} {:>3}%    {:<30} {:<20}",
                truncate(&agent.id, 20),
                status_str,
                agent.progress,
                task,
                last_seen
            ));

            if let Some(blockers) = &agent.blockers {
                lines.push(format!("  → Blockers: {blockers}"));
            }
        }

        lines.join("\n") + "\n"
    }

    fn format_agents_json(&self, agents: &[Agent], liveness_map: &HashMap<String, Liveness>) -> String {
        #[derive(Serialize)]
        struct AgentWithLiveness<'a> {
            #[serde(flatten)]
            agent: &'a Agent,
            liveness: &'static str,
            minutes_since_last_seen: i64,
        }

        let agents_with_liveness: Vec<_> = agents.iter().map(|a| {
            let liveness = liveness_map.get(&a.id).copied().unwrap_or(Liveness::Offline);
            let liveness_str = match liveness {
                Liveness::Active => "active",
                Liveness::Stale => "stale",
                Liveness::Offline => "offline",
            };
            let minutes = chrono::Utc::now().signed_duration_since(a.last_seen).num_minutes();
            
            AgentWithLiveness {
                agent: a,
                liveness: liveness_str,
                minutes_since_last_seen: minutes,
            }
        }).collect();

        serde_json::to_string_pretty(&agents_with_liveness).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn format_messages(&self, messages: &[Message]) -> String {
        match self.format {
            OutputFormat::Human => self.format_messages_human(messages),
            OutputFormat::Json => serde_json::to_string_pretty(messages).unwrap_or_else(|_| "[]".to_string()),
        }
    }

    fn format_messages_human(&self, messages: &[Message]) -> String {
        if messages.is_empty() {
            return "No messages found.\n".to_string();
        }

        let mut lines = Vec::new();

        for msg in messages.iter().rev() {
            let priority_indicator = match msg.priority {
                crate::core::models::message::Priority::Critical => "🔴 ",
                crate::core::models::message::Priority::High => "🟡 ",
                _ => "",
            };

            let time = format_timestamp_human(msg.created_at);
            let tags = if msg.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", msg.tags.join(", "))
            };

            lines.push(format!(
                "#{} {}{}{} ({})",
                msg.id, priority_indicator, msg.from_agent, tags, time
            ));

            // Format content with wrapping
            let content_lines = wrap_text(&msg.content, 80);
            for line in content_lines {
                lines.push(format!("  {line}"));
            }

            if !msg.refs.is_empty() {
                let refs_str: Vec<_> = msg.refs.iter()
                    .map(|r| format!("{}:{}:{}", r.where_, r.what, r.ref_))
                    .collect();
                lines.push(format!("  → Refs: {}", refs_str.join(", ")));
            }

            lines.push(String::new());
        }

        lines.join("\n")
    }

    pub fn format_message_thread(&self, messages: &[Message]) -> String {
        self.format_messages(messages)
    }

    pub fn format_artifacts(&self, artifacts: &[Artifact]) -> String {
        match self.format {
            OutputFormat::Human => self.format_artifacts_human(artifacts),
            OutputFormat::Json => serde_json::to_string_pretty(artifacts).unwrap_or_else(|_| "[]".to_string()),
        }
    }

    fn format_artifacts_human(&self, artifacts: &[Artifact]) -> String {
        if artifacts.is_empty() {
            return "No artifacts found.\n".to_string();
        }

        let mut lines = vec![
            format!("{:<40} {:<15} {:<30}", "PATH", "PRODUCED BY", "DESCRIPTION")
        ];
        lines.push("-".repeat(90));

        for artifact in artifacts {
            let path = if artifact.path.len() > 37 {
                format!("...{}", &artifact.path[artifact.path.len()-34..])
            } else {
                artifact.path.clone()
            };

            let desc = if artifact.description.len() > 27 {
                format!("{}...", &artifact.description[..27])
            } else {
                artifact.description.clone()
            };

            lines.push(format!(
                "{:<40} {:<15} {:<30}",
                path,
                truncate(&artifact.produced_by, 15),
                desc
            ));

            if let Some(version) = &artifact.version {
                lines.push(format!("  → Version: {version}"));
            }

            if !artifact.refs.is_empty() {
                let refs_str: Vec<_> = artifact.refs.iter()
                    .map(|r| format!("{}:{}:{}", r.where_, r.what, r.ref_))
                    .collect();
                lines.push(format!("  → Refs: {}", refs_str.join(", ")));
            }
        }

        lines.join("\n") + "\n"
    }

    pub fn format_summary(&self, summary: &SummaryData) -> String {
        match self.format {
            OutputFormat::Human => self.format_summary_human(summary),
            OutputFormat::Json => serde_json::to_string_pretty(summary).unwrap_or_else(|_| "{}".to_string()),
        }
    }

    fn format_summary_human(&self, summary: &SummaryData) -> String {
        let mut lines = vec!["Blackboard Summary".to_string(), "=".repeat(40), String::new()];

        // Active agents
        lines.push(format!("Active Agents: {}", summary.active_agents.len()));
        for agent in &summary.active_agents {
            let status_str = format!("{} ({}%)", agent.status.as_str(), agent.progress);
            lines.push(format!("  • {} - {}", agent.id, status_str));
            if let Some(blockers) = &agent.blockers {
                lines.push(format!("    ⚠ Blocked: {blockers}"));
            }
        }
        lines.push(String::new());

        // Recent messages
        lines.push(format!("Recent Messages (last 30 min): {}", summary.recent_messages.len()));
        for msg in summary.recent_messages.iter().take(5) {
            let preview = if msg.content.len() > 50 {
                format!("{}...", &msg.content[..50])
            } else {
                msg.content.clone()
            };
            lines.push(format!("  #{} {}: {}", msg.id, msg.from_agent, preview));
        }
        if summary.recent_messages.len() > 5 {
            lines.push(format!("  ... and {} more", summary.recent_messages.len() - 5));
        }
        lines.push(String::new());

        // High priority messages
        if !summary.high_priority_messages.is_empty() {
            lines.push(format!("⚠ High Priority Messages: {}", summary.high_priority_messages.len()));
            for msg in &summary.high_priority_messages {
                lines.push(format!("  #{} {}: {}", msg.id, msg.from_agent, 
                    if msg.content.len() > 40 { format!("{}...", &msg.content[..40]) } else { msg.content.clone() }));
            }
            lines.push(String::new());
        }

        // Recent artifacts
        lines.push(format!("Recent Artifacts (last hour): {}", summary.recent_artifacts.len()));
        for artifact in summary.recent_artifacts.iter().take(5) {
            lines.push(format!("  • {}", artifact.path));
        }

        lines.join("\n") + "\n"
    }

    pub fn format_ref_results(&self, results: &ReferenceResults) -> String {
        match self.format {
            OutputFormat::Human => self.format_ref_results_human(results),
            OutputFormat::Json => serde_json::to_string_pretty(results).unwrap_or_else(|_| "{}".to_string()),
        }
    }

    fn format_ref_results_human(&self, results: &ReferenceResults) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Messages: {}", results.messages.len()));
        for msg in &results.messages {
            lines.push(format!("  #{} {}: {}", msg.id, msg.from_agent,
                if msg.content.len() > 50 { format!("{}...", &msg.content[..50]) } else { msg.content.clone() }));
        }

        lines.push(String::new());
        lines.push(format!("Artifacts: {}", results.artifacts.len()));
        for artifact in &results.artifacts {
            lines.push(format!("  • {} by {}", artifact.path, artifact.produced_by));
        }

        lines.join("\n") + "\n"
    }
}

#[derive(Debug, Serialize)]
pub struct SummaryData {
    pub active_agents: Vec<Agent>,
    pub blocked_agents: Vec<Agent>,
    pub recent_messages: Vec<Message>,
    pub high_priority_messages: Vec<Message>,
    pub recent_artifacts: Vec<Artifact>,
}

fn format_timestamp_human(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else {
        format!("{}d ago", duration.num_days())
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len-3])
    } else {
        s.to_string()
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > width
            && !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(text.to_string());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_format_timestamp_human() {
        let now = chrono::Utc::now();
        assert_eq!(format_timestamp_human(now), "just now");
        assert_eq!(format_timestamp_human(now - chrono::Duration::minutes(5)), "5m ago");
        assert_eq!(format_timestamp_human(now - chrono::Duration::hours(2)), "2h ago");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world this is long", 10), "hello w...");
    }

    #[test]
    fn test_wrap_text() {
        let lines = wrap_text("this is a very long text that should be wrapped", 15);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| l.len() <= 15));
    }

    #[test]
    fn test_output_formatter_agents_human() {
        let formatter = OutputFormatter::new(OutputFormat::Human);
        let agents = vec![Agent::new("test-agent")];
        let mut liveness = HashMap::new();
        liveness.insert("test-agent".to_string(), Liveness::Active);

        let output = formatter.format_agents(&agents, &liveness);
        assert!(output.contains("test-agent"));
        assert!(output.contains("idle"));
    }

    #[test]
    fn test_output_formatter_agents_json() {
        let formatter = OutputFormatter::new(OutputFormat::Json);
        let agents = vec![Agent::new("test-agent")];
        let mut liveness = HashMap::new();
        liveness.insert("test-agent".to_string(), Liveness::Active);

        let output = formatter.format_agents(&agents, &liveness);
        assert!(output.contains("\"id\":\"test-agent\"") || output.contains("\"id\": \"test-agent\""));
        assert!(output.contains("\"liveness\":\"active\"") || output.contains("\"liveness\": \"active\""));
    }
}
