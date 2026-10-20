# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Hubbub is a Windows desktop multi-agent chat app: **Tauri 2** shell, **React 19 + TypeScript + Tailwind v4 + Vite** frontend (`src/`), and a **Rust workspace** core (`crates/*`, `src-tauri/`). Most docs (`README.md`, `docs/`) and user-facing error strings are in Vietnamese. The target platform is Windows x64 (MSVC toolchain); CI runs on `windows-latest`.

## Commands

`justfile` wraps the common tasks:

| Task | Command |
| --- | --- |
| Run the desktop app (dev) | `just dev` (`npm run tauri dev`) |
| Run the frontend only, without Tauri | `just dev-ui` (`npm run dev`) |
| Run all Rust tests | `just test` (nextest) or `just test-cargo` (`cargo test --workspace`) |
| Run Clippy | `just lint` (`cargo clippy --workspace --all-targets -- -D warnings`) |
| Format / check formatting | `just fmt` / `just fmt-check` |
| Type-check, lint and build the frontend | `just lint-ui` (`npx tsc -b`, `npm run lint`, `npx vite build`) |
| Lint the frontend | `npm run lint` (oxlint) |
| Run everything CI runs | `just ci` (fmt-check, clippy, cargo test, lint-ui) |
| Build release bundles | `just build` (output in `target/release/bundle/{msi,nsis}/`) |
| Check licenses and advisories | `just deny` |

To run one crate's tests or a single test:

```bash
cargo test -p hubbub-agent                                    # one crate
cargo test -p hubbub-policy --test policy_tests               # one integration test file
cargo test -p hubbub-agent --test runtime_tests some_test_name  # one test
```

Crate package names use the `hubbub-` prefix (`hubbub-domain`, `hubbub-store`, …). CI installs with `npm ci` against `package-lock.json`. A `pnpm-lock.yaml` also exists, but npm is the package manager CI uses.

## Lint rules that will fail CI

These are set in the workspace `Cargo.toml` and apply to every crate:

- `unsafe_code = forbid`.
- `unwrap_used`, `expect_used`, `panic`, `todo`, `dbg_macro`, and `await_holding_lock` are all `deny`. Use `?` or `unwrap_or_else` instead, including in non-test code paths. Test files may opt out locally.
- CI sets `RUSTFLAGS=-Dwarnings`, so any warning fails the build.
- `rustfmt.toml` uses `max_width = 100` and edition 2024.

## Architecture

The design is clean/hexagonal. The dependency direction is `domain` ← (`policy`, `agent`, adapters) ← `app` ← `src-tauri`.

- **`domain`**: entities (`Agent`, `Conversation`, `Message`, `Run`/`Step`, `Document`, `AuditLog`, `ModelQuota`), `RunEvent`, and **port traits** in `ports/`: `Store`, `LlmProvider`/`LlmStream`, `ToolHost`, `SecretStore`, `EventSink`, and `WorkspaceService`. It has no I/O dependencies.
- **`domain`, `policy`, and `agent` must not import I/O crates** (`std::fs`, `reqwest`, `sqlx`). All I/O goes through the ports.
- **`policy`**: `PathGuard` (blocks traversal and symlink escape), `UrlGuard` (SSRF, private IPs), and `PermissionChecker` (checks per-agent `fs_read`, `fs_write`, and `network` permissions).
- **`agent`**: `AgentRuntime` runs the loop. It alternates between `LlmRunner` (streams the LLM) and `ToolRunner` (executes tool calls), with `BudgetTracker` enforcing steps, tokens, cost, and timeout, and a `CancellationToken` checked each iteration. `ToolRunner::check_policy` maps each tool name to a policy check and records an audit log entry for every call. **When you add a builtin tool, you also have to add a branch there and list it in `BUILTIN_TOOLS` (same file); unlisted tools are denied at run time and rejected when an agent is saved.**
- **Adapters** implement the ports:
  - `llm`: `AnthropicAdapter`, plus `OpenAiCompatAdapter`, which also serves Gemini, Groq, Ollama, and OpenRouter. Both stream over SSE.
  - `tools`: `BuiltinToolHost` with `fs_*`, `web_*`, and `report_*`.
  - `store`: sqlx SQLite in WAL mode with FTS5. Queries are runtime `sqlx::query(...)` calls, not compile-time macros, so no `DATABASE_URL` is needed.
  - `vault`: OS keyring through `keyring-rs`, plus an in-memory version for tests.
  - `workspace`: local files plus a `notify` watcher.
  - `mcp`: a stub.
- **`app`**: `AppService` is the composition root.
  - `AppService::init` creates the workspace subdirectories (`data`, `reports`, `notes`, `sources`, `.versions`), opens `data/hubbub.db`, and wires up the adapters.
  - `send_message` resolves the agent, **infers the provider from the model-name prefix** (`claude*` → anthropic, `gemini*`, `gpt*`/`o1`/`o3` → openai, llama/qwen/… → ollama or groq), fetches `{provider}_api_key` from the vault, builds the LLM with `ProviderFactory` from the `AppConfig.providers` map, and runs `AgentRuntime`.
- **`testkit`**: `FakeLlm`, mock tools, a capturing event sink, and fixtures. Use these in tests instead of calling real APIs. HTTP-level tests use `wiremock`.
- **`src-tauri`**: a thin shell with no business logic.
  - `commands.rs` holds the `#[tauri::command]` wrappers that call `AppService` and map errors to `String`. Register new commands in `generate_handler!` in `lib.rs`.
  - `TauriEventSink` emits every `RunEvent` on the single `"run_event"` channel. `RunEvent` is serialized with `#[serde(tag = "type", content = "data")]`.

### Agents

The live agent definitions come from `crates/app/src/seed.rs` (`default_agents()`). User edits overlay them and are persisted to `<app_data>/data/agents.json`. The TOML files in `resources/agents/` are design references and are **not loaded at runtime**.

### Frontend

- `src/types.ts` mirrors the Rust entities and `RunEvent` by hand. Keep the two in sync when serialized shapes change.
- IPC calls are `invoke('snake_case_command', { camelCaseArgs })`; Tauri converts the argument names.
- `hooks/useRunEvents.ts` subscribes to `run_event`.
- `hooks/useAppInit.ts` loads the initial state.
- `utils/turnGrouper.ts` groups messages into chat turns.
- The chat list is virtualized with TanStack Virtual.
- The report editor is Milkdown.

## Project conventions (from `docs/AI_WORKFLOW.md` and `docs/TESTING_GUIDE.md`)

- Contract-first: define the port or trait in `domain`, then write the tests, then the implementation.
- Every bug fix needs a regression test.
- Prioritize **common** edge cases (Vietnamese diacritics, spaces in paths, empty input, cancellation mid-stream, path traversal, SSRF) over exotic ones.
- Every path and URL an agent touches must go through `PathGuard` or `UrlGuard`. Secrets must never reach logs or the UI.
- Pass a `CancellationToken` to long-running tasks. Use bounded channels. Never hold a lock across `.await`.
- Size limits: files about 400 lines or less, functions about 60 lines or less, PRs about 400 lines of diff or less. Prefer owned types with `Arc` and `async-trait` trait objects at ports, and avoid complex lifetimes or generics.
- Record significant design, library, or security decisions in `docs/DECISION_LOG.md`. ADRs are in `docs/adr/`, and the threat model is in `docs/THREAT_MODEL.md`.
