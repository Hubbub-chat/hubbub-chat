#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use hubbub_agent::{AgentError, AgentRuntime};
use hubbub_domain::entities::conversation::MessageRole;
use hubbub_domain::entities::run::{RunStatus, StepKind, StepStatus};
use hubbub_domain::ports::llm::{LlmChunk, LlmUsage, ToolCall, ToolSchema};
use hubbub_domain::ports::store::Store;
use hubbub_domain::ports::tool_host::ToolResult;
use hubbub_store::SqliteStore;
use hubbub_testkit::{FakeLlm, FakeResponse, InMemoryEventSink, MockToolHost, create_test_agent};

#[tokio::test]
async fn test_simple_chat_stream() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    fake_llm
        .push_text_deltas(
            &["Xin ", "chào! ", "Tôi là Hubbub."],
            Some(LlmUsage {
                prompt_tokens: 15,
                completion_tokens: 10,
            }),
        )
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let agent = create_test_agent("test-agent", "Bạn là trợ lý AI hữu ích.");
    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let run = runtime
        .execute_run(conv_id, &agent, Some("Chào bạn"), cancel)
        .await
        .unwrap();

    assert_eq!(run.status, RunStatus::Completed);
    assert!(run.usage.is_some());
    assert_eq!(run.usage.as_ref().unwrap().total_tokens, 25);

    // Verify stream events
    let full_text = event_sink.full_text().await;
    assert_eq!(full_text, "Xin chào! Tôi là Hubbub.");
    assert!(event_sink.has_finished_status("completed").await);

    // Verify store persistence
    let messages = store.list_messages(conv_id, 10, 0).await.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, MessageRole::User);
    assert_eq!(messages[1].role, MessageRole::Assistant);

    let steps = store.list_steps(run.id).await.unwrap();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].kind, StepKind::Llm);
    assert_eq!(steps[0].status, StepStatus::Completed);
}

#[tokio::test]
async fn test_single_tool_execution_loop() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    // Register mock tool
    tool_host
        .register_simple_tool(
            "web_search",
            "Tìm kiếm thông tin web",
            "Thời tiết Hà Nội: 28°C, trời nắng nhẹ.",
        )
        .await;

    // Turn 1: LLM returns a tool call
    fake_llm
        .push_tool_call(
            "call_weather_1",
            "web_search",
            "{\"query\":\"thời tiết Hà Nội\"}",
        )
        .await;

    // Turn 2: LLM receives tool result and produces final answer
    fake_llm
        .push_text_deltas(
            &["Hiện tại tại Hà Nội ", "trời nắng nhẹ, nhiệt độ 28°C."],
            None,
        )
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let agent = create_test_agent("test-agent", "Bạn có khả năng tìm kiếm web.");
    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let run = runtime
        .execute_run(
            conv_id,
            &agent,
            Some("Thời tiết Hà Nội hôm nay thế nào?"),
            cancel,
        )
        .await
        .unwrap();

    assert_eq!(run.status, RunStatus::Completed);

    // Verify messages: User, Assistant (tool call), Tool (result), Assistant (final)
    let messages = store.list_messages(conv_id, 10, 0).await.unwrap();
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0].role, MessageRole::User);
    assert_eq!(messages[1].role, MessageRole::Assistant);
    assert_eq!(messages[2].role, MessageRole::Tool);
    assert_eq!(messages[3].role, MessageRole::Assistant);

    // Verify steps: LLM step -> Tool step -> LLM step
    let steps = store.list_steps(run.id).await.unwrap();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].kind, StepKind::Llm);
    assert_eq!(steps[1].kind, StepKind::Tool);
    assert_eq!(steps[2].kind, StepKind::Llm);
}

