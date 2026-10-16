//! Validation of agent definitions coming from outside (the UI's `upsert_agent`).
//!
//! The runtime already enforces permissions per call; this rejects definitions that
//! could never be safe, before they are stored.

use std::path::Path;

use hubbub_domain::entities::agent::Agent;

use crate::errors::PolicyError;
use crate::path_guard::PathGuard;

pub struct AgentValidator;

impl AgentValidator {
    /// `known_builtin_tools` is the set of builtin tools the runtime has a policy for.
    pub fn validate(agent: &Agent, known_builtin_tools: &[&str]) -> Result<(), PolicyError> {
        let invalid = |reason: String| PolicyError::InvalidAgent {
            agent_id: agent.id.clone(),
            reason,
        };

        let id_ok = !agent.id.is_empty()
            && agent.id.len() <= 64
            && agent
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
        if !id_ok {
            return Err(invalid(
                "id must be 1-64 characters of a-z, 0-9, '_' or '-'".to_string(),
            ));
        }

        if let Some(tool) = agent
            .tools
            .builtin
            .iter()
            .find(|t| !known_builtin_tools.contains(&t.as_str()))
        {
            return Err(invalid(format!("unknown builtin tool '{tool}'")));
        }

        for pattern in &agent.permissions.fs_read {
            Self::check_pattern(pattern, false).map_err(|r| invalid(format!("fs_read: {r}")))?;
        }
        for pattern in &agent.permissions.fs_write {
            Self::check_pattern(pattern, true).map_err(|r| invalid(format!("fs_write: {r}")))?;
        }
        Ok(())
    }

    /// Patterns are `**`/`*` (everything), `dir/**`, `dir/*`, or an exact relative path.
    fn check_pattern(pattern: &str, is_write: bool) -> Result<(), String> {
        let pattern = pattern.trim();
        if pattern == "**" || pattern == "*" {
            return if is_write {
                // Would include agents/, .versions/, data/ ...
                Err(format!("'{pattern}' would allow writing protected paths"))
            } else {
                Ok(())
            };
        }

        let base = pattern
            .strip_suffix("/**")
            .or_else(|| pattern.strip_suffix("/*"))
            .unwrap_or(pattern);
        // Same rules as a real path: no traversal, absolute paths, drives or ADS
        PathGuard::resolve_within_workspace(Path::new("."), base)
            .map_err(|e| format!("'{pattern}': {e}"))?;
        if PathGuard::is_app_data_path(base) {
            return Err(format!("'{pattern}' is inside the app data directory"));
        }
        if is_write && PathGuard::is_protected_path(base) {
            return Err(format!("'{pattern}' is a protected path"));
        }
        Ok(())
    }
}
