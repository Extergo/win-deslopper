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

## Guest preparation

The exact developer handoff, schema-version-two approval template, checkpoint
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

## Physical-laptop preparation

The 2026-08-03 Windows 11 Pro 25H2 laptop pass is documented in
`physical-validation-target-readiness.md`. Its local draft remains pending,
recovery-not-ready, unapproved, and ignored. The original development-host
hash is unavailable on the laptop and must later be exported from the original
PC as a hash only. Zero live mutation tests were attempted.

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
