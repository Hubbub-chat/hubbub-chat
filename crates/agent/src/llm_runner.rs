//! LLM step execution helper for agent runtime.

use std::sync::Arc;
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use hubbub_domain::entities::agent::Agent;
use hubbub_domain::entities::run::{Step, StepKind, StepStatus};
use hubbub_domain::events::RunEvent;
use hubbub_domain::ports::event_sink::EventSink;
use hubbub_domain::ports::llm::{
    LlmChunk, LlmConfig, LlmMessage, LlmProvider, ToolCall, ToolSchema,
};
use hubbub_domain::ports::store::Store;

use crate::budget::BudgetTracker;
use crate::errors::AgentError;

pub struct LlmStepParams<'a> {
    pub run_id: Uuid,
    pub agent: &'a Agent,
    pub working_messages: &'a [LlmMessage],
    pub active_tools: &'a [ToolSchema],
    pub step_idx: u32,
    pub cancellation_token: &'a CancellationToken,
}

pub struct LlmRunner<'a> {
    pub store: &'a Arc<dyn Store>,
    pub llm_provider: &'a Arc<dyn LlmProvider>,
    pub event_sink: &'a Arc<dyn EventSink>,
}

impl<'a> LlmRunner<'a> {
    pub async fn run_step(
        &self,
        params: LlmStepParams<'_>,
        budget: &mut BudgetTracker,
    ) -> Result<(String, Vec<ToolCall>, u64, u64), AgentError> {
        let step_id = Uuid::now_v7();
        let step_start = Instant::now();
        let mut step = Step {
            id: step_id,
            run_id: params.run_id,
            idx: params.step_idx,
            kind: StepKind::Llm,
            input: serde_json::json!({
                "model": &params.agent.model,
                "messages_count": params.working_messages.len(),
            }),
            output: None,
            status: StepStatus::Running,
            duration_ms: None,
            created_at: chrono::Utc::now(),
        };
        self.store.create_step(&step).await?;

        let (mut text, tool_calls, mut p_tokens, mut c_tokens, finish_reason) =
            match self.execute_llm_stream(&params, budget).await {
                Ok(res) => res,
                Err(e) => {
                    self.record_step_failure(&mut step, step_start, &e).await;
                    return Err(e);
                }
            };

        if finish_reason.as_deref() == Some("length") && tool_calls.is_empty() && !text.is_empty() {
            let _ = self
                .run_continuation(&params, budget, &mut text, &mut p_tokens, &mut c_tokens)
                .await;
        }

        step.status = StepStatus::Completed;
        step.duration_ms = Some(step_start.elapsed().as_millis() as u64);
        step.output = Some(serde_json::json!({
            "text_length": text.len(),
            "tool_calls_count": tool_calls.len(),
        }));
        let _ = self.store.create_step(&step).await;

        Ok((text, tool_calls, p_tokens, c_tokens))
    }

    async fn execute_llm_stream(
        &self,
        params: &LlmStepParams<'_>,
        budget: &mut BudgetTracker,
    ) -> Result<(String, Vec<ToolCall>, u64, u64, Option<String>), AgentError> {
        let llm_config = LlmConfig {
            model: params.agent.model.clone(),
            max_tokens: None,
            temperature: Some(0.7),
        };

        let stream_result = tokio::select! {
            _ = params.cancellation_token.cancelled() => return Err(AgentError::Cancelled),
            res = self.llm_provider.chat_stream(
                params.working_messages.to_vec(),
                params.active_tools.to_vec(),
                llm_config,
            ) => res
        };

        let mut stream = stream_result.map_err(AgentError::Domain)?;
        let mut text = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut p_tokens = 0u64;
        let mut c_tokens = 0u64;
        let mut finish_reason: Option<String> = None;

        loop {
            let chunk_opt = tokio::select! {
                _ = params.cancellation_token.cancelled() => return Err(AgentError::Cancelled),
                chunk = stream.next() => chunk
            };

            match chunk_opt {
                Some(Ok(LlmChunk::Delta(delta))) => {
                    text.push_str(&delta);
                    self.event_sink
                        .emit(RunEvent::MessageDelta {
                            run_id: params.run_id.to_string(),
                            content: delta,
                        })
                        .await?;
                }
                Some(Ok(LlmChunk::ToolCall(tc))) => tool_calls.push(tc),
                Some(Ok(LlmChunk::Usage(usage))) => {
                    p_tokens += usage.prompt_tokens;
                    c_tokens += usage.completion_tokens;
                    budget.record_usage(&usage);
                }
                Some(Ok(LlmChunk::FinishReason(r))) => finish_reason = Some(r),
                Some(Ok(LlmChunk::Done)) | None => break,
                Some(Err(e)) => return Err(AgentError::Domain(e)),
            }
        }

        Ok((text, tool_calls, p_tokens, c_tokens, finish_reason))
    }

