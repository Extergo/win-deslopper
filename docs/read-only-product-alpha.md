# Read-Only Product Alpha

## Product boundary

Deslopper Product Alpha is a local, unelevated Windows inspection application.
It observes a fixed catalogue of 20 Windows components, explains uncertainty,
stores redacted history locally, and lets a user describe a desired state. It
does not optimise, debloat, repair, remove, restore, or modify Windows.

The normal binary does not compile or register mutation commands. The main
window has only Tauri event listen/unlisten capability for inspection progress.
There is no shell, filesystem, network, updater, authentication, telemetry,
plugin, backend, or elevation permission.

## First-run and daily use

First run presents a compact boundary explanation:

- inspection is read-only;
- unknown does not mean broken;
- permission-limited does not mean absent;
- results and redacted history stay local;
- desired states and previews do not execute anything.

The user can start an inspection or continue to the dashboard. The dashboard
reports freshness, drift, managed states, unknown/permission-limited results,
and detector failures without turning uncertainty into a health score.

The component catalogue exposes all 20 Rust-owned component definitions. It
supports search and filters for category, state, freshness, management,
confidence, applicability, desired-state difference, and drift. Details show
current state, authority, confidence, applicability, completeness, warnings,
errors, restart implications, redacted evidence, a local timeline, and the
trade-offs of a possible desired state.

## Desired states and previews

A desired state is a local comparison preference, not an instruction to
Windows. Rust validates allowed values and records revisions. Previews contain
the current and desired values, authority, mechanism, restart implications,
known risk, rollback considerations, source inspection, generation time, and
schema version. Every Product Alpha preview declares that execution is
unavailable and creates no approval nonce, transaction, or mutation plan.

## Drift and history

Drift is calculated only from usable observations. Failed and cancelled
detectors are excluded. Normal package version servicing is not treated as
configuration drift. A user can mark a drift event reviewed; review does not
claim resolution. A later observation that returns to the desired state marks
the event resolved and records that return explicitly.

History provides inspection snapshots, component timelines, package details,
and comparison between snapshots from the same machine identity. Different
machine identities are rejected rather than presented as ordinary drift.

The default retention period is 180 days. The user can select 0, 30, 90, 180,
or 365 days. Pruning always preserves the latest snapshot. Clear local history
requires confirmation and removes only read-only product records; internal
feature-gated audit tables are not erased.

## Diagnostics and privacy

Diagnostics are opt-in and save only after the user reviews the included
categories and chooses a destination. The allowlisted JSON contains product and
build information, non-identifying Windows version data, inspection summary,
detector status, redacted errors/evidence, schema versions, capability summary,
and explicit privacy flags.

Diagnostics exclude machine identity, hostname, usernames, emails, account
identifiers, profile and OneDrive paths, filenames, package install paths,
development-host protection, scenario or approval records, approval nonces,
credentials, recovery material, raw command output, and the local database.
There is no automatic or network upload.

## Packaging and support statement

The normal package version is `0.1.0-alpha.1`. Windows packaging produces an
NSIS installer. It is unsigned, so Windows may show an unknown-publisher
warning. The installer uses Tauri's WebView2 download-bootstrapper mode when a
compatible runtime is missing. Uninstalling the application does not promise to
remove user data; local history can be cleared from Settings before uninstall.

This alpha is intentionally narrow. It is not production-ready and does not
claim coverage of every Windows edition, build, management configuration, or
component representation. Detector limitations and permission failures remain
visible instead of being silently converted into conclusions.

## Validation status

The read-only engine has synthetic/redacted fixtures and automated Rust,
frontend, schema, privacy, capability, and package-boundary tests. Internal
mutation research remains a separate compile-gated surface: all three handlers
are synthetic-tested and zero are live-validated.

On 2026-08-05 the normal feature-off release binary was launched on the
original development PC without mutation flags. Keyboard navigation reached
the inspection action and two real read-only runs completed 20/20 detectors
with 14 warnings, 0 failed detectors, 13 unknown/permission-limited results,
and 0 detected drift. The app showed no UAC prompt, persisted the results, and
reported partial evidence explicitly. The smoke found and fixed modal focus
escape and stale terminal progress copy before packaging.

The previously documented Windows 11 Pro 25H2 physical laptop was sold before
approval or any mutation attempt. It is withdrawn and unavailable; this is not
a mutation failure. Any future mutation research requires a new disposable
target, recovery evidence, explicit approval, and separate authorization. The
original development PC remains permanently denied.
