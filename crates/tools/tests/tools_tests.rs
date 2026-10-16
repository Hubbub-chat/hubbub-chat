#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use uuid::Uuid;

use hubbub_domain::ports::tool_host::{ToolContext, ToolHost};
use hubbub_tools::{BuiltinToolHost, WebFetchTool, WebSearchTool};

#[test]
fn test_web_search_html_parsing() {
    let sample_html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <table>
                <tr>
                    <td>1.&nbsp;</td>
                    <td><a class="result-link" href="https://rust-lang.org/">Rust Programming Language</a></td>
                </tr>
                <tr>
                    <td>&nbsp;</td>
                    <td class="result-snippet">A language empowering everyone to build reliable and efficient software.</td>
                </tr>
                <tr>
                    <td>2.&nbsp;</td>
                    <td><a class="result-link" href="https://en.wikipedia.org/wiki/Rust">Rust (programming language)</a></td>
                </tr>
                <tr>
                    <td>&nbsp;</td>
                    <td class="result-snippet">Rust is a multi-paradigm, general-purpose programming language.</td>
                </tr>
            </table>
        </body>
        </html>
    "#;

    let items = WebSearchTool::parse_lite_html(sample_html);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].title, "Rust Programming Language");
    assert_eq!(items[0].url, "https://rust-lang.org/");
    assert!(items[0].snippet.contains("reliable and efficient software"));

    assert_eq!(items[1].title, "Rust (programming language)");
    assert_eq!(items[1].url, "https://en.wikipedia.org/wiki/Rust");
    assert!(items[1].snippet.contains("multi-paradigm"));
}

#[test]
fn test_web_fetch_html_to_markdown() {
    let sample_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Trang tin tức Công nghệ</title>
            <script>alert("malicious script");</script>
            <style>body { color: red; }</style>
        </head>
        <body>
            <header>Menu navigation</header>
            <article>
                <h1>Bản tin Trí tuệ Nhân tạo 2026</h1>
                <p>Mô hình AI mới đạt bước tiến lớn trong khả năng suy luận đa bước.</p>
                <h2>Điểm nổi bật</h2>
                <ul>
                    <li>Tốc độ xử lý tăng gấp đôi</li>
                    <li>Khả năng gọi công cụ chính xác hơn</li>
                </ul>
            </article>
            <footer>Bản quyền 2026</footer>
        </body>
        </html>
    "#;

    let md = WebFetchTool::html_to_markdown(sample_html, "https://example.com/news");

    assert!(md.contains("# Trang tin tức Công nghệ"));
    assert!(md.contains("*Nguồn: https://example.com/news*"));
    assert!(md.contains("Bản tin Trí tuệ Nhân tạo 2026"));
    assert!(md.contains("Điểm nổi bật"));
    assert!(md.contains("- Tốc độ xử lý tăng gấp đôi"));
    assert!(md.contains("- Khả năng gọi công cụ chính xác hơn"));
    // Scripts and styles should not appear as content
    assert!(!md.contains("alert("));
    assert!(!md.contains("color: red"));
}

#[tokio::test]
async fn test_web_fetch_blocks_ssrf_targets_before_network() {
    let tool = WebFetchTool::new();

    let ssrf_targets = [
        "http://127.0.0.1:8080/metrics",
        "http://localhost/admin",
        "http://169.254.169.254/latest/meta-data",
        "http://192.168.1.1/router",
        "http://10.0.0.1/internal",
        "file:///etc/passwd",
    ];

    for target in ssrf_targets {
        let res = tool.fetch(target).await;
        assert!(
            res.is_err(),
            "Expected SSRF block for '{target}', but got Ok"
        );
    }
}

#[tokio::test]
async fn test_builtin_tool_host_execution() {
    let host = BuiltinToolHost::new();
    let ctx = ToolContext {
        agent_id: "researcher".to_string(),
        run_id: Uuid::now_v7(),
        workspace_root: ".".to_string(),
    };

    // Missing query
    let res_empty = host
        .execute(
            "web_search",
            serde_json::json!({ "query": "" }),
            ctx.clone(),
        )
        .await
        .unwrap();
    assert!(!res_empty.success);
    assert!(
        res_empty
            .content
            .contains("Lỗi: Tham số 'query' không được để trống")
    );

    // Missing url
    let res_no_url = host
        .execute("web_fetch", serde_json::json!({ "url": "" }), ctx.clone())
        .await
        .unwrap();
    assert!(!res_no_url.success);
    assert!(
        res_no_url
            .content
            .contains("Lỗi: Tham số 'url' không được để trống")
    );

    // SSRF URL execution via host
    let res_ssrf = host
        .execute(
            "web_fetch",
            serde_json::json!({ "url": "http://127.0.0.1/admin" }),
            ctx.clone(),
        )
        .await
        .unwrap();
    assert!(!res_ssrf.success);
    assert!(res_ssrf.content.contains("SSRF protection"));
}

