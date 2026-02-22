use std::env;
use crate::core::errors::BBResult;

pub fn run(agent_type: Option<&str>) -> BBResult<()> {
    // Get the path to the bb binary
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    println!("Blackboard MCP Installation");
    println!("==========================\n");
    
    println!("bb binary location: {exe_path_str}\n");

    // Determine agent type
    let agent_types: Vec<&str> = match agent_type {
        Some("claude") => vec!["claude"],
        Some("kimi") => vec!["kimi"],
        Some("kilo") => vec!["kilo"],
        _ => vec!["claude", "kimi", "kilo"],
    };

    for agent in agent_types {
        match agent {
            "claude" => print_claude_config(&exe_path_str),
            "kimi" => print_kimi_config(&exe_path_str),
            "kilo" => print_kilo_config(&exe_path_str),
            _ => {}
        }
        println!();
    }

    println!("\nNotes:");
    println!("  - Replace <agent-id> with your preferred ID (e.g., claude-01, kimi-01, kilo-01)");
    println!("  - You can also set BB_AGENT_ID environment variable instead of using --agent");
    println!("  - Restart your agent after configuration");

    Ok(())
}

fn print_claude_config(exe_path: &str) {
    println!("Claude Code Configuration");
    println!("-------------------------");
    println!("Option A (CLI with env):");
    println!("  claude mcp add --transport stdio --env BB_AGENT_ID=claude-01 blackboard -- {exe_path} mcp");
    println!();
    println!("Option B (CLI with args):");
    println!("  claude mcp add --transport stdio blackboard -- {exe_path} mcp --agent claude-01");
    println!();
    println!("Option C (.mcp.json in project root):");
    println!(r#"  {{
    "mcpServers": {{
      "blackboard": {{
        "command": "{exe_path}",
        "args": ["mcp", "--agent", "claude-01"],
        "env": {{}}
      }}
    }}
  }}"#);
}

fn print_kimi_config(exe_path: &str) {
    println!("Kimi Code Configuration");
    println!("-----------------------");
    println!("CLI:");
    println!("  kimi mcp add --transport stdio blackboard -- {exe_path} mcp --agent kimi-01");
    println!();
    println!("Config (~/.kimi/mcp.json):");
    println!(r#"  {{
    "mcpServers": {{
      "blackboard": {{
        "command": "{exe_path}",
        "args": ["mcp", "--agent", "kimi-01"],
        "env": {{}}
      }}
    }}
  }}"#);
}

fn print_kilo_config(exe_path: &str) {
    println!("Kilo Code Configuration");
    println!("-----------------------");
    println!("Project config (.kilocode/mcp.json):");
    println!(r#"  {{
    "mcpServers": {{
      "blackboard": {{
        "command": "{exe_path}",
        "args": ["mcp", "--agent", "kilo-01"],
        "env": {{}},
        "alwaysAllow": [
          "bb_identify",
          "bb_set_status",
          "bb_get_status",
          "bb_post_message",
          "bb_read_messages",
          "bb_register_artifact",
          "bb_list_artifacts",
          "bb_find_refs",
          "bb_summary"
        ]
      }}
    }}
  }}"#);
    println!();
    println!("Global config (~/.kilocode/cli/global/settings/mcp_settings.json):");
    println!(r#"  {{
    "mcpServers": {{
      "blackboard": {{
        "command": "{exe_path}",
        "args": ["mcp", "--agent", "kilo-01"],
        "env": {{}}
      }}
    }}
  }}"#);
}
