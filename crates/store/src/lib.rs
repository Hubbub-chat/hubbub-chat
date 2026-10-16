//! # hubbub-store
//!
//! SQLite implementation of the persistent storage layer for Hubbub.
//! Uses `sqlx` with WAL mode, foreign keys, and FTS5 full-text search.

pub mod connection;
pub mod errors;
pub mod repositories;

use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use hubbub_domain::entities::audit_log::AuditLog;
use hubbub_domain::entities::conversation::{Conversation, Message};
use hubbub_domain::entities::document::Document;
use hubbub_domain::entities::run::{Run, Step};
use hubbub_domain::errors::DomainError;
use hubbub_domain::ports::store::Store;

pub use errors::StoreError;

/// SQLite-backed persistent store implementing the `Store` port.
#[derive(Clone, Debug)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Open or create an SQLite database at the given path or connection string.
    pub async fn open(database_path_or_url: &str) -> Result<Self, StoreError> {
        let pool = connection::create_pool(database_path_or_url).await?;
        Ok(Self { pool })
    }

    /// Open a new in-memory SQLite database (primarily for testing).
    pub async fn open_in_memory() -> Result<Self, StoreError> {
        let pool = connection::create_in_memory_pool().await?;
        Ok(Self { pool })
    }

    /// Get a reference to the underlying `SqlitePool`.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl Store for SqliteStore {
    // --- Conversations ---

    async fn create_conversation(&self, conv: &Conversation) -> Result<(), DomainError> {
        repositories::conversation::create(&self.pool, conv)
            .await
            .map_err(Into::into)
    }

    async fn get_conversation(&self, id: Uuid) -> Result<Option<Conversation>, DomainError> {
        repositories::conversation::get(&self.pool, id)
            .await
            .map_err(Into::into)
    }

    async fn list_conversations(&self) -> Result<Vec<Conversation>, DomainError> {
        repositories::conversation::list(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn update_conversation(&self, conv: &Conversation) -> Result<(), DomainError> {
        repositories::conversation::update(&self.pool, conv)
            .await
            .map_err(Into::into)
    }

    // --- Messages ---

    async fn append_message(&self, msg: &Message) -> Result<(), DomainError> {
        repositories::message::append(&self.pool, msg)
            .await
            .map_err(Into::into)
    }

    async fn list_messages(
        &self,
        conversation_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Message>, DomainError> {
        repositories::message::list(&self.pool, conversation_id, limit, offset)
            .await
            .map_err(Into::into)
    }

    async fn search_messages(&self, query: &str) -> Result<Vec<Message>, DomainError> {
        repositories::message::search(&self.pool, query)
            .await
            .map_err(Into::into)
    }

    // --- Runs ---

    async fn create_run(&self, run: &Run) -> Result<(), DomainError> {
        repositories::run::create_run(&self.pool, run)
            .await
            .map_err(Into::into)
    }

    async fn update_run(&self, run: &Run) -> Result<(), DomainError> {
        repositories::run::update_run(&self.pool, run)
            .await
            .map_err(Into::into)
    }

    async fn get_run(&self, id: Uuid) -> Result<Option<Run>, DomainError> {
        repositories::run::get_run(&self.pool, id)
            .await
            .map_err(Into::into)
    }

    async fn fail_interrupted_runs(&self) -> Result<u64, DomainError> {
        repositories::run::fail_interrupted_runs(&self.pool)
            .await
            .map_err(Into::into)
    }

    // --- Steps ---

    async fn create_step(&self, step: &Step) -> Result<(), DomainError> {
        repositories::run::create_step(&self.pool, step)
            .await
            .map_err(Into::into)
    }

    async fn list_steps(&self, run_id: Uuid) -> Result<Vec<Step>, DomainError> {
        repositories::run::list_steps(&self.pool, run_id)
            .await
            .map_err(Into::into)
    }

    // --- Documents ---

    async fn upsert_document(&self, doc: &Document) -> Result<(), DomainError> {
        repositories::document::upsert(&self.pool, doc)
            .await
            .map_err(Into::into)
    }

    async fn list_documents(&self) -> Result<Vec<Document>, DomainError> {
        repositories::document::list(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn search_documents(&self, query: &str) -> Result<Vec<Document>, DomainError> {
        repositories::document::search(&self.pool, query)
            .await
            .map_err(Into::into)
    }

    // --- Audit Logs ---

    async fn record_audit_log(&self, entry: &AuditLog) -> Result<(), DomainError> {
        repositories::audit_log::record(&self.pool, entry)
            .await
            .map_err(Into::into)
    }

    async fn list_audit_logs(&self, limit: u32) -> Result<Vec<AuditLog>, DomainError> {
        repositories::audit_log::list(&self.pool, limit)
            .await
            .map_err(Into::into)
    }

    // --- Model Quota & Usage ---

    async fn get_model_usage_stats(
        &self,
    ) -> Result<Vec<hubbub_domain::entities::model_quota::ModelUsageStat>, DomainError> {
        repositories::model_usage::get_model_usage_stats(&self.pool)
            .await
            .map_err(Into::into)
    }
}
