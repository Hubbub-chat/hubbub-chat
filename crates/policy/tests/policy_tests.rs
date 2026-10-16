#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use hubbub_domain::entities::agent::{AgentPermissions, NetworkPolicy};
use hubbub_policy::{PathGuard, PermissionChecker, PolicyError, UrlGuard};

#[test]
fn test_url_guard_allowed_public_urls() {
    let valid_urls = [
        "https://example.com",
        "https://www.google.com/search?q=rust+lang",
        "http://html.duckduckgo.com/html/?q=test",
        "https://api.tavily.com/search",
        "https://news.ycombinator.com/item?id=123",
    ];

    for u in valid_urls {
        let res = UrlGuard::validate_url(u);
        assert!(res.is_ok(), "Expected valid URL '{u}', got {:?}", res.err());
    }
}

#[test]
fn test_url_guard_disallows_non_http_schemes() {
    let invalid_schemes = [
        "file:///etc/passwd",
        "ftp://ftp.example.com",
        "gopher://example.com",
        "javascript:alert(1)",
        "data:text/html,<h1>hi</h1>",
    ];

    for u in invalid_schemes {
        let err = UrlGuard::validate_url(u).unwrap_err();
        match err {
            PolicyError::DisallowedScheme(_) => {}
            other => panic!("Expected DisallowedScheme for '{u}', got {other:?}"),
        }
    }
}

#[test]
fn test_url_guard_blocks_ssrf_ipv4_private_and_loopback() {
    let ssrf_urls = [
        "http://127.0.0.1/admin",
        "http://127.0.0.2:8080",
        "http://10.0.0.1",
        "http://10.254.0.1",
        "http://172.16.0.1",
        "http://172.31.255.255",
        "http://192.168.1.1",
        "http://192.168.0.254",
        "http://169.254.169.254/latest/meta-data", // AWS / GCP metadata IP
        "http://0.0.0.0:3000",
        "http://100.64.0.1", // Carrier-grade NAT
    ];

    for u in ssrf_urls {
        let err = UrlGuard::validate_url(u).unwrap_err();
        match err {
            PolicyError::SsrfBlocked { .. } => {}
            other => panic!("Expected SsrfBlocked for '{u}', got {other:?}"),
        }
    }
}

#[test]
fn test_url_guard_blocks_ssrf_ipv6_and_internal_hostnames() {
    let ssrf_urls = [
        "http://[::1]/",
        "http://[fe80::1]/",
        "http://[fc00::1]/",
        "http://localhost:8000",
        "http://myhost.localhost",
        "http://service.local",
        "http://app.internal",
        "http://metadata.google.internal/computeMetadata/v1/",
    ];

    for u in ssrf_urls {
        let err = UrlGuard::validate_url(u).unwrap_err();
        match err {
            PolicyError::SsrfBlocked { .. } => {}
            other => panic!("Expected SsrfBlocked for '{u}', got {other:?}"),
        }
    }
}

#[test]
fn test_path_guard_allows_safe_relative_paths() {
    let root = Path::new("/workspace");
    let safe_paths = [
        "notes.md",
        "reports/2026/summary.md",
        "sub/folder/file.json",
        "./clean.txt",
    ];

    for p in safe_paths {
        let res = PathGuard::resolve_within_workspace(root, p);
        assert!(res.is_ok(), "Expected safe path '{p}', got {:?}", res.err());
    }
}

#[test]
fn test_path_guard_blocks_traversal_and_escapes() {
    let root = Path::new("/workspace");
    let malicious_paths = [
        "../secret.txt",
        "reports/../../passwords.txt",
        "sub/../../../root.key",
        "folder/../..",
    ];

    for p in malicious_paths {
        let err = PathGuard::resolve_within_workspace(root, p).unwrap_err();
        match err {
            PolicyError::PathTraversal { .. } => {}
            other => panic!("Expected PathTraversal for '{p}', got {other:?}"),
        }
    }
}

#[test]
fn test_path_guard_blocks_absolute_paths_and_windows_attacks() {
    let root = Path::new("/workspace");
    let invalid_paths = [
        "/etc/passwd",
        "\\server\\share\\file.txt",
        "C:\\Windows\\System32",
        "file.txt:hidden_stream",
        "file.txt\0evil.jpg",
    ];

    for p in invalid_paths {
        let err = PathGuard::resolve_within_workspace(root, p).unwrap_err();
        assert!(
            matches!(
                err,
                PolicyError::AbsolutePathDisallowed(_) | PolicyError::InvalidPath(_)
            ),
            "Expected AbsolutePathDisallowed or InvalidPath for '{p}', got {err:?}"
        );
    }
}

