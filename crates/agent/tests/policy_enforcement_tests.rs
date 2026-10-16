#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use hubbub_agent::AgentRuntime;
use hubbub_domain::entities::agent::{AgentPermissions, NetworkPolicy};
use hubbub_domain::entities::conversation::MessageRole;
use hubbub_domain::entities::run::{RunStatus, StepStatus};
use hubbub_domain::ports::store::Store;
use hubbub_store::SqliteStore;
use hubbub_testkit::{FakeLlm, InMemoryEventSink, MockToolHost, create_test_agent};

#[tokio::test]
async fn test_agent_blocked_from_network_when_policy_none() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    tool_host
        .register_simple_tool("web_search", "Search web", "Should not be reached")
        .await;

    // Step 1: LLM wants to call web_search
    fake_llm
        .push_tool_call("call_web_1", "web_search", r#"{"query":"rust news"}"#)
        .await;

    // Step 2: After rejection, LLM finishes run
    fake_llm
        .push_text_deltas(
            &["Tôi không thể truy cập Internet do chính sách bảo mật."],
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

    let mut agent = create_test_agent("offline-agent", "You are an offline assistant.");
    agent.tools.builtin = vec!["web_search".to_string()];
    agent.permissions = AgentPermissions {
        fs_read: vec![],
        fs_write: vec![],
        network: NetworkPolicy::None, // Completely blocked
    };

    let conv_id = Uuid::now_v7();
    let cancel = CancellationToken::new();

    let run = runtime
        .execute_run(conv_id, &agent, Some("Tìm kiếm tin tức"), cancel)
        .await
        .unwrap();

    assert_eq!(run.status, RunStatus::Completed);

    // Verify tool execution was intercepted and marked failed
    let steps = store.list_steps(run.id).await.unwrap();
    assert_eq!(steps.len(), 3); // LLM -> Tool -> LLM
    let tool_step = &steps[1];
    assert_eq!(tool_step.status, StepStatus::Failed);
    let output = tool_step.output.as_ref().unwrap().as_str().unwrap();
    assert!(
        output.contains("Chính sách bảo mật từ chối"),
        "Unexpected tool step output: {output}"
    );

    // Verify tool result message in store
    let messages = store.list_messages(conv_id, 10, 0).await.unwrap();
    let tool_msg = messages
        .iter()
        .find(|m| m.role == MessageRole::Tool)
        .unwrap();
    let parts_str = serde_json::to_string(&tool_msg.parts).unwrap();
    assert!(parts_str.contains("Chính sách bảo mật từ chối"));
}

#[tokio::test]
async fn test_agent_blocked_from_path_traversal() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    tool_host
        .register_simple_tool("fs_read", "Read file", "Should not be reached")
        .await;

    // Step 1: Malicious path traversal tool call
    fake_llm
        .push_tool_call(
            "call_traversal_1",
            "fs_read",
            r#"{"path":"../../windows/system32/config/sam"}"#,
        )
        .await;

    // Step 2: Final response
    fake_llm
        .push_text_deltas(&["Truy cập tệp thất bại."], None)
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        Some("D:/Personal/Vibe_coding/Hubbub".to_string()),
    );

    let mut agent = create_test_agent("fs-agent", "You are a filesystem agent.");
    agent.tools.builtin = vec!["fs_read".to_string()];
    agent.permissions = AgentPermissions {
        fs_read: vec!["**".to_string()],
        fs_write: vec![],
        network: NetworkPolicy::None,
    };

    let conv_id = Uuid::now_v7();
    let run = runtime
        .execute_run(conv_id, &agent, Some("Đọc SAM"), CancellationToken::new())
        .await
        .unwrap();

    let steps = store.list_steps(run.id).await.unwrap();
    let tool_step = &steps[1];
    assert_eq!(tool_step.status, StepStatus::Failed);
    let output = tool_step.output.as_ref().unwrap().as_str().unwrap();
    assert!(
        output.contains("Chính sách bảo mật từ chối") && output.contains("Path traversal"),
        "Unexpected tool step output: {output}"
    );
}

