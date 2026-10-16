//! Web fetch tool to safely retrieve and convert web pages to clean text/Markdown.

use scraper::{Html, Selector};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

use crate::errors::ToolError;
use hubbub_domain::text::truncate_chars;
use hubbub_policy::{PolicyError, UrlGuard};

const MAX_REDIRECTS: usize = 5;
const MAX_BODY_BYTES: usize = 5 * 1024 * 1024;

pub struct WebFetchTool {
    client: reqwest::Client,
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves hostnames and keeps only public addresses, so the address that is checked
/// is the one connected to (a hostname can resolve to 127.0.0.1 or a LAN address).
struct PublicOnlyResolver;

impl reqwest::dns::Resolve for PublicOnlyResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let addrs = WebFetchTool::resolve_public_addrs(&host).await?;
            Ok(Box::new(addrs.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

impl WebFetchTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            // Redirects are followed by hand so every hop is validated
            .redirect(reqwest::redirect::Policy::none())
            .dns_resolver(Arc::new(PublicOnlyResolver))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Hubbub/0.2.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    /// DNS lookup that fails unless at least one resolved address is public.
    pub async fn resolve_public_addrs(host: &str) -> Result<Vec<SocketAddr>, ToolError> {
        let resolved = tokio::net::lookup_host((host, 0)).await.map_err(|e| {
            ToolError::Network(format!("Không phân giải được tên miền {host}: {e}"))
        })?;
        let public: Vec<SocketAddr> = resolved
            .filter(|a| !UrlGuard::is_private_or_restricted_ip(&a.ip()))
            .collect();
        if public.is_empty() {
            return Err(PolicyError::SsrfBlocked {
                url: host.to_string(),
                ip: "private or restricted address".to_string(),
            }
            .into());
        }
        Ok(public)
    }

    /// Resolve a `Location` header against the current URL and validate the result.
    pub fn redirect_target(current: &Url, location: &str) -> Result<Url, ToolError> {
        let next = current
            .join(location)
            .map_err(|e| ToolError::Network(format!("Chuyển hướng không hợp lệ: {e}")))?;
        Ok(UrlGuard::validate_url(next.as_str())?)
    }

    /// Read at most `max_bytes` of the body, so a huge response can't exhaust memory.
    pub async fn read_body_limited(
        mut res: reqwest::Response,
        max_bytes: usize,
    ) -> Result<Vec<u8>, ToolError> {
        let mut body = Vec::new();
        while let Some(chunk) = res
            .chunk()
            .await
            .map_err(|e| ToolError::Network(format!("Lỗi đọc nội dung phản hồi: {e}")))?
        {
            let room = max_bytes - body.len();
            if chunk.len() >= room {
                body.extend_from_slice(&chunk[..room]);
                break;
            }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }

    /// Fetch a web page and convert its content to clean Markdown/text.
    pub async fn fetch(&self, raw_url: &str) -> Result<String, ToolError> {
        let trimmed_url = raw_url.trim();
        if trimmed_url.is_empty() {
            return Err(ToolError::MissingParameter("url"));
        }

        // 1. SSRF and URL validation guard (the resolver also checks resolved IPs)
        let mut url = UrlGuard::validate_url(trimmed_url)?;

        // 2. GET, following redirects by hand and re-validating every hop
        let mut redirects = 0;
        let res = loop {
            let res = self
                .client
                .get(url.as_str())
                .send()
                .await
                .map_err(|e| ToolError::Network(format!("Không thể truy cập URL: {e}")))?;
            if !res.status().is_redirection() {
                break res;
            }
            redirects += 1;
            if redirects > MAX_REDIRECTS {
                return Err(ToolError::Network("Quá nhiều lần chuyển hướng".to_string()));
            }
            let location = res
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| ToolError::Network("Chuyển hướng thiếu Location".to_string()))?;
            url = Self::redirect_target(&url, location)?;
        };
        let final_url = url.to_string();

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

        // 3. Read a bounded body (a cut mid-character becomes U+FFFD, never a panic)
        let bytes = Self::read_body_limited(res, MAX_BODY_BYTES).await?;
        let raw_body = String::from_utf8_lossy(&bytes).into_owned();

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
