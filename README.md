# Deslopper

**Windows without the slop.**

Deslopper is a safety-focused Windows management application built in Rust and Slint. Its long-term goal is to help people review optional Windows components and make supported, reversible changes without relying on opaque scripts or aggressive debloating.

## Current milestone: UI preview

The current build is deliberately non-destructive. It presents mock component information and lets users assemble a local preview plan, but it does not inspect or modify Windows.

- Windows Cleanup mock catalogue with local search and category filters
- UI-only planned-change review flow
- Extensions coming-soon screen
- No elevation, network access, shell execution, plugin execution, telemetry, authentication, payments, backend, or real Windows operations

The preview catalogue currently discusses OneDrive, Copilot, and Windows promotional content. These entries are examples, not live detections or removal promises.

## Safety and gaming compatibility

Stable systems matter more than aggressive debloating. Future Windows operations must use supported mechanisms, determine compatibility first, explain impact, capture original state, verify results, and provide rollback. Gaming, anti-cheat, Xbox, and Game Pass compatibility must be tested rather than assumed.

## Development

Run locally with:

```powershell
cargo run
```

Contributors and coding agents must begin with `AGENTS.md`. The project architecture and safety posture are defined in `ARCHITECTURE.md` and `SECURITY.md`.
