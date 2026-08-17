# Owner Mode M3 current-user cleanup physical smoke test

**Status:** Completed for the available exact DWORD representations. Owner Mode
M3 physical validation is PASS. Tips and suggestions and Notification
suggestions completed Apply, process exit/relaunch, durable Undo, and exact
restoration. Welcome experience and Suggested content in Settings were absent
and remained absent under the unsupported-missing contract. See
`owner-mode-m3-physical-validation.md` for the authoritative evidence record.

This file remains the reproducible manual protocol for the installed normal
Deslopper `0.3.0` product. It is not an automated test, approval system, or
authorization for additional operations.

Run on a Windows 11 x64 machine as the ordinary current user, without UAC or
elevation. Test one operation at a time. Record the installer SHA-256, installed
executable path, Windows edition/build, user/elevation state, Deslopper commit,
timestamps, and every command result. Do not test on a managed setting without
the owner's approval, and never remove or change policy to make an operation
actionable.

## Accepted-operation inventory

All preference values are fixed under:

`HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`

All policy values are read-only checks under:

`HKLM\Software\Policies\Microsoft\Windows\CloudContent`

| Component | Preference DWORD | Matching policy DWORD | Meaning |
| --- | --- | --- | --- |
| Welcome experience | `SubscribedContent-310093Enabled` | `DisableWindowsSpotlightWindowsWelcomeExperience` | Preference 1 enabled / 0 disabled; configured policy is authoritative |
| Tips and suggestions | `SoftLandingEnabled` | `DisableSoftLanding` | Preference 1 enabled / 0 disabled; configured policy is authoritative |
| Notification suggestions | `SubscribedContent-338389Enabled` | `DisableWindowsSpotlightOnActionCenter` | Preference 1 enabled / 0 disabled; configured policy is authoritative |
| Suggested content in Settings | `SubscribedContent-338393Enabled` | `DisableWindowsSpotlightOnSettings` | Preference 1 enabled / 0 disabled; configured policy is authoritative |

A missing preference or a type/value other than exact `REG_DWORD` 0 or 1 is not
actionable in M3. Lock-screen suggestions are not in this inventory: their
broader Spotlight policy does not prove exact parity with
`RotatingLockScreenOverlayEnabled`.

## Operation 1: Welcome experience

### A. Record pre-state

```powershell
reg.exe query "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" /v "SubscribedContent-310093Enabled"
reg.exe query "HKLM\Software\Policies\Microsoft\Windows\CloudContent" /v "DisableWindowsSpotlightWindowsWelcomeExperience"
```

Record existence, type, and exact value. Deslopper must report enabled only for
preference DWORD 1, disabled only for DWORD 0, unknown for missing/invalid, and
managed when the matching policy is configured.

### B. Optional same-state writability probe

Only if the preference already exists as exact DWORD 0 or 1, an owner may write
the same recorded value unelevated before asking Deslopper to change it:

```powershell
reg.exe add "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" /v "SubscribedContent-310093Enabled" /t REG_DWORD /d <recorded-0-or-1> /f
```

Do not run this probe if the value is absent. Continue with sections C and D.

## Operation 2: Tips and suggestions

### A. Record pre-state

```powershell
reg.exe query "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" /v "SoftLandingEnabled"
reg.exe query "HKLM\Software\Policies\Microsoft\Windows\CloudContent" /v "DisableSoftLanding"
```

Record existence, type, and exact value. Apply the same DWORD and policy rules
from the inventory.

### B. Optional same-state writability probe

Only for an existing exact DWORD 0 or 1:

```powershell
reg.exe add "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" /v "SoftLandingEnabled" /t REG_DWORD /d <recorded-0-or-1> /f
```

Do not create an absent value. Continue with sections C and D.

## Operation 3: Notification suggestions

### A. Record pre-state

```powershell
reg.exe query "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" /v "SubscribedContent-338389Enabled"
reg.exe query "HKLM\Software\Policies\Microsoft\Windows\CloudContent" /v "DisableWindowsSpotlightOnActionCenter"
```

Record existence, type, and exact value. Apply the same DWORD and policy rules
from the inventory.

### B. Optional same-state writability probe

Only for an existing exact DWORD 0 or 1:

```powershell
reg.exe add "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" /v "SubscribedContent-338389Enabled" /t REG_DWORD /d <recorded-0-or-1> /f
```

Do not create an absent value. Continue with sections C and D.

## Operation 4: Suggested content in Settings

### A. Record pre-state

```powershell
reg.exe query "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" /v "SubscribedContent-338393Enabled"
reg.exe query "HKLM\Software\Policies\Microsoft\Windows\CloudContent" /v "DisableWindowsSpotlightOnSettings"
```

Record existence, type, and exact value. Apply the same DWORD and policy rules
from the inventory.

### B. Optional same-state writability probe

Only for an existing exact DWORD 0 or 1:

```powershell
reg.exe add "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" /v "SubscribedContent-338393Enabled" /t REG_DWORD /d <recorded-0-or-1> /f
```

Do not create an absent value. Continue with sections C and D.

## C. Deslopper test — repeat independently for each operation

1. Launch the installed normal product from
   `%LOCALAPPDATA%\Deslopper\win-deslopper.exe`. Use no CLI flags, repository
   working directory, validation files, or elevation.
2. Run a fresh inspection. Confirm the component detector agrees with the
   independently recorded preference and policy state.
3. If the component is unknown, unsupported, managed, or direct change
   unavailable, confirm Apply is unavailable and stop that operation without
   forcing it.
4. For a ready exact DWORD state, click one offered Apply direction once.
   Record Deslopper's outcome, exact pre-state, target, verification result,
   and Undo availability.
5. Re-run the operation-specific `reg.exe query` from section A. Confirm the
   same fixed value is exact `REG_DWORD` 0 or 1 in the requested direction.
6. Run a fresh Deslopper inspection and confirm the matching detector agrees.
   Record visible Windows behavior if any; do not claim a visible effect when
   none can be observed reliably.
7. Close Deslopper completely. Confirm no process remains:

   ```powershell
   Get-Process win-deslopper -ErrorAction SilentlyContinue
   ```

8. Query the preference while Deslopper is closed and confirm the applied state
   persists.
9. Relaunch the installed product normally. Confirm the durable transaction and
   Undo remain available for the same operation and owner scope.
10. Click Undo once. Query the preference again and confirm exact restoration:
    the original DWORD value must be restored. (M3 does not offer Apply from an
    originally missing representation.)
11. Run a final fresh inspection and confirm the detector matches the restored
    state. Save a concise evidence record; do not paste private machine or user
    identifiers into repository documentation.

## D. Stop conditions — apply to every operation

Stop immediately and preserve evidence without retrying blindly on any of:

- detector state differs from the independently queried fixed representation;
- Deslopper or instructions reference an unexpected key, value, hive, or type;
- a relevant policy exists but Deslopper offers to override it;
- access denied occurs unexpectedly (record unchanged post-attempt state and do
  not elevate, change ACLs, take ownership, or weaken Windows protections);
- the post-attempt state is ambiguous or differs from both original and target;
- a verified successful change has no durable Undo;
- Undo is offered after a rejected unchanged write;
- the transaction disappears after complete process exit and relaunch;
- Undo does not restore the exact original DWORD representation;
- direct and detector verification disagree; or
- the operation requests Explorer termination, sign-out, reboot, PowerShell,
  AppX changes, HKLM writes, services, tasks, or administrator rights.

Record the operation as PASS only after the full independent
detect/apply/direct-verify/detector-verify/exit/relaunch/Undo/exact-restore/final-
verify sequence succeeds.
