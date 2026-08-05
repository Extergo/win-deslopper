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
boundaries, keyboard-native controls, and the absence of Product Alpha mutation
calls. Svelte checking, ESLint, Prettier, static production build, Rust
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

Live-validation hardening tests cover the separate gate, development-host
refusal, scenario/machine/checkpoint/database mismatch, validation maturity,
rejected handlers, three-dimensional visual outcomes, Settings disagreement,
shell-ignore and reversion outcomes, evidence redaction/duplicates, and
cross-build comparison. Checked-in live evidence is validated for closed
operation IDs and redaction; an empty evidence directory truthfully means zero
completed live runs.

Generic validation-target tests reject incomplete physical recovery fields,
pending preparation, expired approval, missing development-host identity, and
wrong type/build/identity. The physical preparation matrix row never
contributes to completed live-mutation counts. It is explicitly
`withdrawn_sold`: the laptop was sold before approval or mutation, so it is
unavailable rather than failed.

`tools/scan-mutation-boundary.ps1` rejects generic registry/process surfaces and
prohibited Windows operations. Live apply/rollback testing follows
`docs/vm-mutation-protocol.md`; CI never performs mutation.
