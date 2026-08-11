# Owner Mode M3 handoff

Deslopper `0.3.0` implements Owner Mode M3. The normal release observes 21
components and exposes exactly six closed current-user operations: Widgets,
Task View, welcome experience, tips and suggestions, notification suggestions,
and suggested content in Settings.

The four cleanup handlers use fixed DWORD values under
`HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`.
DWORD `1` is enabled and `0` is disabled. Missing and non-binary values are
unknown/unsupported. Matching configured values under
`HKLM\Software\Policies\Microsoft\Windows\CloudContent` are authoritative and
make the operation managed; Deslopper does not override policy. Lock-screen
suggestions remain inspection-only because `RotatingLockScreenOverlayEnabled`
and the broader `ConfigureWindowsSpotlight` policy do not establish exact parity.

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
- M3 Tips and suggestions `SoftLandingEnabled` was DWORD 1. An independent
  non-elevated same-state write succeeded. Deslopper applied DWORD 0; the
  installed process fully exited; relaunch preserved durable Undo; and
  independent final verification confirmed exact restoration to DWORD 1.
- M3 Notification suggestions `SubscribedContent-338389Enabled` followed the
  same successful sequence from DWORD 1 to 0 and back to exact DWORD 1 after
  complete process exit, relaunch, and Undo.
- Welcome experience `SubscribedContent-310093Enabled` and Suggested content in
  Settings `SubscribedContent-338393Enabled` were absent before and after the
  test. They were not created or mutated. This is the intended unsupported-
  missing contract, not a failed operation.
- `TaskbarSd` was absent and was not tested. Do not touch or productize it.

## Safety contract

- Normal commands are only closed owner actionability, closed owner Apply,
  transaction-bound Undo, and scoped history.
- No registry path, value name, raw value, script, or command comes from the UI.
- The app remains unelevated, Windows 11 x64/current-user scoped, local-only,
  and without shell, filesystem, network, updater, service, or helper access.
- Automated tests use fake backends. Implementation and CI perform no live
  registry mutation.
- Historical `owner-widgets.1` and `owner-taskbar.2` transactions remain
  readable; new intents use `owner-cleanup.3`.

The completed physical evidence is in
`docs/owner-mode-m2-physical-validation.md`; the reproducible protocol remains
in `docs/owner-mode-task-view-smoke-test.md`. Do not rerun it as part of coding
or CI.

M3 physical validation is recorded as PASS in
`docs/owner-mode-m3-physical-validation.md`. The original practical checklist
remains in `docs/owner-mode-m3-current-user-cleanup-smoke-test.md`; never turn it
into an automated or approval-file workflow.
