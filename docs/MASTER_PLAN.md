# HUBBUB — Master Development Plan
### Personal Multi-Agent Chat App for Learning & Research
**Phiên bản:** 1.0 · **Ngày:** 02/10/2026 · **Cập nhật lần cuối:** 02/10/2026

---

## 0. TẦM NHÌN & PHẠM VI v1

### Sản phẩm
Ứng dụng Desktop cá nhân (1 user duy nhất) tích hợp nhiều AI Agent chuyên biệt,
hỗ trợ học tập và nghiên cứu thông qua chat, tìm kiếm web, đọc tài liệu cục bộ,
và tạo báo cáo Markdown — tất cả được quản lý tập trung trong app.

### Phạm vi v1 — CÓ
- Chat văn bản với nhiều Agent
- Quản lý, điều phối nhiều Agent (Supervisor → Workers)
- Agent truy cập Internet (search, fetch) có kiểm soát
- Agent truy cập thư mục/file cục bộ được cấp quyền
- Agent tạo báo cáo Markdown, quản lý tập trung trong Workspace
- Hệ thống bảo mật capability-based
- Nền tảng: Windows 10/11

### Phạm vi v1 — KHÔNG
- Voice (STT/TTS) — sẽ xem xét ở v2 nếu cần
- Đồng bộ đa thiết bị
- Multi-user / Authentication
- Code execution / Shell access
- Cloud sync
- Android (kiến trúc sẵn sàng, triển khai ở Phase tương lai)

### Nguyên tắc đóng băng (thay đổi phải viết ADR + cập nhật DECISION_LOG.md)
1. LLM KHÔNG phải security authority — mọi quyền thực thi ở Rust.
2. Agent KHÔNG truy cập OS trực tiếp — mọi hành động qua Tool Gateway → Policy Engine.
3. Modular Monolith — chỉ tách service khi có bottleneck thực tế.
4. Markdown là first-class data — không phải attachment.
5. Mọi Agent execution phải có Run/Step/Trace.
6. Kiến trúc core phải platform-neutral để tái sử dụng cho Mobile.
7. Ưu tiên fix bug/vấn đề tiềm ẩn TRƯỚC KHI phát triển tính năng mới.
8. Test-first: viết test case (đặc biệt common edge cases) trước khi viết code.

---

## 1. KIẾN TRÚC HỆ THỐNG

### 1.1 Tổng quan

```
                          ┌──────────────────┐
                          │       USER       │
                          └────────┬─────────┘
                                   │ Text input
                                   ▼
                  ┌────────────────────────────────┐
                  │         DESKTOP UI              │
                  │   Tauri 2 + React + TypeScript  │
                  │                                 │
                  │  Chat │ Agents │ Files │ Runs   │
                  └───────────────┬─────────────────┘
                                  │ Tauri IPC (invoke + events)
                                  ▼
                  ┌────────────────────────────────┐
                  │         RUST APP CORE          │
                  │                                │
                  │  ┌─ Application ─────────────┐ │
                  │  │  Use-case facades         │ │
                  │  │  Composition root          │ │
                  │  └────────────┬───────────────┘ │
                  │               │                 │
                  │  ┌─ Domain ───┴───────────────┐ │
                  │  │  Entities, Value Objects   │ │
                  │  │  Ports (traits)             │ │
                  │  │  Domain Events              │ │
                  │  └────────────┬───────────────┘ │
                  │               │                 │
                  │  ┌─ Agent ────┴───────────────┐ │
                  │  │  Runtime loop              │ │
                  │  │  Orchestrator              │ │
                  │  │  Context builder           │ │
                  │  └────────────┬───────────────┘ │
                  │               │                 │
                  │  ┌─ Security ─┴───────────────┐ │
                  │  │  Policy Engine             │ │
                  │  │  PathGuard, UrlGuard       │ │
                  │  │  Approval Manager          │ │
                  │  │  Capability checks         │ │
                  │  └────────────────────────────┘ │
                  └───────┬──────────┬──────────────┘
                          │          │
               ┌──────────┴──┐  ┌───┴──────────────┐
               │  SQLite     │  │   Workspace      │
               │  + FTS5     │  │   Markdown/Files  │
               └─────────────┘  └──────────────────┘
                          │
               ┌──────────┴───────────┐
               ▼                      ▼
        ┌─────────────┐       ┌──────────────────┐
        │  LOCAL AI   │       │     EXTERNAL     │
        │  Ollama     │       │  Cloud LLM APIs  │
        │  llama.cpp  │       │  Web Search      │
        └─────────────┘       └──────────────────┘
```

### 1.2 Kiến trúc phân lớp (Clean Architecture)

```
UI (React + TypeScript)
  │  Chỉ hiển thị và nhập liệu. Không chứa logic nghiệp vụ.
  │  Giao tiếp với Rust qua Tauri IPC commands + events.
  │
Tauri Shell (Rust)
  │  Khai báo IPC commands, chuyển tiếp đến Application layer.
  │  Quản lý window, lifecycle, sidecar processes.
  │
Application Layer (Rust crate: app)
  │  Composition root. Use-case facades.
  │  Nơi DUY NHẤT biết adapter cụ thể nào được dùng.
  │
Domain Layer (Rust crate: domain)
  │  Entity, Value Object, Ports (trait), Error types.
  │  KHÔNG phụ thuộc I/O. KHÔNG import reqwest, sqlx, keyring...
  │
Agent Layer (Rust crate: agent)
  │  Runtime loop, orchestrator, context builder/compressor.
  │  Phụ thuộc domain ports, KHÔNG phụ thuộc adapter cụ thể.
  │
Security Layer (Rust crate: policy)
  │  PathGuard, UrlGuard, PermissionChecker, ApprovalManager.
  │  KHÔNG phụ thuộc I/O. Thuần logic validation.
  │
Infrastructure / Adapters (Rust crates: llm, tools, store, workspace, vault, mcp)
  │  Implement domain ports.
  │  llm: openai_compat, anthropic, ollama adapters
  │  tools: web_search, web_fetch, fs_read, report_write...
  │  store: SQLite via sqlx
  │  workspace: filesystem operations
  │  vault: OS keyring via keyring crate
  │  mcp: MCP client via rmcp
```

