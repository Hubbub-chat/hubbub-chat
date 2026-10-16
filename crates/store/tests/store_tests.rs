#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::Utc;
use hubbub_domain::entities::audit_log::AuditLog;
use hubbub_domain::entities::conversation::{Conversation, Message, MessagePart, MessageRole};
use hubbub_domain::entities::document::Document;
use hubbub_domain::entities::run::{Run, RunStatus, Step, StepKind, StepStatus, UsageInfo};
use hubbub_domain::ports::store::Store;
use hubbub_store::SqliteStore;
use uuid::Uuid;

#[tokio::test]
async fn test_create_and_get_conversation_with_vietnamese_diacritics() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let id = Uuid::now_v7();
    let title = "Nghiên cứu về Trí tuệ Nhân tạo & Xử lý Ngôn ngữ Tự nhiên tiếng Việt".to_string();
    let now = Utc::now();

    let conv = Conversation {
        id,
        title: title.clone(),
        agent_id: "researcher".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };

    store
        .create_conversation(&conv)
        .await
        .expect("Failed to create conversation");

    let retrieved = store
        .get_conversation(id)
        .await
        .expect("Failed to query conversation")
        .expect("Conversation not found");

    assert_eq!(retrieved.id, id);
    assert_eq!(retrieved.title, title);
    assert_eq!(retrieved.agent_id, "researcher");
    assert!(!retrieved.archived);
}

#[tokio::test]
async fn test_non_existent_conversation_returns_none() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let non_existent_id = Uuid::now_v7();
    let result = store
        .get_conversation(non_existent_id)
        .await
        .expect("Query failed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_conversation_list_and_update() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let id1 = Uuid::now_v7();
    let id2 = Uuid::now_v7();
    let now = Utc::now();

    let conv1 = Conversation {
        id: id1,
        title: "Cuộc hội thoại 1".to_string(),
        agent_id: "tutor".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };

    let conv2 = Conversation {
        id: id2,
        title: "Cuộc hội thoại 2".to_string(),
        agent_id: "researcher".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };

    store
        .create_conversation(&conv1)
        .await
        .expect("create conv1");
    store
        .create_conversation(&conv2)
        .await
        .expect("create conv2");

    let list = store
        .list_conversations()
        .await
        .expect("list conversations");
    assert_eq!(list.len(), 2);

    // Test update (e.g. archive conversation 1)
    let updated_conv1 = Conversation {
        id: id1,
        title: "Cuộc hội thoại 1 (Đã đổi tên)".to_string(),
        agent_id: "tutor".to_string(),
        created_at: now,
        updated_at: Utc::now(),
        archived: true,
    };

    store
        .update_conversation(&updated_conv1)
        .await
        .expect("update conv1");

    let retrieved = store
        .get_conversation(id1)
        .await
        .expect("get conv1")
        .expect("found conv1");

    assert_eq!(retrieved.title, "Cuộc hội thoại 1 (Đã đổi tên)");
    assert!(retrieved.archived);
}

#[tokio::test]
async fn test_append_and_list_messages_with_emojis_and_code() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let conv_id = Uuid::now_v7();
    let now = Utc::now();

    let conv = Conversation {
        id: conv_id,
        title: "Kiểm tra Tin Nhắn Emoji & Code".to_string(),
        agent_id: "researcher".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };
    store.create_conversation(&conv).await.expect("create conv");

    let msg_id1 = Uuid::now_v7();
    let msg1 = Message {
        id: msg_id1,
        conversation_id: conv_id,
        run_id: None,
        role: MessageRole::User,
        parts: vec![MessagePart::Text(
            "Chào bạn! Hãy tìm hiểu về Rust và WebAssembly nhé 🚀 🎉".to_string(),
        )],
        created_at: now,
    };

    let msg_id2 = Uuid::now_v7();
    let msg2 = Message {
        id: msg_id2,
        conversation_id: conv_id,
        run_id: None,
        role: MessageRole::Assistant,
        parts: vec![
            MessagePart::Text("Chào bạn! Dưới đây là code Rust mẫu:".to_string()),
            MessagePart::Text("```rust\nfn add(a: i32, b: i32) -> i32 { a + b }\n```".to_string()),
        ],
        created_at: now,
    };

    store.append_message(&msg1).await.expect("append msg1");
    store.append_message(&msg2).await.expect("append msg2");

    let messages = store
        .list_messages(conv_id, 10, 0)
        .await
        .expect("list messages");

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].id, msg_id1);
    assert_eq!(messages[1].id, msg_id2);

    match &messages[0].parts[0] {
        MessagePart::Text(text) => assert!(text.contains("🚀")),
        _ => panic!("Expected Text part"),
    }
}

