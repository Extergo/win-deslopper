# Deslopper roadmap

This roadmap records sequencing, not authorization. `AGENTS.md`, `SECURITY.md`,
`ARCHITECTURE.md`, and `.deslopper/policy.toml` remain authoritative.

## Current milestone - Owner Mode M1: Widgets

**Status:** Implemented and packaged; manual Windows smoke test pending.

M1 productizes only Taskbar Widgets visibility in the ordinary release. Apply
is one click and must preflight, capture, journal, write, directly verify, verify
through the matching detector, and recover safely. Undo is one click when the
current state still equals Deslopper's applied state.

The normal build remains unelevated, local, Windows 11 x64/current-user scoped,
and has no generic mutation surface. Task View and Show Desktop remain
non-actionable until matching detectors exist.

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
All evidence is synthetic and zero handlers are live validated. The former
physical target was withdrawn because it was sold before approval or mutation;
no mutation failure occurred. Any future work requires a new disposable target,
complete recovery evidence, explicit approval, and a separately authorized
milestone. The original development host is permanently prohibited.

## Sequencing rules

- Inspection failure is never absence.
- Normal package version servicing is not configuration drift.
- Preview state is never represented as completed work.
- No Apply control appears until an explicitly authorized, tested, reversible
  operation reaches the normal product boundary. M1 authorizes Widgets only.
- Do not weaken gaming, anti-cheat, Xbox, Game Pass, or Windows security
  compatibility to increase removal coverage.
