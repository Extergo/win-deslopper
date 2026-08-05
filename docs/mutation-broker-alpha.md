# Mutation broker alpha

## Release gates

Mutation commands compile only with Cargo feature `mutation-alpha`. The broker
also requires a debug/internal build, exact `--enable-mutation-alpha` CLI flag,
in-app warning acknowledgement, fresh completed inspection, and valid plan. VM
detection is not a gate. Ordinary builds register no mutation Tauri commands;
the frontend hides the panel when its status command is absent.

Live mutation now additionally requires `--enable-live-validation`, a fixed
local scenario manifest, exact scenario/machine/checkpoint arguments,
edition/build/UBR match, a database owned by the guest, and a local development
host denylist. The internal UI always shows computer name, machine-ID prefix,
platform, informational VM detection, and scenario. All three operations remain
“Internal alpha - not live validated.”

## Scoped live approvals and explicit target policy

Scoped version 2 introduced operation-direction approval and remains an
explicit VM-only compatibility format. Physical live validation requires
version 3. It
binds a non-empty list of explicit operation scopes, with target states listed
per operation, plus handler version, machine/platform, source commit,
inspection/evidence, denylist identity, expiry, maximum plans, and maximum
executions. Version 3 also binds strict physical data, recovery route/media,
alternate-device, restart, BitLocker, management, automatic-repair,
final-plan-approval, short-lifetime, reimage, and final-disposition evidence.
Compiled handlers are never implicitly approved, and omitted scope never means
all operations.

Broker options are derived from the validated scope, but filtering is not the
security boundary. Plan generation, execution, and the final pre-write check
independently require the exact operation/target pair. For version 3 physical
validation the broker re-reads the committed target policy, ignored local
denylist, and ignored local approval and requires the
startup approval ID, bindings, scope, and limits to remain identical and
unexpired. Journal usage is counted per approval ID, preventing unlimited
retries beyond its plan and execution allowances.

Rollback authority is transaction-bound rather than granted by a target state.
An approved Widgets-to-enabled transaction may restore its captured absent
pre-state after approval expiry, while a standalone Widgets-to-disabled plan
remains unauthorized. Recovery retains the existing guest, scenario, source,
platform, database, policy, and denylist checks.

Version 1 remains a deliberate legacy virtual-machine path, and version 2 is a
scoped VM path. Neither is silently promoted into physical-target authority.
The original development-host denial overrides every approval and has no
runtime override. All three
handlers remain not live validated, and zero live mutation scenarios are
complete.

## Closed registry

Exactly three current-user, unelevated operations exist:

| Operation ID | Fixed representation | Verification and rollback |
|---|---|---|
| `set_taskbar_widgets_visibility` | HKCU Explorer Advanced `TaskbarDa`, DWORD 0/1 | Re-read; restore exact DWORD or original absence. Never removes Widgets/Web Experience Pack. |
| `set_taskbar_task_view_visibility` | HKCU Explorer Advanced `ShowTaskViewButton`, DWORD 0/1 | Re-read; restore exact representation. Virtual desktops remain available. |
| `set_taskbar_show_desktop_enabled` | HKCU Explorer Advanced `TaskbarSd`, DWORD 0/1 | Re-read; restore exact representation. Only the far-corner gesture changes. |

