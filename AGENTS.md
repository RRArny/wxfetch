# Hermes Agent Configuration

## Coding Standards

### Clippy Pedantic Lints
All code must pass `cargo clippy --all-targets -- -W clippy::pedantic` with zero warnings. To enforce this, always run:

```sh
cargo clippy --all-targets -- -W clippy::pedantic
```

This catches:
- `uninlined_format_args` — use `format!("{var}")` instead of `format!("{}", var)`
- `redundant_closure_for_method_calls` — use method paths directly
- `cast_possible_truncation` — prefer `try_from` over `as` casts
- `implicit_clone` — use `.clone()` explicitly instead of `.to_string()` on references
- `needless_borrows_for_generic_args` — pass `[elem]` not `&[elem]`
- `needless_raw_string_hashes` — use `r"..."` unless inner `"` or `\` needed
- `semicolon_if_nothing_returned` — add `;` to statements that should be unit expressions
- `float_cmp` — use `#[allow(clippy::float_cmp)]` in tests where exact comparison is intentional

### Pre-existing Issues
The `async fn` / `let chains` edition-hint warnings in test modules are pre-existing and require a Rust edition bump (currently blocked by toolchain config). These are acknowledged and should be revisited when the project upgrades to edition 2024 enforcement.