# Coding standards

- Use stable Rust, edition 2024, with no `unsafe` absent an approved documented exception.
- Do not use `unwrap()` or `expect()` in production paths. Model meaningful failures explicitly.
- Prefer narrow modules, a small composition root, explicit ownership, typed models, and enums for finite state.
- Do not use hidden global mutable state, giant callback functions, business logic in Slint, or Windows constants in UI files.
- Keep code warning-free and live: no dead code, ignored warnings, blanket lint suppression, or placeholder pathways.
- Comments explain intent, invariants, or safety rather than syntax. Document non-obvious public interfaces.
- Add focused tests for non-trivial domain and state logic; do not couple tests to irrelevant private details.
- Use clear descriptive file, module, type, and function names.

Required gates are `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`. Never weaken lint configuration because generated or poorly structured code warns.

