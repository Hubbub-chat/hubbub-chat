//! Agent runtime loop with budget tracking, tool execution, and streaming.

use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use hubbub_domain::entities::agent::Agent;
use hubbub_domain::entities::conversation::{Conversation, Message, MessagePart, MessageRole};
use hubbub_domain::entities::run::{Run, RunStatus, UsageInfo};
use hubbub_domain::events::RunEvent;
use hubbub_domain::ports::event_sink::EventSink;
use hubbub_domain::ports::llm::{LlmMessage, LlmProvider, ToolCall, ToolSchema};
use hubbub_domain::ports::store::Store;
use hubbub_domain::ports::tool_host::ToolHost;

use crate::budget::BudgetTracker;
use crate::context::ContextBuilder;
use crate::errors::AgentError;
use crate::llm_runner::{LlmRunner, LlmStepParams};
use crate::tool_runner::ToolRunner;

pub struct AgentRuntime {
    store: Arc<dyn Store>,
    llm_provider: Arc<dyn LlmProvider>,
    tool_host: Arc<dyn ToolHost>,
    event_sink: Arc<dyn EventSink>,
    workspace_root: String,
}

impl AgentRuntime {
    pub fn new(
        store: Arc<dyn Store>,
        llm_provider: Arc<dyn LlmProvider>,
        tool_host: Arc<dyn ToolHost>,
        event_sink: Arc<dyn EventSink>,
        workspace_root: Option<String>,
    ) -> Self {
        Self {
            store,
            llm_provider,
            tool_host,
            event_sink,
            workspace_root: workspace_root.unwrap_or_else(|| ".".to_string()),
        }
    }

    /// Execute an agent run within a conversation.
    pub async fn execute_run(
        &self,
        conversation_id: Uuid,
        agent: &Agent,
        user_prompt: Option<&str>,
        cancellation_token: CancellationToken,
    ) -> Result<Run, AgentError> {
        self.execute_run_with_id(
            conversation_id,
            agent,
            user_prompt,
            cancellation_token,
            None,
        )
        .await
    }

    /// Execute an agent run with an optional pre-determined run_id (useful for cancellation tracking).
    pub async fn execute_run_with_id(
        &self,
        conversation_id: Uuid,
        agent: &Agent,
        user_prompt: Option<&str>,
        cancellation_token: CancellationToken,
        run_id: Option<Uuid>,
    ) -> Result<Run, AgentError> {
        self.prepare_conversation(conversation_id, agent, user_prompt)
            .await?;
        let mut run = self.init_run(conversation_id, agent, run_id).await?;

        // Every exit path past this point must leave the run in a terminal state
        match self.run_loop(&mut run, agent, &cancellation_token).await {
            Ok(()) => Ok(run),
            Err(AgentError::Cancelled) => self.handle_cancelled(&mut run).await,
            Err(e) => self.fail_run(&mut run, e).await,
        }
    }

    async fn run_loop(
        &self,
        run: &mut Run,
        agent: &Agent,
        cancellation_token: &CancellationToken,
    ) -> Result<(), AgentError> {
        let history = self
            .store
            .list_messages(run.conversation_id, 1000, 0)
            .await?;
        let mut working_messages = ContextBuilder::build(agent, &history, None);
        let active_tools = self.resolve_active_tools(agent);

        let mut budget = BudgetTracker::new(agent.budget.clone());
        let mut total_prompt_tokens = 0u64;
        let mut total_completion_tokens = 0u64;
        let mut step_idx = 0u32;

        let llm_runner = LlmRunner {
            store: &self.store,
            llm_provider: &self.llm_provider,
            event_sink: &self.event_sink,
        };

        loop {
            if cancellation_token.is_cancelled() {
                return Err(AgentError::Cancelled);
            }
            budget.record_step()?;
            step_idx += 1;

            let step_params = LlmStepParams {
                run_id: run.id,
                agent,
                working_messages: &working_messages,
                active_tools: &active_tools,
                step_idx,
                cancellation_token,
            };

            let (text, tool_calls, p_tok, c_tok) =
                llm_runner.run_step(step_params, &mut budget).await?;

            total_prompt_tokens += p_tok;
            total_completion_tokens += c_tok;

            if tool_calls.is_empty() {
                // Keep the answer even if it pushed us over the token budget: it was already streamed
                self.append_assistant_text(run, text).await?;
                budget.check_token_limit()?;
                return self
                    .complete_run(run, total_prompt_tokens, total_completion_tokens)
                    .await;
            }

            self.handle_assistant_tool_calls(run, &mut working_messages, &text, &tool_calls)
                .await?;

            step_idx = self
                .execute_tool_calls_batch(
                    run,
                    agent,
                    &mut working_messages,
                    &tool_calls,
                    step_idx,
                    cancellation_token,
                )
                .await?;
        }
    }

    async fn prepare_conversation(
        &self,
        conversation_id: Uuid,
        agent: &Agent,
        user_prompt: Option<&str>,
    ) -> Result<(), AgentError> {
        let conv_opt = self.store.get_conversation(conversation_id).await?;
        if conv_opt.is_none() {
            let new_conv = Conversation {
                id: conversation_id,
                title: user_prompt.unwrap_or("New Conversation").to_string(),
                agent_id: agent.id.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                archived: false,
            };
            self.store.create_conversation(&new_conv).await?;
        }

        if let Some(prompt) = user_prompt.filter(|p| !p.is_empty()) {
            let user_msg = Message {
                id: Uuid::now_v7(),
                conversation_id,
                run_id: None,
                role: MessageRole::User,
                parts: vec![MessagePart::Text(prompt.to_string())],
                created_at: Utc::now(),
            };
            self.store.append_message(&user_msg).await?;
        }

        Ok(())
    }

