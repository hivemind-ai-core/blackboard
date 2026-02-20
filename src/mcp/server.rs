use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;

use crate::core::errors::BBError;
use crate::mcp::identity::IdentityResolver;
use crate::mcp::tools::*;

// For now, we'll implement a simpler MCP server without the full rmcp integration
// The rmcp crate API is complex and this basic implementation provides the functionality

pub struct BlackboardMcpServer {
    identity: Arc<Mutex<IdentityResolver>>,
    project_dir: std::path::PathBuf,
}

impl BlackboardMcpServer {
    pub fn new(identity: IdentityResolver, project_dir: &Path) -> Self {
        Self {
            identity: Arc::new(Mutex::new(identity)),
            project_dir: project_dir.to_path_buf(),
        }
    }

    async fn handle_request(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, BBError> {
        match method {
            "bb_identify" => {
                let input: IdentifyInput = params
                    .map(|v| serde_json::from_value(v).map_err(|e| BBError::InvalidInput(format!("Parse error: {}", e))))
                    .transpose()?
                    .ok_or_else(|| BBError::InvalidInput("Missing params".to_string()))?;
                
                bb_identify(self.identity.clone(), input).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            "bb_set_status" => {
                let input: SetStatusInput = params
                    .map(|v| serde_json::from_value(v).map_err(|e| BBError::InvalidInput(format!("Parse error: {}", e))))
                    .transpose()?
                    .ok_or_else(|| BBError::InvalidInput("Missing params".to_string()))?;
                
                bb_set_status(self.identity.clone(), &self.project_dir, input).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            "bb_get_status" => {
                let input: GetStatusInput = params
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                
                bb_get_status(self.identity.clone(), &self.project_dir, input).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            "bb_post_message" => {
                let input: PostMessageInput = params
                    .map(|v| serde_json::from_value(v).map_err(|e| BBError::InvalidInput(format!("Parse error: {}", e))))
                    .transpose()?
                    .ok_or_else(|| BBError::InvalidInput("Missing params".to_string()))?;
                
                bb_post_message(self.identity.clone(), &self.project_dir, input).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            "bb_read_messages" => {
                let input: ReadMessagesInput = params
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                
                bb_read_messages(&self.project_dir, input).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            "bb_register_artifact" => {
                let input: RegisterArtifactInput = params
                    .map(|v| serde_json::from_value(v).map_err(|e| BBError::InvalidInput(format!("Parse error: {}", e))))
                    .transpose()?
                    .ok_or_else(|| BBError::InvalidInput("Missing params".to_string()))?;
                
                bb_register_artifact(self.identity.clone(), &self.project_dir, input).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            "bb_list_artifacts" => {
                let input: ListArtifactsInput = params
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                
                bb_list_artifacts(&self.project_dir, input).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            "bb_find_refs" => {
                let input: FindRefsInput = params
                    .map(|v| serde_json::from_value(v).map_err(|e| BBError::InvalidInput(format!("Parse error: {}", e))))
                    .transpose()?
                    .ok_or_else(|| BBError::InvalidInput("Missing params".to_string()))?;
                
                bb_find_refs(&self.project_dir, input).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            "bb_summary" => {
                bb_summary(&self.project_dir).await
                    .map(|r| serde_json::to_value(r).unwrap())
            }

            _ => Err(BBError::InvalidInput(format!("Unknown method: {}", method))),
        }
    }
}

pub async fn run_mcp_server(
    fixed_agent: Option<String>,
    env_agent: Option<String>,
    project_dir: &Path,
) -> crate::core::errors::BBResult<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    
    // Check if initialized
    crate::db::schema::ensure_initialized(project_dir)?;

    let identity = IdentityResolver::new(fixed_agent.clone(), env_agent.clone());
    let server = Arc::new(BlackboardMcpServer::new(identity, project_dir));

    // Log identity source for debugging
    let identity_source = if fixed_agent.is_some() {
        "arg"
    } else if env_agent.is_some() {
        "env"
    } else {
        "none"
    };
    tracing::debug!("MCP server identity source: {}", identity_source);
    
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = stdout;
    
    // MCP protocol over stdio: read JSON-RPC requests, write responses
    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        
        // Parse request
        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    },
                    "id": null
                });
                let _ = stdout.write_all(format!("{}\n", response).as_bytes()).await;
                let _ = stdout.flush().await;
                continue;
            }
        };
        
        let id = request.get("id").cloned();
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned();
        
        // Handle initialize
        if method == "initialize" {
            let response = json!({
                "jsonrpc": "2.0",
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "blackboard",
                        "version": "0.1.0"
                    }
                },
                "id": id
            });
            let _ = stdout.write_all(format!("{}\n", response).as_bytes()).await;
            let _ = stdout.flush().await;
            continue;
        }
        
        // Handle tools/list
        if method == "tools/list" {
            let response = json!({
                "jsonrpc": "2.0",
                "result": {
                    "tools": [
                        { "name": "bb_identify", "description": "Establish agent identity", "inputSchema": { "type": "object", "properties": { "agent_id": { "type": "string" }}, "required": ["agent_id"]}},
                        { "name": "bb_set_status", "description": "Update agent status", "inputSchema": { "type": "object", "properties": { "current_task": { "type": "string" }, "progress": { "type": "integer" }, "status": { "type": "string" }, "blockers": { "type": "string" }}}},
                        { "name": "bb_get_status", "description": "Get agent status", "inputSchema": { "type": "object", "properties": { "agent_id": { "type": "string" }}}},
                        { "name": "bb_post_message", "description": "Post a message", "inputSchema": { "type": "object", "properties": { "content": { "type": "string" }, "tags": { "type": "array" }, "priority": { "type": "string" }, "in_reply_to": { "type": "integer" }, "refs": { "type": "array" }}, "required": ["content"]}},
                        { "name": "bb_read_messages", "description": "Read messages", "inputSchema": { "type": "object", "properties": { "since": { "type": "string" }, "tags": { "type": "array" }, "from_agent": { "type": "string" }, "priority": { "type": "string" }, "limit": { "type": "integer" }}}},
                        { "name": "bb_register_artifact", "description": "Register artifact", "inputSchema": { "type": "object", "properties": { "path": { "type": "string" }, "description": { "type": "string" }, "version": { "type": "string" }, "refs": { "type": "array" }}, "required": ["path", "description"]}},
                        { "name": "bb_list_artifacts", "description": "List artifacts", "inputSchema": { "type": "object", "properties": { "produced_by": { "type": "string" }, "limit": { "type": "integer" }}}},
                        { "name": "bb_find_refs", "description": "Find references", "inputSchema": { "type": "object", "properties": { "where": { "type": "string" }, "what": { "type": "string" }, "ref": { "type": "string" }}, "required": ["where", "what", "ref"]}},
                        { "name": "bb_summary", "description": "Get summary", "inputSchema": { "type": "object", "properties": {}}}
                    ]
                },
                "id": id
            });
            let _ = stdout.write_all(format!("{}\n", response).as_bytes()).await;
            let _ = stdout.flush().await;
            continue;
        }
        
        // Handle tool calls
        if method == "tools/call" {
            let tool_name = params.as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let tool_params = params.as_ref()
                .and_then(|p| p.get("arguments"))
                .cloned();
            
            let result = server.handle_request(tool_name, tool_params).await;
            
            let response = match result {
                Ok(content) => {
                    json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "content": [{"type": "text", "text": content.to_string()}]
                        },
                        "id": id
                    })
                }
                Err(e) => {
                    let (code, message) = match e {
                        BBError::NotInitialized => (-32001, "No blackboard found. Run 'bb init' to create one.".to_string()),
                        BBError::IdentityRequired => (-32002, "Identity required. Configure --agent, set BB_AGENT_ID, or call bb_identify.".to_string()),
                        BBError::InvalidInput(msg) => (-32003, msg),
                        BBError::NotFound(msg) => (-32004, msg),
                        BBError::InvalidRefFormat(msg) => (-32005, msg),
                        BBError::PathTraversal(msg) => (-32006, msg),
                        BBError::DatabaseBusy => (-32007, "Database busy. Please retry.".to_string()),
                        _ => (-32000, e.to_string()),
                    };
                    json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": code,
                            "message": message
                        },
                        "id": id
                    })
                }
            };
            
            let _ = stdout.write_all(format!("{}\n", response).as_bytes()).await;
            let _ = stdout.flush().await;
            continue;
        }
        
        // Unknown method
        let response = json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            },
            "id": id
        });
        let _ = stdout.write_all(format!("{}\n", response).as_bytes()).await;
        let _ = stdout.flush().await;
    }
    
    Ok(())
}

// Default implementations for GetStatusInput and ReadMessagesInput
impl Default for GetStatusInput {
    fn default() -> Self {
        Self { agent_id: None }
    }
}

impl Default for ReadMessagesInput {
    fn default() -> Self {
        Self {
            since: None,
            tags: None,
            from_agent: None,
            priority: None,
            ref_where: None,
            ref_what: None,
            ref_ref: None,
            limit: None,
        }
    }
}

impl Default for ListArtifactsInput {
    fn default() -> Self {
        Self {
            produced_by: None,
            ref_where: None,
            ref_what: None,
            ref_ref: None,
            limit: None,
        }
    }
}
