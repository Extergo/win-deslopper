# Agent operating rules

`AGENTS.md` is the mandatory entry point for every coding agent working in this repository. The rules are mandatory even while Deslopper is early. The authoritative rule hierarchy is:

1. Safety and security rules
2. Architectural boundaries
3. Product principles
4. Design-system rules
5. Coding standards
6. Testing and quality requirements
7. Feature-specific implementation instructions

When rules conflict, stop and document the conflict; the higher rule wins. Never guess around governance.

## Required reading

Before editing, read this file, `SECURITY.md`, `ARCHITECTURE.md`, `.deslopper/policy.toml`, and every relevant file under `docs/`. Read `CONTRIBUTING.md` for workflow and `README.md` for current scope. Inspect the existing architecture before proposing changes.

A behavioural, architectural, security, dependency, or visual-system change is incomplete until its authoritative documentation is updated.

## Non-negotiable conduct

- Preserve safety boundaries and do not implement beyond the approved milestone.
- Never weaken tests, linting, validation, security controls, or mandatory wording merely to pass CI.
- Never silently alter architecture; substantial boundary changes require a decision record and maintainer approval.
- Follow `docs/dependency-policy.md` before adding or updating a dependency.
- Never perform real Windows mutations unless the current approved milestone explicitly permits them.
- Never introduce arbitrary script or shell execution, directly or through a helper.
- Never store credentials, signing material, tokens, personal data, or other secrets in the repository.
- Never delete governance files, significantly weaken them, reclassify mandatory rules as suggestions, or add undocumented exceptions without explicit maintainer approval.
- Report every created, modified, renamed, and deleted file, every command executed, failed checks honestly, and known limitations.

## Anti-bypass rules

A helper script is still shell execution. A hidden feature flag is still an implemented capability. A disabled control wired to destructive code is still destructive code. Test-only production pathways count if shipped. Calling PowerShell through another executable is still PowerShell execution. A backend returning executable instructions is remote command execution. Moving unsafe code to another crate does not remove the unsafe-code concern. Disabling a lint globally is not a fix. Catching and ignoring an error is not error handling. A `preview` label does not excuse misleading behaviour. Splitting a prohibited operation across modules does not permit it. Indirect dependencies still require review. Generated code does not excuse security or licensing review.

## Owner-mode Windows operations

The normal product may execute only registered owner-mode operations approved
by the current milestone. One deliberate Apply click authorizes one closed
operation and one deliberate Undo click authorizes an exact, conflict-free
restore. Safety lives in Rust's fixed operation registry, preflight, exact state
capture, durable journal, verification, and rollback. Frontend input must never
provide a registry path, value name, command, script, or arbitrary data mapping.

Owner Mode M1 permits only current-user Widgets visibility through the fixed
`TaskbarDa` handler. It remains unelevated and Windows 11 x64 only. Task View,
Show Desktop, AppX, HKLM, services, scheduled tasks, generic PowerShell, generic
registry editing, and automatic Explorer restart are outside this milestone.
Automated tests must use fake backends and must not perform a live Windows
mutation.

Historical Mutation Alpha target, approval, scenario, deny-list, evidence, and
Hyper-V infrastructure may remain as an engineering validation harness. It does
not authorize or block the installed owner product. Manual live smoke testing
is a separate maintainer action after reviewing the release artifact and
`docs/owner-mode-widgets-smoke-test.md`; implementation and CI do not run it.

## Pre-change checklist

- [ ] Read `AGENTS.md` and all governance documents relevant to the task.
- [ ] Inspect `Cargo.toml`, current module boundaries, and existing tests.
- [ ] Identify security, Windows operations, plugins, networking, persistence, updates, billing, authentication, and elevation impact.
- [ ] Confirm `.deslopper/policy.toml` and the approved milestone permit the proposed work.
- [ ] Define the smallest coherent scope and identify risks before editing.

## Post-change checklist

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test`.
- [ ] Run and visually review the application where practical.
- [ ] Confirm no prohibited Windows operation or capability was introduced.
- [ ] Confirm documentation remains accurate and no unnecessary dependency was added.
- [ ] Review the complete Git diff and report files, commands, failures, and limitations.
