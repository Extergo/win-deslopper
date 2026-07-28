# Architecture

This document owns Deslopper's layer boundaries. The current milestone implements only the first three layers.

## Layers

1. **Tauri and SvelteKit presentation — current.** Tauri owns the native window and serves a statically built SvelteKit SPA. Svelte renders input, accessibility-oriented presentation, and serializable view models; it contains no Windows-operation or domain logic.
2. **Application state and orchestration — current.** Rust owns navigation, local search/filter state, the planned ID set, and review-panel visibility. Two explicitly registered Tauri commands read the current view or dispatch a typed in-memory preview action.
3. **Domain models — current.** Rust-owned component IDs, categories, risk, restart, compatibility, catalogue entries, and pure planning/filtering behaviour. Domain types do not depend on Tauri, SvelteKit, or presentation models.
4. **Windows inspection — future.** Read-only, build-aware discovery through documented Windows mechanisms.
5. **Windows operation planning — future.** Compatibility checks, impact explanation, rollback requirements, and immutable operation plans; never applies changes.
6. **Privileged operation host — future.** Separate, narrowly scoped elevated process accepting validated operations, not commands.
7. **Persistence and rollback receipts — future.** Captures original state and verified outcomes; never silently applies operations.
8. **Plugin protocol — future.** Out-of-process, capability-scoped extension communication with no unrestricted privilege.
9. **Backend API client — future.** Accounts, commerce, entitlements, catalogue metadata, and signed package delivery; never remote command execution.
10. **Update and package verification — future.** Signature, identity, version, and integrity verification before installation.

## Dependency rules

Dependencies flow from Svelte presentation through typed local Tauri IPC to Rust orchestration and domain abstractions. Domain models must not depend on Tauri, serde transport shapes, or SvelteKit. TypeScript must not duplicate domain rules or know registry paths, service names, shell commands, or DISM arguments. Tauri plugins are not enabled in the current milestone, the production frontend is embedded locally, and the capability manifest grants no core plugin permissions. Inspection must not depend on marketplace or billing. Billing and entitlements never determine technical safety. Plugins receive no unrestricted privileged access. A backend cannot instruct arbitrary client commands. Persistence cannot apply operations. Elevation is isolated from the main UI process.

The current UI data is explicitly mock-only. Future layers must not be introduced behind flags or inert controls. Any substantial boundary change or exception requires an approved entry in `docs/decision-log.md` before code lands.

