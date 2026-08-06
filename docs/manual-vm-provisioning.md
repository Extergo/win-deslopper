# Manual disposable-VM provisioning and live-validation handoff

This runbook is the handoff for the current infrastructure blocker. It does not
authorize mutation on the development host and it does not make VM creation,
media download, account setup, domain join, MDM enrolment, or licensing an
automated Deslopper action.

## Current blocker and safest first target

The 2026-08-02 host audit found the Windows hypervisor layer, but no Hyper-V
PowerShell management commands, registered guests, checkpoints, Windows
Sandbox executable, third-party hypervisor, or supplied installation media.
The largest detected free volume had about 48 GiB available. No VM was created,
started, or mutated.

Provision `win11-pro-24h2-clean-local` first. Use legitimately supplied Windows
11 Pro 24H2 media, a local administrator account, no domain join, no MDM
enrolment, and no personal account or data. Recommended minimums are generation
2, Secure Boot, 4 virtual processors, 8 GiB memory, a 64 GiB dynamically
expanding system disk, and at least 80 GiB free on the chosen host volume before
creation. Do not proceed with the current 48 GiB free-volume condition.

## Developer provisioning checklist

1. Enable or install a supported hypervisor and its management tooling using
   the normal Windows administration workflow, then reboot if Windows requires
   it. Confirm `Get-Command Get-VM` for the repository's Hyper-V helper.
2. Obtain licensed installation media from an approved source. Verify its
   publisher and integrity outside Deslopper.
3. Create one disposable VM with no shared host folders, personal credentials,
   production domain membership, production MDM enrolment, or host secrets.
4. Install Windows 11 Pro 24H2 and all updates intended for the scenario. Keep
   the scenario's edition/build/account facts exact; do not relabel another
   build as 24H2.
5. Install the guest integration service needed for deliberate file transfer.
   Do not enable uncontrolled host-drive sharing.
6. Run a normal read-only Deslopper inspection inside the guest so its database
   is guest-owned.
7. Shut down the guest and create the checkpoint
   `deslopper-clean-win11-pro-24h2-local-v1`. Do not reuse a checkpoint after a
   mutation run.
8. Start the guest, compute its fingerprint using the command below, and have a
   developer compare the reported edition/build with the scenario. Never
   compute this value on the development host for use as a guest identity.
9. Create the ignored approval file
   `.deslopper/local/approved-validation-vms.json` from the manifest template
   below. Replace every angle-bracket token with an observed or approved value.
   Validate it against `validation/approved-validation-vms.schema.json`.
10. Run the inventory and checkpoint checks. Stop if the VM name, generation,
    identity, edition, build family, checkpoint, or disposable declaration is
    not exact.

Inside the guest, compute the identity without exposing the computer name:

```powershell
$cv = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$inputValue = "$($env:COMPUTERNAME.ToUpperInvariant()):$($cv.EditionID):$($cv.CurrentBuild)"
$sha = [Security.Cryptography.SHA256]::Create()
try {
  $machineId = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($inputValue))).Replace('-', '').ToLowerInvariant()
} finally { $sha.Dispose() }
[pscustomobject]@{ MachineId = $machineId; Edition = $cv.EditionID; Build = [uint32]$cv.CurrentBuild; UBR = [uint32]$cv.UBR }
```

## Local approval manifest template

This is a template, not an approval. It must remain ignored and local. The
machine ID must be the 64-character lowercase value computed in the guest.

```json
{
  "schemaVersion": 2,
  "vms": [
    {
      "scenarioId": "win11-pro-24h2-clean-local",
      "vmName": "<approved Hyper-V VM name>",
      "generation": 2,
      "checkpointName": "deslopper-clean-win11-pro-24h2-local-v1",
      "expectedMachineId": "<64 lowercase hexadecimal guest fingerprint>",
      "expectedEdition": "Professional",
      "expectedBuildFamily": "24H2",
      "accountClass": "local-administrator",
      "domainJoined": false,
      "mdmEnrolled": false,
      "disposable": true,
      "approvedOperations": [
        "set_widgets_visibility",
        "set_task_view_visibility",
        "set_show_desktop_corner"
      ],
      "approvedTargetStates": {
        "set_widgets_visibility": ["enabled", "disabled"],
        "set_task_view_visibility": ["enabled", "disabled"],
        "set_show_desktop_corner": ["enabled", "disabled"]
      },
      "evidenceExportLocation": "C:\\DeslopperValidation\\.deslopper\\local\\live-evidence",
      "checkpointRestoreProcedure": "tools/Invoke-DeslopperHyperVValidation.ps1 -Action RestoreCheckpoint -ScenarioId win11-pro-24h2-clean-local -ConfirmVmStateChange"
    }
  ]
}
```

