#![allow(clippy::unwrap_used, clippy::expect_used)]

use hubbub_domain::entities::agent::Agent;
use hubbub_policy::AgentValidator;
use hubbub_testkit::create_test_agent;

const KNOWN: &[&str] = &[
    "web_search",
    "web_fetch",
    "fs_read",
    "fs_list",
    "report_write",
];

fn valid_agent() -> Agent {
    let mut agent = create_test_agent("chuyen_gia_devops", "Prompt");
    agent.tools.builtin = vec!["web_search".to_string(), "report_write".to_string()];
    agent.permissions.fs_read = vec!["reports/**".to_string(), "notes/**".to_string()];
    agent.permissions.fs_write = vec!["reports/**".to_string()];
    agent
}

fn with(edit: impl FnOnce(&mut Agent)) -> Agent {
    let mut agent = valid_agent();
    edit(&mut agent);
    agent
}

#[test]
fn accepts_a_well_formed_agent() {
    assert!(AgentValidator::validate(&valid_agent(), KNOWN).is_ok());
    assert!(
        AgentValidator::validate(&with(|a| a.permissions.fs_read = vec!["**".into()]), KNOWN)
            .is_ok()
    );
}

#[test]
fn rejects_bad_ids() {
    for id in ["", "Chuyên gia", "../evil", "a/b", &"x".repeat(65)] {
        let agent = with(|a| a.id = id.to_string());
        assert!(
            AgentValidator::validate(&agent, KNOWN).is_err(),
            "id {id:?}"
        );
    }
}

#[test]
fn rejects_unknown_builtin_tools() {
    let agent = with(|a| a.tools.builtin.push("shell_exec".into()));
    assert!(AgentValidator::validate(&agent, KNOWN).is_err());
}

#[test]
fn rejects_unsafe_read_patterns() {
    for pattern in [
        "../**",
        "/etc/**",
        "C:/Windows/**",
        "data/**",
        "DATA/hubbub.db",
        "",
    ] {
        let agent = with(|a| a.permissions.fs_read = vec![pattern.to_string()]);
        assert!(
            AgentValidator::validate(&agent, KNOWN).is_err(),
            "fs_read {pattern:?}"
        );
    }
}

#[test]
fn rejects_write_patterns_covering_protected_paths() {
    for pattern in [
        "**",
        "*",
        "agents/**",
        ".versions/**",
        "data/**",
        "config.toml",
        "../**",
    ] {
        let agent = with(|a| a.permissions.fs_write = vec![pattern.to_string()]);
        assert!(
            AgentValidator::validate(&agent, KNOWN).is_err(),
            "fs_write {pattern:?}"
        );
    }
}