### 1.3 Quy tắc phụ thuộc (CI kiểm tra tự động)

```
domain ← policy, agent, adapters ← app ← tauri shell
```

- `domain`, `policy`, `agent`: KHÔNG được import crate I/O (reqwest, sqlx, keyring, notify).
- UI (TypeScript): KHÔNG gọi fetch/HTTP trực tiếp, chỉ qua Tauri invoke.
- Script `scripts/check-deps.sh` chạy trong CI để enforce.

### 1.4 Luồng dữ liệu chính

```
[User gõ tin nhắn]
       │
       ▼
[React UI] ──(Tauri invoke: send_message)──▶ [Rust App Core]
                                                    │
                                                    ▼
                                            [Agent Runtime]
                                              1. Dựng context (system prompt + history + tool schemas)
                                              2. Gọi LlmProvider (stream)
                                              3. Có tool_call?
                                              │   ├── Yes → Policy.check() → Approval? → Execute → ghi Step → quay lại 2
                                              │   └── No  → hoàn thành
                                              4. Dừng khi: hết tool_call | hết budget | bị huỷ | lỗi
                                                    │
[React UI] ◀──(Tauri event: run_event stream)──────┤
       │                                            ▼
       │                                    [Ghi file .md vào Workspace]
       │                                            │
       │◀──(Tauri event: documents_changed)─────────┘
       ▼
[UI cập nhật: hiển thị tin nhắn + mở tài liệu mới]
```

---

## 2. TECH STACK

### 2.1 Quyết định chính & Lý do

| Thành phần | Lựa chọn | Lý do |
|---|---|---|
| **Desktop Runtime** | Tauri 2 (Rust) | Bundle nhẹ (<15MB), native performance, IPC an toàn, hệ sinh thái ổn định |
| **UI Framework** | React 19 + TypeScript | Ecosystem lớn nhất, Markdown editor tốt nhất (Milkdown), component library phong phú |
| **UI Bundler** | Vite | Nhanh, cấu hình đơn giản, HMR tốt |
| **UI Styling** | TailwindCSS + shadcn/ui | Utility-first, component sẵn có, dễ tuỳ biến |
| **Markdown Editor** | Milkdown (ProseMirror) | WYSIWYG trên nền ProseMirror, hỗ trợ GFM, KaTeX, Mermaid, plugin ecosystem |
| **State Management** | Zustand | Nhẹ, TypeScript-first, không boilerplate |
| **Core Language** | Rust + Tokio | Memory safety, performance, async runtime, platform-neutral |
| **Database** | SQLite + WAL + FTS5 | Embedded, zero-config, reliable, full-text search built-in |
| **DB Access** | sqlx | Compile-time query checking, async, migration support |
| **LLM Integration** | Tự viết adapter qua trait | Không lock-in framework. Hỗ trợ: OpenAI-compat, Anthropic, Ollama |
| **Agent Framework** | Tự viết (Rust) | Kiểm soát hoàn toàn runtime, budget, cancellation. Tham khảo: Goose (Apache-2.0) |
| **Web Search** | Brave Search API (mặc định) | Qua trait SearchProvider, thay được SearXNG/Tavily |
| **Tool Protocol** | MCP (rmcp - official Rust SDK) | Interoperability, nhưng KHÔNG phải security boundary |
| **Secrets** | OS Keyring (crate keyring) | Windows Credential Manager, không plaintext |
| **Observability** | tracing + tracing-subscriber | Structured logging, Run tracing |

### 2.2 WebView2 RAM Optimization (quan trọng — Gate 0 sẽ validate)

Chiến lược tối ưu RAM cho WebView2 trên Windows:

1. **MemoryUsageTargetLevel**: Set `Low` khi app minimize/mất focus, `Normal` khi active.
2. **Content Virtualization**: Chỉ render ~100-200 tin nhắn gần nhất trong DOM, lazy-load khi cuộn.
3. **Markdown AST Cache**: Cache parsed AST theo message ID, giải phóng khi ra khỏi viewport.
4. **Stream Optimization**: Gom delta 16-32ms trước khi cập nhật DOM, chỉ re-parse block cuối khi streaming.
5. **Image/Media Lazy Load**: Không preload media ngoài viewport.
6. **Mục tiêu**: Idle ≤ 180MB với 1.000 tin nhắn. Cold start < 3s.

### 2.3 Đã loại bỏ có chủ đích

| Loại bỏ | Lý do |
|---|---|
| Python sidecar / FastAPI | Phức tạp 2-process, bundle nặng (PyInstaller ~150MB+), lifecycle khó quản lý |
| LangGraph / LangChain | Over-engineering cho app cá nhân, API thay đổi liên tục, lock-in |
| Dedicated vector DB (Qdrant) | Quá nặng cho v1. SQLite FTS5 đủ dùng. Thêm embedding ở phase sau |
| Voice (STT/TTS) | Phức tạp không cần thiết cho v1. Xem xét ở v2 |
| Electron | RAM quá cao (~300MB+), bundle nặng |
| Microservices / Redis / Kafka | Overkill cho app cá nhân desktop |
| Shell execution | Rủi ro bảo mật quá cao |

### 2.4 Phương án dự phòng

Nếu Gate 0 cho thấy WebView2 KHÔNG đạt mục tiêu RAM (>200MB idle với 1.000 tin nhắn):
- Chuyển UI sang **Flutter Desktop** + `flutter_rust_bridge`.
- Rust Core giữ nguyên 100% — chỉ thay lớp bridge.
- Trade-off: Markdown editor yếu hơn (dùng textarea + preview thay vì WYSIWYG).

---

## 3. MÔ HÌNH DỮ LIỆU

### 3.1 SQLite Schema (migration chỉ tiến, ID = UUID v7)

