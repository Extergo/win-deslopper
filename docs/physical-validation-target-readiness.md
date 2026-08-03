# Physical validation-target readiness

The first real Windows 11 Pro 25H2 physical-laptop pass was performed on
2026-08-03. This is read-only preparation evidence, not a live mutation target
approval. No live mutation, mutation-alpha launch, mutation runtime flag,
taskbar registry write, or approved live scenario was used.

## Read-only application result

The initial laptop inspection persisted successfully, but the frontend stayed
at Preparing because `capabilities/main-window.json` granted no event command.
The generated Tauri 2.11 ACL separates registration and cleanup, so the normal
window now receives only `core:event:allow-listen` and
`core:event:allow-unlisten`. It does not receive the broader event default or
frontend emit permissions. Frontend tests cover registration, phase and
terminal delivery, listener denial, and cleanup; the capability test rejects
any broader normal permission set.

After the correction, the normal feature-off app visibly moved from Preparing
through detector progress to 20/20 terminal completion. Inspection
`inspection-1785741112941-1` completed with partial evidence in 31,449 ms: 9
successful, 11 unknown, 0 failed, and 0 cancelled. Closing and reopening the
normal app preserved both saved runs and the completed counts. The loader now
uses persisted `started_at`, rather than `completed_at`, as the snapshot start
time so the 31,449 ms duration also remains authoritative after restart.

The normal UI exposed no mutation controls. The normal build still compiles no
mutation module and registers no mutation commands. Deslopper remained
unelevated and displayed no UAC prompt.

## OneDrive and AppX

The original fixed `onedrive_metadata` query exited 1 with no JSON. The failing
statement was the startup-value expression: a missing OneDrive value under the
existing Run key caused `Get-ItemPropertyValue` to prevent construction of the
entire object. The fixed query reads the Run key once and treats only the
missing OneDrive property as false. Missing account/policy branches remain
valid absence or unlinked inputs, while non-zero execution, null preference
values, and incomplete per-root evidence remain failed or unknown as
appropriate. No account identity, email, sync-root path, profile path, or file
name is retained. The second inspection reported OneDrive successfully.

The unelevated AppX result remains deliberately incomplete. Current-user
evidence is usable, all-user access is permission-limited, and provisioning
failure is separate. Machine-wide absence is never inferred. Incomplete scope
keeps full-absence desired-state preview unavailable or requiring review; the
normal app is not elevated to improve inventory coverage.

## Recovery audit and readiness

`tools/Invoke-DeslopperRecoveryAudit.ps1` is a separate fixed, read-only audit
tool. It requires ordinary interactive UAC and writes only a redacted record
under ignored `.deslopper/local`. It never requests key protectors, protector
IDs, recovery passwords, key packages, TPM owner material, product keys,
serials, or account identity.

The 2026-08-03 audit found Windows RE enabled, a recovery partition present,
and an obvious built-in recovery route. The operating-system volume was fully
decrypted: 0% encrypted, BitLocker protection off, and no encryption method.
CBS and Windows Update reboot flags were clear. The existing pending-file-
rename queue means Windows still reports a restart pending.

Recovery readiness remains `not_ready`. The tool cannot confirm important-data
status, backup completion, expendability, reinstall acceptance, recovery-media
availability, access to another media-creation device or external drive, or
permission to create a later dedicated local test account.

## Preparation versus approval

`validation/approved-validation-targets.schema.json` supports
`virtual_machine` and `physical_laptop`. The legacy approved-VM inventory schema
remains available for Hyper-V tooling. A generic record is explicitly either
`preparation` or `approval`. Preparation must stay pending and cannot authorize
mutation.

An approved physical target requires exact hashed identity, edition/build/UBR,
account and management state, source checkpoint commit, handler versions,
closed operations and target states, ignored evidence location, recovery
readiness, important-data and backup confirmations, reinstall acceptance,
WinRE verification, safe BitLocker recovery state, recovery media,
expendability, restore/reimage procedure, explicit approval timestamp,
expiration, and validation maturity.

The live gate additionally requires an ignored
`.deslopper/local/approved-validation-target.json`. It rejects a preparation
draft, incomplete recovery fields, an expired approval, a wrong target
type/build/identity, an empty development-host denylist, or a target identity
equal to a denylisted development-host identity. The original development-host
hash is not available on this laptop and must be transferred locally from the
original PC; it must never be committed.

The local physical-laptop draft and recovery audit remain ignored. Its status
is `preparation_incomplete`, `recovery_not_ready`, `not_approved`, and
`mutation_not_attempted`. The physical preparation row in
`validation/matrix.json` does not count as a completed mutation target.

The inspection reported MDM evidence even though the separate direct domain,
Entra workplace, and MDM checks did not find enrollment. That discrepancy is
recorded in the ignored draft and must be resolved before any approval; the
preparation record does not convert either observation into an authoritative
management classification.
