# Owner Mode M2 handoff

The active product milestone is Deslopper Owner Mode M2. The normal release
observes 21 components and exposes only two closed current-user operations:
Widgets visibility and Task View visibility.

## Physical evidence that controls the product

- Widgets `TaskbarDa` was DWORD 0. Deslopper and an independent unelevated
  write both received access denied and the exact value remained unchanged.
  This is a rejected-unchanged result, not partial mutation and not a generic
  writer failure. The stable machine/account scope must show **Direct change
  unavailable** for Widgets, with no Undo. Do not elevate, edit ACLs, or suppress
  Widgets globally.
- Task View `ShowTaskViewButton` was DWORD 0. An unelevated same-state write
  succeeded. The subsequent M2 physical run passed the complete live sequence:
  fresh hidden detection, Apply to DWORD 1, direct and detector verification,
  visible Explorer refresh without restart, full process exit, persistence,
  relaunch with durable Undo, exact restoration to DWORD 0, and independent
  final verification.
- `TaskbarSd` was absent and was not tested. Do not touch or productize it.

## Safety contract

- Normal commands are only Widgets/Task View actionability, closed owner Apply,
  transaction-bound Undo, and scoped history.
- No registry path, value name, raw value, script, or command comes from the UI.
- The app remains unelevated, Windows 11 x64/current-user scoped, local-only,
  and without shell, filesystem, network, updater, service, or helper access.
- Automated tests use fake backends. Implementation and CI perform no live
  registry mutation.
- Historical `owner-widgets.1` transactions remain readable; new intents use
  `owner-taskbar.2`.

The completed physical evidence is in
`docs/owner-mode-m2-physical-validation.md`; the reproducible protocol remains
in `docs/owner-mode-task-view-smoke-test.md`. Do not rerun it as part of coding
or CI.
