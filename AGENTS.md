# Agent Instructions for wxfetch

This document provides guidelines for AI agents working on the `wxfetch` codebase.

## Build, Lint, and Test

- **Build:** `cargo build`
- **Run:** `cargo run`
- **Lint:** `cargo clippy -- -W clippy::pedantic` (strict linting is enforced)
- **Test:** `cargo test`
- **Run a single test:** `cargo test -- --nocapture test_function_name`
- **CI Pipeline:** See `.github/workflows/rust.yml` for the full sequence of checks.

## Code Style and Conventions

- **Formatting:** Use standard `rustfmt`.
- **Imports:** Group standard, crate, and module imports separately.
- **Types:** Use specific types where possible. Use `anyhow::Result` for functions that can fail.
- **Naming:** Follow Rust's standard naming conventions (e.g., `snake_case` for functions/variables, `PascalCase` for structs/enums).
- **Error Handling:** Use `?` for error propagation. Use `expect()` only for unrecoverable errors (e.g., config loading).
- **Asynchronous Code:** Use `tokio` for async operations. Tests are written with `#[tokio::test]`.
- **Modularity:** Keep parsing, API calls, and data structures in separate modules.
- **Dependencies:** Use `serde` for JSON serialization/deserialization.
- **Comments:** Add comments to explain complex logic, not to describe what the code does.
