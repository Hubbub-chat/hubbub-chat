#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use hubbub_app::{AppConfig, AppError, AppService};
use hubbub_store::SqliteStore;
use hubbub_testkit::MockToolHost;
use hubbub_vault::InMemoryVault;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_app_config_load_and_save() {
    let tmp_file = NamedTempFile::new().unwrap();
    let path = tmp_file.path().to_path_buf();

    let config = AppConfig {
        default_provider: "anthropic".to_string(),
        default_agent: "librarian".to_string(),
        ..Default::default()
    };

    config.save_to_path(&path).unwrap();

    let loaded = AppConfig::load_from_path(&path).unwrap();
    assert_eq!(loaded.default_provider, "anthropic");
    assert_eq!(loaded.default_agent, "librarian");
    assert!(loaded.providers.contains_key("openai"));
    assert!(loaded.providers.contains_key("anthropic"));
    assert!(loaded.providers.contains_key("ollama"));
    assert!(loaded.providers.contains_key("gemini"));
}

#[tokio::test]
async fn test_app_service_conversation_lifecycle() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let vault = Arc::new(InMemoryVault::new());
    let tool_host = Arc::new(MockToolHost::new());
    let config = AppConfig::default();

    let service = AppService::new(config, store.clone(), vault, tool_host);

    // 1. Create conversation with Vietnamese title
    let conv = service
        .create_conversation(
            Some("Nghiên cứu thị trường AI Việt Nam 🇻🇳".to_string()),
            Some("researcher".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(conv.title, "Nghiên cứu thị trường AI Việt Nam 🇻🇳");
    assert_eq!(conv.agent_id, "researcher");

    // 2. List conversations
    let list = service.list_conversations().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, conv.id);

    // 3. Get conversation
    let retrieved = service.get_conversation(conv.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, conv.id);
}

#[tokio::test]
async fn test_app_service_secret_management() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let vault = Arc::new(InMemoryVault::new());
    let tool_host = Arc::new(MockToolHost::new());
    let config = AppConfig::default();

    let service = AppService::new(config, store, vault, tool_host);

    // Initially no key
    assert!(!service.has_provider_key("openai").await.unwrap());

    // Set key
    service
        .set_provider_key("openai", "sk-proj-secret-12345")
        .await
        .unwrap();
    assert!(service.has_provider_key("openai").await.unwrap());

    // Delete key
    service.delete_provider_key("openai").await.unwrap();
    assert!(!service.has_provider_key("openai").await.unwrap());
}

#[tokio::test]
async fn test_app_service_agents() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let vault = Arc::new(InMemoryVault::new());
    let tool_host = Arc::new(MockToolHost::new());
    let config = AppConfig::default();

    let service = AppService::new(config, store, vault, tool_host);

    let agents = service.list_agents().await;
    assert_eq!(agents.len(), 6);

    let researcher = service.get_agent("researcher").await;
    assert!(researcher.is_some());
    let r = researcher.unwrap();
    assert_eq!(r.id, "researcher");
    assert!(r.tools.builtin.contains(&"web_search".to_string()));

    let analyst = service.get_agent("analyst").await;
    assert!(analyst.is_some());
    let a = analyst.unwrap();
    assert_eq!(a.id, "analyst");
    assert_eq!(a.model, "gemini-3.8-flash");

    // Test set_agent_model
    service
        .set_agent_model("analyst", "gpt-4o-mini")
        .await
        .unwrap();
    let updated = service.get_agent("analyst").await.unwrap();
    assert_eq!(updated.model, "gpt-4o-mini");

    // Test upsert_agent (create custom functional agent)
    let mut custom = a.clone();
    custom.id = "custom_devops".to_string();
    custom.name = "Chuyên gia DevOps".to_string();
    custom.model = "gemini-3.8-flash".to_string();
    service.upsert_agent(custom).await.unwrap();
    assert!(service.get_agent("custom_devops").await.is_some());

    // Test delete_agent
    service.delete_agent("custom_devops").await.unwrap();
    assert!(service.get_agent("custom_devops").await.is_none());
}

#[tokio::test]
async fn test_app_service_build_llm_provider() {
    let store = Arc::new(SqliteStore::open_in_memory().await.unwrap());
    let vault = Arc::new(InMemoryVault::new());
    let tool_host = Arc::new(MockToolHost::new());
    let config = AppConfig::default();

    let service = AppService::new(config, store, vault, tool_host);

    let openai_provider = service.build_llm_provider("openai", Some("sk-test".to_string()));
    assert!(openai_provider.is_ok());

    let anthropic_provider =
        service.build_llm_provider("anthropic", Some("sk-ant-test".to_string()));
    assert!(anthropic_provider.is_ok());

    let gemini_provider = service.build_llm_provider("gemini", Some("AIzaSy-test".to_string()));
    assert!(gemini_provider.is_ok());

    let unknown_provider = service.build_llm_provider("unknown_provider", None);
    match unknown_provider {
        Err(AppError::Provider { provider, .. }) => assert_eq!(provider, "unknown_provider"),
        Ok(_) => panic!("Expected Err(AppError::Provider), got Ok"),
        Err(other) => panic!("Expected AppError::Provider, got {other:?}"),
    }
}

#[tokio::test]
async fn test_app_service_workspace_and_reports() {
    let tmp = tempfile::tempdir().unwrap();
    let config = AppConfig {
        workspace_dir: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let service = AppService::init(config).await.unwrap();

    // 1. Initially no reports
    let reports = service.list_reports().await.unwrap();
    assert!(reports.is_empty());

    // 2. Write a report via workspace
    service
        .workspace()
        .write_file("reports/test-report.md", "# Báo cáo Test\n\nNội dung.")
        .await
        .unwrap();

    // 3. List reports
    let reports = service.list_reports().await.unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0], "reports/test-report.md");

    // 4. Read report
    let content = service.read_report("test-report.md").await.unwrap();
    assert!(content.contains("# Báo cáo Test"));
}

