# Deslopper roadmap

This roadmap describes the intended order of work after the current `ui-preview` milestone. It is directional, not authorization to implement future capabilities.

Safety and security rules, architectural boundaries, and `.deslopper/policy.toml` take priority over this roadmap. A future milestone may begin only after its scope is explicitly approved and the policy file is updated deliberately. No milestone may skip the lifecycle defined in `docs/windows-safety-model.md`.

## Current milestone — UI preview

**Status:** Complete

The current application provides a polished, non-destructive shell with:

- a mock catalogue for OneDrive, Windows Copilot, and promotional content;
- local search and category filtering;
- a preview-only planned-change and review flow;
- an Extensions coming-soon screen;
- no real Windows inspection, mutation, elevation, networking, backend, telemetry, or plugin execution.

## Milestone 1 — Preview hardening

**Goal:** Make the existing preview a dependable development baseline before introducing Windows-specific code.

Planned work:

- Add GitHub Actions for formatting, Clippy with warnings denied, and tests.
- Replace CODEOWNERS placeholders with real maintainers before enabling branch protection.
- Add a project licence and define supported Windows versions and architectures.
- Add practical UI state tests and a repeatable visual-review process.
- Test keyboard navigation, resizing, and standard Windows scaling levels.
- Validate the catalogue language with users and document supported terminology.

Exit criteria:

- Required checks run automatically on pull requests.
- Governance-sensitive files require configured maintainer review.
- Supported test environments are documented.
- The preview remains fully non-destructive.

## Milestone 2 — Read-only inspection alpha

**Goal:** Replace selected mock detection states with trustworthy, read-only Windows inspection while retaining zero mutation and zero elevation.

Before implementation:

- Obtain explicit approval for real inspection.
- Update `.deslopper/policy.toml` so only `allow_real_windows_inspection` changes to `true`.
- Add an architecture decision covering the inspection boundary and evidence model.
- Define the initially supported Windows builds and documented inspection mechanisms.

Planned architecture:

- Introduce a narrow inspection interface below application orchestration.
- Keep domain models independent of Slint and Windows implementation details.
- Retain a fake provider for deterministic tests and development.
- Start with one component, preferably OneDrive, before expanding the catalogue.

Inspection results must distinguish:

- `Detected`
- `Not detected`
- `Unavailable`
- `Unsupported Windows build`
- `Unknown — inspection failed`

Every result should record when it was obtained, its evidence source, and whether elevated access was unavailable or unnecessary. Inspection failure must never be treated as absence.

Exit criteria:

- The main UI process still runs without administrator privileges.
- No inspection path mutates Windows or executes arbitrary commands.
- Fake-provider unit tests and Windows integration tests pass.
- Mock and live data are visibly distinguishable.

## Milestone 3 — Compatibility and planning engine

**Goal:** Convert inspection evidence into precise, immutable proposed-change plans without applying them.

Planned work:

- Determine which actions are available for the exact Windows build and component form.
- Explain impact, compatibility assumptions, restart requirements, and rollback prerequisites.
- Detect conflicts, missing prerequisites, and dependent components.
- Keep plan creation separate from execution.
- Add state-transition and compatibility-matrix tests.

Exit criteria:

- Plans contain typed operations rather than shell commands or loosely related strings.
- Unsupported and unknown states cannot produce an applicable plan.
- No Apply capability or mutation implementation exists.

## Milestone 4 — Rollback and privileged-operation design

**Goal:** Design the complete trust boundary and recovery model before implementing any real change.

Required design work:

- Original-state capture
- Immutable operation receipts
- Result verification
- Failure and partial-failure handling
- Rollback and rollback verification
- Narrow privileged-helper protocol
- Authentication between the UI and privileged helper
- Audit history and corrupted-state handling

This milestone requires architecture and security review. Elevation must remain isolated from the main UI, and the helper must accept typed, allow-listed operations rather than commands.

Exit criteria:

- Threat model and protocol are documented and reviewed.
- Rollback is defined for the proposed first operation.
- No mutation ships merely because the design exists.

## Milestone 5 — First reversible Windows operation

**Goal:** Implement one narrowly scoped, documented, verifiable, and reversible operation.

Selection criteria:

- Uses a supported Windows mechanism.
- Has explicit build compatibility.
- Can capture and restore original state.
- Can verify both application and rollback.
- Does not require broad package removal, wildcard matching, or arbitrary scripts.

The first operation should not be a generic debloat action or broad AppX removal. Additional operations must repeat the same compatibility, verification, receipt, and rollback analysis.

Exit criteria:

- Explicit user approval is required immediately before mutation.
- Success is reported only after verification.
- Rollback is available and verified.
- Virtual-machine and supported-build testing passes.

## Milestone 6 — Extensions foundation

**Goal:** Define a safe extension system only after the core Windows safety model is proven.

Required properties:

- Out-of-process isolation
- Signed and verified packages
- Versioned protocols
- Explicit capability grants
- No unrestricted administrator access
- No arbitrary shell execution
- Verified updates and revocation planning

Marketplace, accounts, payments, downloads, and extension execution are not part of the current application and must not be added speculatively.

## Sequencing rules

- Do not start mutation work before inspection, compatibility planning, and rollback design are complete.
- Do not start extensions before the privileged-operation and package-trust models are reviewed.
- Do not add backend dependencies for functionality that can safely work locally.
- Do not weaken gaming, anti-cheat, Xbox, Game Pass, or Windows security compatibility to increase removal coverage.
- Keep milestones small enough that safety claims can be tested rather than assumed.

