use crate::entities::audit_log::AuditLog;
use crate::entities::conversation::{Conversation, Message};
use crate::entities::document::Document;
use crate::entities::run::{Run, Step};
use crate::errors::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

/// Port for persistent storage operations.
#[async_trait]
pub trait Store: Send + Sync {
    // Conversations
    async fn create_conversation(&self, conv: &Conversation) -> Result<(), DomainError>;
    async fn get_conversation(&self, id: Uuid) -> Result<Option<Conversation>, DomainError>;
    async fn list_conversations(&self) -> Result<Vec<Conversation>, DomainError>;
    async fn update_conversation(&self, conv: &Conversation) -> Result<(), DomainError>;

    // Messages
    async fn append_message(&self, msg: &Message) -> Result<(), DomainError>;
    async fn list_messages(
        &self,
        conversation_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Message>, DomainError>;
    /// The newest `limit` messages, returned oldest-first.
    async fn list_recent_messages(
        &self,
        conversation_id: Uuid,
        limit: u32,
    ) -> Result<Vec<Message>, DomainError>;

    // Runs
    async fn create_run(&self, run: &Run) -> Result<(), DomainError>;
    async fn update_run(&self, run: &Run) -> Result<(), DomainError>;
    async fn get_run(&self, id: Uuid) -> Result<Option<Run>, DomainError>;
    /// Mark runs left `queued`/`running` by a previous process as failed. Returns how many.
    async fn fail_interrupted_runs(&self) -> Result<u64, DomainError>;

    // Steps
    async fn create_step(&self, step: &Step) -> Result<(), DomainError>;
    async fn list_steps(&self, run_id: Uuid) -> Result<Vec<Step>, DomainError>;

    // Documents
    async fn upsert_document(&self, doc: &Document) -> Result<(), DomainError>;
    async fn list_documents(&self) -> Result<Vec<Document>, DomainError>;
    async fn search_documents(&self, query: &str) -> Result<Vec<Document>, DomainError>;
    async fn search_messages(&self, query: &str) -> Result<Vec<Message>, DomainError>;

    // Audit Logs
    async fn record_audit_log(&self, entry: &AuditLog) -> Result<(), DomainError>;
    async fn list_audit_logs(&self, limit: u32) -> Result<Vec<AuditLog>, DomainError>;

    // Model Quota & Usage
    async fn get_model_usage_stats(
        &self,
    ) -> Result<Vec<crate::entities::model_quota::ModelUsageStat>, DomainError>;
}