    async fn run_continuation(
        &self,
        params: &LlmStepParams<'_>,
        budget: &mut BudgetTracker,
        text: &mut String,
        p_tokens: &mut u64,
        c_tokens: &mut u64,
    ) -> Result<(), AgentError> {
        let mut continuations = 0;
        let max_continuations = 3;

        while continuations < max_continuations {
            if params.cancellation_token.is_cancelled() || budget.check_limits().is_err() {
                break;
            }
            continuations += 1;
            tracing::info!(
                "Phản hồi bị chạm giới hạn token (finish_reason=length). Đang kích hoạt tiếp tục sinh #{continuations}..."
            );

            let mut cont_messages = params.working_messages.to_vec();
            cont_messages.push(LlmMessage {
                role: "assistant".to_string(),
                content: text.clone(),
                tool_calls: None,
                tool_call_id: None,
            });
            cont_messages.push(LlmMessage {
                role: "user".to_string(),
                content: "Vui lòng tiếp tục viết tiếp phần còn lại từ đúng điểm dừng trên, không lặp lại đoạn văn bản đã viết.".to_string(),
                tool_calls: None,
                tool_call_id: None,
            });

            let cont_config = LlmConfig {
                model: params.agent.model.clone(),
                max_tokens: None,
                temperature: Some(0.7),
            };

            let stream_res = tokio::select! {
                _ = params.cancellation_token.cancelled() => return Err(AgentError::Cancelled),
                res = self.llm_provider.chat_stream(cont_messages, vec![], cont_config) => res
            };

            let mut stream = match stream_res {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("Lỗi khi khởi tạo stream tiếp tục: {e}");
                    break;
                }
            };

            let mut reason = None;
            loop {
                let chunk_opt = tokio::select! {
                    _ = params.cancellation_token.cancelled() => return Err(AgentError::Cancelled),
                    chunk = stream.next() => chunk
                };

                match chunk_opt {
                    Some(Ok(LlmChunk::Delta(delta))) => {
                        text.push_str(&delta);
                        self.event_sink
                            .emit(RunEvent::MessageDelta {
                                run_id: params.run_id.to_string(),
                                content: delta,
                            })
                            .await?;
                    }
                    Some(Ok(LlmChunk::Usage(usage))) => {
                        *p_tokens += usage.prompt_tokens;
                        *c_tokens += usage.completion_tokens;
                        budget.record_usage(&usage);
                    }
                    Some(Ok(LlmChunk::FinishReason(r))) => reason = Some(r),
                    Some(Ok(LlmChunk::Done)) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }

            if reason.as_deref() != Some("length") {
                break;
            }
        }

        Ok(())
    }

    async fn record_step_failure(&self, step: &mut Step, step_start: Instant, err: &AgentError) {
        step.status = StepStatus::Failed;
        step.duration_ms = Some(step_start.elapsed().as_millis() as u64);
        step.output = Some(serde_json::json!({ "error": err.to_string() }));
        // No Error event here: the runtime reports the failure once when it finalizes the run
        let _ = self.store.create_step(step).await;
    }
}
