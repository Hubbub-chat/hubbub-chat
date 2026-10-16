use chrono::Utc;
use hubbub_domain::entities::agent::Agent;
use hubbub_domain::entities::conversation::{Message, MessagePart, MessageRole};
use hubbub_domain::ports::llm::{LlmMessage, ToolCall};

pub struct ContextBuilder;

impl ContextBuilder {
    pub fn build(
        agent: &Agent,
        history: &[Message],
        current_prompt: Option<&str>,
    ) -> Vec<LlmMessage> {
        let mut messages = Vec::new();

        // 1. System Prompt with Current Time and Language Preference
        let now_utc = Utc::now().to_rfc3339();
        let system_content = format!(
            "{}\n\n[Current Time (UTC): {}]\n[Instruction: Provide comprehensive, well-structured, in-depth answers. Do not rush or artificially truncate your explanations. When explaining concepts or code, provide complete runnable examples with thorough step-by-step reasoning. Always conclude your thoughts cleanly and close all open code blocks, lists, and JSON blocks properly. Reply in Vietnamese when the user asks in Vietnamese.]",
            agent.system_prompt.trim(),
            now_utc
        );

        messages.push(LlmMessage {
            role: "system".to_string(),
            content: system_content,
            tool_calls: None,
            tool_call_id: None,
        });

        // 2. Conversation History
        for msg in history {
            match msg.role {
                MessageRole::User => {
                    let mut text = String::new();
                    for part in &msg.parts {
                        if let MessagePart::Text(t) = part {
                            text.push_str(t);
                        }
                    }
                    if !text.is_empty() {
                        // Merge consecutive user messages to avoid confusing LLMs
                        if let Some(last) = messages.last_mut().filter(|m| m.role == "user") {
                            if last.content != text {
                                last.content.push_str("\n\n");
                                last.content.push_str(&text);
                            }
                            continue;
                        }
                        messages.push(LlmMessage {
                            role: "user".to_string(),
                            content: text,
                            tool_calls: None,
                            tool_call_id: None,
                        });
                    }
                }
                MessageRole::Assistant => {
                    let mut text = String::new();
                    let mut tool_calls = Vec::new();

                    for part in &msg.parts {
                        match part {
                            MessagePart::Text(t) => text.push_str(t),
                            MessagePart::ToolCall {
                                id,
                                name,
                                arguments,
                                extra_content,
                            } => {
                                tool_calls.push(ToolCall {
                                    id: id.clone(),
                                    name: name.clone(),
                                    arguments: arguments.to_string(),
                                    extra_content: extra_content.clone(),
                                });
                            }
                            _ => {}
                        }
                    }

                    // Skip empty assistant messages (e.g. from failed or aborted runs)
                    if text.is_empty() && tool_calls.is_empty() {
                        continue;
                    }

                    messages.push(LlmMessage {
                        role: "assistant".to_string(),
                        content: text,
                        tool_calls: if tool_calls.is_empty() {
                            None
                        } else {
                            Some(tool_calls)
                        },
                        tool_call_id: None,
                    });
                }
                MessageRole::Tool => {
                    for part in &msg.parts {
                        if let MessagePart::ToolResult {
                            tool_call_id,
                            result,
                        } = part
                        {
                            let content_str = match result {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                            messages.push(LlmMessage {
                                role: "tool".to_string(),
                                content: content_str,
                                tool_calls: None,
                                tool_call_id: Some(tool_call_id.clone()),
                            });
                        }
                    }
                }
                MessageRole::System => {}
            }
        }

        let mut messages = Self::pair_tool_results(messages);

        // 3. New User Prompt (if provided)
        if let Some(prompt) = current_prompt.filter(|p| !p.is_empty()) {
            if let Some(last) = messages.last_mut().filter(|m| m.role == "user") {
                if last.content != prompt {
                    last.content.push_str("\n\n");
                    last.content.push_str(prompt);
                }
                return messages;
            }
            messages.push(LlmMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        messages
    }

    /// Providers reject history where an assistant tool call has no result right after it,
    /// or a tool result answers no call. A run cancelled or failed mid-batch leaves exactly
    /// that, so fill missing results and drop orphans instead of breaking the conversation.
    fn pair_tool_results(messages: Vec<LlmMessage>) -> Vec<LlmMessage> {
        let mut out = Vec::with_capacity(messages.len());
        let mut pending: Vec<String> = Vec::new();

        for msg in messages {
            if msg.role == "tool" {
                let answered = msg
                    .tool_call_id
                    .as_ref()
                    .and_then(|id| pending.iter().position(|p| p == id));
                if let Some(idx) = answered {
                    pending.remove(idx);
                    out.push(msg);
                }
                continue;
            }
            Self::fill_missing_results(&mut out, &mut pending);
            if let Some(calls) = &msg.tool_calls {
                pending = calls.iter().map(|c| c.id.clone()).collect();
            }
            out.push(msg);
        }
        Self::fill_missing_results(&mut out, &mut pending);
        out
    }

    fn fill_missing_results(out: &mut Vec<LlmMessage>, pending: &mut Vec<String>) {
        for id in pending.drain(..) {
            out.push(LlmMessage {
                role: "tool".to_string(),
                content: "Tool call was cancelled before it produced a result.".to_string(),
                tool_calls: None,
                tool_call_id: Some(id),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, calls: Option<Vec<&str>>, call_id: Option<&str>) -> LlmMessage {
        LlmMessage {
            role: role.to_string(),
            content: String::new(),
            tool_calls: calls.map(|ids| {
                ids.into_iter()
                    .map(|id| ToolCall::new(id, "t", "{}"))
                    .collect()
            }),
            tool_call_id: call_id.map(str::to_string),
        }
    }

    #[test]
    fn pairs_every_tool_call_and_drops_orphan_results() {
        let out = ContextBuilder::pair_tool_results(vec![
            msg("user", None, None),
            msg("tool", None, Some("orphan")),
            msg("assistant", Some(vec!["a", "b"]), None),
            msg("tool", None, Some("b")),
            msg("user", None, None),
        ]);
        let shape: Vec<_> = out
            .iter()
            .map(|m| (m.role.as_str(), m.tool_call_id.as_deref()))
            .collect();
        assert_eq!(
            shape,
            vec![
                ("user", None),
                ("assistant", None),
                ("tool", Some("b")),
                ("tool", Some("a")),
                ("user", None),
            ]
        );
    }
}
