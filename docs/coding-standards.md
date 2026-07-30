# Coding standards

- Use stable Rust, edition 2024, with no `unsafe` absent an approved documented exception.
- Do not use `unwrap()` or `expect()` in production paths. Model meaningful failures explicitly.
- Prefer narrow modules, a small composition root, explicit ownership, typed models, and enums for finite state.
- Do not use hidden global mutable state, giant command handlers, business logic in Svelte, or Windows constants in UI files.
- Keep Tauri commands few, typed, explicitly registered, and limited to orchestration. Do not add a plugin or core permission when an existing in-memory Rust operation is sufficient.
- Use strict TypeScript, semantic HTML, visible focus, central CSS tokens, and keyed Svelte lists. Treat Svelte diagnostics, ESLint errors, and formatting drift as failures.
- Keep code warning-free and live: no dead code, ignored warnings, blanket lint suppression, or placeholder pathways.
- Comments explain intent, invariants, or safety rather than syntax. Document non-obvious public interfaces.
- Add focused tests for non-trivial domain and state logic; do not couple tests to irrelevant private details.
- Use clear descriptive file, module, type, and function names.

Required Rust gates are `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`. Required frontend gates are `bun install --frozen-lockfile`, `bun run check`, `bun run lint`, `bun run test`, and `bun run build`. Never weaken lint configuration because generated or poorly structured code warns.

