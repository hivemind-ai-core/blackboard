use crate::core::errors::{BBError, BBResult};
use crate::core::validation::limits::validate_agent_id;

#[derive(Debug, Clone)]
pub struct IdentityResolver {
    fixed_agent: Option<String>, // From --agent
    env_agent: Option<String>,   // From BB_AGENT_ID
    resolved: Option<String>,    // From bb_identify
}

impl IdentityResolver {
    pub fn new(fixed_agent: Option<String>, env_agent: Option<String>) -> Self {
        Self {
            fixed_agent,
            env_agent,
            resolved: None,
        }
    }

    pub fn resolve(&self) -> Option<&str> {
        self.fixed_agent
            .as_deref()
            .or(self.env_agent.as_deref())
            .or(self.resolved.as_deref())
    }

    pub fn identify(&mut self, agent_id: &str) -> BBResult<IdentifyResponse> {
        validate_agent_id(agent_id)?;

        // Check if identity is fixed by --agent
        if self.fixed_agent.is_some() {
            return Err(BBError::InvalidInput(
                "identity already fixed by --agent".into(),
            ));
        }

        // Check if identity is set by env
        if let Some(env_id) = &self.env_agent {
            if env_id != agent_id {
                return Err(BBError::InvalidInput(
                    "cannot change identity set by BB_AGENT_ID".into(),
                ));
            }
            // Same value, succeed as no-op
            return Ok(IdentifyResponse {
                agent_id: agent_id.to_string(),
                source: "env".to_string(),
            });
        }

        // Check if already resolved via identify
        if let Some(existing) = &self.resolved {
            if existing != agent_id {
                return Err(BBError::InvalidInput(
                    "identity already set to a different value".into(),
                ));
            }
            // Same value, succeed as no-op
            return Ok(IdentifyResponse {
                agent_id: agent_id.to_string(),
                source: "identify".to_string(),
            });
        }

        // Set the resolved identity
        self.resolved = Some(agent_id.to_string());

        Ok(IdentifyResponse {
            agent_id: agent_id.to_string(),
            source: "identify".to_string(),
        })
    }

    pub fn require_identity(&self) -> BBResult<&str> {
        self.resolve().ok_or(BBError::IdentityRequired)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IdentifyResponse {
    pub agent_id: String,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_precedence() {
        // --agent takes precedence
        let resolver =
            IdentityResolver::new(Some("from-arg".to_string()), Some("from-env".to_string()));
        assert_eq!(resolver.resolve(), Some("from-arg"));

        // env takes precedence over resolved
        let resolver = IdentityResolver::new(None, Some("from-env".to_string()));
        assert_eq!(resolver.resolve(), Some("from-env"));
    }

    #[test]
    fn test_identify_sets_identity() {
        let mut resolver = IdentityResolver::new(None, None);

        let result = resolver.identify("agent-1").unwrap();
        assert_eq!(result.agent_id, "agent-1");
        assert_eq!(result.source, "identify");

        assert_eq!(resolver.resolve(), Some("agent-1"));
    }

    #[test]
    fn test_identify_fails_when_fixed() {
        let mut resolver = IdentityResolver::new(Some("fixed".to_string()), None);

        assert!(resolver.identify("agent-1").is_err());
    }

    #[test]
    fn test_identify_fails_when_different_env() {
        let mut resolver = IdentityResolver::new(None, Some("from-env".to_string()));

        assert!(resolver.identify("different").is_err());
    }

    #[test]
    fn test_identify_succeeds_when_same_env() {
        let mut resolver = IdentityResolver::new(None, Some("agent-1".to_string()));

        let result = resolver.identify("agent-1").unwrap();
        assert_eq!(result.source, "env");
    }

    #[test]
    fn test_identify_noop_when_same() {
        let mut resolver = IdentityResolver::new(None, None);

        resolver.identify("agent-1").unwrap();
        let result = resolver.identify("agent-1").unwrap();

        assert_eq!(result.agent_id, "agent-1");
        assert_eq!(resolver.resolve(), Some("agent-1"));
    }

    #[test]
    fn test_identify_fails_when_different_resolved() {
        let mut resolver = IdentityResolver::new(None, None);

        resolver.identify("agent-1").unwrap();
        assert!(resolver.identify("agent-2").is_err());
    }

    #[test]
    fn test_require_identity() {
        let resolver = IdentityResolver::new(None, None);
        assert!(resolver.require_identity().is_err());

        let resolver = IdentityResolver::new(Some("agent-1".to_string()), None);
        assert_eq!(resolver.require_identity().unwrap(), "agent-1");
    }
}