#[test]
fn test_builtin_tool_host_available_tools_schema() {
    let host = BuiltinToolHost::new();
    let tools = host.available_tools("researcher");

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"web_search"));
    assert!(names.contains(&"web_fetch"));
    assert!(names.contains(&"fs_read"));
    assert!(names.contains(&"fs_list"));
    assert!(names.contains(&"report_write"));
    assert!(names.contains(&"report_read"));
    assert!(names.contains(&"report_list"));
}

#[tokio::test]
async fn test_builtin_tool_host_fs_and_report_tools() {
    use hubbub_workspace::LocalWorkspaceService;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let ws = Arc::new(LocalWorkspaceService::new(dir.path()));
    let host = BuiltinToolHost::with_workspace(ws, None);

    let ctx = ToolContext {
        agent_id: "researcher".to_string(),
        run_id: Uuid::now_v7(),
        workspace_root: dir.path().to_string_lossy().to_string(),
    };

    // 1. report_write
    let write_res = host
        .execute(
            "report_write",
            serde_json::json!({
                "title": "Báo cáo AI Quốc tế 2026",
                "content": "Nội dung báo cáo chi tiết về AI.",
                "filename": "ai-report.md"
            }),
            ctx.clone(),
        )
        .await
        .unwrap();
    assert!(write_res.success);
    assert!(write_res.content.contains("reports/ai-report.md"));

    // 2. report_read
    let read_res = host
        .execute(
            "report_read",
            serde_json::json!({ "filename": "ai-report.md" }),
            ctx.clone(),
        )
        .await
        .unwrap();
    assert!(read_res.success);
    assert!(read_res.content.contains("Báo cáo AI Quốc tế 2026"));
    assert!(
        read_res
            .content
            .contains("Nội dung báo cáo chi tiết về AI.")
    );

    // 3. report_list
    let list_res = host
        .execute("report_list", serde_json::json!({}), ctx.clone())
        .await
        .unwrap();
    assert!(list_res.success);
    assert!(list_res.content.contains("ai-report.md"));

    // 4. fs_read
    let fs_read_res = host
        .execute(
            "fs_read",
            serde_json::json!({ "path": "reports/ai-report.md" }),
            ctx.clone(),
        )
        .await
        .unwrap();
    assert!(fs_read_res.success);
    assert!(fs_read_res.content.contains("Báo cáo AI Quốc tế 2026"));

    // 5. fs_list
    let fs_list_res = host
        .execute(
            "fs_list",
            serde_json::json!({ "path": "reports" }),
            ctx.clone(),
        )
        .await
        .unwrap();
    assert!(fs_list_res.success);
    assert!(fs_list_res.content.contains("reports/ai-report.md"));
}

#[tokio::test]
async fn test_report_write_long_vietnamese_title_does_not_panic() {
    use hubbub_workspace::LocalWorkspaceService;
    use std::sync::Arc;

    let dir = tempfile::tempdir().unwrap();
    let ws = Arc::new(LocalWorkspaceService::new(dir.path()));
    let host = BuiltinToolHost::with_workspace(ws, None);
    let ctx = ToolContext {
        agent_id: "researcher".to_string(),
        run_id: Uuid::now_v7(),
        workspace_root: dir.path().to_string_lossy().to_string(),
    };

    // Slug keeps non-ASCII letters; byte 50 of "a" + "ệ"* falls mid-character
    let title = format!("A{}", "ệ".repeat(60));
    let res = host
        .execute(
            "report_write",
            serde_json::json!({ "title": title, "content": "Nội dung." }),
            ctx,
        )
        .await
        .unwrap();
    assert!(res.success, "{}", res.content);
    assert!(res.content.contains(&format!("a{}.md", "ệ".repeat(49))));
}

#[test]
fn test_web_fetch_truncation_is_char_safe() {
    let body = "ệ".repeat(30_000);
    let out = WebFetchTool::truncate_for_llm(body);
    assert!(out.starts_with(&"ệ".repeat(25_000)));
    assert!(!out.starts_with(&"ệ".repeat(25_001)));
    assert!(out.contains("cắt bớt"));
}
