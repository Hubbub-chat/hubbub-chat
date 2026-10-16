//! Tool execution helper for agent runtime.

use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use hubbub_domain::entities::agent::Agent;
use hubbub_domain::entities::audit_log::AuditLog;
use hubbub_domain::entities::conversation::{Message, MessagePart, MessageRole};
use hubbub_domain::entities::run::{Step, StepKind, StepStatus};
use hubbub_domain::events::RunEvent;
use hubbub_domain::ports::event_sink::EventSink;
use hubbub_domain::ports::llm::{LlmMessage, ToolCall};
use hubbub_domain::ports::store::Store;
use hubbub_domain::ports::tool_host::{ToolContext, ToolHost};
use hubbub_domain::text::truncate_with_ellipsis;
use hubbub_policy::{PathGuard, PermissionChecker, PolicyError, UrlGuard};

use crate::errors::AgentError;

pub struct ToolRunner<'a> {
    pub store: &'a Arc<dyn Store>,
    pub tool_host: &'a Arc<dyn ToolHost>,
    pub event_sink: &'a Arc<dyn EventSink>,
    pub workspace_root: &'a str,
}

impl<'a> ToolRunner<'a> {
    pub async fn execute_tool_call(
        &self,
        run_id: Uuid,
        conversation_id: Uuid,
        agent: &Agent,
        tc: &ToolCall,
        step_idx: u32,
        cancellation_token: &CancellationToken,
    ) -> Result<LlmMessage, AgentError> {
        let tool_step_start = Instant::now();
        let args_val: Value =
            serde_json::from_str(&tc.arguments).unwrap_or(Value::String(tc.arguments.clone()));

        self.emit_tool_start(run_id, &tc.name, &tc.arguments)
            .await?;

        let tool_step = self
            .create_initial_step(run_id, step_idx, args_val.clone())
            .await?;

        let tool_context = ToolContext {
            agent_id: agent.id.clone(),
            run_id,
            workspace_root: self.workspace_root.to_string(),
        };

        let (content, success, decision) = self
            .dispatch_tool(agent, &tc.name, args_val, tool_context, cancellation_token)
            .await?;

        self.record_audit(run_id, &tc.name, &tc.arguments, decision, &content)
            .await;

        let elapsed = tool_step_start.elapsed().as_millis() as u64;
        self.finish_step(tool_step, success, elapsed, &content)
            .await?;

        self.emit_tool_finished(run_id, &tc.name, success, &content)
            .await?;

        self.persist_tool_message(conversation_id, run_id, &tc.id, &content)
            .await?;

        Ok(LlmMessage {
            role: "tool".to_string(),
            content,
            tool_calls: None,
            tool_call_id: Some(tc.id.clone()),
        })
    }

