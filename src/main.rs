mod cli;
mod core;
mod db;
mod mcp;
mod util;

use cli::{Cli, Commands, get_project_dir};
use cli::output::OutputFormat;
use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn get_agent_id(as_arg: Option<String>) -> String {
    as_arg.or_else(|| std::env::var("BB_AGENT_ID").ok())
        .unwrap_or_else(|| "human".to_string())
}

async fn run(cli: Cli) -> core::errors::BBResult<()> {
    let format = if cli.json { OutputFormat::Json } else { OutputFormat::Human };
    
    match cli.command {
        Commands::Init => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::init::run(&project_dir)
        }
        Commands::Install { agent_type } => {
            cli::commands::install::run(agent_type.as_deref())
        }
        Commands::Destroy { confirm } => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::destroy::run(&project_dir, confirm)
        }
        Commands::Status { command } => {
            let project_dir = get_project_dir(cli.dir)?;
            match command {
                None => cli::commands::status::status(&project_dir, format),
                Some(cli::StatusCommands::Set { task, progress, status, blockers }) => {
                    let agent_id = get_agent_id(cli.as_);
                    cli::commands::status::status_set(&project_dir, &agent_id, &task, progress, status, blockers.as_deref())
                }
                Some(cli::StatusCommands::Get { agent_id }) => {
                    cli::commands::status::status_get(&project_dir, &agent_id, format)
                }
                Some(cli::StatusCommands::Clear) => {
                    let agent_id = get_agent_id(cli.as_);
                    cli::commands::status::status_clear(&project_dir, &agent_id)
                }
            }
        }
        Commands::Log { since, tags, from, priority, ref_where, ref_what, ref_ref, limit } => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::message::log(&project_dir, since.as_deref(), tags, from.as_deref(), priority, ref_where.as_deref(), ref_what.as_deref(), ref_ref.as_deref(), limit, format)
        }
        Commands::Post { content, tags, priority, reply_to, refs } => {
            let project_dir = get_project_dir(cli.dir)?;
            let agent_id = get_agent_id(cli.as_);
            cli::commands::message::post(&project_dir, &agent_id, &content, tags, priority, reply_to, refs)
        }
        Commands::Message { id } => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::message::show_message(&project_dir, id, format)
        }
        Commands::Artifacts { by, ref_where, ref_what, ref_ref, limit } => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::artifact::list(&project_dir, by.as_deref(), ref_where.as_deref(), ref_what.as_deref(), ref_ref.as_deref(), limit, format)
        }
        Commands::ArtifactAdd { path, description, version, refs } => {
            let project_dir = get_project_dir(cli.dir)?;
            let agent_id = get_agent_id(cli.as_);
            cli::commands::artifact::add(&project_dir, &path, &agent_id, &description, version.as_deref(), refs)
        }
        Commands::ArtifactShow { path } => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::artifact::show(&project_dir, &path, format)
        }
        Commands::Refs { reference } => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::ref_::find(&project_dir, &reference, format)
        }
        Commands::Clear { messages_before, reset_offline, artifacts, confirm } => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::clear::clear(&project_dir, messages_before.as_deref(), reset_offline, artifacts, confirm)
        }
        Commands::Export => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::export::export(&project_dir)
        }
        Commands::Summary => {
            let project_dir = get_project_dir(cli.dir)?;
            cli::commands::summary::summary(&project_dir, format)
        }
        Commands::Mcp { agent } => {
            let project_dir = get_project_dir(cli.dir)?;
            mcp::run_mcp_server(agent, std::env::var("BB_AGENT_ID").ok(), &project_dir).await
        }
    }
}
