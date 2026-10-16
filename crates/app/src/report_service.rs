use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use hubbub_domain::entities::audit_log::AuditLog;
use hubbub_domain::entities::document::Document;
use hubbub_domain::ports::store::Store;
use hubbub_domain::ports::workspace_service::WorkspaceService;
use hubbub_domain::text::truncate_chars;
use hubbub_workspace::{WorkspaceWatcher, WorkspaceWatcherSink};

use crate::errors::AppError;

struct StoreDocumentSink(Arc<dyn Store>);

impl WorkspaceWatcherSink for StoreDocumentSink {
    fn on_document_indexed(&self, doc: Document) {
        let store = self.0.clone();
        tokio::spawn(async move {
            let _ = store.upsert_document(&doc).await;
        });
    }
}

pub struct ReportService {
    workspace: Arc<dyn WorkspaceService>,
    store: Arc<dyn Store>,
    _watcher: Option<WorkspaceWatcher>,
}

impl ReportService {
    pub fn new(workspace: Arc<dyn WorkspaceService>, store: Arc<dyn Store>) -> Self {
        let sink = Arc::new(StoreDocumentSink(store.clone()));
        let root = PathBuf::from(workspace.workspace_root());
        let _watcher = WorkspaceWatcher::start(&root, workspace.clone(), sink).ok();
        Self {
            workspace,
            store,
            _watcher,
        }
    }

    pub async fn list_reports(&self) -> Result<Vec<String>, AppError> {
        let files = self.workspace.list_files("reports").await?;
        Ok(files.into_iter().filter(|f| f.ends_with(".md")).collect())
    }

    pub async fn read_report(&self, filename: &str) -> Result<String, AppError> {
        let clean = if filename.starts_with("reports/") || filename.starts_with("reports\\") {
            filename.to_string()
        } else {
            format!("reports/{filename}")
        };
        Ok(self.workspace.read_file(&clean).await?)
    }

    pub async fn write_report(
        &self,
        title: &str,
        content: &str,
        filename_opt: Option<&str>,
    ) -> Result<String, AppError> {
        let filename = match filename_opt {
            Some(f) if !f.trim().is_empty() => {
                let clean = f.trim().replace(['/', '\\'], "_");
                if clean.ends_with(".md") {
                    clean
                } else {
                    format!("{clean}.md")
                }
            }
            _ => {
                let date_str = Utc::now().format("%Y-%m-%d");
                let slug = Self::slugify(title);
                format!("{date_str}-{slug}.md")
            }
        };

        let path = format!("reports/{filename}");
        let final_content = if !content.trim_start().starts_with('#') {
            format!("# {title}\n\n{content}")
        } else {
            content.to_string()
        };

        self.workspace.write_file(&path, &final_content).await?;
        if let Ok(Some(mut doc)) = self.workspace.reindex_file(&path).await {
            doc.title = title.to_string();
            let _ = self.store.upsert_document(&doc).await;
        }

        Ok(path)
    }

    pub async fn list_documents(&self) -> Result<Vec<Document>, AppError> {
        Ok(self.store.list_documents().await?)
    }

    pub async fn search_documents(&self, query: &str) -> Result<Vec<Document>, AppError> {
        Ok(self.store.search_documents(query).await?)
    }

    pub async fn list_audit_logs(&self, limit: u32) -> Result<Vec<AuditLog>, AppError> {
        Ok(self.store.list_audit_logs(limit).await?)
    }

    fn slugify(title: &str) -> String {
        let slug: String = title
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
            .collect();
        let cleaned = slug.to_lowercase().replace(' ', "-");
        let trimmed = cleaned.trim_matches('-').to_string();
        if trimmed.is_empty() {
            format!("report-{}", &Uuid::now_v7().to_string()[..8])
        } else if trimmed.chars().count() > 50 {
            truncate_chars(&trimmed, 50).trim_matches('-').to_string()
        } else {
            trimmed
        }
    }
}
