# Hubbub

<div align="center">

![Hubbub Desktop Banner](src/assets/hero.png)

### The Privacy-First Multi-Agent Desktop AI Workspace

[![Release](https://img.shields.io/badge/release-v0.1.0-6366f1.svg?style=flat-square)](https://github.com/thanghn14/hubbub-chat/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-0284c7.svg?style=flat-square)](https://tauri.app)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg?style=flat-square)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/tauri-v2.12-24c8db.svg?style=flat-square)](https://tauri.app)
[![React](https://img.shields.io/badge/react-19.2-61dafb.svg?style=flat-square)](https://react.dev)
[![TypeScript](https://img.shields.io/badge/typescript-6.0-3178c6.svg?style=flat-square)](https://www.typescriptlang.org)
[![Database](https://img.shields.io/badge/database-SQLite3%20%2B%20FTS5-003b57.svg?style=flat-square)](https://sqlite.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-emerald.svg?style=flat-square)](LICENSE)

*An enterprise-grade, memory-efficient desktop workspace that orchestrates multi-provider AI agents with zero cloud credential leakage and full-text workspace indexing.*

[Key Features](#key-features) • [Technical Highlights](#technical-highlights) • [Getting Started](#getting-started) • [Security & Privacy](#security--privacy) • [Documentation](#documentation)

</div>

---

## Overview

**Hubbub** is a next-generation personal desktop application for developers, engineers, and researchers. Built on **Rust + Tauri 2** with **React 19**, Hubbub reimagines the AI-assisted workflow by bringing Multi-Agent Orchestration, a Security Policy Guard, and Full-Text Search (FTS5) indexing of reports directly onto the user's local machine.

Unlike typical web applications, Hubbub is built to protect privacy:
- **All API keys** are encrypted and stored exclusively in the **Windows Credential Manager (OS Keyring)** — never saved as plaintext in configuration files or the database.
- **Chat history and documents** are stored locally in an embedded SQLite database and are not sent to any intermediary server.
- **Efficient resource usage**: the Rust core runs in the background using only about 3.2 MB of RAM, the whole application stays under 180 MB of RAM, and it can stream responses at 50+ tokens per second smoothly.

---

## Key Features

### 1. Role-based Functional Agents
- Set up multiple agents, each for a specialized purpose: **System Analyst**, **Software Developer**, **Researcher**, **Technical Writer**, and **Academic Tutor**.
- Each agent has its own System Prompt, its own limits (Token Budget, Max Steps, Cost Guard), and a set of granted tools.

### 2. Hybrid Multi-Provider Gateway
- Instant support for leading AI providers:
  - **Google Gemini**: Gemini 3.8 Flash, Gemini 1.5 Pro, Gemini 2.5 Flash.
  - **Anthropic Claude**: Claude Sonnet 4, Claude 3.5 Sonnet.
  - **OpenAI**: GPT-4o, GPT-4o-mini.
  - **Groq Cloud**: Llama 3.3 70B Versatile with very fast inference.
  - **Ollama Local**: Fully offline operation without Internet access, with Qwen 2.5 Coder and Llama 3.2.
  - **OpenRouter**: Access to hundreds of open-source models worldwide.

### 3. Enterprise Security: OS Keyring & Sandbox Policy
- **Zero Cloud Leakage**: Uses `keyring-rs` to interact directly with the Windows Data Protection API (DPAPI).
- **Path Guard**: Blocks Path Traversal attacks (`../`) and confines all file reads and writes to the Workspace.
- **URL Guard**: Blocks access to internal network ranges (SSRF prevention) and only allows document loading from safe domains.
- **Audit Logs**: Records every tool execution in detail, including a digest of its parameters and the allow/deny decision.

### 4. Model Quota & Cost Monitoring
- Automatically tracks the number of API calls per day (Requests Per Day - RPD) and the call rate (RPM).
- Shows detailed statistics for prompt tokens, completion tokens, and estimated cost in USD in real time.
- Visual warnings when usage approaches the API quota limit.

### 5. Workspace Report Library & FTS5 Full-Text Search
- Lets the AI automatically export research results as structured Markdown documents into the `reports/` folder.
- Includes the **Milkdown** real-time WYSIWYG editor (Nord theme).
- The **SQLite FTS5** search engine provides full-text lookup across documents and conversations with latency under 5 ms.

### 6. Developer-focused Interface (Antigravity & Cursor-inspired Aesthetic)
- Layered, elegant Dark Mode design (Obsidian / Zinc / Slate), with subtle rounded corners and hairline glow borders.
- A smart **Collapsible Sidebar** that fits both small split-screen windows (720px – 900px) and large 4K ultrawide displays.
- A modern input box with full Vietnamese IME support (UniKey, EVKey) and no character glitches.

---

## Technical Highlights

The system follows **Clean Architecture** principles, with the user interface and the core processing system cleanly separated:

- **Desktop Shell & IPC**: **Tauri 2** with asynchronous, strongly-typed IPC (typed commands and events).
- **Core Runtime**: A backend written entirely in **Rust** with the **Tokio** async engine, aiming for maximum speed and memory safety.
- **Storage Subsystem**: Embedded SQLite with WAL mode transactions and an FTS5 full-text index.
- **Credential Storage**: Windows Credential Manager via DPAPI, so API keys are protected at the operating system account level.
- **Frontend Stack**: A client built on **React 19**, **TypeScript**, **Tailwind CSS v4**, and **Vite 8**, using **TanStack Virtual** for list virtualization so thousands of messages scroll smoothly.

---

## Getting Started

### Prerequisites

1. **Operating system**: Windows 10/11 x64.
2. **Rust toolchain**: The latest stable Rust (MSVC target).
   ```bash
   rustup default stable-x86_64-pc-windows-msvc
   ```
3. **C++ Build Tools**: Visual Studio 2022 with the "Desktop development with C++" workload.
4. **Node.js**: Version `>= 20.x`, with the bundled `npm`.
5. **Just Command Runner** (optional but recommended):
   ```bash
   cargo install just
   ```

### Installation & Running

```bash
# 1. Clone the repository
git clone https://github.com/thanghn14/hubbub-chat.git
cd hubbub-chat

# 2. Install the desktop app dependencies
npm install

# 3. Start development mode
just dev
# Or run the command directly if you don't have just installed:
# cargo tauri dev --manifest-path src-tauri/Cargo.toml
```

### Testing & Building

```bash
# Run the full Rust core test suite
just test

# Check linting and source code formatting
just check

# Build the Windows release packages (.msi, .exe, portable .zip)
just build
```

After a successful build, the installer packages are located in:
`target/release/bundle/msi/` and `target/release/bundle/nsis/`.

---

## Shortcuts & Usage

| Shortcut / Action | Description |
| :--- | :--- |
| `Enter` | Send a message to the selected AI agent |
| `Shift + Enter` | Insert a new line in the input box |
| `Toggle Panel` | Collapse/expand the left conversation panel to free up space |
| `Copy Code` | Copy an entire code block from a response with one click |
| `Save Report` | Save research content to the Workspace's `reports/` folder |
| `Model Switch` | Switch the LLM model instantly from the title bar |

---

## Security & Privacy

Hubbub is designed around defense-in-depth security:

1. **Zero-Knowledge Core**: The application includes no hidden telemetry or user-tracking tools.
2. **Encrypted Keyring**: Secret keys are loaded into RAM only when making an API request and are cleared immediately afterward.
3. **Workspace Isolation**: AI agents may only interact with files inside the Workspace folder the user has specified; any request outside it is blocked immediately by the Policy Guard.

---

## Documentation

Detailed technical documentation is maintained in the `docs/` folder:

- [Master Plan](docs/MASTER_PLAN.md)
- [Decision Log](docs/DECISION_LOG.md)
- [System Architecture](docs/ARCHITECTURE.md)
- [Threat Model](docs/THREAT_MODEL.md)
- [AI Development Workflow](docs/AI_WORKFLOW.md)

---

## License

This project is distributed under the **Apache License 2.0**. See [LICENSE](LICENSE) for details.

<div align="center">
<sub>Designed and developed with a strong commitment to performance, security, and developer experience.</sub>
</div>
