//! Report writing tool for saving markdown research reports to reports/.

use chrono::Utc;
use hubbub_domain::errors::DomainError;
use hubbub_domain::ports::store::Store;
use hubbub_domain::ports::tool_host::ToolContext;
use hubbub_domain::ports::workspace_service::WorkspaceService;
use hubbub_domain::text::truncate_chars;
use std::sync::Arc;
use uuid::Uuid;

pub struct ReportWriteTool {
    workspace: Arc<dyn WorkspaceService>,
    store: Option<Arc<dyn Store>>,
}

impl ReportWriteTool {
    pub fn new(workspace: Arc<dyn WorkspaceService>, store: Option<Arc<dyn Store>>) -> Self {
        Self { workspace, store }
    }

    pub async fn write_report(
        &self,
        title: &str,
        content: &str,
        filename_opt: Option<&str>,
        context: &ToolContext,
    ) -> Result<String, DomainError> {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Ok("Lỗi: Tiêu đề báo cáo không được để trống.".to_string());
        }

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
                let slug = Self::slugify_title(trimmed_title);
                format!("{date_str}-{slug}.md")
            }
        };

        let relative_path = format!("reports/{filename}");

        let final_content = if !content.trim_start().starts_with('#') {
            format!("# {trimmed_title}\n\n{content}")
        } else {
            content.to_string()
        };

        if let Err(e) = self
            .workspace
            .write_file(&relative_path, &final_content)
            .await
        {
            return Ok(format!("Lỗi khi lưu báo cáo vào '{relative_path}': {e}"));
        }

        let size_bytes = final_content.len();

        if let Ok(Some(mut doc)) = self.workspace.reindex_file(&relative_path).await {
            doc.title = trimmed_title.to_string();
            doc.agent_id = Some(context.agent_id.clone());
            doc.run_id = Some(context.run_id);

            if let Some(store) = &self.store {
                let _ = store.upsert_document(&doc).await;
            }
        }

        Ok(format!(
            "✅ Báo cáo đã được lưu thành công!\n\n- **Tiêu đề:** {trimmed_title}\n- **Đường dẫn:** `{relative_path}`\n- **Dung lượng:** {size_bytes} bytes\n\nTài liệu đã được đánh chỉ mục và có thể tra cứu hoặc mở xem trực tiếp trong Workspace."
        ))
    }

    fn slugify_title(title: &str) -> String {
        let mut slug = String::new();
        for ch in title.chars() {
            if ch.is_alphanumeric() {
                slug.push(ch.to_ascii_lowercase());
            } else if (ch == ' ' || ch == '-' || ch == '_')
                && !slug.ends_with('-')
                && !slug.is_empty()
            {
                slug.push('-');
            }
        }
        let trimmed = slug.trim_matches('-').to_string();
        if trimmed.is_empty() {
            format!("report-{}", &Uuid::now_v7().to_string()[..8])
        } else if trimmed.chars().count() > 50 {
            truncate_chars(&trimmed, 50).trim_matches('-').to_string()
        } else {
            trimmed
        }
    }
}