```
── Core ──────────────────────────────────
profile             id, display_name, created_at
setting             key, value_json, updated_at

── AI ────────────────────────────────────
agent               id, slug, name, model_id, system_prompt,
                    config_json, capabilities_json, budget_json,
                    version, created_at, updated_at, archived
model_provider      id, name, kind(cloud|local), base_url,
                    config_json, active

── Conversation ──────────────────────────
conversation        id, title, agent_id, created_at, updated_at, archived
message             id, conversation_id, run_id, role(user|assistant|system|tool),
                    parts_json, created_at
                    -- parts: Text | ToolCall | ToolResult | FileRef
                    -- append-only, không sửa/xoá

── Execution ─────────────────────────────
run                 id, conversation_id, agent_id, parent_run_id,
                    status(queued|running|waiting_approval|completed|failed|cancelled),
                    usage_json, cost_usd, started_at, finished_at
step                id, run_id, idx, kind(llm|tool|approval),
                    input_json, output_json, status, duration_ms, created_at

── Workspace ─────────────────────────────
document            id, path, title, tags_json, agent_id, run_id,
                    content_hash, size_bytes, created_at, updated_at

── Security ──────────────────────────────
audit_log           id, timestamp, run_id, tool_name, args_digest,
                    decision(allow|deny|approval), result_digest

── Search ────────────────────────────────
messages_fts        FTS5 virtual table (BM25)
documents_fts       FTS5 virtual table (BM25)
```

### 3.2 Workspace (Filesystem)

```
Workspace/ (mặc định: Documents\Hubbub\Workspace\)
├── agents/          # Agent definitions (agent.toml + prompt.md)
├── reports/         # Báo cáo do Agent tạo
├── notes/           # Ghi chú của user hoặc Agent
├── sources/         # Tài liệu nguồn đã import
└── .versions/       # Lưu 20 bản cũ mỗi file (auto-versioning)

App Data: %APPDATA%\Hubbub\
├── config.toml      # Cấu hình app + model providers
├── app.db           # SQLite database
├── logs/            # Rotating log files (3 × 5MB)
└── backups/         # Auto backup SQLite
```

### 3.3 Agent Definition (file-based, sửa không cần compile)

```toml
# Workspace/agents/researcher/agent.toml
id = "researcher"
name = "Research Agent"
model = "anthropic/claude-sonnet-4-20250514"

[tools]
builtin = ["web_search", "web_fetch", "report_write", "report_read"]
mcp = []

[permissions]
fs_read = []                        # Không đọc file cục bộ
fs_write = ["reports/**"]           # Chỉ ghi vào reports/
network = "open"                    # none | search_only | allowlist | open

[budget]
max_steps = 30
max_tokens = 200_000
max_cost_usd = 1.0
timeout_s = 600

[delegation]
can_delegate_to = []                # Không delegate
```

Kèm `Workspace/agents/researcher/prompt.md` — system prompt dạng Markdown.

**Validation khi nạp**: `fs_read` ≠ rỗng AND `network = "open"` → **TỪ CHỐI** (chống rò rỉ dữ liệu cục bộ ra internet).

---

## 4. MÔ HÌNH BẢO MẬT

### 4.1 Nguyên tắc

```
Least Privilege + Explicit Capability + Human-in-the-loop
```

Agent KHÔNG ĐƯỢC:
- Toàn quyền filesystem
- Toàn quyền network
- Truy cập shell
- Đọc secrets
- Tự sửa policy/permission của chính nó
- Ghi vào `agents/`, `config.toml`, hay bất kỳ nơi chứa định nghĩa quyền

### 4.2 Tool Execution Flow

```
Agent request tool_call
       ↓
Schema validation (input matches tool schema?)
       ↓
Policy Engine check
  ├── Capability check (agent có tool này không?)
  ├── Scope check (path/url trong scope cho phép?)
  └── Risk level check
       ↓
Approval (nếu cần, tuỳ risk level)
  ├── auto           → thực thi ngay
  ├── ask_once       → hỏi 1 lần/run
  └── always_ask     → hỏi mỗi lần
       ↓
Execute trong sandbox/worker
       ↓
Ghi Audit Log
       ↓
Return Result (đã sanitize)
```

### 4.3 PathGuard

- `canonicalize()` mọi đường dẫn trước khi thao tác.
- Chặn: `../`, symlink escape, UNC path, ADS (NTFS alternate data streams), Windows device names (CON, PRN, NUL...).
- Tool chỉ ghi được: `reports/**`, `notes/**`.
- `agents/`, `config.toml` **bất khả xâm phạm** với mọi tool.
- Property test (proptest) cho PathGuard.

### 4.4 UrlGuard (SSRF Protection)

- Chỉ cho phép `http://` và `https://`.
- Resolve DNS trước, kiểm IP: chặn loopback, private ranges, link-local, `169.254.169.254`, IPv6 tương đương.
- Kiểm tra lại mỗi redirect (tối đa 5 hops).
- Giới hạn: response ≤ 5MB, timeout 30s.
- Content-type validation.

### 4.5 Untrusted Content

- Mọi nội dung từ web/file/PDF = **untrusted**.
- Bọc trong thẻ `<untrusted>...</untrusted>` + system prompt nhắc rõ.
- Test bằng **FakeLlm** "ngoan ngoãn làm theo injection" để chứng minh policy chặn được, **độc lập với LLM**.

### 4.6 Secrets

- Lưu qua OS Keyring (Windows Credential Manager).
- UI chỉ **ghi**, không đọc lại giá trị.
- Không bao giờ xuất hiện trong: log, context LLM, audit, error messages.

### 4.7 MCP Security

- Chỉ chạy server khai báo trong `config.toml`, pin version/hash.
- Mỗi MCP server: cwd riêng, lọc biến môi trường.
- Cấm `npx -y` không pin version.
- MCP tool lần đầu = `always_ask`.
- MCP KHÔNG được bypass Policy Engine.

---

## 5. AGENT SYSTEM

### 5.1 Agent Runtime Loop

```
1) Dựng context:
   - System prompt (từ prompt.md)
   - Lịch sử đã nén (context compression cho hội thoại dài)
   - Tool schemas (chỉ những tool agent có quyền)
   - Ngày giờ hiện tại + ngôn ngữ trả lời

2) Gọi LlmProvider (stream) → phát RunEvent::MessageDelta

3) Nhận tool_call:
   → policy.check()
   → (hỏi phê duyệt nếu cần)
   → thực thi tool
   → ghi Step (TRƯỚC và SAU khi thực thi)
   → kết quả trả về LLM → quay lại bước 2

4) Dừng khi:
   - Hết tool_call (LLM trả text thuần)
   - Hết budget (steps/tokens/cost/time)
   - Bị huỷ (CancellationToken)
   - Lỗi không phục hồi được
```