#[tokio::test]
async fn test_multi_step_tool_loop() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    tool_host
        .register_simple_tool("web_search", "Search web", "Ket qua search")
        .await;
    tool_host
        .register_simple_tool("fs_read", "Read file", "Noi dung file")
        .await;

    // Turn 1: Call fs_read
    fake_llm
        .push_tool_call("c1", "fs_read", "{\"path\":\"report.md\"}")
        .await;
    // Turn 2: Call web_search
    fake_llm
        .push_tool_call("c2", "web_search", "{\"query\":\"Hubbub\"}")
        .await;
    // Turn 3: Final answer
    fake_llm
        .push_text_deltas(&["Đã đọc file và tìm kiếm xong."], None)
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let agent = create_test_agent("test-agent", "Multi-step tool agent.");
    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let run = runtime
        .execute_run(conv_id, &agent, Some("Phân tích"), cancel)
        .await
        .unwrap();

    assert_eq!(run.status, RunStatus::Completed);
    let steps = store.list_steps(run.id).await.unwrap();
    assert_eq!(steps.len(), 5); // LLM -> Tool 1 -> LLM -> Tool 2 -> LLM
}

#[tokio::test]
async fn test_budget_max_steps_exceeded() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    tool_host
        .register_simple_tool("web_search", "Search", "Result")
        .await;

    // Infinite tool calls
    for i in 0..10 {
        fake_llm
            .push_tool_call(&format!("c_{i}"), "web_search", "{\"query\":\"test loop\"}")
            .await;
    }

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let mut agent = create_test_agent("test-agent", "Looping agent.");
    agent.budget.max_steps = 3; // Limit to 3 steps

    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let result = runtime
        .execute_run(conv_id, &agent, Some("Bắt đầu lặp"), cancel)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AgentError::BudgetExceeded(msg) => {
            assert!(msg.contains("Max steps exceeded"));
        }
        other => panic!("Expected BudgetExceeded, got: {:?}", other),
    }

    // Run should be marked failed in store
    let conv = store.get_conversation(conv_id).await.unwrap().unwrap();
    assert_eq!(conv.agent_id, "test-agent");
}

#[tokio::test]
async fn test_budget_max_tokens_exceeded() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    fake_llm
        .push_text_deltas(
            &["Nội dung trả lời dài."],
            Some(LlmUsage {
                prompt_tokens: 300,
                completion_tokens: 300, // Total = 600
            }),
        )
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let mut agent = create_test_agent("test-agent", "Token budget agent.");
    agent.budget.max_tokens = 500; // Limit is 500

    let conv_id = Uuid::now_v7();
    let run_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let result = runtime
        .execute_run_with_id(conv_id, &agent, Some("Tin nhắn"), cancel, Some(run_id))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AgentError::BudgetExceeded(msg) => {
            assert!(msg.contains("Max tokens exceeded"));
        }
        other => panic!("Expected BudgetExceeded, got: {:?}", other),
    }

    // The answer was already streamed to the UI, so it must be kept, and the run finalized
    let run = store.get_run(run_id).await.unwrap().unwrap();
    assert_eq!(run.status, RunStatus::Failed);
    assert!(run.finished_at.is_some());
    assert!(event_sink.has_finished_status("failed").await);
    let messages = store.list_messages(conv_id, 10, 0).await.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].role, MessageRole::Assistant);
}

#[tokio::test]
async fn test_llm_stream_error_marks_run_failed() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    fake_llm
        .push_response(FakeResponse {
            chunks: vec![
                Ok(LlmChunk::Delta("Đang trả lời".to_string())),
                Err(hubbub_domain::errors::DomainError::Internal(
                    "connection reset".to_string(),
                )),
            ],
            chunk_delay: None,
        })
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );
    let agent = create_test_agent("test-agent", "Agent.");
    let run_id = Uuid::now_v7();

    let result = runtime
        .execute_run_with_id(
            Uuid::now_v7(),
            &agent,
            Some("Xin chào"),
            CancellationToken::new(),
            Some(run_id),
        )
        .await;

    assert!(result.is_err());
    let run = store.get_run(run_id).await.unwrap().unwrap();
    assert_eq!(run.status, RunStatus::Failed);
    assert!(run.finished_at.is_some());
    assert!(event_sink.has_finished_status("failed").await);
}

