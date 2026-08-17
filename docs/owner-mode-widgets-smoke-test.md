# Owner Mode Widgets smoke test

Use this checklist only on a Windows 11 x64 test account where changing the
current user's Widgets button is acceptable. Test the normal installed
`Deslopper_0.1.0_x64-setup.exe` release. Stop immediately on any mismatch; do
not restart Explorer, sign out, or force a rollback over a conflict.

## Record the starting point

- [ ] Record Windows edition, build, and UBR from `winver` and
      `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion`.
- [ ] Record the installer path, Deslopper version, and installer SHA-256.
- [ ] Inspect
      `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDa`.
      Record whether it is absent or present, its registry type, and its exact
      value. Do not create or normalize it.

## Apply, verify, and relaunch

- [ ] Install and launch Deslopper normally, with no CLI flags and no repository
      working directory.
- [ ] Run **Scan**. Confirm Widgets actionability agrees with the recorded
      `TaskbarDa` representation and that managed/unknown evidence disables Apply.
- [ ] Click the one available opposite action: **Hide Widgets button** or
      **Show Widgets button**.
- [ ] Confirm the UI progresses through Checking, Applying, and Verifying, then
      reports Changed (or a truthful failure result).
- [ ] Re-read `TaskbarDa`; confirm DWORD `0` means hidden and DWORD `1` means
      shown.
- [ ] Confirm the Deslopper Widgets detector agrees. Observe and record whether
      the visible taskbar updates immediately; Deslopper must not restart
      Explorer to force repaint.
- [ ] Expand technical details and confirm a durable transaction exists with
      exact pre-state and rollback availability.
- [ ] Close Deslopper, relaunch normally, return to Widgets, and confirm the
      transaction and **Undo last Widgets change** remain available.

## Undo and exact restoration

- [ ] Click **Undo last Widgets change** once.
- [ ] Confirm Deslopper reports Restored.
- [ ] Re-read the registry and confirm the exact original representation:
      original DWORD value and type, or value absence if it was originally
      absent.
- [ ] Confirm the detector agrees and record the visible taskbar result.
- [ ] Request the already-current state once and confirm Already set/no-op is
      reported with no rollback-only change invented.

## Optional conflict test

- [ ] Apply a Widgets change in Deslopper, then change `TaskbarDa` independently
      before clicking Undo.
- [ ] Confirm Undo refuses to overwrite the newer value, reports Needs attention,
      and shows original/applied/current representations in technical details.
- [ ] Restore the test account manually to its recorded starting representation
      after documenting the result.
