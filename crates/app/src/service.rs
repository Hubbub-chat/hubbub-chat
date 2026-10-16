use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use hubbub_agent::{AgentError, AgentRuntime, BUILTIN_TOOLS};
use hubbub_domain::entities::agent::Agent;
use hubbub_domain::entities::audit_log::AuditLog;
use hubbub_domain::entities::conversation::{Conversation, Message};
use hubbub_domain::entities::document::Document;
use hubbub_domain::entities::run::Run;
use hubbub_domain::ports::event_sink::EventSink;
use hubbub_domain::ports::llm::LlmProvider;
use hubbub_domain::ports::secret_store::SecretStore;
use hubbub_domain::ports::store::Store;
use hubbub_domain::ports::tool_host::ToolHost;
use hubbub_domain::ports::workspace_service::WorkspaceService;
use hubbub_policy::AgentValidator;
use hubbub_store::SqliteStore;
use hubbub_tools::BuiltinToolHost;
use hubbub_vault::KeyringVault;
use hubbub_workspace::LocalWorkspaceService;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::report_service::ReportService;
use crate::run_guard::ActiveConversations;
use crate::seed::default_agents;

pub struct AppService {
    config: AppConfig,
    store: Arc<dyn Store>,
    vault: Arc<dyn SecretStore>,
    tool_host: Arc<dyn ToolHost>,
    workspace: Arc<dyn WorkspaceService>,
    reports: Arc<ReportService>,
    agents: Arc<RwLock<HashMap<String, Agent>>>,
    active_cancellations: Arc<Mutex<HashMap<Uuid, CancellationToken>>>,
    active_conversations: ActiveConversations,
}

impl AppService {
    pub fn new(
        config: AppConfig,
        store: Arc<dyn Store>,
        vault: Arc<dyn SecretStore>,
        tool_host: Arc<dyn ToolHost>,
    ) -> Self {
        let workspace = Arc::new(LocalWorkspaceService::new(&config.workspace_dir));
        Self::with_workspace(config, store, vault, tool_host, workspace)
    }

    pub fn with_workspace(
        config: AppConfig,
        store: Arc<dyn Store>,
        vault: Arc<dyn SecretStore>,
        tool_host: Arc<dyn ToolHost>,
        workspace: Arc<dyn WorkspaceService>,
    ) -> Self {
        let mut map = HashMap::new();
        for agent in default_agents() {
            map.insert(agent.id.clone(), agent);
        }

        let reports = Arc::new(ReportService::new(workspace.clone(), store.clone()));

        Self {
            config,
            store,
            vault,
            tool_host,
            workspace,
            reports,
            agents: Arc::new(RwLock::new(map)),
            active_cancellations: Arc::new(Mutex::new(HashMap::new())),
            active_conversations: ActiveConversations::default(),
        }
    }

    /// Initialize default AppService with SQLite store, Keyring vault, and Workspace service.
    pub async fn init(config: AppConfig) -> Result<Self, AppError> {
        for sub in &["data", "reports", "notes", "sources", ".versions"] {
            std::fs::create_dir_all(config.workspace_dir.join(sub))?;
        }

        let data_dir = config.workspace_dir.join("data");
        let db_path = data_dir.join("hubbub.db");
        let db_str = db_path.to_string_lossy().to_string();
        let store = Arc::new(SqliteStore::open(&db_str).await?);
        // Runs still `running` at startup belong to a process that died; nothing will finish them
        store.fail_interrupted_runs().await?;
        let vault = Arc::new(KeyringVault::new("hubbub"));

        let workspace = Arc::new(LocalWorkspaceService::new(&config.workspace_dir));
        let tool_host = Arc::new(BuiltinToolHost::with_workspace(
            workspace.clone(),
            Some(store.clone()),
        ));
        let reports = Arc::new(ReportService::new(workspace.clone(), store.clone()));

        let agents_path = data_dir.join("agents.json");
        let mut map = HashMap::new();
        for def in default_agents() {
            map.insert(def.id.clone(), def);
        }

        let saved_agents_opt: Option<Vec<Agent>> = std::fs::read_to_string(&agents_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok());
        if let Some(saved_agents) = saved_agents_opt {
            for saved in saved_agents {
                map.insert(saved.id.clone(), saved);
            }
        }

        // Migrate legacy analyst name if present
        if let Some(analyst) = map
            .get_mut("analyst")
            .filter(|a| a.name == "Gemini Analyst")
        {
            analyst.name = "Chuyên viên Phân tích".to_string();
        }

        Ok(Self {
            config,
            store,
            vault,
            tool_host,
            workspace,
            reports,
            agents: Arc::new(RwLock::new(map)),
            active_cancellations: Arc::new(Mutex::new(HashMap::new())),
            active_conversations: ActiveConversations::default(),
        })
    }