### 5.2 Context Management (chất lượng agent)

1. **Nén lịch sử cũ** thành tóm tắt khi vượt ngưỡng token.
2. **Cắt tool result quá dài** + gợi ý "đọc tiếp theo offset".
3. **Không nhét nguyên trang web/PDF** vào context — chỉ đưa phần relevant.
4. **Phần đầu prompt ổn định** (tận dụng prompt caching của provider).
5. **Lỗi tool trả về dạng có cấu trúc** để model tự sửa; retry có giới hạn.

### 5.3 Multi-Agent Orchestration

```
Mặc định: User chat trực tiếp với 1 agent.

Delegation pattern:
  User
    ↓
  Orchestrator Agent (có can_delegate_to)
    ↓ delegate(agent_id, task) → tạo run con
    ├── Researcher (web search, fetch)
    ├── Librarian (đọc thư mục cục bộ, network = none)
    └── Writer (tổng hợp, viết báo cáo)
    ↓
  Kết quả tổng hợp → Markdown Report

Ràng buộc:
- Độ sâu delegation ≤ 2
- Budget con trừ từ budget cha
- Events con lồng vào run cha qua parent_run_id
- Tối đa 4 run đồng thời; semaphore riêng cho từng provider
```

### 5.4 Built-in Tools

| Tool | Risk | Approval | Mô tả |
|---|---|---|---|
| `web_search` | LOW | auto | Tìm kiếm web qua SearchProvider |
| `web_fetch` | MEDIUM | auto | Fetch URL, trả về Markdown (sanitized) |
| `fs_read` | MEDIUM | auto | Đọc file trong scope cho phép |
| `fs_list` | LOW | auto | Liệt kê thư mục trong scope |
| `report_write` | HIGH | auto | Ghi file Markdown vào reports/ hoặc notes/ |
| `report_read` | LOW | auto | Đọc file Markdown trong Workspace |
| `report_list` | LOW | auto | Liệt kê reports |
| `delegate` | MEDIUM | ask_once | Giao task cho agent khác |
| `delete_file` | CRITICAL | always_ask | Xoá file — luôn hỏi user |

### 5.5 Agent Quality Harness (8 hạng mục)

1. **Tool schema chất lượng**: tên nhất quán, mô tả rõ khi nào dùng/không dùng, enum thay chuỗi tự do.
2. **Context management**: nén, cắt, không nhồi nguyên tài liệu.
3. **Structured error**: lỗi tool có cấu trúc để model tự sửa; retry có giới hạn; không nuốt lỗi.
4. **Citation bắt buộc**: báo cáo phải có trích dẫn nguồn; workflow deep_research có bước verify.
5. **Sub-agent context sạch**: mỗi sub-agent chạy với context riêng, trả về tóm tắt gọn.
6. **Prompt caching**: phần đầu prompt ổn định; luôn cấp ngày giờ + ngôn ngữ.
7. **Run inspector**: hiển thị context đã gửi, tool calls, tokens, chi phí — để debug chất lượng.
8. **Eval framework**: 5-10 golden tasks + checklist chấm điểm; chạy lại khi đổi model/prompt/tool.

---

## 6. CẤU TRÚC CODE BASE

```
hubbub/
├── Cargo.toml                        # Rust workspace
├── rust-toolchain.toml                # Pin Rust version
├── rustfmt.toml                      # Format config
├── deny.toml                         # cargo-deny config
├── justfile                          # Task runner (just)
├── .github/workflows/ci.yml          # GitHub Actions CI
│
├── docs/                             # Tài liệu dự án
│   ├── MASTER_PLAN.md                # Bản kế hoạch này
│   ├── DECISION_LOG.md               # Nhật ký quyết định (BẮT BUỘC cập nhật)
│   ├── ARCHITECTURE.md               # Kiến trúc chi tiết + diagrams
│   ├── TESTING_GUIDE.md              # Hướng dẫn viết test (quan trọng)
│   ├── AI_WORKFLOW.md                # Quy trình AI viết code + review checklist
│   ├── THREAT_MODEL.md               # Mô hình đe doạ
│   ├── RUNBOOK.md                    # Hướng dẫn vận hành, debug, backup/restore
│   └── adr/                          # Architecture Decision Records
│       ├── 0001-tauri-react.md
│       ├── 0002-rust-core.md
│       ├── 0003-sqlite-fts5.md
│       ├── 0004-custom-agent-runtime.md
│       ├── 0005-capability-security.md
│       ├── 0006-workspace-markdown.md
│       ├── 0007-milkdown-editor.md
│       └── ...
│
├── scripts/                          # CI & dev scripts
│   ├── check-deps.sh                # Kiểm tra dependency rules
│   ├── check-imports.sh             # Kiểm tra UI không import I/O trực tiếp
│   └── gen-icons.sh                 # Generate app icons
│
├── crates/                           # Rust workspace members
│   ├── domain/                       # Entity, Value Object, Ports (trait), Error
│   │   └── src/
│   │       ├── entities/             # Conversation, Message, Agent, Run, Step, Document...
│   │       ├── ports/                # LlmProvider, ToolHost, Store, Workspace, SecretStore,
│   │       │                         # EventSink, SearchProvider, Clock
│   │       ├── events/               # RunEvent, DocumentEvent...
│   │       └── errors/               # Domain errors (thiserror)
│   │
│   ├── policy/                       # Security: PathGuard, UrlGuard, PermissionChecker,
│   │   └── src/                      # ApprovalManager, AgentValidator
│   │
│   ├── agent/                        # Runtime loop, Orchestrator, Context builder/compressor
│   │   └── src/
│   │
│   ├── llm/                          # LLM adapters: openai_compat, anthropic, ollama, sse, retry
│   │   └── src/
│   │
│   ├── tools/                        # Built-in tools: web_search, web_fetch, fs_read,
│   │   └── src/                      # report_write, report_read, delegate, todo
│   │
│   ├── mcp/                          # MCP client: registry, spawn, env isolation
│   │   └── src/
│   │
│   ├── store/                        # SQLite repos, migrations, FTS
│   │   └── src/
│   │   └── migrations/
│   │
│   ├── workspace/                    # Markdown service: atomic write, front matter,
│   │   └── src/                      # versioning, file watcher, indexer
│   │
│   ├── vault/                        # SecretStore implementation (keyring)
│   │   └── src/
│   │
│   ├── app/                          # Composition root, use-case facades, config loader, backup
│   │   └── src/
│   │
│   └── testkit/                      # Shared test utilities
│       └── src/                      # FakeLlm, record/replay cassettes, tmp workspace, fixtures
│
├── src-tauri/                        # Tauri 2 shell (Rust: commands, lifecycle)
│   ├── src/
│   │   ├── commands/                 # IPC command handlers
│   │   └── main.rs                   # App entry
│   ├── tauri.conf.json
│   ├── capabilities/                 # Tauri permission configs
│   └── Cargo.toml
│
├── src/                              # React UI
│   ├── app/                          # Router, theme, providers, shell layout
│   ├── components/
│   │   ├── layout/                   # Shell, Sidebar, SplitView
│   │   ├── chat/                     # MessageList, MessageBubble, Composer, StreamRenderer
│   │   ├── agents/                   # AgentList, AgentConfig, AgentCard
│   │   ├── files/                    # FileTree, MilkdownEditor, BreadCrumb, SearchBar
│   │   ├── runs/                     # RunInspector, StepTimeline, TokenUsage
│   │   └── settings/                 # ProviderConfig, WorkspaceConfig, SecurityConfig
│   ├── hooks/                        # useChatStream, useAgents, useDocuments, useRuns
│   ├── stores/                       # Zustand stores
│   ├── services/                     # Tauri IPC wrappers
│   └── types/                        # TypeScript types (mirror Rust domain entities)
│
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts
│
└── resources/
    └── agents/                       # Agent mặc định, seed vào Workspace lần chạy đầu
        ├── researcher/
        ├── librarian/
        ├── tutor/
        └── orchestrator/
```

