# Hubbub Justfile — Task runner
# Usage: just <recipe>

# Default recipe
default:
    @just --list

# === Development ===

# Run desktop app in dev mode
dev:
    cd apps/desktop && npm run tauri dev

# Run frontend only (no Tauri)
dev-ui:
    cd apps/desktop && npm run dev

# === Testing ===

# Run all Rust tests
test:
    cargo nextest run --workspace

# Run Rust tests (standard cargo test if nextest not installed)
test-cargo:
    cargo test --workspace

# Run clippy on all crates
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format check
fmt-check:
    cargo fmt --all -- --check

# Format fix
fmt:
    cargo fmt --all

# Run TypeScript checks
lint-ui:
    cd apps/desktop && npx tsc --noEmit

# === Build ===

# Build release
build:
    cd apps/desktop && npm run tauri build

# Check workspace compiles
check:
    cargo check --workspace

# === Security ===

# Run cargo-deny checks
deny:
    cargo deny check

# === All checks (CI equivalent) ===

# Run all checks that CI runs
ci: fmt-check lint test-cargo lint-ui
    @echo "All CI checks passed!"
