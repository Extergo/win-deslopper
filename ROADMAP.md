# Deslopper roadmap

This roadmap records sequencing, not authorization. `AGENTS.md`, `SECURITY.md`,
`ARCHITECTURE.md`, and `.deslopper/policy.toml` remain authoritative.

## Current milestone - Read-Only Product Alpha

**Status:** Implemented on `read-only-product-alpha`; validation and packaging
gates must pass before the milestone tag is created.

The Product Alpha turns the 20-component read-only engine into a useful Windows
application: compact onboarding, an honest system summary, component browsing,
desired-state validation, non-executable previews, reviewed/resolved drift,
snapshot comparison, retained history, privacy-safe diagnostics, local settings,
and a normal feature-off installer.

The normal build remains unelevated and has no usable mutation surface. It does
not claim optimisation, removal, repair, production readiness, or universal
Windows support.

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

The compile-gated broker still contains exactly three closed taskbar operations.
All evidence is synthetic and zero handlers are live validated. The former
physical target was withdrawn because it was sold before approval or mutation;
no mutation failure occurred. Any future work requires a new disposable target,
complete recovery evidence, explicit approval, and a separately authorized
milestone. The original development host is permanently prohibited.

## Sequencing rules

- Inspection failure is never absence.
- Normal package version servicing is not configuration drift.
- Preview state is never represented as completed work.
- No automatic restoration or Apply control appears until an explicitly
  authorized, tested, reversible operation reaches the normal product boundary.
- Do not weaken gaming, anti-cheat, Xbox, Game Pass, or Windows security
  compatibility to increase removal coverage.
