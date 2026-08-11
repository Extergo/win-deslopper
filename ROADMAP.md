# Deslopper roadmap

This roadmap records sequencing, not authorization. `AGENTS.md`, `SECURITY.md`,
`ARCHITECTURE.md`, and `.deslopper/policy.toml` remain authoritative.

## Current milestone - Owner Mode M3: Current-user cleanup pack

**Status:** Implemented and packaged as `0.3.0`; physical validation is pending
the manual smoke protocol. Automated implementation and verification performed
no live Windows mutation.

M3 productizes four existing current-user findings alongside the M1/M2 taskbar
operations: welcome experience, tips and suggestions, notification suggestions,
and suggested content in Settings. Each owns one fixed Content Delivery Manager
DWORD plus its matching read-only Cloud Content policy check. Missing and
non-binary representations remain unsupported; policy-owned states are refused.
Lock-screen suggestions remain inspection-only because their current preference
and broader Spotlight policy do not prove exact setting parity.

M2 productized Taskbar Task View visibility alongside the M1 Widgets operation.
Apply is one click and must preflight, capture, journal, write, directly verify,
verify through the matching detector, and recover safely. Undo is one click
when the current state still equals Deslopper's applied state.

The normal build remains unelevated, local, Windows 11 x64/current-user scoped,
and has no generic mutation surface. The physical M1 result established that
Widgets writes are denied unchanged on the tested machine/account, so that
scope is surfaced as Direct change unavailable without elevation. Task View's
same-state probe succeeded and now has a matching detector. Show Desktop and
`TaskbarSd` remain non-productized and untouched.

The first M2 physical sequence verified Task View detection from DWORD 0,
Apply to DWORD 1, direct and detector verification, visible Explorer refresh,
persistence after complete process exit, durable Undo after relaunch, exact
restoration to DWORD 0, and independent final verification. See
`docs/owner-mode-m2-physical-validation.md`.

## Read-only validation and hardening

- Capture Home, Pro, Enterprise, 24H2, 25H2, managed, OneDrive, Store-serviced,
  and permission-limited standard-user fixtures.
- Validate keyboard/screen-reader behavior and 125%/150%/200% scaling on more
  Windows systems.
- Exercise disk-full, locked database, interrupted migration, installer,
  uninstall, and retained-user-data behavior with disposable data.
- Keep detector source review dates and Windows applicability current.
- Add private Windows CI without creating live mutation pathways.

## Internal mutation research remains separate

The explicit engineering harness still contains exactly three closed taskbar operations.
Historical harness evidence remains synthetic. A later Owner Mode physical test
recorded an unchanged Widgets access denial and a successful unelevated Task
View same-state write; those product facts do not graduate harness maturity or
authorize Show Desktop. Any broader work requires a separately authorized
milestone. The original development host remains permanently prohibited by the
engineering harness.

## Sequencing rules

- Inspection failure is never absence.
- Normal package version servicing is not configuration drift.
- Preview state is never represented as completed work.
- No Apply control appears until an explicitly authorized, tested, reversible
  operation reaches the normal product boundary. M3 authorizes exactly the six
  operations listed above; Show Desktop and Taskbar Search remain excluded.
- Do not weaken gaming, anti-cheat, Xbox, Game Pass, or Windows security
  compatibility to increase removal coverage.
