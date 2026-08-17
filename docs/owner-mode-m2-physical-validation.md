# Owner Mode M2 physical validation record

**Result:** PASS

**Recorded:** 2026-08-11

This record captures maintainer-reported physical evidence from the first live
Owner Mode M2 validation. It records observations only and grants no authority
for additional product operations.

## Release and environment

- Product: Deslopper `0.2.0`
- Source commit: `d4cd3390633be20f8ce13cd26f41af739340f298`
- Installer SHA-256: `841060FF252FA6A43C5FB419D73F02E515EE381A6720E55CBD326BEB6CFCAE75`
- Installed executable: `%LOCALAPPDATA%\Deslopper\win-deslopper.exe`
- Platform: Windows 11 x64 owner machine
- Execution: ordinary current-user process; no UAC or elevation

No SID, MachineGuid, username, hostname, or profile path is retained in this
record.

## Task View live sequence

The initial fixed current-user representation was:

```text
HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced
ShowTaskViewButton
REG_DWORD 0
```

A fresh Deslopper inspection correctly detected Task View as hidden and offered
only **Show Task View button**. Apply was executed once. Deslopper reported:

- outcome `changed`
- target `enabled`
- exact pre-state `{"dword":0}`
- rollback available
- verification `direct_and_detector_verified`

Independent `reg.exe` verification after Apply observed
`ShowTaskViewButton REG_DWORD 1`. Explorer displayed the Task View button
without Deslopper restarting Explorer.

Deslopper was then completely closed. `Get-Process` confirmed that no Deslopper
process remained, while independent registry verification still observed
`ShowTaskViewButton REG_DWORD 1`. The result was therefore persistent Windows
state, not process-local state.

The installed application was relaunched. The durable transaction survived
process exit and Undo remained available. Undo was executed once. Deslopper
reported outcome `restored` and: **The exact original Task View setting was
restored.** Independent `reg.exe` verification then observed
`ShowTaskViewButton REG_DWORD 0`.

The complete physical sequence passed:

```text
DETECT
-> APPLY
-> REAL WINDOWS WRITE
-> DIRECT VERIFY
-> DETECTOR VERIFY
-> DURABLE TRANSACTION
-> PROCESS EXIT
-> RELAUNCH
-> UNDO
-> EXACT RESTORATION
-> VERIFY
```

## Earlier Widgets physical finding

The earlier physical Widgets evidence remains scoped and unchanged:

- `TaskbarDa` was DWORD 0.
- Deslopper received `PermissionDenied`.
- An independent non-elevated `reg.exe` same-value write also received Access
  denied.
- `TaskbarDa` remained unchanged.

This evidence does not currently support classifying the result as a generic
Deslopper registry-writer failure. Deslopper must not add elevation, alter
registry ownership or ACLs, weaken Windows protections, or brute-force the
setting merely to force Widgets visibility.

## Disposition

Owner Mode M2 / Task View physical validation is **PASS**. This validates the
released Task View Detect -> Apply -> Verify -> durable relaunch -> Undo ->
exact-restoration path on the recorded environment. It does not productize or
validate Show Desktop, `TaskbarSd`, AppX, services, HKLM writes, elevation, or
generic execution.
