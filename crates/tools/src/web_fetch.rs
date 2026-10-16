//! Web fetch tool to safely retrieve and convert web pages to clean text/Markdown.

use scraper::{Html, Selector};
use std::time::Duration;

use crate::errors::ToolError;
use hubbub_domain::text::truncate_chars;
use hubbub_policy::UrlGuard;

pub struct WebFetchTool {
    client: reqwest::Client,
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebFetchTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .redirect(reqwest::redirect::Policy::limited(5))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Hubbub/0.2.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    /// Fetch a web page and convert its content to clean Markdown/text.
    pub async fn fetch(&self, raw_url: &str) -> Result<String, ToolError> {
        let trimmed_url = raw_url.trim();
        if trimmed_url.is_empty() {
            return Err(ToolError::MissingParameter("url"));
        }

        // 1. SSRF and URL validation guard
        let validated_url = UrlGuard::validate_url(trimmed_url)?;

        // 2. Perform HTTP GET request
        let res = self
            .client
            .get(validated_url.as_str())
            .send()
            .await
            .map_err(|e| ToolError::Network(format!("Không thể truy cập URL: {e}")))?;

        // 3. Check final redirect URL against SSRF
        let final_url = res.url().to_string();
        if final_url != trimmed_url {
            let _ = UrlGuard::validate_url(&final_url)?;
        }

        let status = res.status();
        if !status.is_success() {
            return Err(ToolError::Network(format!(
                "Máy chủ trang web trả về mã lỗi HTTP {status}"
            )));
        }

        let content_type = res
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_lowercase();

        let raw_body = res
            .text()
            .await
            .map_err(|e| ToolError::Network(format!("Lỗi đọc nội dung phản hồi: {e}")))?;

        // 4. Process content based on content-type
        let formatted = if content_type.contains("text/html") || raw_body.contains("<html") {
            Self::html_to_markdown(&raw_body, &final_url)
        } else {
            raw_body
        };

        // 5. Truncate if exceeds token-friendly limit
        Ok(Self::truncate_for_llm(formatted))
    }

    /// Cap page content at 25,000 characters (not bytes, so multi-byte text never splits).
    pub fn truncate_for_llm(formatted: String) -> String {
        const MAX_CHARS: usize = 25_000;
        let truncated = truncate_chars(&formatted, MAX_CHARS);
        if truncated.len() < formatted.len() {
            format!(
                "{truncated}\n\n[... Nội dung trang web đã được cắt bớt do vượt quá giới hạn độ dài {MAX_CHARS} ký tự ...]"
            )
        } else {
            formatted
        }
    }

    /// Convert HTML body into readable Markdown representation.
    pub fn html_to_markdown(html: &str, source_url: &str) -> String {
        let document = Html::parse_document(html);

        // Extract title
        let title = if let Ok(title_sel) = Selector::parse("title") {
            document
                .select(&title_sel)
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        // Extract main content from <main>, <article>, or <body>
        let content_sel = Selector::parse("main, article, div.content, div.main, body").ok();

        let mut lines = Vec::new();
        if !title.is_empty() {
            lines.push(format!("# {title}"));
            lines.push(format!("*Nguồn: {source_url}*\n---"));
        }

        let ps_sel = Selector::parse("h1, h2, h3, h4, p, li, pre, code").ok();
        if let Some((container, ps)) = content_sel
            .and_then(|sel| document.select(&sel).next())
            .zip(ps_sel)
        {
            for el in container.select(&ps) {
                let text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !text.is_empty() {
                    let tag_name = el.value().name();
                    match tag_name {
                        "h1" => lines.push(format!("\n# {text}")),
                        "h2" => lines.push(format!("\n## {text}")),
                        "h3" => lines.push(format!("\n### {text}")),
                        "li" => lines.push(format!("- {text}")),
                        _ => lines.push(text),
                    }
                }
            }
        }

        if lines.is_empty() {
            // Fallback: collect all text elements
            document.root_element().text().collect::<Vec<_>>().join(" ")
        } else {
            lines.join("\n\n")
        }
    }
}
