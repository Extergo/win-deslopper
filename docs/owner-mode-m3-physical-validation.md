# Owner Mode M3 physical validation record

**Result:** PASS

**Recorded:** 2026-08-12

This is the authoritative physical evidence record for the Owner Mode M3
current-user cleanup pack. It records observations from the installed release
only. It does not authorize additional settings or broaden the mutation
boundary.

## Validated release and environment

- Product: Deslopper `0.3.0`
- Commit: `2216c8eaac7f7efa961e04251f6b7eb670b8f83d`
- Installer SHA-256:
  `1902BB5C5392CE5600C742CEBC037BE1AB26BE416BCF8511E759876BE3E12813`
- Installed executable: `%LOCALAPPDATA%\Deslopper\win-deslopper.exe`
- Product form: installed NSIS product, not the repository release executable
- Platform: Windows 11 x64 owner machine
- Execution: current Windows user, ordinary non-elevated token, no UAC

## Pre-validation inventory

The fixed current-user key was:

`HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager`

The key was independently exported before testing. Its relevant initial
representations were:

| M3 component | Value | Exact pre-state |
| --- | --- | --- |
| Welcome experience | `SubscribedContent-310093Enabled` | Absent |
| Tips and suggestions | `SoftLandingEnabled` | `REG_DWORD 1` |
| Notification suggestions | `SubscribedContent-338389Enabled` | `REG_DWORD 1` |
| Suggested content in Settings | `SubscribedContent-338393Enabled` | Absent |

No absent value was created merely to make it testable.

## Independent same-state writability probes

Ordinary non-elevated `reg.exe` same-state writes were performed only for the
two existing DWORDs:

- `SoftLandingEnabled`: DWORD 1 -> DWORD 1, completed successfully.
- `SubscribedContent-338389Enabled`: DWORD 1 -> DWORD 1, completed successfully.

These probes independently established that both physically exercised M3
representations were writable by the ordinary current-user token before
Deslopper changed them.

## Tips and suggestions live sequence

Deslopper detected the existing state and applied the fixed Tips and
suggestions operation once:

`SoftLandingEnabled REG_DWORD 1 -> REG_DWORD 0`

Independent `reg.exe` verification after Apply observed exact `REG_DWORD 0`.
After Deslopper fully exited, process inspection found no Deslopper process.
The installed product was relaunched, the durable owner transaction and Undo
remained available, and Undo was executed once. Independent final verification
observed exact `SoftLandingEnabled REG_DWORD 1`.

Tips and suggestions live Apply/Undo: **PASS**.

## Notification suggestions live sequence

Deslopper detected the existing state and applied the fixed Notification
suggestions operation once:

`SubscribedContent-338389Enabled REG_DWORD 1 -> REG_DWORD 0`

Independent `reg.exe` verification after Apply observed exact `REG_DWORD 0`.
After the same complete process exit and installed-product relaunch, the durable
owner transaction and Undo remained available. Undo was executed once.
Independent final verification observed exact
`SubscribedContent-338389Enabled REG_DWORD 1`.

Notification suggestions live Apply/Undo: **PASS**.

For both physically changed operations, the supplied physical record confirms:

```text
DETECT
-> APPLY
-> REAL WINDOWS WRITE
-> VERIFY
-> DURABLE TRANSACTION
-> PROCESS EXIT
-> RELAUNCH
-> UNDO
-> EXACT RESTORATION
-> VERIFY
```

## Absence integrity

After both Apply/Undo sequences, the two initially absent values were queried
independently:

- `SubscribedContent-310093Enabled`: absent.
- `SubscribedContent-338393Enabled`: absent.

Both queries returned that the specified registry key or value could not be
found. M3 therefore did not create unrelated missing representations while
changing the two existing DWORDs.

Welcome experience and Suggested content in Settings were not physically
mutated because their values were absent. This is not a failure: M3 deliberately
treats missing values as unsupported instead of inventing effective-state
semantics or creating a value.

Absence preservation: **PASS**.

## Cumulative Owner Mode physical evidence

The repository now records successful installed-product Apply/Undo across three
independent registered operations:

- Task View: `ShowTaskViewButton` DWORD 0 -> 1 -> process exit/relaunch -> Undo
  -> exact DWORD 0.
- Tips and suggestions: `SoftLandingEnabled` DWORD 1 -> 0 -> process
  exit/relaunch -> Undo -> exact DWORD 1.
- Notification suggestions: `SubscribedContent-338389Enabled` DWORD 1 -> 0 ->
  process exit/relaunch -> Undo -> exact DWORD 1.

Widgets remains a distinct scoped direct-change-unavailable result. Deslopper
and an independent non-elevated same-state `reg.exe` write both received access
denied for `TaskbarDa`, and the value remained unchanged. This is not evidence
of a generic `WindowsSettingStore` failure and does not justify elevation,
ownership/ACL changes, or weakened Windows protections.

Lock-screen suggestions remain intentionally inspection-only because they were
not accepted into M3.

## Disposition

Owner Mode M3 physical validation is **PASS** for the recorded release and
environment. Tips and suggestions and Notification suggestions passed real
write, process-boundary durability, relaunch Undo, and exact restoration.
Initially absent M3 representations remained absent.

This evidence does not claim live mutation coverage for Welcome experience or
Suggested content in Settings from a present DWORD, does not generalize the
results to every Windows build/account/policy configuration, and does not
authorize AppX, HKLM writes, services, elevation, Taskbar Search, Show Desktop,
`TaskbarSd`, arbitrary registry input, or any other operation.