---

## 7. TÀI LIỆU DỰ ÁN

### 7.1 Danh sách tài liệu bắt buộc

| File | Mục đích | Khi nào cập nhật |
|---|---|---|
| `MASTER_PLAN.md` | Kế hoạch tổng thể (file này) | Khi thay đổi scope/strategy lớn |
| `DECISION_LOG.md` | **Nhật ký quyết định** | **BẮT BUỘC** khi có quyết định/thay đổi quan trọng |
| `ARCHITECTURE.md` | Kiến trúc chi tiết + diagrams | Khi thay đổi kiến trúc |
| `TESTING_GUIDE.md` | Hướng dẫn viết test | Khi thêm quy tắc test mới |
| `AI_WORKFLOW.md` | Quy trình AI viết code + review | Khi thay đổi quy trình |
| `THREAT_MODEL.md` | Mô hình đe doạ | Khi thêm attack surface mới |
| `RUNBOOK.md` | Vận hành, debug, backup | Khi thêm quy trình vận hành |
| `adr/XXXX-*.md` | Architecture Decision Records | Mỗi quyết định kiến trúc lớn |

### 7.2 DECISION_LOG.md — Format

```markdown
# Decision Log

Mọi quyết định quan trọng và thay đổi lớn trong quá trình phát triển
PHẢI được ghi lại tại đây. Sắp xếp theo thứ tự thời gian (mới nhất ở trên).

## [YYYY-MM-DD] Tiêu đề quyết định

**Bối cảnh:** Tại sao cần quyết định này?
**Quyết định:** Chọn phương án nào?
**Lý do:** Tại sao chọn phương án này? Đã cân nhắc gì?
**Hệ quả:** Quyết định này ảnh hưởng gì đến hệ thống?
**Trạng thái:** Đã áp dụng / Đang thử nghiệm / Đã huỷ
```

**QUY TẮC BẮT BUỘC:**
- Mọi Dev hoặc Agent (AI) **PHẢI** cập nhật file này khi:
  - Thay đổi tech stack / dependency quan trọng
  - Thay đổi kiến trúc hoặc data model
  - Thay đổi security policy
  - Fix bug quan trọng (ghi rõ nguyên nhân gốc + cách fix)
  - Thay đổi quy trình phát triển
  - Quyết định loại bỏ hoặc thêm tính năng
- PR/commit thay đổi quan trọng mà KHÔNG cập nhật DECISION_LOG.md sẽ bị reject.

### 7.3 Yêu cầu tài liệu cho AI/Dev mới

Mọi tài liệu phải đảm bảo:
- **Cấu trúc rõ ràng**: Heading, bullet points, bảng, diagrams.
- **Self-contained**: Đọc 1 file hiểu được scope của nó mà không cần đọc file khác.
- **Có ví dụ cụ thể**: Không chỉ lý thuyết, phải có code snippet/command ví dụ.
- **Cập nhật**: Ghi rõ ngày cập nhật cuối. Tài liệu lỗi thời nguy hiểm hơn không có tài liệu.

---

## 8. CHIẾN LƯỢC TESTING

### 8.1 Nguyên tắc Test-First

```
1. Viết spec ngắn (input/output/behavior mong đợi)
2. Viết test cases (TRƯỚC code) — đặc biệt common edge cases
3. Viết code implementation
4. CI chạy toàn bộ test
5. Review: test có cover đủ common edge cases không?
```

### 8.2 Quy tắc edge case — QUAN TRỌNG

> **VẤN ĐỀ THƯỜNG GẶP:** AI có xu hướng viết test cho edge cases hiếm gặp
> (VD: chuỗi 10 triệu ký tự, Unicode cổ đại) nhưng bỏ sót các edge cases
> phổ biến mà user thực sự gặp hàng ngày.

**Quy tắc: Ưu tiên viết test cho COMMON edge cases trước, RARE edge cases sau.**

Danh sách common edge cases theo domain (bắt buộc có test):

#### Chat / Messages
- [ ] Tin nhắn rỗng (chỉ khoảng trắng)
- [ ] Tin nhắn chứa tiếng Việt có dấu (ê, ơ, ư, ắ, ồ...)
- [ ] Tin nhắn chứa emoji 🎉 và Unicode symbols
- [ ] Tin nhắn dài (>5KB) — không bị cắt, UI không vỡ
- [ ] Gửi tin liên tiếp nhanh (debounce)
- [ ] Gửi tin khi stream trước chưa xong
- [ ] Huỷ stream giữa chừng — state phải consistent

