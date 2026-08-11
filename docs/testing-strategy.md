# Testing strategy

Rust tests cover lifecycle ordering and counts, cancellation during shared AppX
inventory, double cancellation, fresh inspection after cancellation, query
sharing, permission-limited package scope, exact identity migration,
multi-package aggregation, authority alternatives, applicability by
edition/build/servicing prerequisite, drift classification and causes, schema
creation and v1-to-v4 migrations, unsupported newer schema, package/lifecycle
round-trip, desired revisions, retention, read-only clearing with mutation-audit
preservation, catalogue completeness, closed diagnostics, alpha JSON salvage,
import rollback, and privacy redaction.

Vitest covers the typed command contract, progress subscription, cancellation,
history/timeline/package routing, desired-state validation/save payloads,
onboarding copy, honest status summaries, catalogue filters, freshness,
permission-limited display, non-executable previews, diagnostics/save
boundaries, keyboard-native controls, the closed Widgets and Task View owner
requests, and the absence of Show Desktop or arbitrary registry input in normal UI.
Svelte checking, ESLint, Prettier, static production build, Rust
formatting/check/Clippy/tests, normal and all-feature compile/test, debug and
release Tauri builds, installer packaging, interactive launch, and visual
review remain release gates.

Capability regression tests require exactly event listen/unlisten in the normal
window and reject event emission or broader capabilities. Frontend listener
tests cover progress, terminal completion, denial, and cleanup. Persistence
round-trip tests preserve inspection start/completion duration. OneDrive tests
cover missing branches, unlinked clients, null values, 25H2, non-zero execution,
and redaction. Parser self-tests cover recovery-audit variants, hash-only
identity export, malformed/empty denylist rejection, and source-only script
parsing without mutating Windows.

`validation/fixtures` contains deterministic redacted Windows captures.
`validation::tests::every_checked_in_vm_fixture_matches_current_detectors` loads
every bundle, replays production parsers/detectors, and fails on expected-state
regression. `tools/validate-vm-fixtures.ps1` produces the checked-in report.
Fixture coverage never replaces manual verification of Windows UI, policy
provenance, OneDrive client state, and non-elevated permission behavior.

The Windows 11 matrix covers Home, Pro, and Enterprise/Evaluation on 24H2 and
25H2; local, Microsoft, domain, and MDM authority contexts; OneDrive states;
feature and Store servicing; package registration/provisioning; and
permission-limited standard-user inspection. Windows 10 is optional legacy
observation only.

Mutation-alpha tests run separately with `cargo test --all-features`. They
cover closed requests, gates, the exact registry, source/context rejection,
apply failure, verified apply, exact rollback, durable history, cancellation,
and interrupted-write recovery. Synthetic mutation fixtures replay state only
and must state that no live write occurred. Fault injection is Rust-test-only.
All-feature test or compile commands must never be paired with mutation runtime
flags on the development host.

Default-feature Rust tests exercise Owner Mode entirely through fake backends.
They cover Widgets/Task View-only registration, unsupported/managed/unknown refusal,
no-op behavior, durable pre-state before write, exact apply and detector
verification, actual-state capture after write errors, automatic restoration,
rollback failure, unusable-Undo prevention, exact DWORD/absence restoration,
conflict refusal, relaunch then Undo, tamper resistance, and stable
machine-plus-user scope. Task View tests cover hidden/shown/default/invalid and
policy-controlled detection, apply, relaunch-safe Undo, and exact restoration.
Widgets tests cover PermissionDenied unchanged classification and scope-local
capability suppression. Automated tests never write the live registry.

Vitest also exercises the internal UI workflow with a mocked typed backend:
feature-off and development-host denial, missing approval, Widgets-enabled-only
scope, single-operation plan defense, expiry/elevation/Explorer-termination
rejection, exact phrase approval, journaled execution phases, manual visual
confirmation without a second write, transaction-bound absence rollback,
terminal reuse prevention, history redaction, and plain-language safety errors.
The Owner Mode source regression confirms that the engineering entry point is
selected only by exact Rust build metadata and that the normal page calls only
the closed Widgets and Task View Apply/Undo APIs.

Live-validation hardening tests cover the separate gate, development-host
refusal, scenario/machine/checkpoint/database mismatch, validation maturity,
rejected handlers, three-dimensional visual outcomes, Settings disagreement,
shell-ignore and reversion outcomes, evidence redaction/duplicates, and
cross-build comparison. Checked-in live evidence is validated for closed
operation IDs and redaction; an empty evidence directory truthfully means zero
completed live runs.

Generic validation-target tests reject incomplete physical recovery fields,
pending preparation, expired approval, missing development-host identity, and
wrong type/build/identity. Virtual-machine approval and the legacy VM inventory
schema remain supported. The physical lifecycle preserves retirement followed
by preparation-only reactivation. Neither event contributes to completed or
failed live-mutation counts, approved-target counts, or evidence that a handler
works live.

The separate schema-v3 physical Widgets approval-review draft is
non-authorizing and binds committed policy schema version 2. It fixes
the scope to one Widgets visibility handler, requires the opposite of the
user-confirmed visible state, preserves the exact original representation for
rollback, expires within 30 minutes, and requires mutation, elevation, Explorer
termination, automatic repair, and generic registry paths to remain disabled.
It also requires strict recovery, important-data, restart, unmanaged-target,
identity-separation, and reset-before-sale evidence while excluding approval
timestamps, plans, nonces, and transactions.

Preparation-schema coverage requires structured states for all nine user-only
recovery confirmations, restart verification, redacted recovery audit summary,
toolchain state, missing development-host protection, and the invariant that
mutation remains prohibited. Identity-export and denylist-validator parser
self-tests prove hash-only output plus empty, malformed, and target-equal
rejection without displaying an identity.

Scoped version 3 physical tests accept Widgets-to-enabled only and reject empty or
duplicate scopes/states, unknown operations/states/fields, wrong handler/source/
inspection/evidence bindings, expiration, and exhausted plan or execution
allowances. Policy tests reject missing, malformed, legacy VM-only, empty,
duplicate, and unknown target-type configuration. Physical tests reject every
missing data, recovery, restart, encryption, management, automatic-repair,
final-plan, expiry, reimage, and disposition requirement. Version 2 VM
compatibility is tested separately. Broker tests prove that options contain only the approved operation
and direction, direct calls reject every unapproved operation/direction pair,
one-use plans and nonces cannot replay, and exact rollback derives only from the
durable transaction. All apply/rollback coverage is synthetic; no test command
uses live runtime flags.

The non-authorizing review draft now expires within 30 minutes and carries one
Widgets-enabled scope plus one-plan/one-execution limits. It creates no plan,
transaction, nonce, approval, or live manifest.

`tools/scan-mutation-boundary.ps1` rejects generic registry/process surfaces and
prohibited Windows operations, proves the default production feature is Owner
Mode, checks the normal command registration, and verifies that raw registry
tokens and non-product operations do not enter the normal page. Live Owner Mode
smoke testing follows `docs/owner-mode-widgets-smoke-test.md`; CI never performs
mutation. The separate engineering harness continues to follow
`docs/vm-mutation-protocol.md`.
