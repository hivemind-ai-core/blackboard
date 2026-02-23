use crate::cli::InstallTool;
use crate::core::errors::BBResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_allow: Option<Vec<String>>,
}

pub fn run(tool: Option<String>, global: bool, local: bool) -> BBResult<()> {
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    println!("Blackboard MCP Installation");
    println!("==========================\n");

    let agent_types: Vec<&str> = match tool.as_ref().and_then(|t| InstallTool::from_str(t)) {
        Some(InstallTool::Claude) => vec!["claude"],
        Some(InstallTool::Kimi) => vec!["kimi"],
        Some(InstallTool::Kilo) => vec!["kilo"],
        None => vec!["claude", "kimi", "kilo"],
    };

    for agent in agent_types {
        match agent {
            "claude" => install_claude(&exe_path_str, global, local)?,
            "kimi" => install_kimi(&exe_path_str, global, local)?,
            "kilo" => install_kilo(&exe_path_str, global, local)?,
            _ => {}
        }
        println!();
    }

    println!("Note: Restart your agent after configuration");

    Ok(())
}

fn install_claude(exe_path: &str, global: bool, local: bool) -> BBResult<()> {
    if global {
        let config_path = dirs::home_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?
            .join(".claude.json");
        install_to_config(&config_path, exe_path, "claude-01", true)?;
    }

    if local || (!global && !local) {
        let config_path = PathBuf::from(".mcp.json");
        install_to_config(&config_path, exe_path, "claude-01", true)?;
        add_to_gitignore(".mcp.json")?;
    }

    Ok(())
}

fn install_kimi(exe_path: &str, global: bool, local: bool) -> BBResult<()> {
    if global {
        let config_path = dirs::home_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?
            .join(".kimi")
            .join("mcp.json");
        install_to_config(&config_path, exe_path, "kimi-01", false)?;
    }

    if local || (!global && !local) {
        let config_path = PathBuf::from(".mcp.json");
        install_to_config(&config_path, exe_path, "kimi-01", false)?;
        add_to_gitignore(".mcp.json")?;
    }

    Ok(())
}

fn install_kilo(exe_path: &str, global: bool, local: bool) -> BBResult<()> {
    if global {
        let config_path = dirs::home_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?
            .join(".kilocode")
            .join("cli")
            .join("global")
            .join("settings")
            .join("mcp_settings.json");
        install_to_config(&config_path, exe_path, "kilo-01", true)?;
    }

    if local || (!global && !local) {
        let config_path = PathBuf::from(".kilocode").join("mcp.json");
        install_to_config(&config_path, exe_path, "kilo-01", true)?;
        add_to_gitignore(".kilocode/mcp.json")?;
    }

    Ok(())
}

fn add_to_gitignore(entry: &str) -> BBResult<()> {
    let gitignore_path = PathBuf::from(".gitignore");

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        for line in content.lines() {
            if line.trim() == entry {
                return Ok(());
            }
        }
        let mut new_content = content;
        new_content.push_str("\n");
        new_content.push_str(entry);
        new_content.push_str("\n");
        fs::write(&gitignore_path, new_content)?;
    } else {
        fs::write(&gitignore_path, format!("{}\n", entry))?;
    }

    println!("Added {} to .gitignore", entry);
    Ok(())
}

fn install_to_config(
    config_path: &PathBuf,
    exe_path: &str,
    agent_id: &str,
    with_always_allow: bool,
) -> BBResult<()> {
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| McpConfig {
            mcp_servers: HashMap::new(),
        })
    } else {
        McpConfig {
            mcp_servers: HashMap::new(),
        }
    };

    let mut env = HashMap::new();
    env.insert("BB_AGENT_ID".to_string(), agent_id.to_string());

    let mut server = McpServer {
        command: exe_path.to_string(),
        args: vec![
            "mcp".to_string(),
            "--agent".to_string(),
            agent_id.to_string(),
        ],
        env,
        always_allow: None,
    };

    if with_always_allow {
        server.always_allow = Some(vec![
            "bb_identify".to_string(),
            "bb_set_status".to_string(),
            "bb_get_status".to_string(),
            "bb_post_message".to_string(),
            "bb_read_messages".to_string(),
            "bb_register_artifact".to_string(),
            "bb_list_artifacts".to_string(),
            "bb_find_refs".to_string(),
            "bb_summary".to_string(),
        ]);
    }

    config.mcp_servers.insert("blackboard".to_string(), server);

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, content)?;

    println!("Installed to: {}", config_path.display());

    Ok(())
}
