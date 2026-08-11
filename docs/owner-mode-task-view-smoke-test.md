# Owner Mode M2 Task View physical smoke test

This is a deliberate post-build maintainer procedure. It is not an automated
test and must not be run during implementation or CI.

## Preconditions

- Use the intended Windows 11 x64 machine and the same standard Windows account.
- Confirm the release installer hash and install the normal Owner Mode build.
- Confirm Deslopper is not elevated and no UAC prompt appears.
- Keep recovery access available. Do not change registry permissions, ACLs,
  ownership, policy, Explorer lifecycle, or `TaskbarSd`.
- The starting Task View evidence is `ShowTaskViewButton` DWORD 0 (hidden).

## Sequence

1. Launch Deslopper and run a fresh inspection.
2. Open **Taskbar Task View button**. Confirm the state is hidden, actionability
   is ready, and the only offered target is **Show Task View button**.
3. Open **Taskbar Widgets button**. On the previously tested machine/account,
   confirm it says **Direct change unavailable**, explains that Windows
   prevented the setting from being changed, and offers neither Apply nor Undo
   for the rejected transaction.
4. Return to Task View and click **Show Task View button** once. Keep the app
   open until verification finishes.
5. Confirm Deslopper reports a verified change and exposes exactly one safe
   Task View Undo. Confirm the Windows taskbar shows Task View after its natural
   refresh. Do not restart Explorer.
6. Close and relaunch Deslopper. Confirm the durable Task View Undo remains
   available for the same machine/account scope.
7. Click **Undo last Task View change** once. Confirm Deslopper reports exact
   restoration and the taskbar returns to hidden after its natural refresh.
8. Run one final fresh inspection. Confirm Task View is hidden and no Undo is
   offered for the completed rollback.

## Required evidence

Record the installer SHA-256, app version, Windows edition/build/architecture,
standard-user status, source and final Task View detector states, apply and undo
transaction IDs/statuses, direct representations, detector verification,
absence of UAC/Explorer restart, and any warning or delay. Do not record SID,
MachineGuid, username, hostname, profile path, or unredacted registry exports.

Abort without retrying if state becomes unreadable, policy appears, the current
value differs from Deslopper's recorded applied state, a write result is
ambiguous, or exact rollback cannot be proven. Preserve the local journal for
review. Never force rollback over a conflict.