All use Microsoft’s [Windows 11 settings reference](https://learn.microsoft.com/windows/apps/develop/settings/settings-windows-11)
and [taskbar guidance](https://support.microsoft.com/windows/experience/personalization/customize-the-taskbar-in-windows).
Paths, names, types, and mappings are compile-time constants. The existing key
must exist; handlers do not create arbitrary keys.

Each mutation-only pre-state capture also performs fixed, read-only policy
checks. Any configured Widgets policy blocks Widgets planning; the documented
`HideTaskViewButton` policy blocks Task View planning at either user or device
scope; and an enabled `NoSetTaskbar` policy blocks all three operations. The
same checks run again immediately before apply, during verification, and before
rollback, so a policy race fails closed even when the broader 20-component
inspection has no catalogue entry for a substitute setting. The fixed policy
locations come from Microsoft Learn's [Start Policy CSP](https://learn.microsoft.com/windows/client-management/mdm/policy-csp-start#hidetaskviewbutton)
and [ADMX Start Menu Policy CSP](https://learn.microsoft.com/windows/client-management/mdm/policy-csp-admx-startmenu#nosettaskbar).

### Required substitutions

The requested Taskbar Search mode remains detectable, but current official
documentation describes its UI modes and a Windows 11 24H2+ device policy; it
does not document `SearchboxTaskbarMode` as a stable current-user programming
surface. No writer was added.

Search Highlights was also substituted. Microsoft documents
`AllowSearchHighlights` as device scope for Pro/Enterprise/Education, backed by
machine policy. It is unsuitable for this unelevated current-user alpha. No
`EnableDynamicContentInWSB` or unofficial SearchSettings write exists. Task View
and Show Desktop are the next two low-risk taskbar settings with explicit
current-user representations. Start recommendations and all prohibited items
remain untouched.

## Plan binding and approval

Plan generation accepts a closed operation ID, closed target, and source
inspection ID. Rust binds machine, source observation, exact current
representation, authority/applicability, build/edition, material evidence,
five-minute expiry, rollback method, sources, privilege, side effects, handler
version, and SHA-256 canonical hash. A nonce is returned once; only its hash is
stored.

Apply accepts plan ID, nonce, and acknowledgement only. The broker reloads and
hash-checks the plan, rejects expiry/replay, confirms the latest stored source,
runs a fresh full inspection, captures pre-state again, and rejects material
changes, including a newly configured fixed policy representation. The plan is
consumed before the fixed write. Saving desired state never executes mutation.

## Lifecycle, verification, rollback, and cancellation

Every transition and redacted step is durable. Closing final review consumes
the plan as `cancelled_before_mutation`. No cancellation endpoint can interrupt
an atomic write or verification.

After a setter returns, the handler re-reads and requires the target. A second
full inspection re-evaluates platform, authority, and applicability. Unverified
writes are never labelled success: `applied` remains a recoverable transitional
state until that full detector pass advances the transaction to
`rollback_available`.

Pre-state captures exact DWORD or absence. Rollback callers cannot submit state.
The broker rechecks machine/build/edition/authority plus the operation-specific
fixed policy paths, detects a later change, and requires explicit conflict
approval before overwrite. Restoration is re-read and hashed, then another
full detector pass re-evaluates authority, applicability, build, edition, and
machine identity after rollback.

## Locking and recovery

A process mutex prevents same-process concurrency. A Windows share-deny lock
adjacent to SQLite prevents concurrent app instances. Plan uniqueness prevents
replay. Startup inspects transitional transactions and marks target present,
pre-state present, or unexpected/uncertain as `recovery_required`; it never
silently repeats apply. There is no worker, task, service, startup repair, or
automatic drift reconciliation.

## SQLite v4 and fault injection

`mutation_plans`, `mutation_transactions`, `mutation_steps`,
`mutation_state_captures`, and `mutation_rollbacks` store typed audit data,
hashes, redacted evidence, and recovery state. No raw command output, account
data, arbitrary path, secret, or plaintext nonce is stored.
Schema v4 adds read-only Product Alpha retention and drift-review fields without
changing or deleting these internal audit tables.

Test-only fault injection covers before/after capture, before/after write,
verification, commit, rollback, and rollback verification. It has no Tauri or
frontend control.

The guest validation runner is this same application path, not a handler
shortcut. Typed manual visual observations can be exported only after a durable
transaction. See `live-validation-environment.md` and
`live-evidence-format.md`.

## Future-operation checklist

Require explicit approval, stable subject ID, current official source,
supported representation, build/edition/authority matrix, typed targets, exact
pre-state, idempotent apply, effective verification, exact verified rollback,
crash recovery, privacy review, unit/fixture/disposable-VM tests, security
review, and a boundary-scan update. Never add a generic registry/command runner.