    async fn init_run(
        &self,
        conversation_id: Uuid,
        agent: &Agent,
        run_id: Option<Uuid>,
    ) -> Result<Run, AgentError> {
        let run = Run {
            id: run_id.unwrap_or_else(Uuid::now_v7),
            conversation_id,
            agent_id: agent.id.clone(),
            parent_run_id: None,
            status: RunStatus::Running,
            usage: None,
            cost_usd: None,
            started_at: Utc::now(),
            finished_at: None,
        };
        self.store.create_run(&run).await?;
        Ok(run)
    }

    fn resolve_active_tools(&self, agent: &Agent) -> Vec<ToolSchema> {
        self.tool_host
            .available_tools(&agent.id)
            .into_iter()
            .filter(|t| agent.tools.builtin.contains(&t.name) || agent.tools.mcp.contains(&t.name))
            .collect()
    }

    async fn append_assistant_text(&self, run: &Run, text: String) -> Result<(), AgentError> {
        let assistant_msg = Message {
            id: Uuid::now_v7(),
            conversation_id: run.conversation_id,
            run_id: Some(run.id),
            role: MessageRole::Assistant,
            parts: vec![MessagePart::Text(text)],
            created_at: Utc::now(),
        };
        self.store.append_message(&assistant_msg).await?;
        Ok(())
    }

    async fn complete_run(
        &self,
        run: &mut Run,
        p_tokens: u64,
        c_tokens: u64,
    ) -> Result<(), AgentError> {
        run.status = RunStatus::Completed;
        run.finished_at = Some(Utc::now());
        run.usage = Some(UsageInfo {
            prompt_tokens: p_tokens,
            completion_tokens: c_tokens,
            total_tokens: p_tokens + c_tokens,
        });
        self.store.update_run(run).await?;

        // Best-effort: the run is already stored as completed, a closed window must not undo that
        let _ = self
            .event_sink
            .emit(RunEvent::RunFinished {
                run_id: run.id.to_string(),
                status: "completed".to_string(),
            })
            .await;

        Ok(())
    }

    async fn handle_assistant_tool_calls(
        &self,
        run: &mut Run,
        working_messages: &mut Vec<LlmMessage>,
        text: &str,
        tool_calls: &[ToolCall],
    ) -> Result<(), AgentError> {
        let mut parts = Vec::new();
        if !text.is_empty() {
            parts.push(MessagePart::Text(text.to_string()));
        }
        for tc in tool_calls {
            let parsed_args =
                serde_json::from_str(&tc.arguments).unwrap_or(Value::String(tc.arguments.clone()));
            parts.push(MessagePart::ToolCall {
                id: tc.id.clone(),
                name: tc.name.clone(),
                arguments: parsed_args,
                extra_content: tc.extra_content.clone(),
            });
        }

        let assistant_msg = Message {
            id: Uuid::now_v7(),
            conversation_id: run.conversation_id,
            run_id: Some(run.id),
            role: MessageRole::Assistant,
            parts,
            created_at: Utc::now(),
        };
        self.store.append_message(&assistant_msg).await?;

        working_messages.push(LlmMessage {
            role: "assistant".to_string(),
            content: text.to_string(),
            tool_calls: Some(tool_calls.to_vec()),
            tool_call_id: None,
        });

        Ok(())
    }

    async fn execute_tool_calls_batch(
        &self,
        run: &mut Run,
        agent: &Agent,
        working_messages: &mut Vec<LlmMessage>,
        tool_calls: &[ToolCall],
        mut step_idx: u32,
        cancellation_token: &CancellationToken,
    ) -> Result<u32, AgentError> {
        let runner = ToolRunner {
            store: &self.store,
            tool_host: &self.tool_host,
            event_sink: &self.event_sink,
            workspace_root: &self.workspace_root,
        };

        for tc in tool_calls {
            step_idx += 1;
            let msg = runner
                .execute_tool_call(
                    run.id,
                    run.conversation_id,
                    agent,
                    tc,
                    step_idx,
                    cancellation_token,
                )
                .await?;

            working_messages.push(msg);
        }

        Ok(step_idx)
    }

    async fn handle_cancelled(&self, run: &mut Run) -> Result<Run, AgentError> {
        run.status = RunStatus::Cancelled;
        run.finished_at = Some(Utc::now());
        let _ = self.store.update_run(run).await;

        let _ = self
            .event_sink
            .emit(RunEvent::RunFinished {
                run_id: run.id.to_string(),
                status: "cancelled".to_string(),
            })
            .await;

        Err(AgentError::Cancelled)
    }

    /// Mark the run failed and notify the UI. Store/emit errors are ignored so the
    /// original error is the one returned.
    async fn fail_run(&self, run: &mut Run, err: AgentError) -> Result<Run, AgentError> {
        run.status = RunStatus::Failed;
        run.finished_at = Some(Utc::now());
        let _ = self.store.update_run(run).await;

        let _ = self
            .event_sink
            .emit(RunEvent::Error {
                message: err.to_string(),
                recoverable: false,
            })
            .await;

        let _ = self
            .event_sink
            .emit(RunEvent::RunFinished {
                run_id: run.id.to_string(),
                status: "failed".to_string(),
            })
            .await;

        Err(err)
    }
}
