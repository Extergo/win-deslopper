# Contributing

Read `AGENTS.md` before starting. A behavioural, architectural, security, dependency, or visual-system change is incomplete until the relevant authoritative documentation changes with it.

## Workflow

Use `feature/<short-name>`, `fix/<short-name>`, `refactor/<short-name>`, `docs/<short-name>`, or `security/<short-name>` branches. Prefer focused commits beginning with `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `build:`, `ci:`, `security:`, or `chore:`.

Follow `docs/development-workflow.md`. Avoid unrelated refactors. Pull requests must explain what changed, why, what was deliberately excluded, safety and architecture impact, dependency impact, tests performed, and rollback considerations. Attach screenshots for visible UI work and disclose limitations.

## Reviews

- Architecture review is mandatory for layer-boundary changes, new long-lived modules, persistence, protocols, or process boundaries.
- Security review is mandatory for Windows operations, elevation, plugins, networking, authentication, billing, telemetry, updates, package verification, secrets, or new trust boundaries.
- UI review is mandatory for visible changes; review keyboard focus, resizing, contrast, copy, and consistency with `docs/ui-design-system.md`.
- Windows compatibility testing is mandatory when behaviour depends on a Windows build, edition, region, account, architecture, security feature, or installed component.
- Sensitive paths listed in `.github/CODEOWNERS` require explicit maintainer review once real owner handles are configured.

Propose dependencies using the record required by `docs/dependency-policy.md`. Propose substantial architecture changes by adding a decision entry to `docs/decision-log.md` before implementation. Update code and documentation together whenever behaviour or architecture changes.

Before review, run the commands in `.deslopper/policy.toml` and complete the pull-request template. Never weaken a check to obtain a green result.

