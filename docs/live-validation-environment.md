# Live-validation environment and host safety

Live mutation remains prohibited on the development host. The ignored
`.deslopper/local/development-host-denylist.json` contains only a SHA-256 host
fingerprint; it contains no computer name or other identifying source value.
The application fails closed when the denylist is absent or invalid.
An empty denylist is invalid; it is never equivalent to development-host
protection.

## Development-host identity transfer

Run the fixed export only on the original development PC:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Export-DeslopperValidationIdentity.ps1
```

It creates ignored `.deslopper/local/validation-identity-export.json` with
exactly this shape and no hostname, username, serial, or other source value:

```json
{
  "schemaVersion": 1,
  "developmentHostFingerprints": ["<64 lowercase hexadecimal hash>"]
}
```

Transfer that file privately to the validation laptop without pasting the hash
into chat. Store it only as
`.deslopper/local/development-host-denylist.json`, then run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Test-DeslopperDevelopmentHostDenylist.ps1
```

The validator displays no hash. It rejects a missing, empty, malformed,
duplicate, or extra-field denylist and rejects equality with the current target
identity. Both files remain ignored and must never be committed or pushed.

Every live target also requires ignored
`.deslopper/local/approved-validation-target.json`, validated against
`validation/approved-validation-targets.schema.json`. This is distinct from a
preparation draft. The live gate rejects pending or expired approval, missing
recovery/user confirmations, a target/build/identity mismatch, or a target
identity equal to the development-host denylist. The legacy approved-VM schema
remains supported by Hyper-V host tooling.

## Scoped approval formats

The generic version 1 target schema remains readable only through the
deliberate legacy virtual-machine path. Scoped version 2 is also VM-only
compatibility. Physical live validation requires
`validation/approved-validation-targets-v3.schema.json`. Version 3 retains
operation-specific target states, matching handler versions, exact source/
inspection/evidence and denylist binding, and positive bounded plan/execution
limits. It additionally requires explicit important-data status, recovery
route and media, another recovery-capable device, clear restart state, safe
BitLocker state, unmanaged domain/Entra/workplace/MDM state, disabled automatic
repair, final-plan approval, a maximum 30-minute approval lifetime, reimage
procedure, and final disposition. A missing scope never authorizes compiled
handlers.

The committed live-validation policy is parsed in internal mutation builds and
explicitly allowlists `virtual_machine` and `physical_laptop`. A missing,
malformed, legacy VM-only, empty, or unknown-type policy fails closed. Policy
permission is necessary but never sufficient: it creates no approval.

`tools/New-DeslopperLiveScenarioManifest.ps1` therefore requires at least one
explicit `-OperationScope`, such as
`set_taskbar_widgets_visibility=enabled`, and emits the additional
`--expected-source-commit=<commit>` launch binding. Unknown, empty, or duplicate
operations and states fail closed.

The first physical execution handoff stopped correctly because version 1 could
not express the explicit Widgets-to-enabled-only boundary. Version 2 corrected
that scope limitation; version 3 carries it into the stricter physical-target
model without approving or executing the test. The intended
review draft is one Widgets scope, enabled only, at most one plan and one
execution, and no more than 30 minutes of review lifetime. Zero live scenarios
remain completed.

The physical review-draft schema is version 3 and records policy schema 2,
strict unmanaged/recovery/restart/data/disposition evidence, explicit
non-authorizing and execution-disabled states, exact source inspection and
commit bindings, and transaction-bound restoration of the captured pre-state.
The generator independently rejects a live approval manifest, a matching
development-host identity, active management, pending restart, a configured
Widgets/taskbar policy, stale recovery evidence, or a changed Widgets
representation. The ignored draft cannot be renamed into an approval.

## Guest preparation

The exact developer handoff, VM approval template, checkpoint
commands, and evidence transfer steps are in `docs/manual-vm-provisioning.md`.

1. Provision a licensed disposable Windows VM manually and add it to the local,
   ignored approved-VM inventory. Never download media or create a VM through
   Deslopper tooling.
2. Restore the approved checkpoint.
3. Run a normal read-only inspection in the guest so its SQLite database is
   bound to that guest.
4. Copy the development-host denylist into the guest working directory.
5. Supply a separate, explicitly approved validation-target manifest locally.
   A preparation record cannot be renamed or reused as approval.
6. From the guest checkout, run `tools/New-DeslopperLiveScenarioManifest.ps1`.
   The tool refuses a fingerprint present in the denylist and writes the fixed
   ignored `.deslopper/local/live-validation-scenario.json`.
7. Launch the internal build with all values printed by the tool:

   `--enable-mutation-alpha --enable-live-validation --validation-scenario=<id> --expected-machine-id=<fingerprint> --expected-checkpoint-id=<checkpoint>`

8. Confirm the visible environment banner before acknowledging the warning.

The live gate requires the CLI flag, selected scenario, checkpoint identifier,
machine fingerprint, edition, build, optional UBR, guest-owned database,
denylist, normal alpha gates, and explicit UI acknowledgement. VM detection is
informational only and cannot make an environment eligible.

The original development PC may perform development, compilation, read-only
inspection, and synthetic tests, but it is permanently ineligible for live
mutation. The local denylist overrides policy, approval, target type, and CLI
arguments and has no bypass.

## Physical-laptop preparation

The 2026-08-03 Windows 11 Pro 25H2 laptop pass is documented in
`physical-validation-target-readiness.md`. On 2026-08-05 the target was
`retired_before_mutation` and `target_withdrawn` because the hardware is being
sold. It remained read-only validated, unapproved, and mutation not attempted;
zero live mutation scenarios exist. A later explicit event temporarily
reactivated it for Widgets-only preparation before the final reset. The
retirement remains historical. Preparation reached approval-review readiness,
but reactivation is still unapproved, permits no mutation, and does not count as
completed, failed, approved, or live-handler evidence.

## Optional Hyper-V host tooling

`tools/Invoke-DeslopperHyperVValidation.ps1` reads only the ignored
`.deslopper/local/approved-validation-vms.json`. It can inventory approved VMs,
confirm/create/restore one named checkpoint, and copy the fixed debug binary,
manifest, and denylist. State-changing actions require
`-ConfirmVmStateChange`. It never creates VMs, downloads media, starts the
guest, acknowledges Deslopper, or invokes mutation. Result collection only
lists manually transferred JSON from the fixed scenario inbox.

The 2026-08-02 read-only local audit confirmed that the development-host
fingerprint is denied, no Deslopper process or live scenario manifest targets
the host, and no approved VM inventory exists. The Windows hypervisor layer is
present, but Hyper-V management commands, registered guests/checkpoints,
Windows Sandbox, VMware, VirtualBox, libvirt, Multipass, and supplied VM media
were not available. The largest detected free volume had about 48 GiB, below
the runbook's safe provisioning allowance. No VM was created, started, or run,
and no live mutation was performed.

## Guest runner

The ordinary feature-gated Tauri application is the guest runner. There is no
second handler path. It uses normal inspection, plan generation, approval,
broker execution, detector verification, history, conflict handling, rollback,
and recovery. The added evidence form records typed manual visual/Settings
observations only after a real transaction. It cannot provide commands, paths,
registry data, or operation parameters.
