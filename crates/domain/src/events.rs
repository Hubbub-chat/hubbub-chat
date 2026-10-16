//! Domain events emitted during agent runs.

use serde::{Deserialize, Serialize};

/// Events emitted during an agent run, streamed to the UI.
///
/// Every run-scoped event carries `run_id`, so the UI can ignore events from a
/// run that is not the one it is showing (e.g. after switching conversation).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RunEvent {
    /// The run exists and can be cancelled. Always the first event of a run.
    RunStarted {
        run_id: String,
        conversation_id: String,
    },

    /// Incremental text from the LLM.
    MessageDelta { run_id: String, content: String },

    /// A tool execution has started.
    ToolStarted {
        run_id: String,
        tool_name: String,
        args_preview: String,
    },

    /// User approval is required to proceed.
    ApprovalRequired {
        approval_id: String,
        tool_name: String,
        description: String,
    },

    /// A tool execution has completed.
    ToolFinished {
        run_id: String,
        tool_name: String,
        success: bool,
        summary: String,
    },

    /// The run has completed.
    RunFinished { run_id: String, status: String },

    /// An error occurred during the run.
    Error {
        run_id: String,
        message: String,
        recoverable: bool,
    },

    /// Documents in the workspace have changed.
    DocumentsChanged { paths: Vec<String> },
}