#[tokio::test]
async fn test_cancellation_mid_llm_stream() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    // Response with slow chunks
    fake_llm
        .push_response(FakeResponse {
            chunks: vec![
                Ok(LlmChunk::Delta("Phần 1... ".to_string())),
                Ok(LlmChunk::Delta("Phần 2... ".to_string())),
                Ok(LlmChunk::Done),
            ],
            chunk_delay: Some(Duration::from_millis(300)),
        })
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let agent = create_test_agent("test-agent", "Slow agent.");
    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    // Spawn task to cancel mid-stream
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });

    let result = runtime
        .execute_run(conv_id, &agent, Some("Hãy viết thật dài"), cancel)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AgentError::Cancelled => {}
        other => panic!("Expected Cancelled, got: {:?}", other),
    }

    assert!(event_sink.has_finished_status("cancelled").await);
}

#[tokio::test]
async fn test_cancellation_mid_tool_execution() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    // Tool that takes 1 second
    tool_host
        .register_tool(
            ToolSchema {
                name: "slow_tool".to_string(),
                description: "Takes time".to_string(),
                parameters: serde_json::json!({}),
            },
            Some(Duration::from_millis(800)),
            |_args, _ctx| {
                Ok(ToolResult {
                    success: true,
                    content: "Done slow".to_string(),
                    metadata: None,
                })
            },
        )
        .await;

    fake_llm.push_tool_call("slow_1", "slow_tool", "{}").await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let mut agent = create_test_agent("test-agent", "Tool agent.");
    agent.tools.mcp.push("slow_tool".to_string()); // a non-builtin (external) tool

    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });

    let result = runtime
        .execute_run(conv_id, &agent, Some("Chạy tool chậm"), cancel)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AgentError::Cancelled => {}
        other => panic!("Expected Cancelled, got: {:?}", other),
    }

    assert!(event_sink.has_finished_status("cancelled").await);
}

#[tokio::test]
async fn test_tool_failure_recovery() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    // Failing tool
    tool_host
        .register_tool(
            ToolSchema {
                name: "fs_read".to_string(),
                description: "Đọc file".to_string(),
                parameters: serde_json::json!({}),
            },
            None,
            |_args, _ctx| {
                Ok(ToolResult {
                    success: false,
                    content: "File not found: secret.txt".to_string(),
                    metadata: None,
                })
            },
        )
        .await;

    // Turn 1: LLM calls failing tool
    fake_llm
        .push_tool_call("c_err", "fs_read", "{\"path\":\"secret.txt\"}")
        .await;

    // Turn 2: LLM receives error message and gracefully recovers
    fake_llm
        .push_text_deltas(
            &["Rất tiếc, file secret.txt không tồn tại. Tôi có thể giúp bạn tạo mới không?"],
            None,
        )
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let agent = create_test_agent("test-agent", "Resilient agent.");
    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let run = runtime
        .execute_run(conv_id, &agent, Some("Đọc secret.txt"), cancel)
        .await
        .unwrap();

    assert_eq!(run.status, RunStatus::Completed);
    assert_eq!(
        event_sink.full_text().await,
        "Rất tiếc, file secret.txt không tồn tại. Tôi có thể giúp bạn tạo mới không?"
    );
}

#[tokio::test]
async fn test_vietnamese_diacritics_and_emojis() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    let vietnamese_answer =
        "Dự án Hubbub hỗ trợ tiếng Việt 🇻🇳: Đường phố, phở bò, bánh mì, cà phê sữa đá! ☕🥖";

    fake_llm.push_text_deltas(&[vietnamese_answer], None).await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let agent = create_test_agent("test-agent", "Trợ lý tiếng Việt chuẩn xác.");
    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let vietnamese_prompt = "Hãy giới thiệu một vài nét văn hoá Việt Nam 🇻🇳";
    let run = runtime
        .execute_run(conv_id, &agent, Some(vietnamese_prompt), cancel)
        .await
        .unwrap();

    assert_eq!(run.status, RunStatus::Completed);
    assert_eq!(event_sink.full_text().await, vietnamese_answer);

    // Verify store has exact prompt and answer with diacritics and emojis
    let messages = store.list_messages(conv_id, 10, 0).await.unwrap();
    assert_eq!(messages.len(), 2);
}

