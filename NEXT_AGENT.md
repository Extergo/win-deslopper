# Next agent handoff

This file defines the next approved implementation slice for Deslopper. It supplements `AGENTS.md`; it does not override repository governance, architecture, security rules, or `.deslopper/policy.toml`.

## Start here

Before editing anything, read these files completely:

1. `AGENTS.md`
2. `README.md`
3. `ROADMAP.md`
4. `.deslopper/policy.toml`
5. `ARCHITECTURE.md`
6. `SECURITY.md`
7. `CONTRIBUTING.md`
8. `docs/development-workflow.md`
9. `docs/coding-standards.md`
10. `docs/dependency-policy.md`
11. `docs/testing-strategy.md`

Inspect `Cargo.toml`, the current Rust/Slint boundaries, existing tests, Git status, and current GitHub configuration before proposing changes.

## Current state

- The `ui-preview` milestone is complete.
- The application contains a polished mock catalogue, local filters/search, a preview plan, a mock review flow, and an Extensions coming-soon screen.
- All catalogue results and actions are mock-only.
- Five Rust unit tests cover search, filtering, planning, review state, and enum display mappings.
- There is no real Windows inspection, mutation, elevation, networking, backend, persistence, telemetry, authentication, payment, or plugin execution.
- `.deslopper/policy.toml` intentionally prohibits all of those capabilities.

## Next approved work package

Implement the first slice of **Roadmap Milestone 1 — Preview hardening**:

> Add a minimal Windows-based GitHub Actions workflow that runs the repository's existing quality gates on pull requests and pushes to `main`.

### Required outcome

Create `.github/workflows/ci.yml` with:

- `windows-latest` as the runner;
- triggers for pull requests and pushes to `main`;
- least-privilege workflow permissions (`contents: read`);
- a concurrency group that cancels superseded runs for the same branch or pull request;
- a stable Rust toolchain with `rustfmt` and `clippy` available;
- these commands, without weakening or replacing them:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`

Update `docs/testing-strategy.md` and `docs/development-workflow.md` so they accurately describe the new automated pull-request gate. Keep local commands authoritative and required.

### Constraints

- Do not change application behaviour or UI in this work package.
- Do not change `.deslopper/policy.toml`.
- Do not start read-only Windows inspection; that is a separately approved future milestone.
- Do not add or update Rust crates.
- Do not add application runtime shell execution, networking, elevation, or Windows APIs.
- Do not add repository secrets, tokens, signing material, publishing, releases, deployment, or package upload steps.
- Do not use `continue-on-error` for required checks.
- Do not suppress warnings, skip tests, or make CI pass by weakening validation.
- Pin third-party GitHub Actions to an immutable commit SHA, or document and obtain maintainer approval for a different supply-chain policy before use. Official GitHub actions should still use an explicitly reviewed version.
- Keep the workflow small; do not build a custom CI platform, policy engine, or documentation generator.

### Acceptance criteria

- The workflow is valid GitHub Actions YAML.
- It runs only the four existing non-interactive quality gates listed above.
- It uses Windows and stable Rust.
- It has least-privilege permissions and concurrency control.
- No application dependency or capability changes.
- Relevant governance documentation matches the workflow.
- All four commands pass locally before committing.
- `git diff --check` passes.
- The final report lists every file and command, and distinguishes local validation from GitHub-hosted validation.

## Decisions the next agent must not guess

Stop and ask a maintainer before doing any of the following:

- choosing or adding a project licence;
- replacing CODEOWNERS placeholders with usernames;
- claiming specific supported Windows builds or architectures;
- enabling branch protection or changing repository settings;
- adding dependency caches or third-party actions without supply-chain review;
- changing the current milestone or permitting real inspection;
- implementing any Windows operation, persistence, update, backend, extension, authentication, payment, or telemetry capability.

## Suggested implementation sequence

1. Confirm the worktree is clean and the current branch is appropriate.
2. Review existing GitHub files and the dependency policy.
3. Draft the smallest compliant workflow.
4. Update testing and workflow documentation.
5. Run all four local quality gates.
6. Run `git diff --check` and review the complete diff.
7. Commit with a conventional message such as `ci: add Windows quality gates`.
8. Report that GitHub-hosted execution remains unverified until the pushed workflow completes.

## After this work package

Do not automatically continue into another roadmap item. Report the result and ask for the next approved slice. Likely follow-up work includes selecting a licence, configuring maintainers, documenting supported environments, and expanding practical UI validation before Milestone 2 is considered.

