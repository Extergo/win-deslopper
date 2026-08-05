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

## Internal live-validation targets

Live mutation is permitted only on an explicitly approved disposable validation
target whose target type is allowed by the committed repository policy. The
currently allowed types are a disposable `virtual_machine` and an expendable
`physical_laptop`. A physical target is exceptional, internal, and local-only;
all strict recovery, identity, management, approval, expiry, reinstallation,
and final-disposition requirements must pass. Missing, stale, malformed, or
contradictory evidence requires stopping.

The original development PC is permanently prohibited from live mutation. It
may be used for development, compilation, read-only inspection, and synthetic
testing only. The development-host denylist has no override flag, and neither a
maintainer CLI switch nor a prompt may bypass it.

Governance changes require a committed maintainer-approved source change before
execution. The authorizing approval and execution must bind to the same
committed code. A live-execution handoff cannot override repository governance
by prompt alone, and changing governance grants no target or operation
approval.

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