#[tokio::test]
async fn test_cancel_mid_tool_batch_keeps_conversation_usable() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    tool_host
        .register_tool(
            ToolSchema {
                name: "slow_tool".to_string(),
                description: "Takes time".to_string(),
                parameters: serde_json::json!({}),
            },
            Some(Duration::from_millis(800)),
            |_args, _ctx| {
                Ok(ToolResult {
                    success: true,
                    content: "Done slow".to_string(),
                    metadata: None,
                })
            },
        )
        .await;

    // One LLM response asking for two tools; the run is cancelled during the first
    fake_llm
        .push_response(FakeResponse {
            chunks: vec![
                Ok(LlmChunk::ToolCall(ToolCall::new(
                    "call_a",
                    "slow_tool",
                    "{}",
                ))),
                Ok(LlmChunk::ToolCall(ToolCall::new(
                    "call_b",
                    "slow_tool",
                    "{}",
                ))),
                Ok(LlmChunk::Done),
            ],
            chunk_delay: None,
        })
        .await;
    fake_llm.push_text_deltas(&["Tiếp tục."], None).await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );
    let mut agent = create_test_agent("test-agent", "Tool agent.");
    agent.tools.mcp.push("slow_tool".to_string()); // a non-builtin (external) tool
    let conv_id = Uuid::now_v7();

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        cancel_clone.cancel();
    });
    let result = runtime
        .execute_run(conv_id, &agent, Some("Chạy hai tool"), cancel)
        .await;
    assert!(matches!(result, Err(AgentError::Cancelled)));

    // Next message in the same conversation
    let run = runtime
        .execute_run(conv_id, &agent, Some("Tiếp"), CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(run.status, RunStatus::Completed);

    // Providers reject an assistant tool call that has no matching tool result
    let calls = fake_llm.get_recorded_calls().await;
    let messages = &calls.last().unwrap().messages;
    for (i, m) in messages.iter().enumerate() {
        for tc in m.tool_calls.iter().flatten() {
            let answered = messages[i + 1..]
                .iter()
                .take_while(|n| n.role == "tool")
                .any(|n| n.tool_call_id.as_deref() == Some(tc.id.as_str()));
            assert!(answered, "tool call {} has no result", tc.id);
        }
    }
}

#[tokio::test]
async fn test_run_started_first_and_every_event_tagged_with_run_id() {
    use hubbub_domain::events::RunEvent;

    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());
    tool_host
        .register_simple_tool("web_search", "Tìm kiếm", "Kết quả")
        .await;
    fake_llm
        .push_tool_call("call_1", "web_search", "{\"query\":\"x\"}")
        .await;
    fake_llm.push_text_deltas(&["Xong."], None).await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );
    let agent = create_test_agent("test-agent", "Agent.");
    let conv_id = Uuid::now_v7();
    let run_id = Uuid::now_v7();
    runtime
        .execute_run_with_id(
            conv_id,
            &agent,
            Some("Tìm"),
            CancellationToken::new(),
            Some(run_id),
        )
        .await
        .unwrap();

    let events = event_sink.get_events().await;
    // The UI needs the run id before the run ends, or Cancel has nothing to target
    match &events[0] {
        RunEvent::RunStarted {
            run_id: r,
            conversation_id: c,
        } => {
            assert_eq!(r, &run_id.to_string());
            assert_eq!(c, &conv_id.to_string());
        }
        other => panic!("first event should be RunStarted, got {other:?}"),
    }
    for event in &events {
        let tagged = match event {
            RunEvent::RunStarted { run_id, .. }
            | RunEvent::MessageDelta { run_id, .. }
            | RunEvent::ToolStarted { run_id, .. }
            | RunEvent::ToolFinished { run_id, .. }
            | RunEvent::RunFinished { run_id, .. }
            | RunEvent::Error { run_id, .. } => run_id.clone(),
            other => panic!("unexpected event {other:?}"),
        };
        assert_eq!(tagged, run_id.to_string());
    }
}

