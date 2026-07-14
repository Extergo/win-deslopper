# Architecture

This document owns Deslopper's layer boundaries. The current milestone implements only the first three layers.

## Layers

1. **Slint UI — current.** Rendering, input, accessibility-oriented presentation, and typed callbacks. Slint files contain no business or Windows-operation logic.
2. **Application state and orchestration — current.** Owns navigation, local search/filter state, the planned ID set, and review-panel visibility; coordinates UI callbacks.
3. **Domain models — current.** Rust-owned component IDs, categories, risk, restart, compatibility, catalogue entries, and pure planning/filtering behaviour. Domain types do not depend on Slint.
4. **Windows inspection — future.** Read-only, build-aware discovery through documented Windows mechanisms.
5. **Windows operation planning — future.** Compatibility checks, impact explanation, rollback requirements, and immutable operation plans; never applies changes.
6. **Privileged operation host — future.** Separate, narrowly scoped elevated process accepting validated operations, not commands.
7. **Persistence and rollback receipts — future.** Captures original state and verified outcomes; never silently applies operations.
8. **Plugin protocol — future.** Out-of-process, capability-scoped extension communication with no unrestricted privilege.
9. **Backend API client — future.** Accounts, commerce, entitlements, catalogue metadata, and signed package delivery; never remote command execution.
10. **Update and package verification — future.** Signature, identity, version, and integrity verification before installation.

## Dependency rules

Dependencies flow from UI to orchestration to domain abstractions. Domain models must not depend on Slint-generated types. Slint must not invoke Windows APIs or know registry paths, service names, shell commands, or DISM arguments. Inspection must not depend on marketplace or billing. Billing and entitlements never determine technical safety. Plugins receive no unrestricted privileged access. A backend cannot instruct arbitrary client commands. Persistence cannot apply operations. Elevation is isolated from the main UI process.

The current UI data is explicitly mock-only. Future layers must not be introduced behind flags or inert controls. Any substantial boundary change or exception requires an approved entry in `docs/decision-log.md` before code lands.