    // --- Conversation Management ---

    pub async fn create_conversation(
        &self,
        title: Option<String>,
        agent_id: Option<String>,
    ) -> Result<Conversation, AppError> {
        let agent = agent_id.unwrap_or_else(|| self.config.default_agent.clone());
        let conv = Conversation {
            id: Uuid::now_v7(),
            title: title.unwrap_or_else(|| "Cuộc trò chuyện mới".to_string()),
            agent_id: agent,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            archived: false,
        };

        self.store.create_conversation(&conv).await?;
        Ok(conv)
    }

    pub async fn list_conversations(&self) -> Result<Vec<Conversation>, AppError> {
        let list = self.store.list_conversations().await?;
        Ok(list)
    }

    pub async fn get_conversation(&self, id: Uuid) -> Result<Option<Conversation>, AppError> {
        let conv = self.store.get_conversation(id).await?;
        Ok(conv)
    }

    pub async fn list_messages(
        &self,
        conversation_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Message>, AppError> {
        let msgs = self
            .store
            .list_messages(conversation_id, limit, offset)
            .await?;
        Ok(msgs)
    }

    /// The newest `limit` messages, oldest-first (what a chat view shows).
    pub async fn list_recent_messages(
        &self,
        conversation_id: Uuid,
        limit: u32,
    ) -> Result<Vec<Message>, AppError> {
        Ok(self
            .store
            .list_recent_messages(conversation_id, limit)
            .await?)
    }

    // --- Secret & Provider Key Management ---

    pub async fn set_provider_key(&self, provider: &str, api_key: &str) -> Result<(), AppError> {
        let key_name = format!("{provider}_api_key");
        self.vault.set(&key_name, api_key).await?;
        Ok(())
    }

    pub async fn has_provider_key(&self, provider: &str) -> Result<bool, AppError> {
        let key_name = format!("{provider}_api_key");
        let val = self.vault.get(&key_name).await?;
        Ok(val.is_some())
    }

    pub async fn delete_provider_key(&self, provider: &str) -> Result<(), AppError> {
        let key_name = format!("{provider}_api_key");
        self.vault.delete(&key_name).await?;
        Ok(())
    }

    // --- Model Quota & Usage ---

    pub async fn get_model_usage_stats(
        &self,
    ) -> Result<Vec<hubbub_domain::entities::model_quota::ModelUsageStat>, AppError> {
        let stats = self.store.get_model_usage_stats().await?;
        Ok(stats)
    }

    // --- Agents ---

    pub async fn list_agents(&self) -> Vec<Agent> {
        let map = self.agents.read().await;
        let mut list: Vec<Agent> = map.values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    pub async fn get_agent(&self, id: &str) -> Option<Agent> {
        let map = self.agents.read().await;
        map.get(id).cloned()
    }

    pub async fn upsert_agent(&self, agent: Agent) -> Result<(), AppError> {
        // Definitions come from the webview; reject ones that could never be safe
        AgentValidator::validate(&agent, BUILTIN_TOOLS)
            .map_err(|e| AppError::Config(e.to_string()))?;
        {
            let mut map = self.agents.write().await;
            map.insert(agent.id.clone(), agent);
        }
        let _ = self.save_agents_to_disk().await;
        Ok(())
    }

    pub async fn set_agent_model(&self, agent_id: &str, model: &str) -> Result<(), AppError> {
        {
            let mut map = self.agents.write().await;
            if let Some(agent) = map.get_mut(agent_id) {
                agent.model = model.to_string();
            } else {
                return Err(AppError::NotFound(format!(
                    "Không tìm thấy agent: {agent_id}"
                )));
            }
        }
        let _ = self.save_agents_to_disk().await;
        Ok(())
    }

    pub async fn delete_agent(&self, agent_id: &str) -> Result<(), AppError> {
        {
            let mut map = self.agents.write().await;
            map.remove(agent_id);
        }
        let _ = self.save_agents_to_disk().await;
        Ok(())
    }

    async fn save_agents_to_disk(&self) -> Result<(), AppError> {
        let data_dir = self.config.workspace_dir.join("data");
        if !data_dir.exists() {
            return Ok(());
        }
        let agents_path = data_dir.join("agents.json");
        let map = self.agents.read().await;
        let agents: Vec<Agent> = map.values().cloned().collect();
        let json = serde_json::to_string_pretty(&agents)
            .map_err(|e| AppError::Agent(format!("Lỗi serialize agents: {e}")))?;
        tokio::fs::write(&agents_path, json)
            .await
            .map_err(AppError::Io)?;
        Ok(())
    }

    // --- Provider Factory ---

    pub fn build_llm_provider(
        &self,
        provider_name: &str,
        api_key: Option<String>,
    ) -> Result<Arc<dyn LlmProvider>, AppError> {
        crate::provider_factory::ProviderFactory::build(&self.config, provider_name, api_key)
    }

    // --- Agent Run Execution ---

    pub async fn send_message(
        &self,
        conversation_id: Uuid,
        agent_id: Option<String>,
        prompt: &str,
        event_sink: Arc<dyn EventSink>,
    ) -> Result<Run, AppError> {
        let _conversation_guard = self
            .active_conversations
            .try_acquire(conversation_id)
            .ok_or_else(|| {
                AppError::Agent(
                    "Cuộc trò chuyện này đang có một lượt trả lời chưa hoàn tất. Vui lòng chờ hoặc dừng lượt đó trước.".to_string(),
                )
            })?;

        // 1. Resolve agent
        let agent_slug = agent_id.unwrap_or_else(|| self.config.default_agent.clone());
        let agent = self.get_agent(&agent_slug).await.ok_or_else(|| {
            AppError::NotFound(format!("Không tìm thấy agent với slug: {agent_slug}"))
        })?;

        // 2. Resolve provider & API key
        let provider_name = if agent.model.starts_with("claude") {
            "anthropic"
        } else if agent.model.starts_with("gemini") {
            "gemini"
        } else if agent.model.starts_with("llama") {
            if self.has_provider_key("groq").await.unwrap_or(false)
                && agent.model.contains("versatile")
            {
                "groq"
            } else {
                "ollama"
            }
        } else if agent.model.to_lowercase().contains("qwen")
            || agent.model.to_lowercase().contains("deepseek")
            || agent.model.to_lowercase().contains("mistral")
            || agent.model.to_lowercase().contains("phi")
        {
            "ollama"
        } else if agent.model.starts_with("gpt")
            || agent.model.starts_with("o1")
            || agent.model.starts_with("o3")
        {
            "openai"
        } else {
            &self.config.default_provider
        };

        let key_name = format!("{provider_name}_api_key");
        let api_key = self.vault.get(&key_name).await?;

        // Require API key for cloud providers unless it's Ollama or local
        if api_key.is_none() && provider_name != "ollama" {
            return Err(AppError::Provider {
                provider: provider_name.to_string(),
                message: format!(
                    "Chưa cấu hình API key cho provider '{provider_name}'. Vui lòng cấu hình API key trong Cài đặt hoặc đổi Model của Agent sang mô hình đã có key (như gemini-3.8-flash)."
                ),
            });
        }

        let llm = self.build_llm_provider(provider_name, api_key)?;

        // 3. Setup runtime and cancellation token
        let token = CancellationToken::new();
        let runtime = AgentRuntime::new(
            self.store.clone(),
            llm,
            self.tool_host.clone(),
            event_sink,
            Some(self.config.workspace_dir.to_string_lossy().to_string()),
        );

        let run_id = Uuid::now_v7();
        {
            let mut active = self.active_cancellations.lock().await;
            active.insert(run_id, token.clone());
        }

        let exec_result = runtime
            .execute_run_with_id(conversation_id, &agent, Some(prompt), token, Some(run_id))
            .await;

        {
            let mut active = self.active_cancellations.lock().await;
            active.remove(&run_id);
        }

        match exec_result {
            Ok(run) => Ok(run),
            // A cancel the user asked for is a normal outcome, not an error to show
            Err(AgentError::Cancelled) => self
                .store
                .get_run(run_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Run {run_id}"))),
            Err(e) => Err(AppError::Agent(e.to_string())),
        }
    }

    pub async fn cancel_run(&self, run_id: Uuid) -> Result<bool, AppError> {
        let active = self.active_cancellations.lock().await;
        if let Some(token) = active.get(&run_id) {
            token.cancel();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // --- Workspace & Reports ---

    pub fn workspace(&self) -> &Arc<dyn WorkspaceService> {
        &self.workspace
    }

    pub async fn list_reports(&self) -> Result<Vec<String>, AppError> {
        self.reports.list_reports().await
    }

    pub async fn read_report(&self, filename: &str) -> Result<String, AppError> {
        self.reports.read_report(filename).await
    }

    pub async fn write_report(
        &self,
        title: &str,
        content: &str,
        filename: Option<&str>,
    ) -> Result<String, AppError> {
        self.reports.write_report(title, content, filename).await
    }

    pub async fn list_documents(&self) -> Result<Vec<Document>, AppError> {
        self.reports.list_documents().await
    }

    pub async fn search_documents(&self, query: &str) -> Result<Vec<Document>, AppError> {
        self.reports.search_documents(query).await
    }

    pub async fn list_audit_logs(&self, limit: u32) -> Result<Vec<AuditLog>, AppError> {
        self.reports.list_audit_logs(limit).await
    }
}
