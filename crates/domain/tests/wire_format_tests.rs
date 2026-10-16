//! The frontend mirrors these shapes by hand in `src/types.ts`. These tests pin the
//! wire format so a change here is caught before it silently breaks the UI.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use hubbub_domain::entities::agent::{Agent, NetworkPolicy};
use hubbub_domain::entities::conversation::MessagePart;
use serde_json::json;

#[test]
fn new_agent_payload_from_agent_modal_deserializes() {
    // Shape built by AgentModal for a new agent
    let payload = json!({
        "id": "chuyen_gia_devops",
        "name": "Chuyên gia DevOps",
        "model": "gemini-3.8-flash",
        "system_prompt": "Bạn là chuyên gia DevOps.",
        "tools": { "builtin": ["web_search", "report_write"], "mcp": [] },
        "permissions": { "fs_read": [], "fs_write": ["reports/**"], "network": "search_only" },
        "budget": { "max_steps": 25, "max_tokens": 200000, "max_cost_usd": 0.5, "timeout_s": 300 },
        "delegation": { "can_delegate_to": [] },
        "version": 1
    });

    let agent: Agent = serde_json::from_value(payload).unwrap();
    assert_eq!(agent.permissions.network, NetworkPolicy::SearchOnly);
}

#[test]
fn agent_payload_without_permissions_is_rejected() {
    // What AgentModal used to send; it must not silently get default permissions
    let payload = json!({
        "id": "a", "name": "A", "model": "m", "system_prompt": "",
        "tools": { "builtin": [], "mcp": [] },
        "budget": { "max_steps": 1, "max_tokens": 1, "max_cost_usd": 0.0, "timeout_s": 1 }
    });
    assert!(serde_json::from_value::<Agent>(payload).is_err());
}

#[test]
fn message_parts_nest_fields_under_content() {
    let call = MessagePart::ToolCall {
        id: "call_1".to_string(),
        name: "web_search".to_string(),
        arguments: json!({ "query": "x" }),
        extra_content: None,
    };
    assert_eq!(
        serde_json::to_value(&call).unwrap(),
        json!({
            "type": "ToolCall",
            "content": { "id": "call_1", "name": "web_search", "arguments": { "query": "x" } }
        })
    );

    let result = MessagePart::ToolResult {
        tool_call_id: "call_1".to_string(),
        result: json!("ok"),
    };
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        json!({ "type": "ToolResult", "content": { "tool_call_id": "call_1", "result": "ok" } })
    );

    assert_eq!(
        serde_json::to_value(MessagePart::Text("Xin chào".to_string())).unwrap(),
        json!({ "type": "Text", "content": "Xin chào" })
    );
}
