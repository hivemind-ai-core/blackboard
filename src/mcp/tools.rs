use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::errors::{BBError, BBResult};
use crate::core::models::agent::{Agent, AgentStatus};
use crate::core::models::artifact::Artifact;
use crate::core::models::message::{Message, Priority};
use crate::core::models::reference::Reference;
use crate::core::operations::agent as agent_ops;
use crate::core::operations::artifact as artifact_ops;
use crate::core::operations::classify_liveness;
use crate::core::operations::message as message_ops;
use crate::core::operations::reference::{ReferenceResults, find_references};
use crate::core::validation::limits::validate_agent_id;
use crate::db::connection::with_connection;
use crate::mcp::identity::IdentityResolver;
use std::path::Path;

// Input types for MCP tools
#[derive(Debug, Deserialize)]
pub struct IdentifyInput {
    pub agent_id: String,
}

#[derive(Debug, Serialize)]
pub struct IdentifyOutput {
    pub agent_id: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct SetStatusInput {
    pub current_task: Option<String>,
    pub progress: Option<u8>,
    pub status: Option<String>,
    pub blockers: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GetStatusInput {
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentWithLiveness {
    #[serde(flatten)]
    pub agent: Agent,
    pub liveness: String,
    pub minutes_since_last_seen: i64,
}

#[derive(Debug, Deserialize)]
pub struct PostMessageInput {
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub priority: Option<String>,
    pub reply_to: Option<i64>,
    pub refs: Option<Vec<RefInput>>,
}

#[derive(Debug, Deserialize)]
pub struct RefInput {
    pub where_: String,
    pub what: String,
    pub ref_: JsonValue,
}

#[derive(Debug, Default, Deserialize)]
pub struct ReadMessagesInput {
    pub since: Option<String>,
    pub tags: Option<Vec<String>>,
    pub from_agent: Option<String>,
    pub priority: Option<String>,
    pub ref_where: Option<String>,
    pub ref_what: Option<String>,
    pub ref_ref: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterArtifactInput {
    pub path: String,
    pub description: String,
    pub version: Option<String>,
    pub refs: Option<Vec<RefInput>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ListArtifactsInput {
    pub by: Option<String>,
    pub ref_where: Option<String>,
    pub ref_what: Option<String>,
    pub ref_ref: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct FindRefsInput {
    pub where_: String,
    pub what: String,
    pub ref_: String,
}

#[derive(Debug, Serialize)]
pub struct SummaryOutput {
    pub agents: Vec<AgentWithLiveness>,
    pub blocked_agents: Vec<AgentWithLiveness>,
    pub recent_messages: Vec<Message>,
    pub high_priority_messages: Vec<Message>,
    pub recent_artifacts: Vec<Artifact>,
}

// Tool implementations
pub async fn identify(
    identity: Arc<Mutex<IdentityResolver>>,
    input: IdentifyInput,
) -> BBResult<IdentifyOutput> {
    let mut resolver = identity.lock().await;
    let result = resolver.identify(&input.agent_id)?;

    Ok(IdentifyOutput {
        agent_id: result.agent_id,
        source: result.source,
    })
}

pub async fn set_status(
    identity: Arc<Mutex<IdentityResolver>>,
    project_dir: &Path,
    input: SetStatusInput,
) -> BBResult<Agent> {
    let resolver = identity.lock().await;
    let agent_id = resolver.require_identity()?.to_string();
    drop(resolver);

    let status = input.status.map(|s| AgentStatus::parse(&s));

    let agent = tokio::task::spawn_blocking({
        let project_dir = project_dir.to_path_buf();
        move || {
            with_connection(&project_dir, |conn| {
                agent_ops::update_agent_status(
                    conn,
                    &agent_id,
                    input.current_task.as_deref(),
                    input.progress,
                    status,
                    input.blockers.as_deref(),
                )
            })
        }
    })
    .await
    .map_err(|e| BBError::InvalidInput(format!("Task join error: {e}")))??;

    Ok(agent)
}

pub async fn get_status(
    identity: Arc<Mutex<IdentityResolver>>,
    project_dir: &Path,
    input: GetStatusInput,
) -> BBResult<Vec<AgentWithLiveness>> {
    // Touch the agent's last_seen if we have an identity
    {
        let resolver = identity.lock().await;
        if let Some(agent_id) = resolver.resolve() {
            let agent_id = agent_id.to_string();
            let project_dir = project_dir.to_path_buf();
            tokio::task::spawn_blocking(move || {
                let _ =
                    with_connection(&project_dir, |conn| agent_ops::touch_agent(conn, &agent_id));
            })
            .await
            .ok();
        }
    }

    let result = tokio::task::spawn_blocking({
        let project_dir = project_dir.to_path_buf();
        let input = input.clone();
        move || {
            with_connection(&project_dir, |conn| {
                if let Some(agent_id) = &input.agent_id {
                    validate_agent_id(agent_id)?;
                    let agent = agent_ops::get_agent(conn, agent_id)?.ok_or_else(|| {
                        BBError::NotFound(format!("agent '{agent_id}' not found"))
                    })?;

                    let liveness = classify_liveness(agent.last_seen);
                    let minutes = chrono::Utc::now()
                        .signed_duration_since(agent.last_seen)
                        .num_minutes();

                    Ok(vec![AgentWithLiveness {
                        liveness: format!("{liveness:?}").to_lowercase(),
                        minutes_since_last_seen: minutes,
                        agent,
                    }])
                } else {
                    let agents = agent_ops::get_all_agents_with_liveness(conn)?;
                    let now = chrono::Utc::now();

                    Ok(agents
                        .into_iter()
                        .map(|a| {
                            let liveness = classify_liveness(a.last_seen);
                            let minutes = now.signed_duration_since(a.last_seen).num_minutes();
                            AgentWithLiveness {
                                liveness: format!("{liveness:?}").to_lowercase(),
                                minutes_since_last_seen: minutes,
                                agent: a,
                            }
                        })
                        .collect())
                }
            })
        }
    })
    .await
    .map_err(|e| BBError::InvalidInput(format!("Task join error: {e}")))??;

    Ok(result)
}

pub async fn post_message(
    identity: Arc<Mutex<IdentityResolver>>,
    project_dir: &Path,
    input: PostMessageInput,
) -> BBResult<Message> {
    let resolver = identity.lock().await;
    let agent_id = resolver.require_identity()?.to_string();
    drop(resolver);

    let priority = input
        .priority
        .map(|p| Priority::parse(&p))
        .unwrap_or(Priority::Normal);

    let refs: Vec<Reference> = input
        .refs
        .map(|refs| {
            refs.into_iter()
                .map(|r| Reference {
                    where_: r.where_,
                    what: r.what,
                    ref_: r.ref_,
                })
                .collect()
        })
        .unwrap_or_default();

    let tags = input.tags.unwrap_or_default();

    let message = tokio::task::spawn_blocking({
        let project_dir = project_dir.to_path_buf();
        move || {
            with_connection(&project_dir, |conn| {
                message_ops::post_message(
                    conn,
                    &agent_id,
                    &input.content,
                    tags,
                    priority,
                    input.reply_to,
                    refs,
                )
            })
        }
    })
    .await
    .map_err(|e| BBError::InvalidInput(format!("Task join error: {e}")))??;

    Ok(message)
}

pub async fn read_messages(project_dir: &Path, input: ReadMessagesInput) -> BBResult<Vec<Message>> {
    let since = if let Some(s) = input.since {
        let duration = crate::util::duration::parse_duration(&s)?;
        Some(chrono::Utc::now() - duration)
    } else {
        None
    };

    let priority = input.priority.map(|p| Priority::parse(&p));
    let tags = input.tags.unwrap_or_default();
    let limit = input.limit.unwrap_or(20);

    let messages = tokio::task::spawn_blocking({
        let project_dir = project_dir.to_path_buf();
        move || {
            with_connection(&project_dir, |conn| {
                message_ops::list_messages(
                    conn,
                    since,
                    &tags,
                    input.from_agent.as_deref(),
                    priority,
                    input.ref_where.as_deref(),
                    input.ref_what.as_deref(),
                    input.ref_ref.as_deref(),
                    limit,
                )
            })
        }
    })
    .await
    .map_err(|e| BBError::InvalidInput(format!("Task join error: {e}")))??;

    Ok(messages)
}

pub async fn register_artifact(
    identity: Arc<Mutex<IdentityResolver>>,
    project_dir: &Path,
    input: RegisterArtifactInput,
) -> BBResult<Artifact> {
    let resolver = identity.lock().await;
    let agent_id = resolver.require_identity()?.to_string();
    drop(resolver);

    let refs: Vec<Reference> = input
        .refs
        .map(|refs| {
            refs.into_iter()
                .map(|r| Reference {
                    where_: r.where_,
                    what: r.what,
                    ref_: r.ref_,
                })
                .collect()
        })
        .unwrap_or_default();

    let artifact = tokio::task::spawn_blocking({
        let project_dir = project_dir.to_path_buf();
        move || {
            with_connection(&project_dir, |conn| {
                artifact_ops::register_artifact(
                    conn,
                    &input.path,
                    &agent_id,
                    &input.description,
                    input.version.as_deref(),
                    refs,
                    &project_dir,
                )
            })
        }
    })
    .await
    .map_err(|e| BBError::InvalidInput(format!("Task join error: {e}")))??;

    Ok(artifact)
}

pub async fn list_artifacts(
    project_dir: &Path,
    input: ListArtifactsInput,
) -> BBResult<Vec<Artifact>> {
    let limit = input.limit.unwrap_or(50);

    let artifacts = tokio::task::spawn_blocking({
        let project_dir = project_dir.to_path_buf();
        move || {
            with_connection(&project_dir, |conn| {
                artifact_ops::list_artifacts(
                    conn,
                    input.by.as_deref(),
                    input.ref_where.as_deref(),
                    input.ref_what.as_deref(),
                    input.ref_ref.as_deref(),
                    limit,
                )
            })
        }
    })
    .await
    .map_err(|e| BBError::InvalidInput(format!("Task join error: {e}")))??;

    Ok(artifacts)
}

pub async fn find_refs(project_dir: &Path, input: FindRefsInput) -> BBResult<ReferenceResults> {
    // Parse the ref value (try number first, then string)
    let ref_value: JsonValue = if let Ok(num) = input.ref_.parse::<i64>() {
        JsonValue::Number(num.into())
    } else {
        JsonValue::String(input.ref_)
    };

    let results = tokio::task::spawn_blocking({
        let project_dir = project_dir.to_path_buf();
        move || {
            with_connection(&project_dir, |conn| {
                find_references(conn, &input.where_, &input.what, &ref_value)
            })
        }
    })
    .await
    .map_err(|e| BBError::InvalidInput(format!("Task join error: {e}")))??;

    Ok(results)
}

pub async fn summary(project_dir: &Path) -> BBResult<SummaryOutput> {
    let result = tokio::task::spawn_blocking({
        let project_dir = project_dir.to_path_buf();
        move || {
            with_connection(&project_dir, |conn| {
                let agents = agent_ops::get_all_agents_with_liveness(conn)?;
                let now = chrono::Utc::now();

                let agents_with_liveness: Vec<_> = agents
                    .into_iter()
                    .map(|a| {
                        let liveness = classify_liveness(a.last_seen);
                        let minutes = now.signed_duration_since(a.last_seen).num_minutes();
                        AgentWithLiveness {
                            liveness: format!("{liveness:?}").to_lowercase(),
                            minutes_since_last_seen: minutes,
                            agent: a,
                        }
                    })
                    .collect();

                let blocked_agents: Vec<_> = agents_with_liveness
                    .iter()
                    .filter(|a| a.agent.status == AgentStatus::Blocked)
                    .cloned()
                    .collect();

                let recent_since = now - chrono::Duration::minutes(30);
                let recent_messages = message_ops::list_messages(
                    conn,
                    Some(recent_since),
                    &[],
                    None,
                    None,
                    None,
                    None,
                    None,
                    20,
                )?;

                let high_priority_messages = message_ops::list_messages(
                    conn,
                    None,
                    &[],
                    None,
                    Some(Priority::High),
                    None,
                    None,
                    None,
                    10,
                )?;

                let artifact_since = now - chrono::Duration::hours(1);
                let recent_artifacts =
                    artifact_ops::list_artifacts(conn, None, None, None, None, 20)?;
                let recent_artifacts: Vec<_> = recent_artifacts
                    .into_iter()
                    .filter(|a| a.created_at >= artifact_since)
                    .collect();

                Ok(SummaryOutput {
                    agents: agents_with_liveness,
                    blocked_agents,
                    recent_messages,
                    high_priority_messages,
                    recent_artifacts,
                })
            })
        }
    })
    .await
    .map_err(|e| BBError::InvalidInput(format!("Task join error: {e}")))??;

    Ok(result)
}
