//! # hubbub-agent
//!
//! Agent runtime, context builder, budget tracker.
//! Depends on domain ports only.

pub mod budget;
pub mod context;
pub mod errors;
pub mod llm_runner;
pub mod runtime;
pub mod tool_runner;

pub use budget::BudgetTracker;
pub use context::ContextBuilder;
pub use errors::AgentError;
pub use llm_runner::LlmRunner;
pub use runtime::AgentRuntime;
pub use tool_runner::{BUILTIN_TOOLS, ToolRunner};