#[tokio::test]
async fn test_message_pagination_edge_cases() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let conv_id = Uuid::now_v7();
    let now = Utc::now();

    let conv = Conversation {
        id: conv_id,
        title: "Test Pagination".to_string(),
        agent_id: "tutor".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };
    store.create_conversation(&conv).await.expect("create conv");

    // Insert 15 messages
    for i in 1..=15 {
        let msg = Message {
            id: Uuid::now_v7(),
            conversation_id: conv_id,
            run_id: None,
            role: if i % 2 == 1 {
                MessageRole::User
            } else {
                MessageRole::Assistant
            },
            parts: vec![MessagePart::Text(format!("Message number {i}"))],
            created_at: now,
        };
        store.append_message(&msg).await.expect("append msg");
    }

    // Page 1: limit 5, offset 0 -> 5 messages
    let p1 = store.list_messages(conv_id, 5, 0).await.expect("p1");
    assert_eq!(p1.len(), 5);

    // Page 2: limit 5, offset 5 -> 5 messages
    let p2 = store.list_messages(conv_id, 5, 5).await.expect("p2");
    assert_eq!(p2.len(), 5);

    // Page 3: limit 5, offset 10 -> 5 messages
    let p3 = store.list_messages(conv_id, 5, 10).await.expect("p3");
    assert_eq!(p3.len(), 5);

    // Edge case 1: offset beyond total count -> returns empty vector (no panic)
    let p_empty = store
        .list_messages(conv_id, 5, 20)
        .await
        .expect("empty page");
    assert!(p_empty.is_empty());

    // Edge case 2: limit 0 -> returns empty vector (no panic)
    let p_zero_limit = store
        .list_messages(conv_id, 0, 0)
        .await
        .expect("zero limit");
    assert!(p_zero_limit.is_empty());
}

#[tokio::test]
async fn test_very_long_message_report() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let conv_id = Uuid::now_v7();
    let now = Utc::now();

    let conv = Conversation {
        id: conv_id,
        title: "Test Long Report".to_string(),
        agent_id: "researcher".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };
    store.create_conversation(&conv).await.expect("create conv");

    // Generate ~60KB long text
    let paragraph = "Dự án nghiên cứu này phân tích toàn diện về các kiến trúc AI cục bộ, bao gồm việc quản lý bộ nhớ, bảo mật đường dẫn và tối ưu hóa thời gian thực thi. ".repeat(400);

    let msg = Message {
        id: Uuid::now_v7(),
        conversation_id: conv_id,
        run_id: None,
        role: MessageRole::Assistant,
        parts: vec![MessagePart::Text(paragraph.clone())],
        created_at: now,
    };

    store
        .append_message(&msg)
        .await
        .expect("append long message");

    let messages = store
        .list_messages(conv_id, 1, 0)
        .await
        .expect("list messages");
    assert_eq!(messages.len(), 1);

    match &messages[0].parts[0] {
        MessagePart::Text(text) => assert_eq!(text.len(), paragraph.len()),
        _ => panic!("Expected text part"),
    }
}

#[tokio::test]
async fn test_fts5_search_messages_vietnamese() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let conv_id = Uuid::now_v7();
    let now = Utc::now();

    let conv = Conversation {
        id: conv_id,
        title: "Hội thoại tìm kiếm".to_string(),
        agent_id: "researcher".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };
    store.create_conversation(&conv).await.expect("create conv");

    let msg1 = Message {
        id: Uuid::now_v7(),
        conversation_id: conv_id,
        run_id: None,
        role: MessageRole::Assistant,
        parts: vec![MessagePart::Text(
            "Quần đảo Hoàng Sa và Trường Sa là một phần máu thịt thiêng liêng của Việt Nam."
                .to_string(),
        )],
        created_at: now,
    };

    let msg2 = Message {
        id: Uuid::now_v7(),
        conversation_id: conv_id,
        run_id: None,
        role: MessageRole::Assistant,
        parts: vec![MessagePart::Text(
            "Thuật toán quy hoạch động và tìm kiếm nhị phân trong khoa học máy tính.".to_string(),
        )],
        created_at: now,
    };

    store.append_message(&msg1).await.expect("msg1");
    store.append_message(&msg2).await.expect("msg2");

    // Search query for "Hoàng Sa"
    let results1 = store
        .search_messages("Hoàng Sa")
        .await
        .expect("search Hoàng Sa");
    assert_eq!(results1.len(), 1);
    assert_eq!(results1[0].id, msg1.id);

    // Search query for "khoa học"
    let results2 = store
        .search_messages("khoa học")
        .await
        .expect("search khoa học");
    assert_eq!(results2.len(), 1);
    assert_eq!(results2[0].id, msg2.id);

    // Search query with no match
    let results3 = store
        .search_messages("KhôngTồnTại123")
        .await
        .expect("search none");
    assert!(results3.is_empty());
}

