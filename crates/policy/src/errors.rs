//! Error types for security policy engine.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PolicyError {
    #[error("URL scheme '{0}' is not allowed (only http and https are permitted)")]
    DisallowedScheme(String),

    #[error("URL '{url}' resolved to blocked private/internal IP '{ip}' (SSRF protection)")]
    SsrfBlocked { url: String, ip: String },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("DNS resolution failed for host '{0}': {1}")]
    DnsResolutionFailed(String, String),

    #[error("Path traversal detected: path '{path}' escapes workspace root")]
    PathTraversal { path: String },

    #[error("Absolute paths are not allowed: '{0}'")]
    AbsolutePathDisallowed(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Agent '{agent_id}' lacks permission for '{operation}': {reason}")]
    PermissionDenied {
        agent_id: String,
        operation: String,
        reason: String,
    },

    #[error("Invalid agent definition '{agent_id}': {reason}")]
    InvalidAgent { agent_id: String, reason: String },
}