## Build, checkpoint, and payload commands

Run these from the repository on the host after the manifest is complete:

```powershell
bun run tauri build --debug --no-bundle --features mutation-alpha
powershell -NoProfile -ExecutionPolicy Bypass -File tools\Invoke-DeslopperHyperVValidation.ps1 -Action Inventory
powershell -NoProfile -ExecutionPolicy Bypass -File tools\Invoke-DeslopperHyperVValidation.ps1 -Action ConfirmCheckpoint -ScenarioId win11-pro-24h2-clean-local
powershell -NoProfile -ExecutionPolicy Bypass -File tools\Invoke-DeslopperHyperVValidation.ps1 -Action CopyPayload -ScenarioId win11-pro-24h2-clean-local -ConfirmVmStateChange
```

Inside `C:\DeslopperValidation` in the guest, create the fixed scenario manifest:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\New-DeslopperLiveScenarioManifest.ps1 -ScenarioId win11-pro-24h2-clean-local -CheckpointId deslopper-clean-win11-pro-24h2-local-v1 -AccountClass local -ManagementContext none
```

Compare its fingerprint, edition, build, and checkpoint with the local approval
manifest. Then start `win-deslopper.exe` with the exact launch arguments printed
by the tool. The developer must visually confirm the guest environment banner
before acknowledging the live-validation warning.

## Per-operation protocol

Run one operation/target pair at a time, beginning with Widgets `disabled`.
For each approved operation, exercise `disabled` and `enabled` separately:

1. Inspect and record the initial representation, detector result, Settings UI,
   and taskbar behavior.
2. Generate and approve one plan. Apply through the mutation broker and record
   the transaction, verification result, and least disruptive refresh needed.
3. Restart Deslopper and verify durable history/recovery state.
4. Approve rollback and verify the exact original representation, detector
   result, Settings UI, and user-visible behavior.
5. Change the setting externally and verify the old plan becomes stale and a
   new plan/approval is required.
6. Exercise the closed failure, rollback-conflict, and recovery cases described
   in `docs/vm-mutation-protocol.md`. Never terminate Explorer to make a handler
   appear successful.
7. Export the typed evidence bundle through the application. Use only redacted
   warning text and labels permitted by `docs/live-evidence-format.md`.

If a handler fails, record the truthful failure/limitation and continue to the
next independent operation unless safety, rollback, guest integrity, or the
shared broker is compromised. Restore the checkpoint before every independent
rerun.

## Evidence handoff and restore

Use an explicit user-initiated transfer to copy the guest's JSON bundles from
`C:\DeslopperValidation\.deslopper\local\live-evidence` to the host's ignored
`validation\inbox\win11-pro-24h2-clean-local` directory. Do not transfer raw
registry exports, profile paths, usernames, computer names, tokens, emails,
SIDs, or screenshots containing personal data.

On the host:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\Invoke-DeslopperHyperVValidation.ps1 -Action CollectResults -ScenarioId win11-pro-24h2-clean-local
powershell -NoProfile -ExecutionPolicy Bypass -File tools\Invoke-DeslopperHyperVValidation.ps1 -Action RestoreCheckpoint -ScenarioId win11-pro-24h2-clean-local -ConfirmVmStateChange
powershell -NoProfile -ExecutionPolicy Bypass -File tools\Invoke-DeslopperHyperVValidation.ps1 -Action ConfirmCheckpoint -ScenarioId win11-pro-24h2-clean-local
powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-vm-fixtures.ps1
```

Only after the evidence passes schema/replay validation may the corresponding
matrix row and handler maturity be changed. Repeat this workflow for Home,
Enterprise Evaluation, 25H2, managed-policy, and standard-user coverage rather
than cloning an unverified result across scenarios.