#[tokio::test]
async fn test_runs_and_steps_lifecycle() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let conv_id = Uuid::now_v7();
    let run_id = Uuid::now_v7();
    let now = Utc::now();

    let conv = Conversation {
        id: conv_id,
        title: "Test Run Lifecycle".to_string(),
        agent_id: "researcher".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };
    store.create_conversation(&conv).await.expect("create conv");

    let run = Run {
        id: run_id,
        conversation_id: conv_id,
        agent_id: "researcher".to_string(),
        parent_run_id: None,
        status: RunStatus::Running,
        usage: None,
        cost_usd: None,
        started_at: now,
        finished_at: None,
    };

    store.create_run(&run).await.expect("create run");

    // Create steps
    let step1 = Step {
        id: Uuid::now_v7(),
        run_id,
        idx: 0,
        kind: StepKind::Llm,
        input: serde_json::json!({"prompt": "Tìm thông tin"}),
        output: Some(serde_json::json!({"call": "web_search"})),
        status: StepStatus::Completed,
        duration_ms: Some(150),
        created_at: now,
    };

    let step2 = Step {
        id: Uuid::now_v7(),
        run_id,
        idx: 1,
        kind: StepKind::Tool,
        input: serde_json::json!({"tool": "web_search", "query": "Rust Tokio"}),
        output: Some(serde_json::json!({"results": ["doc1", "doc2"]})),
        status: StepStatus::Completed,
        duration_ms: Some(300),
        created_at: now,
    };

    store.create_step(&step1).await.expect("create step1");
    store.create_step(&step2).await.expect("create step2");

    let steps = store.list_steps(run_id).await.expect("list steps");
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].idx, 0);
    assert_eq!(steps[1].idx, 1);

    // Update run to completed
    let finished_now = Utc::now();
    let updated_run = Run {
        id: run_id,
        conversation_id: conv_id,
        agent_id: "researcher".to_string(),
        parent_run_id: None,
        status: RunStatus::Completed,
        usage: Some(UsageInfo {
            prompt_tokens: 1200,
            completion_tokens: 450,
            total_tokens: 1650,
        }),
        cost_usd: Some(0.0085),
        started_at: now,
        finished_at: Some(finished_now),
    };

    store.update_run(&updated_run).await.expect("update run");

    let retrieved_run = store
        .get_run(run_id)
        .await
        .expect("get run")
        .expect("found run");
    assert_eq!(retrieved_run.status, RunStatus::Completed);
    assert_eq!(retrieved_run.cost_usd, Some(0.0085));
    assert_eq!(retrieved_run.usage.expect("usage").total_tokens, 1650);
}

#[tokio::test]
async fn test_documents_upsert_and_fts_search() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let doc_id = Uuid::now_v7();
    let now = Utc::now();

    let doc = Document {
        id: doc_id,
        path: "reports/ai_research_2026.md".to_string(),
        title: "Báo Cáo Nghiên Cứu AI và Deep Learning 2026".to_string(),
        tags: vec![
            "ai".to_string(),
            "research".to_string(),
            "deep-learning".to_string(),
        ],
        agent_id: Some("researcher".to_string()),
        run_id: None,
        content_hash: "hash123456789".to_string(),
        size_bytes: 4096,
        created_at: now,
        updated_at: now,
    };

    store.upsert_document(&doc).await.expect("upsert doc");

    let docs = store.list_documents().await.expect("list docs");
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].path, "reports/ai_research_2026.md");

    // Search document
    let found = store
        .search_documents("Nghiên Cứu AI")
        .await
        .expect("search doc");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, doc_id);

    // Upsert update on the same path
    let doc_updated = Document {
        id: doc_id,
        path: "reports/ai_research_2026.md".to_string(),
        title: "Báo Cáo Nghiên Cứu AI và Deep Learning 2026 (Bản sửa đổi)".to_string(),
        tags: vec!["ai".to_string(), "v2".to_string()],
        agent_id: Some("researcher".to_string()),
        run_id: None,
        content_hash: "hash_updated".to_string(),
        size_bytes: 5120,
        created_at: now,
        updated_at: Utc::now(),
    };

    store
        .upsert_document(&doc_updated)
        .await
        .expect("upsert doc update");

    let docs_after = store.list_documents().await.expect("list docs after");
    assert_eq!(docs_after.len(), 1); // Still 1 document
    assert_eq!(
        docs_after[0].title,
        "Báo Cáo Nghiên Cứu AI và Deep Learning 2026 (Bản sửa đổi)"
    );
    assert_eq!(docs_after[0].size_bytes, 5120);
}

