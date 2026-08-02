# Redacted live-evidence bundles

`validation/live-evidence.schema.json` describes the version-one bundle.
Evidence is exported to the fixed ignored `.deslopper/local/live-evidence`
directory with a content-derived identity and create-new semantics. Duplicate
identities are rejected rather than overwritten.

The broker derives scenario, VM identity hash, platform, operation, handler,
typed pre/post/rollback states, transitions, plan/capture hashes, verification,
and rollback data from the validated manifest and durable transaction. The
developer may provide only closed visual-verification enums, redacted warning
text, and screenshot labels containing ASCII letters, numbers, dot, dash, or
underscore. Screenshot paths and image contents are never accepted.

The three independent verification dimensions are:

- `RepresentationVerified`: fixed stored value/type/absence.
- `DetectorVerified`: production detector and authority result.
- `UserVisibleBehaviorVerified`: actual taskbar behavior.

Windows Settings UI is recorded separately. Refresh requirements are
`immediate`, natural taskbar refresh, Settings reopen, Deslopper reopen,
sign-out/sign-in, reboot, unsupported without Explorer termination, or
undetermined. Sign-out/reboot outcomes remain pending until the lifecycle is
manually confirmed. An Explorer-termination requirement rejects the handler.

Bundles reject email-like text, profile paths, token/password markers, SIDs,
and arbitrary screenshot paths. They contain no username, computer name,
email, token, account identifier, recovery material, raw registry export, or
environment dump. SHA-256 is an integrity and correlation mechanism, not an
authenticity boundary against compromised same-user code.

Checked-in bundles are replayed only as data. CI and fixture validation never
perform a Windows write.
