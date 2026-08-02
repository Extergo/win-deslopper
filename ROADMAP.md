# Deslopper roadmap

This roadmap records sequencing, not authorization. `AGENTS.md`, `SECURITY.md`,
`ARCHITECTURE.md`, and `.deslopper/policy.toml` remain authoritative.

## Current milestone — internal mutation broker alpha

**Status:** Implemented behind internal gates; real mutation VM validation remains pending.

The beta provides bounded, non-elevated Windows inspection for the complete
20-component catalogue; structured applicability, package, authority, and
precedence evidence; SQLite history and migration; desired-state recording;
drift inference; cancellation and progress; fixture replay; and dedicated
history and drift UI. The application does not mutate Windows, elevate,
download code, or execute arbitrary commands.

The checked-in fixture is continuously replayed by Rust tests. The broader
Windows Home/Pro/Enterprise, domain, MDM, OneDrive, servicing, and
permission-limited matrix is defined in `validation/matrix.json` and must be
captured on real VMs before platform support claims are promoted. See
`docs/vm-validation.md` and `validation/report.md`.

The alpha adds exactly three current-user taskbar operations behind compile,
debug, CLI, acknowledgement, inspection, plan, and approval gates. Search mode
and Search Highlights were replaced with documented Task View and Show Desktop
settings. Normal-build mutation remains prohibited. See
`docs/mutation-broker-alpha.md` and `docs/vm-mutation-protocol.md`.

All mutation results remain synthetic and no handler is production-ready. Zero
live scenarios are complete and no disposable machine is currently approved.
The next intended validation target is a separately backed-up and explicitly
approved spare laptop after cloning the private checkpoint; its identity must
not be recorded in the repository. Production mutation remains disabled until
the required live-validation matrix passes.

## Next — beta validation and hardening

- Capture and review every required VM scenario without changing detector code
  merely to fit a fixture.
- Expand frontend rendering and accessibility tests for inspection progress,
  history, drift filters, and narrow layouts.
- Exercise database lock, disk-full, interrupted migration, and cancellation
  behavior in packaged Windows builds in addition to deterministic unit tests.
- Keep the detector matrix and applicability source review dates current.
- Add Windows CI for formatting, Clippy, Rust tests, frontend checks, fixture
  replay, and a debug Tauri build.

## Later — compatibility and preview planning

Convert eligible, high-confidence observations into immutable proposed-change
plans. Plans must use typed operations, state exact prerequisites and rollback
sources, and remain unavailable for unsupported, unknown, failed, cancelled,
permission-incomplete, or externally controlled states. This milestone still
does not authorize Apply.

## Later — rollback and privileged-operation design

Design original-state capture, immutable receipts, result verification,
partial-failure handling, rollback verification, and a narrow authenticated
helper protocol. Architecture and security review are required before any
implementation. The main UI must remain unelevated.

## Later — first reversible Windows operation

Only after explicit approval and policy changes, implement one documented,
allow-listed, verifiable, reversible operation. Broad AppX removal, wildcards,
arbitrary scripts, Explorer termination, and generic “debloat” execution are
not acceptable first operations.

## Later — extensions foundation

An extension system requires out-of-process isolation, signed packages,
versioned protocols, explicit capabilities, verified updates, and revocation.
It must never grant arbitrary shell or administrator access.

## Sequencing rules

- Inspection failure is never absence.
- Do not begin mutation before compatibility planning and rollback design are
  complete and explicitly approved.
- Do not weaken gaming, anti-cheat, Xbox, Game Pass, or Windows security
  compatibility to increase removal coverage.
- Keep milestones small enough that safety claims can be tested.
