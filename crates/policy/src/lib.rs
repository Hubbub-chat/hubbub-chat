//! # hubbub-policy
//!
//! Security policy engine: PathGuard, UrlGuard, and PermissionChecker.
//! Enforces zero-trust boundaries on paths, URLs (SSRF prevention), and agent capabilities.

pub mod agent_validator;
pub mod errors;
pub mod path_guard;
pub mod permission_checker;
pub mod url_guard;

pub use agent_validator::AgentValidator;
pub use errors::PolicyError;
pub use path_guard::PathGuard;
pub use permission_checker::PermissionChecker;
pub use url_guard::UrlGuard;
