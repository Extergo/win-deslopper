# Deslopper

**Windows without the slop.**

Deslopper is a safety-focused Windows management application built with Rust, Tauri, and SvelteKit. Its long-term goal is to help people review optional Windows components and make supported, reversible changes without relying on opaque scripts or aggressive debloating.

## Current milestone: UI preview

The current build is deliberately non-destructive. It presents mock component information and lets users assemble a local preview plan, but it does not inspect or modify Windows.

- Windows Cleanup mock catalogue with local search and category filters
- UI-only planned-change review flow
- Extensions coming-soon screen
- No elevation, network access, shell execution, plugin execution, telemetry, authentication, payments, backend, or real Windows operations

The preview catalogue currently discusses OneDrive, Copilot, and Windows promotional content. These entries are examples, not live detections or removal promises.

See `ROADMAP.md` for the proposed sequence from preview hardening through read-only inspection, compatibility planning, rollback design, and carefully scoped future operations.

## Safety and gaming compatibility

Stable systems matter more than aggressive debloating. Future Windows operations must use supported mechanisms, determine compatibility first, explain impact, capture original state, verify results, and provide rollback. Gaming, anti-cheat, Xbox, and Game Pass compatibility must be tested rather than assumed.

## Development

Prerequisites are stable Rust, Node.js 22.12 or newer, npm, and the Microsoft WebView2 runtime provided by supported Windows installations.

Install the locked frontend toolchain and run locally with:

```powershell
npm ci
npm run tauri -- dev
```

Useful validation commands are `npm run check`, `npm run lint`, `npm test`, `npm run build`, and the Rust gates documented in `.deslopper/policy.toml`. `npm run tauri -- build --debug --no-bundle` verifies that the production SvelteKit assets embed into the Windows executable without producing an installer.

Contributors and coding agents must begin with `AGENTS.md`. The project architecture and safety posture are defined in `ARCHITECTURE.md` and `SECURITY.md`.