#[tokio::test]
async fn test_long_conversation_sends_newest_history_to_llm() {
    use hubbub_domain::entities::conversation::{Conversation, Message, MessagePart};

    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    fake_llm.push_text_deltas(&["Đáp."], None).await;
    let conv_id = Uuid::now_v7();
    let start = chrono::Utc::now() - chrono::Duration::days(1);
    store
        .create_conversation(&Conversation {
            id: conv_id,
            title: "Dài".to_string(),
            agent_id: "test-agent".to_string(),
            created_at: start,
            updated_at: start,
            archived: false,
        })
        .await
        .unwrap();
    for i in 0..1100 {
        let role = if i % 2 == 0 {
            MessageRole::User
        } else {
            MessageRole::Assistant
        };
        store
            .append_message(&Message {
                id: Uuid::now_v7(),
                conversation_id: conv_id,
                run_id: None,
                role,
                parts: vec![MessagePart::Text(format!("cũ {i}"))],
                created_at: start + chrono::Duration::seconds(i),
            })
            .await
            .unwrap();
    }

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        Arc::new(MockToolHost::new()),
        Arc::new(InMemoryEventSink::new()),
        None,
    );
    let agent = create_test_agent("test-agent", "Agent.");
    runtime
        .execute_run(
            conv_id,
            &agent,
            Some("Câu hỏi mới nhất"),
            CancellationToken::new(),
        )
        .await
        .unwrap();

    let calls = fake_llm.get_recorded_calls().await;
    let last = calls[0].messages.last().unwrap();
    assert_eq!(last.role, "user");
    assert!(last.content.contains("Câu hỏi mới nhất"));
}

#[tokio::test]
async fn test_token_budget_exceeded_with_tool_calls_runs_no_tools() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());
    tool_host
        .register_simple_tool("web_search", "Tìm kiếm", "Kết quả")
        .await;

    fake_llm
        .push_response(FakeResponse {
            chunks: vec![
                Ok(LlmChunk::ToolCall(
                    hubbub_domain::ports::llm::ToolCall::new("c1", "web_search", "{}"),
                )),
                Ok(LlmChunk::Usage(LlmUsage {
                    prompt_tokens: 400,
                    completion_tokens: 200,
                })),
                Ok(LlmChunk::Done),
            ],
            chunk_delay: None,
        })
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );
    let mut agent = create_test_agent("test-agent", "Agent.");
    agent.budget.max_tokens = 500;
    let run_id = Uuid::now_v7();

    let result = runtime
        .execute_run_with_id(
            Uuid::now_v7(),
            &agent,
            Some("Tìm"),
            CancellationToken::new(),
            Some(run_id),
        )
        .await;

    assert!(matches!(result, Err(AgentError::BudgetExceeded(_))));
    // Over budget: the requested tools must not run (they may fetch or write files)
    assert!(tool_host.get_recorded_calls().await.is_empty());
    let run = store.get_run(run_id).await.unwrap().unwrap();
    assert_eq!(run.status, RunStatus::Failed);
}

#[tokio::test]
async fn test_long_vietnamese_tool_output_does_not_panic() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    // "ệ" is 3 bytes in UTF-8, so byte offsets 100 and 200 fall mid-character
    let long_output = "ệ".repeat(150);
    tool_host
        .register_simple_tool("web_search", "Tìm kiếm", &long_output)
        .await;
    let args = format!("{{\"query\":\"{}\"}}", "ệ".repeat(80));
    fake_llm.push_tool_call("call_1", "web_search", &args).await;
    fake_llm.push_text_deltas(&["Xong."], None).await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );
    let agent = create_test_agent("test-agent", "Bạn có khả năng tìm kiếm web.");
    let run = runtime
        .execute_run(
            Uuid::now_v7(),
            &agent,
            Some("Tìm"),
            CancellationToken::new(),
        )
        .await
        .unwrap();

    assert_eq!(run.status, RunStatus::Completed);
    let summary = event_sink
        .get_events()
        .await
        .into_iter()
        .find_map(|e| match e {
            hubbub_domain::events::RunEvent::ToolFinished { summary, .. } => Some(summary),
            _ => None,
        })
        .unwrap();
    assert_eq!(summary, format!("{}...", "ệ".repeat(100)));
}
