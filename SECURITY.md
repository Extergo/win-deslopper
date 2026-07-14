# Security

Deslopper is intended to manage security-sensitive local state. Its threat model includes malicious or compromised extensions and packages, backend compromise, supply-chain attacks, confused-deputy elevation, tampered rollback data, unsafe compatibility assumptions, and misleading UI that causes unintended changes.

## Security posture

- The main UI process runs without administrator privileges by default.
- Future privileged work must use a narrowly scoped helper with authenticated, typed requests and least privilege.
- No plugin receives unrestricted administrator access; unsigned executable extensions are not trusted by default.
- No remote service may cause arbitrary shell execution. Backend availability must not make safe local functionality unusable.
- Packages and updates require publisher identity, signature, integrity, and rollback verification before release.
- Entitlement checks may control access, never whether an operation is technically safe.
- Telemetry is future-only, consent-based, minimal, documented, and off until explicitly approved.
- Secrets never enter source control. Credentials use an approved platform secret store when such a milestone exists.
- Dependencies and transitive supply-chain risks follow `docs/dependency-policy.md`.
- No operation claims success without verification; no mutation ships without rollback analysis and auditable receipts.
- Destructive user-facing actions require precise compatibility criteria, not vague labels such as `safe`.
- Secure defaults mean no elevation, execution, networking, or mutation occurs merely by opening the application.

The `ui-preview` milestone contains no real inspection, privileged operation, Windows mutation, plugin execution, backend, update, or telemetry path. Its catalogue and plan are local mock state only.

## Reporting

Until a dedicated security address is published, do not disclose a suspected vulnerability publicly. Open a private repository security advisory or contact the project maintainers through the repository's private owner channel. Maintainers must acknowledge receipt, assess affected versions, coordinate remediation and disclosure, and never request secrets or exploit data in a public issue.

