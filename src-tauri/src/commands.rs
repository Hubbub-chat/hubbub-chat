use std::sync::Arc;
use tauri::{AppHandle, State};
use uuid::Uuid;

use hubbub_app::AppService;
use hubbub_domain::entities::agent::Agent;
use hubbub_domain::entities::audit_log::AuditLog;
use hubbub_domain::entities::conversation::{Conversation, Message};
use hubbub_domain::entities::document::Document;
use hubbub_domain::entities::model_quota::ModelUsageStat;
use hubbub_domain::entities::run::Run;

use crate::events::TauriEventSink;

/// Tauri IPC command: health check.
#[tauri::command]
pub fn health_check() -> String {
    "Hubbub is running".to_string()
}

/// Tauri IPC command: get app version.
#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Tauri IPC command: list all conversations.
#[tauri::command]
pub async fn list_conversations(
    service: State<'_, Arc<AppService>>,
) -> Result<Vec<Conversation>, String> {
    service
        .list_conversations()
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: create a new conversation.
#[tauri::command]
pub async fn create_conversation(
    title: Option<String>,
    agent_id: Option<String>,
    service: State<'_, Arc<AppService>>,
) -> Result<Conversation, String> {
    service
        .create_conversation(title, agent_id)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: get a single conversation by ID.
#[tauri::command]
pub async fn get_conversation(
    id: Uuid,
    service: State<'_, Arc<AppService>>,
) -> Result<Option<Conversation>, String> {
    service
        .get_conversation(id)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: list messages in a conversation.
#[tauri::command]
pub async fn list_messages(
    conversation_id: Uuid,
    limit: Option<u32>,
    offset: Option<u32>,
    service: State<'_, Arc<AppService>>,
) -> Result<Vec<Message>, String> {
    service
        .list_messages(conversation_id, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: list available agents.
#[tauri::command]
pub async fn list_agents(service: State<'_, Arc<AppService>>) -> Result<Vec<Agent>, String> {
    Ok(service.list_agents().await)
}

/// Tauri IPC command: set model for an agent.
#[tauri::command]
pub async fn set_agent_model(
    agent_id: String,
    model: String,
    service: State<'_, Arc<AppService>>,
) -> Result<(), String> {
    service
        .set_agent_model(&agent_id, &model)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: create or update an agent.
#[tauri::command]
pub async fn upsert_agent(agent: Agent, service: State<'_, Arc<AppService>>) -> Result<(), String> {
    service.upsert_agent(agent).await.map_err(|e| e.to_string())
}

/// Tauri IPC command: delete an agent.
#[tauri::command]
pub async fn delete_agent(
    agent_id: String,
    service: State<'_, Arc<AppService>>,
) -> Result<(), String> {
    service
        .delete_agent(&agent_id)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: store provider API key (write-only).
#[tauri::command]
pub async fn set_provider_key(
    provider: String,
    api_key: String,
    service: State<'_, Arc<AppService>>,
) -> Result<(), String> {
    service
        .set_provider_key(&provider, &api_key)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: check if provider API key is configured.
#[tauri::command]
pub async fn has_provider_key(
    provider: String,
    service: State<'_, Arc<AppService>>,
) -> Result<bool, String> {
    service
        .has_provider_key(&provider)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: delete provider API key.
#[tauri::command]
pub async fn delete_provider_key(
    provider: String,
    service: State<'_, Arc<AppService>>,
) -> Result<(), String> {
    service
        .delete_provider_key(&provider)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: send message and execute agent run with streaming events.
#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    conversation_id: Uuid,
    agent_id: Option<String>,
    prompt: String,
    service: State<'_, Arc<AppService>>,
) -> Result<Run, String> {
    let event_sink = Arc::new(TauriEventSink::new(app));
    service
        .send_message(conversation_id, agent_id, &prompt, event_sink)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: cancel a running agent run.
#[tauri::command]
pub async fn cancel_run(run_id: Uuid, service: State<'_, Arc<AppService>>) -> Result<bool, String> {
    service.cancel_run(run_id).await.map_err(|e| e.to_string())
}

/// Tauri IPC command: list all reports in the workspace.
#[tauri::command]
pub async fn list_reports(service: State<'_, Arc<AppService>>) -> Result<Vec<String>, String> {
    service.list_reports().await.map_err(|e| e.to_string())
}

/// Tauri IPC command: read a specific report from reports/.
#[tauri::command]
pub async fn read_report(
    filename: String,
    service: State<'_, Arc<AppService>>,
) -> Result<String, String> {
    service
        .read_report(&filename)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: list indexed workspace documents.
#[tauri::command]
pub async fn list_documents(service: State<'_, Arc<AppService>>) -> Result<Vec<Document>, String> {
    service.list_documents().await.map_err(|e| e.to_string())
}

/// Tauri IPC command: search indexed documents with FTS5.
#[tauri::command]
pub async fn search_documents(
    query: String,
    service: State<'_, Arc<AppService>>,
) -> Result<Vec<Document>, String> {
    service
        .search_documents(&query)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: write a report into workspace reports/.
#[tauri::command]
pub async fn write_report(
    title: String,
    content: String,
    filename: Option<String>,
    service: State<'_, Arc<AppService>>,
) -> Result<String, String> {
    service
        .write_report(&title, &content, filename.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: list audit logs.
#[tauri::command]
pub async fn list_audit_logs(
    limit: Option<u32>,
    service: State<'_, Arc<AppService>>,
) -> Result<Vec<AuditLog>, String> {
    service
        .list_audit_logs(limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: get model usage and quota stats.
#[tauri::command]
pub async fn get_model_usage_stats(
    service: State<'_, Arc<AppService>>,
) -> Result<Vec<ModelUsageStat>, String> {
    service
        .get_model_usage_stats()
        .await
        .map_err(|e| e.to_string())
}
