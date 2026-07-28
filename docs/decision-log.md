# Decision log

Each decision below is accepted for the current architecture. Replacements must preserve history and explain migration.

## D-001 — Rust core and Slint UI

- **Status:** Superseded by D-006
- **Context:** Deslopper needs a native, maintainable Windows application with a declarative UI.
- **Decision:** Use stable Rust for domain/orchestration and Slint for presentation.
- **Consequences:** Typed boundaries and small native deployment; toolkit-specific UI bridge is isolated.
- **Alternatives:** Web shells and other native toolkits were rejected for this milestone due to scope or architectural weight.

## D-002 — Material-inspired visual system

- **Status:** Accepted
- **Context:** The product needs an approachable identity distinct from dense administration tools.
- **Decision:** Use stock-Android-inspired Material principles with centralised tokens and no copied trademarks/assets.
- **Consequences:** UI review follows `docs/ui-design-system.md`; other visual languages require a new decision.
- **Alternatives:** Legacy control panels, generic desktop widgets, and mixed aesthetics were rejected.

## D-003 — Non-destructive UI-first milestone

- **Status:** Accepted
- **Context:** Safety architecture is not yet implemented.
- **Decision:** The first milestone implements mock domain data, local planning state, and UI only; no real Windows inspection or operation.
- **Consequences:** Every result is labelled preview; no inert destructive pathway may be pre-wired.
- **Alternatives:** Early PowerShell or registry integration was rejected as unsafe and premature.

## D-004 — Future trust boundaries

- **Status:** Accepted
- **Context:** Privileged operations, extensions, and online services will have different trust levels.
- **Decision:** Future elevation uses a narrow helper; plugins run out of process with capabilities; the backend supports accounts, commerce, entitlements, catalogue, and verified package delivery, never arbitrary remote control.
- **Consequences:** These process/protocol boundaries require later security design and explicit milestone approval.
- **Alternatives:** In-process privileged plugins and backend command channels were rejected.

## D-005 — Governance and central tokens

- **Status:** Superseded in part by D-006
- **Context:** Multiple contributors require durable, discoverable constraints.
- **Decision:** Mandatory repository governance documents own policies, and Slint theme tokens own shared visual values.
- **Consequences:** Relevant documentation changes with behaviour; token exceptions require design review.
- **Alternatives:** Convention-only governance and scattered styling were rejected as difficult to audit.

## D-006 — Tauri shell and SvelteKit presentation

- **Status:** Accepted
- **Date:** 2026-07-27
- **Context:** The maintainers requested that the UI preview move from Slint to Tauri and SvelteKit while preserving its non-destructive scope, Rust-owned domain rules, and Material-inspired visual language.
- **Decision:** Use Tauri 2 as the native Windows window and local IPC boundary, and SvelteKit 2 with static SPA output for presentation. Rust continues to own the catalogue, filters, plan, navigation, and review state. The webview receives serializable view models and sends a small typed action enum through two explicitly registered commands. Shared visual tokens move from Slint globals to a single CSS token file.
- **Consequences:** The application now includes the system WebView2-based Tauri runtime and a Node-based frontend build toolchain. Static production assets are embedded locally; no server, remote origin, Tauri plugin, shell API, updater, filesystem API, or network client is introduced. Rust and frontend quality gates are both required. Dependency review is recorded in `docs/dependency-review-tauri-sveltekit.md`.
- **Security boundary:** Only the main local window receives the core Tauri capability. Custom IPC can read or change in-memory preview state only. It cannot inspect or modify Windows, execute commands, persist data, elevate, authenticate, make payments, load plugins, collect telemetry, or contact a backend.
- **Alternatives:** Keeping Slint did not satisfy the requested migration. A hosted or server-rendered frontend would introduce an unnecessary runtime network boundary. Exposing broad Tauri plugins or moving domain state into TypeScript would weaken the current safety and architecture boundaries.