#### File / Path Operations
- [ ] Đường dẫn có dấu cách: `D:\My Documents\report.md`
- [ ] Đường dẫn có tiếng Việt: `D:\Tài liệu\báo cáo.md`
- [ ] File không tồn tại → lỗi rõ ràng (không panic)
- [ ] File bị khoá bởi process khác → retry hoặc lỗi rõ
- [ ] File > giới hạn cho phép → từ chối sớm, thông báo rõ
- [ ] Ghi file khi đĩa đầy → lỗi rõ ràng
- [ ] Đọc/ghi concurrent cùng file → không corrupt data
- [ ] Path traversal: `../../etc/passwd`, `..\..\Windows\System32`
- [ ] Symlink trỏ ra ngoài workspace

#### Agent / LLM
- [ ] LLM trả về JSON tool_call không hợp lệ → parse error, retry
- [ ] LLM timeout (30s+) → cancel, thông báo user
- [ ] LLM trả response rỗng → xử lý gracefully
- [ ] LLM trả tool_call với argument sai type → validation error, retry
- [ ] Network mất giữa stream → reconnect hoặc thông báo
- [ ] Budget hết giữa run → dừng sạch, thông báo user
- [ ] LLM trả tool_call cho tool không có trong schema → reject

#### Security
- [ ] Path traversal qua `../`
- [ ] Symlink escape
- [ ] URL trỏ tới localhost / private IP (SSRF)
- [ ] Response từ web > 5MB → cắt
- [ ] Prompt injection trong nội dung web → policy chặn
- [ ] Agent cố ghi ra ngoài scope → deny + audit log

#### Database
- [ ] Insert/update với chuỗi rỗng
- [ ] Dữ liệu tiếng Việt trong FTS5 (tìm kiếm có dấu)
- [ ] Concurrent reads khi đang write (WAL mode)
- [ ] Migration trên DB có data cũ

### 8.3 Cấu trúc Test

```
── Unit Tests (mỗi crate) ──────────────────────
  Domain: entity validation, port contracts
  Policy: PathGuard (proptest), UrlGuard (proptest), permission logic
  Agent: state transitions, budget enforcement, context compression
  Store: repository CRUD, FTS queries, migration

── Integration Tests ───────────────────────────
  Agent + FakeLlm + Tools: full run loop
  Store + SQLite: real DB operations
  Workspace + filesystem: atomic write, watcher, versioning
  MCP: spawn + tool call + teardown

── Security Tests ──────────────────────────────
  Prompt injection (FakeLlm tuân theo injection → policy vẫn chặn)
  Path traversal (property test: mọi path ngoài scope đều bị reject)
  SSRF (property test: mọi private IP đều bị chặn)
  Tool abuse (agent cố vượt budget/scope)

── E2E Tests ───────────────────────────────────
  Chat flow: gửi tin → stream → hiển thị
  Agent run: chat → tool call → approval → result
  File flow: agent tạo report → UI hiển thị → user sửa → save
  Backup/restore: backup → delete DB → restore → verify data
```

### 8.4 Test Tools

| Rust | TypeScript |
|---|---|
| `cargo-nextest` (test runner) | `vitest` (unit/component) |
| `wiremock` (mock HTTP/LLM) | `testing-library/react` |
| `insta` (snapshot test) | `playwright` (E2E nếu cần) |
| `proptest` (property test cho guards) | |
| `testkit` crate (FakeLlm, fixtures) | |

---

## 9. QUY TRÌNH PHÁT TRIỂN

### 9.1 Contract-First Workflow (AI viết code, Human review)

```
1. Viết spec ngắn:  Input / Output / Behavior / Error cases
2. Viết trait/port:  Interface trước, implementation sau
3. Viết test cases:  Common edge cases TRƯỚC (xem mục 8.2)
4. AI implement:     Theo spec + trait + test
5. CI chạy:          Format → Lint → Typecheck → Unit test → Integration test
6. Review:           Theo checklist (xem 9.3)
7. Merge
8. Cập nhật:         DECISION_LOG.md nếu có thay đổi quan trọng
```

### 9.2 Quy tắc code

**Rust:**
- `forbid(unsafe_code)` (trừ code sinh tự động)
- `deny`: unwrap_used, expect_used, panic, todo, dbg_macro, await_holding_lock
- `clippy -D warnings`
- File < 400 dòng, hàm < 60 dòng
- Owned types + `Arc` cho shared state, trait object ở ports
- `async-trait` khi cần
- Tránh: lifetime trên struct công khai, macro tự viết, generic quá sâu
- Error handling: `thiserror` cho domain errors, `anyhow` chỉ ở application boundary

**TypeScript:**
- `strict: true` trong tsconfig
- ESLint + Prettier
- Không `any`, không `as` type assertion (trừ trường hợp có comment giải thích)
- Components < 200 dòng

### 9.3 Review Checklist

- [ ] Có unwrap/expect? → Phải thay bằng proper error handling
- [ ] Giữ lock qua .await? → Deadlock risk
- [ ] Tác vụ nền có CancellationToken? → Phải có để graceful shutdown
- [ ] Channel/queue có bounded? → Phải có giới hạn
- [ ] Lỗi có context đầy đủ? → User phải hiểu lỗi gì xảy ra
- [ ] Đường dẫn/URL có đi qua guard? → Tất cả phải qua PathGuard/UrlGuard
- [ ] Log có lộ secret/prompt? → Tuyệt đối không
- [ ] Có test cho nhánh lỗi? → Phải test cả happy path và error path
- [ ] Common edge cases đã có test? → Xem danh sách mục 8.2
- [ ] DECISION_LOG.md đã cập nhật? → Nếu có thay đổi quan trọng

### 9.4 Quy tắc ưu tiên công việc

```
1. Fix bug / vấn đề tiềm ẩn    ← LUÔN LÀM TRƯỚC
2. Fix failing tests
3. Refactor technical debt ảnh hưởng tính năng tiếp theo
4. Phát triển tính năng mới     ← CHỈ LÀM KHI 1-3 XÓA SẠCH
```