#[tokio::test]
async fn test_concurrent_reads_and_writes_wal() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let conv_id = Uuid::now_v7();
    let now = Utc::now();

    let conv = Conversation {
        id: conv_id,
        title: "Concurrent Access Test".to_string(),
        agent_id: "researcher".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };
    store.create_conversation(&conv).await.expect("create conv");

    let mut handles = vec![];

    // Spawn 10 concurrent writers
    for i in 0..10 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            let msg = Message {
                id: Uuid::now_v7(),
                conversation_id: conv_id,
                run_id: None,
                role: MessageRole::User,
                parts: vec![MessagePart::Text(format!("Concurrent message {i}"))],
                created_at: Utc::now(),
            };
            store_clone.append_message(&msg).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("task join").expect("append message");
    }

    let messages = store
        .list_messages(conv_id, 50, 0)
        .await
        .expect("list messages");
    assert_eq!(messages.len(), 10);
}

#[tokio::test]
async fn test_audit_log_record_and_list() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let run_id = Uuid::now_v7();
    let log1 = AuditLog {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        run_id: Some(run_id),
        tool_name: "workspace_read".to_string(),
        args_digest: r#"{"path":"reports/summary.md"}"#.to_string(),
        decision: "allow".to_string(),
        result_digest: Some("250 bytes read".to_string()),
    };
    let log2 = AuditLog {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        run_id: Some(run_id),
        tool_name: "web_fetch".to_string(),
        args_digest: r#"{"url":"http://169.254.169.254"}"#.to_string(),
        decision: "deny".to_string(),
        result_digest: Some("SSRF target blocked".to_string()),
    };

    store.record_audit_log(&log1).await.expect("record log1");
    store.record_audit_log(&log2).await.expect("record log2");

    let logs = store.list_audit_logs(10).await.expect("list logs");
    assert_eq!(logs.len(), 2);
    // Ordered by timestamp DESC
    assert_eq!(logs[0].tool_name, "web_fetch");
    assert_eq!(logs[0].decision, "deny");
    assert_eq!(logs[1].tool_name, "workspace_read");
    assert_eq!(logs[1].decision, "allow");
}

#[tokio::test]
async fn test_model_usage_stats_aggregation() {
    let store = SqliteStore::open_in_memory()
        .await
        .expect("Failed to open in-memory store");

    let conv_id = Uuid::now_v7();
    let run_id = Uuid::now_v7();
    let now = Utc::now();

    let conv = Conversation {
        id: conv_id,
        title: "Test Quota".to_string(),
        agent_id: "analyst".to_string(),
        created_at: now,
        updated_at: now,
        archived: false,
    };
    store.create_conversation(&conv).await.expect("create conv");

    let run = Run {
        id: run_id,
        conversation_id: conv_id,
        agent_id: "analyst".to_string(),
        parent_run_id: None,
        status: RunStatus::Completed,
        usage: Some(UsageInfo {
            prompt_tokens: 150,
            completion_tokens: 50,
            total_tokens: 200,
        }),
        cost_usd: Some(0.002),
        started_at: now,
        finished_at: Some(now),
    };
    store.create_run(&run).await.expect("create run");

    let step = Step {
        id: Uuid::now_v7(),
        run_id,
        idx: 1,
        kind: StepKind::Llm,
        input: serde_json::json!({
            "model": "gemini-3.8-flash",
            "messages_count": 2,
        }),
        output: None,
        status: StepStatus::Completed,
        duration_ms: Some(120),
        created_at: now,
    };
    store.create_step(&step).await.expect("create step");

    let stats = store.get_model_usage_stats().await.expect("get stats");
    assert!(!stats.is_empty());

    let gemini_stat = stats.iter().find(|s| s.model == "gemini-3.8-flash");
    assert!(gemini_stat.is_some());
    let stat = gemini_stat.unwrap();
    assert_eq!(stat.total_tokens, 200);
    assert_eq!(stat.total_runs, 1);
    assert_eq!(stat.requests_today, 1);
    assert_eq!(stat.tokens_today, 200);
    assert_eq!(stat.rpm_limit, 15);
    assert_eq!(stat.rpd_limit, 1500);
    assert!(stat.is_free_tier);
}

#[test]
fn test_migration_files_use_lf_line_endings() {
    // sqlx stores a checksum of each migration's exact bytes. A Windows checkout with
    // core.autocrlf rewrites them as CRLF, and an existing database then refuses to start
    // ("migration 1 was previously applied but has been modified"). .gitattributes pins LF.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            !bytes.contains(&b'\r'),
            "{} has CRLF line endings",
            path.display()
        );
    }
}
