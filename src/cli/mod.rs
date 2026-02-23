pub mod commands;
pub mod output;

use crate::core::errors::BBResult;
use crate::core::models::agent::AgentStatus;
use crate::core::models::message::Priority;
use crate::util::discovery::find_blackboard_dir;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bb")]
#[command(about = "Blackboard - Local coordination for AI agents")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Agent identity (default: human)
    #[arg(long, global = true)]
    pub as_: Option<String>,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Explicit project directory
    #[arg(long, global = true)]
    pub dir: Option<PathBuf>,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new blackboard
    Init,

    /// Print MCP installation instructions
    Install {
        /// Agent tool (claude, kimi, kilo)
        #[arg(long = "tool")]
        tool: Option<String>,

        /// Install for global configuration
        #[arg(long)]
        global: bool,

        /// Install for local project configuration
        #[arg(long)]
        local: bool,
    },

    /// Remove the blackboard (use with caution)
    Destroy {
        /// Confirm destruction
        #[arg(long)]
        confirm: bool,
    },

    /// Show agent status
    Status {
        #[command(subcommand)]
        command: Option<StatusCommands>,
    },

    /// Show message log
    Log {
        /// Show messages since duration (e.g., 10m, 1h, 2d)
        #[arg(long)]
        since: Option<String>,

        /// Filter by tag (repeatable)
        #[arg(long = "tag")]
        tags: Vec<String>,

        /// Filter by agent
        #[arg(long)]
        from: Option<String>,

        /// Filter by minimum priority
        #[arg(long)]
        priority: Option<Priority>,

        /// Filter by reference (where:what:ref)
        #[arg(long)]
        ref_where: Option<String>,

        #[arg(long)]
        ref_what: Option<String>,

        #[arg(long)]
        ref_ref: Option<String>,

        /// Limit number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Post a message
    Post {
        /// Message content
        content: String,

        /// Attach tags (repeatable)
        #[arg(long = "tag")]
        tags: Vec<String>,

        /// Set priority
        #[arg(long, default_value = "normal")]
        priority: Priority,

        /// Reply to message ID
        #[arg(long)]
        reply_to: Option<i64>,

        /// Attach references (where:what:ref, repeatable)
        #[arg(long = "ref")]
        refs: Vec<String>,
    },

    /// Show a specific message and its thread
    Message {
        /// Message ID
        id: i64,
    },

    /// List artifacts
    Artifacts {
        /// Filter by producer
        #[arg(long)]
        by: Option<String>,

        /// Filter by reference
        #[arg(long)]
        ref_where: Option<String>,

        #[arg(long)]
        ref_what: Option<String>,

        #[arg(long)]
        ref_ref: Option<String>,

        /// Limit results
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Add or update an artifact
    ArtifactAdd {
        /// File path
        path: String,

        /// Description
        description: String,

        /// Version
        #[arg(long)]
        version: Option<String>,

        /// References (repeatable)
        #[arg(long = "ref")]
        refs: Vec<String>,
    },

    /// Show artifact details
    ArtifactShow {
        /// File path
        path: String,
    },

    /// Find references
    Refs {
        /// Reference (where:what:ref)
        reference: String,
    },

    /// Clear data
    Clear {
        /// Delete messages before duration
        #[arg(long)]
        messages_before: Option<String>,

        /// Remove offline agents
        #[arg(long)]
        reset_offline: bool,

        /// Clear artifacts
        #[arg(long)]
        artifacts: bool,

        /// Confirm without prompting
        #[arg(long)]
        confirm: bool,
    },

    /// Export all data as JSON
    Export,

    /// Show summary
    Summary,

    /// Run MCP server
    Mcp {
        /// Agent ID for MCP mode
        #[arg(long)]
        agent: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum StatusCommands {
    /// Set your status
    Set {
        /// Current task description
        task: String,

        /// Progress (0-100)
        #[arg(long)]
        progress: Option<u8>,

        /// Status
        #[arg(long)]
        status: Option<AgentStatus>,

        /// Blockers
        #[arg(long)]
        blockers: Option<String>,
    },

    /// Get status for a specific agent
    Get {
        /// Agent ID
        agent_id: String,
    },

    /// Clear your status
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallTool {
    Claude,
    Kimi,
    Kilo,
}

impl InstallTool {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "kimi" => Some(Self::Kimi),
            "kilo" => Some(Self::Kilo),
            _ => None,
        }
    }
}

pub fn get_project_dir(dir_arg: Option<PathBuf>) -> BBResult<PathBuf> {
    if let Some(dir) = dir_arg {
        return Ok(dir);
    }

    if let Some(bb_dir) = find_blackboard_dir(&std::env::current_dir()?) {
        // .bb/ is the parent of the blackboard dir
        Ok(bb_dir.parent().unwrap().to_path_buf())
    } else {
        Ok(std::env::current_dir()?)
    }
}