Khi phát hiện bug trong quá trình phát triển tính năng mới:
- DỪNG tính năng mới
- Tạo issue/ghi chú cho bug
- Fix bug + viết test regression
- Quay lại tính năng mới

---

## 10. CÁC PHASE TRIỂN KHAI

### Phase 0: Nền móng + Gate 0 (~1-2 tuần)

**Mục tiêu:** Thiết lập infrastructure, validate tech stack.

```
[ ] Thiết lập repository, workspace structure, CI (GitHub Actions Windows)
[ ] ADR 0001-0007 (mỗi quyết định tech stack lớn)
[ ] DECISION_LOG.md khởi tạo
[ ] THREAT_MODEL.md v0
[ ] AI_WORKFLOW.md + review checklist
[ ] TESTING_GUIDE.md + common edge cases list
[ ] Workspace lints (Rust + TypeScript), deny.toml, rustfmt.toml

[ ] GATE 0 — Validate Tauri + WebView2 (pass/fail, 2-3 ngày):
    1) Tauri 2 + React app cơ bản trên Windows
    2) Gõ tiếng Việt (UniKey/EVKey/Telex): gõ nhanh, sửa giữa chuỗi,
       undo, dán — không mất/lặp/sai dấu
    3) Render 1.000 tin nhắn Markdown (có bảng, code, danh sách):
       cuộn mượt, chọn/copy text ổn
    4) Đo RAM: Private Working Set ≤ 180MB với 1.000 tin nhắn
    5) Stream simulation 50 token/s: UI không giật
    6) Milkdown editor: tạo/sửa Markdown với GFM + code blocks
```

**Definition of Done:**
- CI xanh (format + lint + typecheck)
- Gate 0 kết quả ghi vào ADR-0001
- Nếu Gate 0 đạt → chốt Tauri + React, tiếp Phase 1
- Nếu Gate 0 trượt RAM → chuyển Flutter, tạo ADR mới, Rust core giữ nguyên

---

### Phase 1: Chat MVP (~3-4 tuần)

**Mục tiêu:** Chat ổn định với nhiều agent, nhiều provider.

```
[ ] domain crate: entities (Conversation, Message, Agent, Run, Step), ports, errors
[ ] store crate: SQLite setup, WAL, migrations, CRUD repos, test với real DB
[ ] llm crate: openai_compat + anthropic adapters (stream, tool-call, usage)
    - Test bằng wiremock (mock HTTP)
    - Test: timeout, invalid JSON, empty response, network error
[ ] agent crate v1: runtime loop, budget enforcement, cancellation
    - Test bằng FakeLlm (testkit)
    - Test: budget exceeded, cancel mid-run, tool-call retry
[ ] vault crate: keyring integration
[ ] app crate: composition root, config loader
[ ] Tauri shell: IPC commands, app lifecycle
[ ] React UI:
    - Shell layout (sidebar + main content)
    - Chat view: MessageList (virtualized), Composer, StreamRenderer
    - Conversation list (paginated)
    - Agent selector
    - Settings: Provider config, API key input (write-only)
    - Markdown rendering trong chat messages
```

**Definition of Done:**
- Chat ổn định với ≥ 2 provider (VD: Anthropic + OpenAI-compat)
- Chạy với ≥ 2 agent khác nhau (chuyển đổi được)
- 1.000 tin nhắn: cuộn mượt, RAM đạt mục tiêu
- Stream/cancel/retry hoạt động
- Kill app giữa chừng không hỏng dữ liệu
- All tests pass, CI xanh
- DECISION_LOG.md cập nhật

---

### Phase 2: Web Research + Báo cáo Markdown (~3-4 tuần)

**Mục tiêu:** Agent tìm kiếm web, tạo báo cáo Markdown có trích dẫn.

```
[ ] policy crate: PathGuard, UrlGuard, PermissionChecker, ApprovalManager
    - Property test (proptest) cho PathGuard + UrlGuard
    - Test SSRF, path traversal, symlink escape
[ ] tools crate: web_search, web_fetch
    - UrlGuard integration
    - Test: SSRF, timeout, oversized response, invalid content-type
[ ] tools crate: report_write, report_read, report_list
    - PathGuard integration
    - Atomic write (tmp → fsync → rename)
    - Test: path traversal, concurrent write, disk full
[ ] workspace crate: file watcher, auto-versioning (.versions/), FTS5 indexing
    - Test: sửa file ngoài app → reindex, self-write ignore
[ ] Audit log: ghi mọi tool call
[ ] UI:
    - Approval dialog (khi tool cần phê duyệt)
    - Files view: FileTree, Breadcrumb, Milkdown Editor
    - Search trong Workspace (FTS5)
    - "Save as report" action
[ ] Security tests:
    - FakeLlm injection test (web content chứa injection → policy chặn)
    - Path traversal battery test
    - SSRF battery test
```

**Definition of Done:**
- Researcher agent tạo báo cáo có trích dẫn nguồn
- Báo cáo lưu vào Workspace, hiển thị + chỉnh sửa được trong app
- Sửa file ngoài app → app tự reindex
- Toàn bộ injection/traversal/SSRF test pass
- DECISION_LOG.md cập nhật

---

### Phase 3: Thư mục cục bộ + MCP + Multi-Agent (~4-5 tuần)

**Mục tiêu:** Agent đọc file cục bộ, MCP tools, orchestration nhiều agent.

```
[ ] tools crate: fs_read, fs_list, fs_search
    - UI quản lý "allowed directories" (sources)
    - PathGuard enforce scope
    - PDF → text extraction (pdfium-render)
[ ] mcp crate: MCP client via rmcp
    - Registry, spawn, env isolation
    - Pin version/hash
    - Test: spawn → call → teardown, malicious server
[ ] agent crate: orchestrator, delegate tool
    - Run lồng nhau (parent_run_id), budget cascading
    - Test: delegation chain, budget exceeded in child
[ ] UI:
    - Run Inspector: timeline, steps, tool calls, tokens, cost
    - Agent management: create/edit/archive
    - Source management: add/remove allowed directories
[ ] Workflow: deep_research (plan → search → read → synthesize → write → verify)
    - Test end-to-end với FakeLlm
[ ] Agent quality eval: 5-10 golden tasks + checklist
```

