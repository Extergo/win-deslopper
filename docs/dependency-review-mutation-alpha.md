# Mutation alpha dependency review

Reviewed 2026-08-02 for the internal `mutation-alpha` Cargo feature.

## `sha2` 0.10.9

- Purpose: SHA-256 integrity binding for immutable plans, state captures, and
  audit records.
- Standard-library gap: Rust does not provide a cryptographic hash.
- Maintenance/licence: maintained RustCrypto crate; MIT OR Apache-2.0.
- Platform/capabilities: portable computation only; no command, filesystem,
  network, privilege, or Windows-management capability.
- Supply-chain/binary impact: already present transitively in the accepted lock
  file; made an optional direct dependency only for the alpha feature.
- Alternatives: `DefaultHasher` is not cryptographic or stable; a handwritten
  SHA-256 implementation would be harder to audit.
- Removal: remove plan/capture SHA-256 binding or replace it with a reviewed
  cryptographic integrity design, then remove the feature edge.

## `winreg` 0.55.0

- Purpose: safe, typed access to three fixed HKCU taskbar presentation values.
- Standard-library gap: Rust has no safe registry API. Calling `reg.exe` or
  PowerShell would create a broader command boundary and is prohibited.
- Maintenance/licence: maintained Windows Rust crate; MIT licence.
- Platform/capabilities: Windows registry access only. It does not execute
  commands, use the network, elevate, or create a service. The dependency was
  already present transitively through the accepted desktop stack and is now
  optional/direct only for `mutation-alpha` on Windows.
- Security/unsafe: its audited implementation wraps Win32 registry APIs; no
  `unsafe` is added to Deslopper. Deslopper does not expose a generic wrapper:
  each handler hard-codes its hive, path, value name, type, and allowed values.
- Binary impact: limited and feature-gated; absent from ordinary dependency use
  paths when the alpha feature is disabled.
- Alternatives: raw Win32 calls require repository `unsafe`; external registry
  tools create an injection-prone process boundary; both are worse.
- Removal: replace the three setting handlers with a reviewed supported Windows
  API, then remove this dependency and its feature edge.