#[tokio::test]
async fn test_agent_blocked_from_ssrf_metadata_ip() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let event_sink = Arc::new(InMemoryEventSink::new());
    let tool_host = Arc::new(MockToolHost::new());

    tool_host
        .register_simple_tool("web_fetch", "Fetch web", "Should not be reached")
        .await;

    // Step 1: SSRF attack against AWS / GCP cloud metadata
    fake_llm
        .push_tool_call(
            "call_ssrf_1",
            "web_fetch",
            r#"{"url":"http://169.254.169.254/latest/meta-data"}"#,
        )
        .await;

    // Step 2: Final response
    fake_llm
        .push_text_deltas(&["Yêu cầu bị từ chối do vi phạm bảo mật."], None)
        .await;

    let runtime = AgentRuntime::new(
        store.clone(),
        fake_llm.clone(),
        tool_host.clone(),
        event_sink.clone(),
        None,
    );

    let mut agent = create_test_agent("fetch-agent", "You are a web agent.");
    agent.tools.builtin = vec!["web_fetch".to_string()];
    agent.permissions = AgentPermissions {
        fs_read: vec![],
        fs_write: vec![],
        network: NetworkPolicy::Open,
    };

    let conv_id = Uuid::now_v7();
    let run = runtime
        .execute_run(
            conv_id,
            &agent,
            Some("Lấy metadata"),
            CancellationToken::new(),
        )
        .await
        .unwrap();

    let steps = store.list_steps(run.id).await.unwrap();
    let tool_step = &steps[1];
    assert_eq!(tool_step.status, StepStatus::Failed);
    let output = tool_step.output.as_ref().unwrap().as_str().unwrap();
    assert!(
        output.contains("Chính sách bảo mật từ chối") && output.contains("SSRF"),
        "Unexpected tool step output: {output}"
    );
}

/// Runs one turn in which the LLM calls `tool` with `args`; returns whether the tool host ran it.
async fn tool_was_executed(
    agent: &hubbub_domain::entities::agent::Agent,
    tool: &str,
    args: &str,
) -> bool {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let fake_llm = Arc::new(FakeLlm::new());
    let tool_host = Arc::new(MockToolHost::new());
    tool_host
        .register_simple_tool(tool, "Tool", "Nội dung")
        .await;
    fake_llm.push_tool_call("call_1", tool, args).await;
    fake_llm.push_text_deltas(&["Xong."], None).await;

    let runtime = AgentRuntime::new(
        store,
        fake_llm,
        tool_host.clone(),
        Arc::new(InMemoryEventSink::new()),
        None,
    );
    runtime
        .execute_run(
            Uuid::now_v7(),
            agent,
            Some("Làm đi"),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    !tool_host.get_recorded_calls().await.is_empty()
}

fn agent_with(tools: &[&str], fs_read: &[&str]) -> hubbub_domain::entities::agent::Agent {
    let mut agent = create_test_agent("policy-agent", "Agent.");
    agent.tools.builtin = tools.iter().map(|t| t.to_string()).collect();
    agent.permissions = AgentPermissions {
        fs_read: fs_read.iter().map(|p| p.to_string()).collect(),
        fs_write: vec![],
        network: NetworkPolicy::None,
    };
    agent
}

#[tokio::test]
async fn test_tool_not_assigned_to_agent_is_denied() {
    // A prompt-injected call to a tool the agent was never given
    let agent = agent_with(&["fs_read"], &["**"]);
    assert!(!tool_was_executed(&agent, "report_list", "{}").await);
}

#[tokio::test]
async fn test_unknown_builtin_tool_is_denied() {
    let agent = agent_with(&["shell_exec"], &["**"]);
    assert!(!tool_was_executed(&agent, "shell_exec", r#"{"cmd":"dir"}"#).await);
}

#[tokio::test]
async fn test_report_list_requires_read_permission_on_reports() {
    assert!(!tool_was_executed(&agent_with(&["report_list"], &[]), "report_list", "{}").await);
    assert!(
        tool_was_executed(
            &agent_with(&["report_list"], &["reports/**"]),
            "report_list",
            "{}"
        )
        .await
    );
}

#[tokio::test]
async fn test_app_data_dir_is_never_readable() {
    let agent = agent_with(&["fs_read", "fs_list"], &["**"]);
    for path in [
        "data/hubbub.db",
        "DATA/agents.json",
        "./data/x",
        "data",
        "Data./hubbub.db",
    ] {
        let args = format!(r#"{{"path":"{path}"}}"#);
        assert!(
            !tool_was_executed(&agent, "fs_read", &args).await,
            "fs_read {path}"
        );
        assert!(
            !tool_was_executed(&agent, "fs_list", &args).await,
            "fs_list {path}"
        );
    }
    assert!(tool_was_executed(&agent, "fs_read", r#"{"path":"notes/data.md"}"#).await);
}

#[tokio::test]
async fn test_listing_workspace_root_requires_root_read_permission() {
    let narrow = agent_with(&["fs_list"], &["reports/**"]);
    assert!(!tool_was_executed(&narrow, "fs_list", r#"{"path":""}"#).await);
    assert!(!tool_was_executed(&narrow, "fs_list", r#"{"path":"."}"#).await);
    assert!(tool_was_executed(&narrow, "fs_list", r#"{"path":"reports"}"#).await);
    let wide = agent_with(&["fs_list"], &["**"]);
    assert!(tool_was_executed(&wide, "fs_list", r#"{"path":"."}"#).await);
}

#[tokio::test]
async fn test_mcp_tool_assigned_to_agent_runs() {
    let mut agent = agent_with(&[], &[]);
    agent.tools.mcp = vec!["github_search".to_string()];
    assert!(tool_was_executed(&agent, "github_search", "{}").await);
}