#[tokio::test]
async fn test_init_fails_runs_interrupted_by_previous_process() {
    use hubbub_domain::entities::conversation::Conversation;
    use hubbub_domain::entities::run::{Run, RunStatus};
    use hubbub_domain::ports::store::Store;

    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let run_id = uuid::Uuid::now_v7();
    {
        let db = data_dir.join("hubbub.db");
        let store = SqliteStore::open(&db.to_string_lossy()).await.unwrap();
        let now = chrono::Utc::now();
        let conv_id = uuid::Uuid::now_v7();
        store
            .create_conversation(&Conversation {
                id: conv_id,
                title: "Bị gián đoạn".to_string(),
                agent_id: "researcher".to_string(),
                created_at: now,
                updated_at: now,
                archived: false,
            })
            .await
            .unwrap();
        store
            .create_run(&Run {
                id: run_id,
                conversation_id: conv_id,
                agent_id: "researcher".to_string(),
                parent_run_id: None,
                status: RunStatus::Running,
                usage: None,
                cost_usd: None,
                started_at: now,
                finished_at: None,
            })
            .await
            .unwrap();
    }

    let config = AppConfig {
        workspace_dir: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let _service = AppService::init(config).await.unwrap();

    let db = data_dir.join("hubbub.db");
    let store = SqliteStore::open(&db.to_string_lossy()).await.unwrap();
    let run = store.get_run(run_id).await.unwrap().unwrap();
    assert_eq!(run.status, RunStatus::Failed);
}

#[tokio::test]
async fn test_write_report_long_vietnamese_title_does_not_panic() {
    let tmp = tempfile::tempdir().unwrap();
    let config = AppConfig {
        workspace_dir: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let service = AppService::init(config).await.unwrap();

    // Slug keeps non-ASCII letters; byte 50 of "a" + "ệ"* falls mid-character
    let title = format!("A{}", "ệ".repeat(60));
    service
        .write_report(&title, "Nội dung.", None)
        .await
        .unwrap();

    let reports = service.list_reports().await.unwrap();
    assert_eq!(reports.len(), 1);
    assert!(
        reports[0].ends_with(&format!("a{}.md", "ệ".repeat(49))),
        "{}",
        reports[0]
    );
}

#[tokio::test]
async fn test_cancel_run_while_streaming_returns_cancelled_run() {
    use hubbub_domain::entities::run::RunStatus;
    use hubbub_domain::events::RunEvent;
    use hubbub_testkit::InMemoryEventSink;
    use std::time::Duration;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // An LLM that takes far longer to answer than the test waits
    let llm_server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(30)))
        .mount(&llm_server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let mut config = AppConfig {
        workspace_dir: tmp.path().to_path_buf(),
        ..Default::default()
    };
    config.providers.get_mut("ollama").unwrap().base_url = Some(llm_server.uri());
    let service = Arc::new(AppService::new(
        config,
        Arc::new(SqliteStore::open_in_memory().await.unwrap()),
        Arc::new(InMemoryVault::new()),
        Arc::new(MockToolHost::new()),
    ));
    service
        .set_agent_model("analyst", "llama3.2")
        .await
        .unwrap(); // Ollama: no key needed

    let sink = Arc::new(InMemoryEventSink::new());
    let conv_id = uuid::Uuid::now_v7();
    let send = tokio::spawn({
        let (service, sink) = (service.clone(), sink.clone());
        async move {
            service
                .send_message(conv_id, Some("analyst".to_string()), "Xin chào", sink)
                .await
        }
    });

    // The UI learns the run id from RunStarted, while send_message is still pending
    let mut run_id = None;
    for _ in 0..200 {
        run_id = sink.get_events().await.iter().find_map(|e| match e {
            RunEvent::RunStarted { run_id, .. } => Some(run_id.clone()),
            _ => None,
        });
        if run_id.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let run_id: uuid::Uuid = run_id.expect("RunStarted emitted").parse().unwrap();
    assert!(service.cancel_run(run_id).await.unwrap());

    // A cancel the user asked for is not an error
    let run = tokio::time::timeout(Duration::from_secs(5), send)
        .await
        .expect("run stops promptly")
        .unwrap()
        .expect("cancelled run is returned, not an error");
    assert_eq!(run.id, run_id);
    assert_eq!(run.status, RunStatus::Cancelled);
}

#[test]
fn test_default_agents_have_permissions_for_their_tools() {
    use hubbub_domain::entities::agent::NetworkPolicy;
    use hubbub_policy::PermissionChecker;

    for agent in hubbub_app::default_agents() {
        let p = &agent.permissions;
        for tool in &agent.tools.builtin {
            let ok = match tool.as_str() {
                "web_search" => {
                    PermissionChecker::check_network(&agent.id, &p.network, true).is_ok()
                }
                "web_fetch" => p.network == NetworkPolicy::Open,
                "fs_read" | "fs_list" => !p.fs_read.is_empty(),
                "report_read" => {
                    PermissionChecker::check_fs_read(&agent.id, p, "reports/a.md").is_ok()
                }
                "report_list" => PermissionChecker::check_fs_read(&agent.id, p, "reports").is_ok(),
                "report_write" => {
                    PermissionChecker::check_fs_write(&agent.id, p, "reports/a.md").is_ok()
                }
                other => panic!("{}: unknown tool {other}", agent.id),
            };
            assert!(
                ok,
                "agent '{}' has tool '{tool}' but no permission to use it",
                agent.id
            );
        }
    }
}
