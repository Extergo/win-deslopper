# Deslopper

**Windows without the slop.**

Deslopper `0.1.0-alpha.1` is a privacy-conscious, installable Windows
configuration inspector. The Read-Only Product Alpha observes 20 documented
Windows components, explains uncertainty and authority, saves local inspection
history, detects meaningful drift, records desired states, and creates
non-executable previews.

The normal build never applies a Windows change. It has no mutation command,
filesystem capability, shell endpoint, network client, updater, elevation
manifest, service, or privileged helper. Unknown does not mean broken,
permission-limited does not mean absent, and a preview is never presented as
completed work.

## Product Alpha features

- Compact first-run explanation and cancellable read-only inspection.
- Honest dashboard summaries without fake health scores or optimisation claims.
- All 20 registered components with applicability, authority, confidence,
  completeness, desired state, drift, trade-offs, and redacted evidence.
- Persisted SQLite history, snapshot comparison, reviewed drift, and retention
  controls.
- Desired-state validation and preview-only planning. Previews create no nonce,
  transaction, approval, or executable operation.
- User-controlled privacy-safe JSON diagnostics saved through the system file
  picker. Nothing is uploaded automatically.
- Settings/About with product/build status, redacted database location, local
  history controls, privacy explanation, and explicit mutation unavailability.

See `docs/read-only-product-alpha.md` for the complete product and privacy
contract.

## Internal mutation broker

The repository retains a separately compiled internal `mutation-alpha` feature
for exactly three closed current-user taskbar experiments. It is not part of the
normal Product Alpha command registration or presentation. All mutation results
remain synthetic: zero handlers are live validated and none is production-ready.

The physical validation laptop completed read-only preparation and was then
retired before mutation because the hardware was being sold. That event remains
in history. It is now temporarily reactivated and ready only for a separate
Widgets approval review before its final reset, but remains unapproved with zero
live scenarios and no mutation attempt. Reactivation and review readiness do not
themselves authorize mutation.

Scoped approval schema version 2 can bind a physical validation approval to an
explicit operation and operation-specific target state. The broker filters
options and independently revalidates the exact pair, source evidence, expiry,
and usage limits. This source change does not constitute approval or live
validation; the Widgets test remains unexecuted.

The original development PC remains permanently denied by its local ignored
`.deslopper/local/development-host-denylist.json`. Never commit or expose that
identity.

## Development and packaging

Prerequisites are stable Rust, Bun 1.3.14, WebView2, and the supported Tauri 2
Windows toolchain. Run the quality gates in `.deslopper/policy.toml`.

```powershell
bun install --frozen-lockfile
cargo test
bun run test
bun run tauri build --no-bundle
bun run tauri build
```

The normal NSIS installer uses Tauri's WebView2 downloaded-bootstrapper mode.
Windows 11 normally supplies WebView2; the installer can obtain it when missing.
The package is not code-signed and no public release is produced by this
milestone. Application binaries uninstall normally; local history under
`%LOCALAPPDATA%\Deslopper` is documented as user data and may remain unless the
user clears it in Settings or removes it deliberately.

Start with `AGENTS.md`, `ARCHITECTURE.md`, `SECURITY.md`, and
`docs/product-principles.md`.