#[test]
fn test_permission_checker_network_policy() {
    // Open policy
    assert!(PermissionChecker::check_network("analyst", &NetworkPolicy::Open, true).is_ok());
    assert!(PermissionChecker::check_network("analyst", &NetworkPolicy::Open, false).is_ok());

    // SearchOnly policy
    assert!(PermissionChecker::check_network("tutor", &NetworkPolicy::SearchOnly, true).is_ok());
    let err =
        PermissionChecker::check_network("tutor", &NetworkPolicy::SearchOnly, false).unwrap_err();
    assert!(matches!(err, PolicyError::PermissionDenied { .. }));

    // None policy
    let err_search =
        PermissionChecker::check_network("librarian", &NetworkPolicy::None, true).unwrap_err();
    assert!(matches!(err_search, PolicyError::PermissionDenied { .. }));
    let err_fetch =
        PermissionChecker::check_network("librarian", &NetworkPolicy::None, false).unwrap_err();
    assert!(matches!(err_fetch, PolicyError::PermissionDenied { .. }));
}

#[test]
fn test_permission_checker_fs_patterns() {
    let permissions = AgentPermissions {
        fs_read: vec!["**".to_string()],
        fs_write: vec!["reports/**".to_string(), "notes/draft.md".to_string()],
        network: NetworkPolicy::None,
    };

    // Read: allowed everywhere
    assert!(PermissionChecker::check_fs_read("dev", &permissions, "any/file.txt").is_ok());

    // Write: allowed in reports/ and notes/draft.md
    assert!(PermissionChecker::check_fs_write("dev", &permissions, "reports/summary.md").is_ok());
    assert!(PermissionChecker::check_fs_write("dev", &permissions, "reports/2026/deep.md").is_ok());
    assert!(PermissionChecker::check_fs_write("dev", &permissions, "notes/draft.md").is_ok());

    // Write: denied outside allowed patterns
    let err = PermissionChecker::check_fs_write("dev", &permissions, "src/main.rs").unwrap_err();
    assert!(matches!(err, PolicyError::PermissionDenied { .. }));

    let err2 =
        PermissionChecker::check_fs_write("dev", &permissions, "notes/other.md").unwrap_err();
    assert!(matches!(err2, PolicyError::PermissionDenied { .. }));
}

proptest::proptest! {
    #[test]
    fn proptest_path_guard_rejects_parent_traversal(prefix in "[a-zA-Z0-9_]{1,10}", suffix in "[a-zA-Z0-9_]{1,10}") {
        let root = Path::new("/workspace");
        let traversal_path = format!("{prefix}/../../{suffix}");
        let res = PathGuard::resolve_within_workspace(root, &traversal_path);
        proptest::prop_assert!(res.is_err());
    }

    #[test]
    fn proptest_path_guard_rejects_absolute_paths(rest in "[a-zA-Z0-9_/]{1,20}") {
        let root = Path::new("/workspace");
        let abs_path = format!("/{rest}");
        let res = PathGuard::resolve_within_workspace(root, &abs_path);
        proptest::prop_assert!(res.is_err());
    }

    #[test]
    fn proptest_url_guard_blocks_all_127_loopback(b in 0u8..=255, c in 0u8..=255, d in 0u8..=255) {
        let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, b, c, d));
        proptest::prop_assert!(UrlGuard::is_private_or_restricted_ip(&ip));
    }

    #[test]
    fn proptest_url_guard_blocks_all_10_private(b in 0u8..=255, c in 0u8..=255, d in 0u8..=255) {
        let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(10, b, c, d));
        proptest::prop_assert!(UrlGuard::is_private_or_restricted_ip(&ip));
    }

    #[test]
    fn proptest_url_guard_blocks_all_192_168_private(c in 0u8..=255, d in 0u8..=255) {
        let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, c, d));
        proptest::prop_assert!(UrlGuard::is_private_or_restricted_ip(&ip));
    }
}

#[test]
fn test_path_guard_protects_system_files() {
    let root = Path::new("/workspace");

    // Allowed paths
    assert!(PathGuard::check_write_path(root, "reports/summary.md").is_ok());
    assert!(PathGuard::check_write_path(root, "notes/todo.txt").is_ok());
    assert!(PathGuard::check_write_path(root, "notes/data/output.json").is_ok());

    // Protected paths must fail
    assert!(PathGuard::check_write_path(root, "data/output.json").is_err()); // app DB lives here
    assert!(PathGuard::check_write_path(root, "config.toml").is_err());
    // Windows treats these as the same paths as the lowercase ones
    assert!(PathGuard::check_write_path(root, "AGENTS/prompt.md").is_err());
    assert!(PathGuard::check_write_path(root, ".Versions./old.md").is_err());
    assert!(PathGuard::check_write_path(root, "Data/hubbub.db").is_err());
    assert!(PathGuard::check_write_path(root, "agents/analyst/agent.toml").is_err());
    assert!(PathGuard::check_write_path(root, "agents/prompt.md").is_err());
    assert!(PathGuard::check_write_path(root, ".versions/old.md").is_err());
    assert!(PathGuard::check_write_path(root, ".git/config").is_err());
    assert!(PathGuard::check_write_path(root, ".env").is_err());
}

#[test]
fn test_url_guard_blocks_localhost_with_trailing_dot() {
    // "localhost." is the same host as "localhost" (fully qualified form)
    for url in [
        "http://localhost./admin",
        "http://LOCALHOST./",
        "http://metadata.google.internal./",
    ] {
        assert!(UrlGuard::validate_url(url).is_err(), "{url}");
    }
}
