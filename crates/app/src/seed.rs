use hubbub_domain::entities::agent::{
    Agent, AgentBudget, AgentDelegation, AgentPermissions, AgentTools, NetworkPolicy,
};

/// Default functional agents available in Hubbub.
/// Agents are organized by FUNCTION / TASK (role-based), not by model.
/// Any model (e.g. Gemini, Claude, GPT-4o, Ollama) can power any agent.
pub fn default_agents() -> Vec<Agent> {
    vec![
        Agent {
            id: "analyst".to_string(),
            name: "Chuyên viên Phân tích".to_string(),
            model: "gemini-3.8-flash".to_string(),
            system_prompt: r#"You are an expert Data & System Analyst.
You specialize in deep analytical thinking, synthesizing complex information, and providing clear, comprehensive, and well-structured answers.
Always deliver detailed, high-quality responses with complete code examples, structured breakdowns, and actionable insights. Do not cut your explanations short or rush to finish.
Respond in the language used by the user."#
                .to_string(),
            tools: AgentTools {
                builtin: vec![
                    "web_search".to_string(),
                    "web_fetch".to_string(),
                    "report_write".to_string(),
                    "report_read".to_string(),
                ],
                mcp: vec![],
            },
            permissions: AgentPermissions {
                fs_read: vec!["reports/**".to_string()],
                fs_write: vec!["reports/**".to_string()],
                network: NetworkPolicy::Open,
            },
            budget: AgentBudget {
                max_steps: 25,
                max_tokens: 200_000,
                max_cost_usd: 0.5,
                timeout_s: 300,
            },
            delegation: AgentDelegation {
                can_delegate_to: vec![],
            },
            version: 2,
        },
        Agent {
            id: "developer".to_string(),
            name: "Kỹ sư Lập trình".to_string(),
            model: "qwen2.5-coder:7b".to_string(),
            system_prompt: r#"You are a Senior Software Engineer and Architect.
You specialize in writing clean, idiomatic, performant code, conducting in-depth code reviews, debugging tricky issues, and designing software architecture.
Follow best practices, adhere to language conventions (Rust, TypeScript, Python, etc.), and provide complete, functional code snippets with clear explanations of design decisions.
Always respond in the language used by the user."#
                .to_string(),
            tools: AgentTools {
                builtin: vec![
                    "fs_read".to_string(),
                    "fs_list".to_string(),
                    "report_write".to_string(),
                ],
                mcp: vec![],
            },
            permissions: AgentPermissions {
                fs_read: vec!["**".to_string()],
                fs_write: vec!["reports/**".to_string()],
                network: NetworkPolicy::None,
            },
            budget: AgentBudget {
                max_steps: 25,
                max_tokens: 200_000,
                max_cost_usd: 0.5,
                timeout_s: 300,
            },
            delegation: AgentDelegation {
                can_delegate_to: vec![],
            },
            version: 2,
        },
        Agent {
            id: "researcher".to_string(),
            name: "Trợ lý Nghiên cứu".to_string(),
            model: "gemini-3.8-flash".to_string(),
            system_prompt: r#"You are a Research Specialist focused on finding, analyzing, and synthesizing information from the web.
## Core Capabilities:
- Search the web for authoritative sources
- Fetch and extract key information from web pages
- Synthesize findings into structured, comprehensive reports
- Always cite source URLs for any empirical facts or claims
Respond in the language used by the user."#
                .to_string(),
            tools: AgentTools {
                builtin: vec![
                    "web_search".to_string(),
                    "web_fetch".to_string(),
                    "report_write".to_string(),
                    "report_read".to_string(),
                    "report_list".to_string(),
                ],
                mcp: vec![],
            },
            permissions: AgentPermissions {
                fs_read: vec!["reports/**".to_string()],
                fs_write: vec!["reports/**".to_string()],
                network: NetworkPolicy::Open,
            },
            budget: AgentBudget {
                max_steps: 30,
                max_tokens: 200_000,
                max_cost_usd: 1.0,
                timeout_s: 600,
            },
            delegation: AgentDelegation {
                can_delegate_to: vec![],
            },
            version: 2,
        },
        Agent {
            id: "writer".to_string(),
            name: "Biên tập & Soạn thảo".to_string(),
            model: "gemini-3.8-flash".to_string(),
            system_prompt: r#"You are an expert Technical Writer and Content Editor.
You excel at drafting clear documentation, technical specifications, engaging articles, concise summaries, and professional communications.
Focus on clarity, tone consistency, logical flow, and immaculate formatting.
Respond in the language used by the user."#
                .to_string(),
            tools: AgentTools {
                builtin: vec![
                    "report_write".to_string(),
                    "report_read".to_string(),
                    "report_list".to_string(),
                ],
                mcp: vec![],
            },
            permissions: AgentPermissions {
                fs_read: vec!["reports/**".to_string(), "notes/**".to_string()],
                fs_write: vec!["reports/**".to_string()],
                network: NetworkPolicy::None,
            },
            budget: AgentBudget {
                max_steps: 20,
                max_tokens: 150_000,
                max_cost_usd: 0.5,
                timeout_s: 300,
            },
            delegation: AgentDelegation {
                can_delegate_to: vec![],
            },
            version: 2,
        },
        Agent {
            id: "tutor".to_string(),
            name: "Gia sư Đồng hành".to_string(),
            model: "gemini-3.8-flash".to_string(),
            system_prompt: r#"You are an engaging, patient Learning Tutor and Mentor.
Help users master complex topics by breaking them down into intuitive concepts, step-by-step reasoning, real-world analogies, and guided practice.
Encourage deep understanding and verify comprehension through targeted questions.
Respond in the language used by the user."#
                .to_string(),
            tools: AgentTools {
                builtin: vec!["web_search".to_string()],
                mcp: vec![],
            },
            permissions: AgentPermissions {
                fs_read: vec![],
                fs_write: vec![],
                network: NetworkPolicy::SearchOnly,
            },
            budget: AgentBudget {
                max_steps: 20,
                max_tokens: 150_000,
                max_cost_usd: 0.3,
                timeout_s: 240,
            },
            delegation: AgentDelegation {
                can_delegate_to: vec![],
            },
            version: 2,
        },
        Agent {
            id: "librarian".to_string(),
            name: "Thủ thư Quản lý Tệp".to_string(),
            model: "gemini-3.8-flash".to_string(),
            system_prompt: r#"You are a Workspace Librarian specialized in managing, cataloging, and querying local files and notes.
You have read/write access to the user workspace but NO internet access.
Ensure accuracy and preserve file integrity at all times.
Respond in the language used by the user."#
                .to_string(),
            tools: AgentTools {
                builtin: vec![
                    "fs_read".to_string(),
                    "fs_list".to_string(),
                    "report_read".to_string(),
                    "report_write".to_string(),
                ],
                mcp: vec![],
            },
            permissions: AgentPermissions {
                fs_read: vec!["**".to_string()],
                fs_write: vec!["notes/**".to_string(), "reports/**".to_string()],
                network: NetworkPolicy::None,
            },
            budget: AgentBudget {
                max_steps: 20,
                max_tokens: 150_000,
                max_cost_usd: 0.5,
                timeout_s: 300,
            },
            delegation: AgentDelegation {
                can_delegate_to: vec![],
            },
            version: 2,
        },
    ]
}
