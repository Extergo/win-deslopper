# Physical validation-target readiness

> **Withdrawn 2026-08-05:** The laptop was sold before mutation approval or
> any mutation attempt and is no longer available. This is historical read-only
> preparation evidence, not a mutation failure, approval, or completed target.
> No local draft, audit record, or earlier preparation state authorizes work on
> a replacement device.

The first real Windows 11 Pro 25H2 physical-laptop pass was performed on
2026-08-03. This is read-only preparation evidence, not a live mutation target
approval. No live mutation, mutation-alpha launch, mutation runtime flag,
taskbar registry write, or approved live scenario was used.

## Retirement

On 2026-08-05 the physical target was withdrawn because the hardware is being
sold. Its committed state is `retired_before_mutation`, `target_withdrawn`,
`read_only_validated`, `not_approved`, and `mutation_not_attempted`, with zero
live mutation scenarios. It is excluded from completed targets, failed mutation
targets, approved targets, and evidence that any handler works live.

The useful read-only conclusions remain: Windows 11 Pro 25H2 inspection was
validated; minimal Tauri event permissions were corrected; the OneDrive
detector was corrected; MDM evidence handling was corrected; and AppX
permission-limited behavior remained explicit. No mutation occurred.

## Reactivation preparation

On 2026-08-05 the user explicitly superseded the operational retirement status
without erasing the retirement event. The target is temporarily
`reactivation_preparation` for `widgets_visibility_preparation_only` before its
final reset. It remains `not_approved`, `mutation_not_attempted`, and at zero
live scenarios. Task View and Show Desktop are outside this preparation scope.

The preparation reached `ready_for_approval_review` without creating an
approval or live scenario. A fresh ignored review draft is limited to the fixed
Widgets visibility handler, one plan, one execution, and no more than 30
minutes. The governed integration upgrades that non-authorizing local draft to
schema version 3 under committed policy schema version 2 and requires strict
physical recovery, unmanaged-state, restart-clear, identity-separation, and
`reset_before_sale` evidence. Its pre-state records
an absent `TaskbarDa` value, no configured Widgets policy, a conservative
detector result of unknown, and the user's confirmation that the Widgets button
is not visible. The proposed opposite state is enabled; exact rollback is value
absence. The draft prohibits mutation until a separate approval decision and
also prohibits automatic repair, elevation, Explorer termination, other
handlers, and generic registry paths.

## Scoped approval correction

The first execution handoff stopped because the version 1 approval model
required all three handlers and both directions. Stretching that approval into
Widgets-to-enabled-only authority would have violated the requested scope, so
the fail-closed stop was correct and no mutation occurred.

The additive version 2 model now supports a unique, non-empty target set per
explicit operation. For this preparation the only representable scope is the
fixed Widgets visibility handler targeting enabled, with maximum one plan,
maximum one execution, and expiration no later than 30 minutes. Task View, Show
Desktop, and standalone Widgets-disabled authority are absent. Exact restoration
to the original absent representation is authorized only by a consumed durable
transaction, not by adding disabled to the approval.

The version 1 format remains deliberate legacy virtual-machine compatibility.
Version 2 preserves scoped VM approval. Physical live validation now requires
version 3, which retains exact source commit, inspection/evidence, handler,
platform, identity, development-host denylist, operation, and direction
bindings while adding strict recovery route/media, alternate-device, restart,
BitLocker, unmanaged domain/Entra/workplace/MDM, automatic-repair,
final-plan-approval, reimage, short-expiry, and final-disposition evidence.

This governance change does not approve the laptop or any operation. The sold/
withdrawn retirement event remains immutable history, and the later
reactivation remains a separate preparation event. Widgets remains
`ready_for_approval_review`, unexecuted, unapproved, and at zero live scenarios.
Task View and Show Desktop remain outside the proposed scope.

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

After the laptop restart and management-query correction, exactly one further
normal feature-off inspection was run. Inspection
`inspection-1785763235399-1` completed all 20 detectors in 26,777 ms: 9
successful, 11 unknown, 0 failed, 0 cancelled, 14 warnings, and 0 errors. It
contained no partial observations. Closing and reopening the normal app
preserved all three saved runs; the newest history row retained its duration,
counts, build 26200 Professional, and Domain/Workplace/MDM `no` summary.

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
CBS RebootPending, Windows Update RebootRequired, pending file renames, pending
computer rename, pending domain join, and the Windows Update COM reboot flag
were all clear. `SessionsPending` contains completed CBS session history only:
its top-level counters are zero and each retained child session is complete.
The final restart-pending state is therefore clear.

The user subsequently confirmed there is no important personal data, so backup
is not applicable; the laptop is expendable; reinstall is acceptable; approved
Windows USB media, another working computer, and an external drive are
available; and a dedicated local test account may be created later. BitLocker
has never been used and is not planned; the audit independently found the OS
volume fully decrypted. Core recovery conditions are ready, but the repository
classification stays `not_ready` while original development-host protection is
missing.

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
identity was transferred through removable media, validated against its
replacement checksum and closed field set, and imported only into the ignored
local denylist. It is distinct from this target and must never be committed.

The historical retirement remains in the target lifecycle. The later
reactivation event is preparation-only and does not convert the target into an
approval or live scenario. The physical row in `validation/matrix.json` records
zero live mutation scenarios and contributes no completed, failed, approved, or
live-handler evidence.

The earlier MDM result was a detector false positive. Three provider-backed
Enrollment registry records contained only metadata: `dsregcmd` reported no
Azure AD, enterprise, domain, or workplace join; no matching OMADM account,
EnterpriseMgmt task, or MDM certificate thumbprint existed. The management
query now keeps the metadata count separate and classifies MDM only when an
enrollment record is corroborated by one of those active artifacts. The
post-restart inspection reports MDM false and attributes zero component states
to MDM. No raw enrollment identifiers are persisted.