    async fn dispatch_tool(
        &self,
        agent: &Agent,
        tool_name: &str,
        args_val: Value,
        context: ToolContext,
        cancellation_token: &CancellationToken,
    ) -> Result<(String, bool, &'static str), AgentError> {
        match Self::check_policy(agent, tool_name, &args_val, self.workspace_root) {
            Ok(()) => {
                let tool_exec_res = tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        return Err(AgentError::Cancelled);
                    }
                    res = self.tool_host.execute(tool_name, args_val, context) => res
                };

                match tool_exec_res {
                    Ok(res) => Ok((res.content, res.success, "allow")),
                    Err(e) => Ok((format!("Tool execution error: {e}"), false, "allow")),
                }
            }
            Err(policy_err) => Ok((
                format!("Chính sách bảo mật từ chối: {policy_err}"),
                false,
                "deny",
            )),
        }
    }

    async fn record_audit(
        &self,
        run_id: Uuid,
        tool: &str,
        args: &str,
        decision: &str,
        result: &str,
    ) {
        let args_digest = truncate_with_ellipsis(args, 200);
        let result_digest = Some(truncate_with_ellipsis(result, 200));

        let entry = AuditLog {
            id: Uuid::now_v7(),
            timestamp: chrono::Utc::now(),
            run_id: Some(run_id),
            tool_name: tool.to_string(),
            args_digest,
            decision: decision.to_string(),
            result_digest,
        };
        let _ = self.store.record_audit_log(&entry).await;
    }

    async fn emit_tool_start(
        &self,
        run_id: Uuid,
        name: &str,
        args: &str,
    ) -> Result<(), AgentError> {
        let preview = truncate_with_ellipsis(args, 100);
        self.event_sink
            .emit(RunEvent::ToolStarted {
                run_id: run_id.to_string(),
                tool_name: name.to_string(),
                args_preview: preview,
            })
            .await?;
        Ok(())
    }

    async fn emit_tool_finished(
        &self,
        run_id: Uuid,
        name: &str,
        success: bool,
        content: &str,
    ) -> Result<(), AgentError> {
        let summary = truncate_with_ellipsis(content, 100);
        self.event_sink
            .emit(RunEvent::ToolFinished {
                run_id: run_id.to_string(),
                tool_name: name.to_string(),
                success,
                summary,
            })
            .await?;
        Ok(())
    }

    async fn create_initial_step(
        &self,
        run_id: Uuid,
        step_idx: u32,
        input: Value,
    ) -> Result<Step, AgentError> {
        let step = Step {
            id: Uuid::now_v7(),
            run_id,
            idx: step_idx,
            kind: StepKind::Tool,
            input,
            output: None,
            status: StepStatus::Running,
            duration_ms: None,
            created_at: chrono::Utc::now(),
        };
        self.store.create_step(&step).await?;
        Ok(step)
    }

    async fn finish_step(
        &self,
        mut step: Step,
        success: bool,
        duration_ms: u64,
        content: &str,
    ) -> Result<(), AgentError> {
        step.status = if success {
            StepStatus::Completed
        } else {
            StepStatus::Failed
        };
        step.duration_ms = Some(duration_ms);
        step.output = Some(Value::String(content.to_string()));
        let _ = self.store.create_step(&step).await;
        Ok(())
    }

    async fn persist_tool_message(
        &self,
        conv_id: Uuid,
        run_id: Uuid,
        call_id: &str,
        content: &str,
    ) -> Result<(), AgentError> {
        let tool_msg = Message {
            id: Uuid::now_v7(),
            conversation_id: conv_id,
            run_id: Some(run_id),
            role: MessageRole::Tool,
            parts: vec![MessagePart::ToolResult {
                tool_call_id: call_id.to_string(),
                result: Value::String(content.to_string()),
            }],
            created_at: chrono::Utc::now(),
        };
        self.store.append_message(&tool_msg).await?;
        Ok(())
    }

    fn check_policy(
        agent: &Agent,
        tool_name: &str,
        args: &Value,
        workspace_root: &str,
    ) -> Result<(), PolicyError> {
        let deny = |reason: &str| PolicyError::PermissionDenied {
            agent_id: agent.id.clone(),
            operation: tool_name.to_string(),
            reason: reason.to_string(),
        };
        // The model is only shown the agent's tools, but a prompt-injected call can name any
        let tools = &agent.tools;
        if !tools
            .builtin
            .iter()
            .chain(&tools.mcp)
            .any(|t| t == tool_name)
        {
            return Err(deny("tool is not assigned to this agent"));
        }

        let ws_path = std::path::Path::new(workspace_root);
        let perms = &agent.permissions;
        let str_arg = |key: &str| args.get(key).and_then(Value::as_str).unwrap_or("").trim();
        match tool_name {
            "web_search" => {
                PermissionChecker::check_network(&agent.id, &perms.network, true)?;
            }
            "web_fetch" => {
                PermissionChecker::check_network(&agent.id, &perms.network, false)?;
                UrlGuard::validate_url(str_arg("url"))?;
            }
            "fs_read" => {
                let path = str_arg("path");
                PathGuard::check_read_path(ws_path, path)?;
                PermissionChecker::check_fs_read(&agent.id, perms, path)?;
            }
            "fs_list" => match str_arg("path") {
                // Listing the workspace root needs read access to all of it
                "" | "." => PermissionChecker::check_fs_read(&agent.id, perms, "")?,
                path => {
                    PathGuard::check_read_path(ws_path, path)?;
                    PermissionChecker::check_fs_read(&agent.id, perms, path)?;
                }
            },
            "report_write" => {
                let filename = match str_arg("filename") {
                    "" => "report.md",
                    f => f,
                };
                let path = format!("reports/{filename}");
                PathGuard::check_write_path(ws_path, &path)?;
                PermissionChecker::check_fs_write(&agent.id, perms, &path)?;
            }
            "report_read" => {
                let path = format!("reports/{}", str_arg("filename"));
                PathGuard::check_read_path(ws_path, &path)?;
                PermissionChecker::check_fs_read(&agent.id, perms, &path)?;
            }
            "report_list" => PermissionChecker::check_fs_read(&agent.id, perms, "reports")?,
            // MCP is not implemented yet; its servers will carry their own policy
            _ if tools.mcp.iter().any(|t| t == tool_name) => {}
            _ => return Err(deny("unknown builtin tool")),
        }
        Ok(())
    }
}