**Definition of Done:**
- Orchestrator delegate task cho Researcher + Librarian thành công
- Run Inspector hiển thị cây run đầy đủ
- MCP server (VD: filesystem MCP) hoạt động trong sandbox
- Eval golden tasks có điểm số cơ sở
- DECISION_LOG.md cập nhật

---

### Phase 4: Hardening & Release (~2-3 tuần)

**Mục tiêu:** Ổn định, bảo mật, sẵn sàng sử dụng hàng ngày.

```
[ ] Integration test smoke suite
[ ] Benchmark: cold start, RAM, chat dài, file operations
[ ] Rà soát bảo mật theo THREAT_MODEL.md
    - Prompt injection test suite
    - Path traversal property test
    - SSRF property test
    - Tool abuse test
    - Secret leakage test
[ ] Backup:
    - Auto backup SQLite (VACUUM INTO), giữ 14 bản
    - Backup trước mỗi DB migration
    - Test restore flow
[ ] Đóng gói: Tauri build Windows (MSI/EXE installer)
[ ] Rotating log (3 × 5MB)
[ ] Cost tracking: usage mỗi run, trần chi phí theo run + tháng
[ ] Documentation review: tất cả docs cập nhật
[ ] Dùng thật 2 tuần — ghi bug, fix, regression test
```

**Definition of Done:**
- Dùng thật 2 tuần không mất dữ liệu
- Toàn bộ security test pass
- Backup + restore verified
- RAM/cold start đạt mục tiêu
- CI xanh (clippy -D warnings, cargo-deny)
- DECISION_LOG.md, THREAT_MODEL.md, RUNBOOK.md cập nhật

---

### Phase 5+ (Tương lai — Backlog)

```
[ ] Android:
    - Đánh giá topology tại thời điểm cần:
      (A) Flutter app + Rust core via flutter_rust_bridge (nhúng .so)
      (B) Headless server (axum) trên desktop + mobile client qua WireGuard/Tailscale
    - Reuse toàn bộ Rust core, chỉ thay UI layer

[ ] Voice (v2 nếu cần):
    - STT: whisper.cpp local
    - TTS: Native OS hoặc Edge-TTS
    - Push-to-talk trước, continuous sau

[ ] Advanced features:
    - Embedding + sqlite-vec (semantic search)
    - LaTeX rendering (KaTeX)
    - Zotero / arXiv integration
    - Flashcards (FSRS algorithm)
    - Agent scheduled tasks
    - Memory system (conversation/project/explicit)
```

---

## 11. QUẢN LÝ RỦI RO

| # | Rủi ro | Xác suất | Tác động | Biện pháp |
|---|---|:---:|:---:|---|
| 1 | WebView2 RAM quá cao | Trung bình | Cao | Gate 0 validate + fallback Flutter |
| 2 | Gõ tiếng Việt lỗi trên WebView2 | Thấp | Cao | Gate 0 test kỹ UniKey/EVKey/Telex |
| 3 | Prompt injection bypass policy | Trung bình | Cao | Policy ở Rust (không dựa vào LLM) + FakeLlm test + audit log |
| 4 | Chất lượng agent output kém | Cao | Cao | 8 hạng mục harness + eval golden tasks + Run inspector debug |
| 5 | Scope phình | Cao | Trung bình | Mỗi phase có "Definition of Done" rõ ràng; không làm ngoài scope |
| 6 | rmcp (MCP SDK) còn trẻ | Trung bình | Thấp | Pin version, bọc sau ToolHost trait, test với mock server |
| 7 | AI viết code sai edge cases | Cao | Trung bình | Test-first, common edge case list bắt buộc, review checklist |
| 8 | Phụ thuộc 1 LLM provider | Thấp | Cao | Trait LlmProvider + ≥ 2 adapter + trần chi phí |

---

## 12. HIỆU NĂNG & VẬN HÀNH

### Mục tiêu hiệu năng
- Cold start: < 3 giây
- Idle RAM (1.000 tin nhắn): ≤ 180MB
- First-token latency: < 1s (không tính LLM latency)
- UI frame rate: > 30fps khi cuộn/stream
- DB query: < 50ms cho CRUD thông thường

### Vận hành
- **Process lifecycle**: Graceful shutdown — cancel running agents, persist state, cleanup.
- **Backup**: SQLite `VACUUM INTO` → file backup, giữ 14 bản gần nhất.
- **Logs**: Rotating file (3 × 5MB), tại `%APPDATA%\Hubbub\logs\`.
- **Workspace data**: Plain Markdown — user có thể sync qua Git/OneDrive/Syncthing.
- **Nâng cấp**: Dữ liệu tách khỏi thư mục cài → cài bản mới an toàn. Rollback = cài lại bản cũ + restore backup.
- **Cost tracking**: Ghi usage mỗi run, bảng giá model trong config.toml, trần chi phí theo run + tháng.

---

## 13. CÁC ADR CẦN TẠO

| ADR | Chủ đề |
|---|---|
| ADR-0001 | Tauri 2 + React (hoặc Flutter nếu Gate 0 trượt) |
| ADR-0002 | Rust Core — platform-neutral business logic |
| ADR-0003 | SQLite + FTS5 — database strategy |
| ADR-0004 | Custom Agent Runtime (không dùng LangGraph/LangChain) |
| ADR-0005 | Capability-based Security Model |
| ADR-0006 | Workspace Markdown as Source of Truth |
| ADR-0007 | Milkdown Editor (hoặc alternative) |
| ADR-0008 | MCP Integration Strategy |
| ADR-0009 | No Voice in v1 |
| ADR-0010 | Test-First + Common Edge Cases Strategy |
| ADR-0011 | Decision Log Requirement |
| ADR-0012 | Agent as TOML Files |
| ADR-0013 | Local-first Data Strategy |
| ADR-0014 | Desktop → Android Strategy |

---

*Kế hoạch này kế thừa và tổng hợp từ Plan 01, Plan 02, Plan 03 — lấy kiến trúc sâu của Plan 02,
tính thực tế và Gate 0 của Plan 03, loại bỏ những gì không phù hợp từ Plan 01.*

*Mọi thay đổi so với kế hoạch này phải được ghi vào `DECISION_LOG.md`.*
