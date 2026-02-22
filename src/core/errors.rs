#[derive(Debug, thiserror::Error)]
pub enum BBError {
    #[allow(dead_code)]
    #[error("No blackboard found. Run 'bb init' to create one.")]
    NotInitialized,

    #[allow(dead_code)]
    #[error("Database busy. Please retry.")]
    DatabaseBusy,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid ref format '{0}': expected where:what:ref")]
    InvalidRefFormat(String),

    #[error("Path traversal not allowed: {0}")]
    PathTraversal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Identity required. Configure --agent, set BB_AGENT_ID, or call bb_identify.")]
    IdentityRequired,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type BBResult<T> = Result<T, BBError>;
