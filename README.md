# Deslopper

**Windows without the slop.**

Deslopper `0.3.0` is a privacy-conscious, installable Windows
configuration inspector with a reversible current-user cleanup pack plus owner-operated Task View and scoped Widgets controls. Owner
Mode observes 21 documented
Windows components, explains uncertainty and authority, saves local inspection
history, detects meaningful drift, records desired states, and creates
non-executable previews.

The normal build can apply and exactly undo six registered current-user changes:
four Windows suggestion/content preferences and the Windows 11 Task View or Widgets taskbar buttons. Widgets
direct change is suppressed for a machine/account scope after a verified unchanged
access denial. It has no generic
mutation command, filesystem capability, shell endpoint, network client,
updater, elevation manifest, service, or privileged helper. Unknown does not mean broken,
permission-limited does not mean absent, and a preview is never presented as
completed work.

## Owner Mode M3 features

- Compact first-run explanation and cancellable read-only inspection.
- Honest dashboard summaries without fake health scores or optimisation claims.
- All 21 registered components with applicability, authority, confidence,
  completeness, desired state, drift, trade-offs, and redacted evidence.
- Persisted SQLite history, snapshot comparison, reviewed drift, and retention
  controls.
- Desired-state validation and preview-only planning. Previews create no nonce,
  transaction, approval, or executable operation.
- User-controlled privacy-safe JSON diagnostics saved through the system file
  picker. Nothing is uploaded automatically.
- Settings/About with product/build status, redacted database location, local
  history controls and privacy explanation.
- Welcome experience, tips and suggestions, notification suggestions, suggested
  content in Settings, Task View, and Widgets actionability integrated into their normal component details. One Apply
  click performs fixed preflight, exact pre-state capture, durable transaction,
  fixed registered DWORD write, direct and detector verification, and safe failure
  recovery. One Undo click restores the exact prior DWORD or prior absence when
  no conflict exists.
- Rejected unchanged writes are classified separately from ambiguous or changed
  failures. They expose no Undo; a proven Widgets permission denial suppresses
  only that machine/account capability without elevation or ACL changes.
- Actionable component pages foreground the real Apply/Undo workflow and concise
  latest-change history. The legacy non-executable desired-state preview remains
  available only for inspection-only components.

The M3 cleanup settings are fixed to `SubscribedContent-310093Enabled`,
`SoftLandingEnabled`, `SubscribedContent-338389Enabled`, and
`SubscribedContent-338393Enabled` under the current user's Content Delivery
Manager key. DWORD `1` is enabled and DWORD `0` is disabled. A missing or
non-binary value is unsupported rather than guessed, and matching configured
Cloud Content policy makes the operation managed and non-actionable. Lock-screen
suggestions were deliberately skipped because the existing detector combines
`RotatingLockScreenOverlayEnabled` with broader `ConfigureWindowsSpotlight`
policy semantics and cannot yet prove exact detector/handler parity.

## Physical validation

Owner Mode M2 Task View is physically validated PASS on a Windows 11 x64 owner
machine under ordinary current-user execution without UAC or elevation. The
released `0.2.0` build completed detection, one live Apply from DWORD 0 to 1,
direct and detector verification, process exit, durable relaunch, Undo, exact
restoration to DWORD 0, and independent registry verification. See
`docs/owner-mode-m2-physical-validation.md` for the release identity and full
evidence record.

M3 automated verification uses fake/fault-injection backends only. The practical
installed-product validation has now passed for Tips and suggestions and
Notification suggestions. Both fixed DWORDs changed from 1 to 0, survived a
complete process exit as durable transactions, and were exactly restored to 1
through Undo after relaunch. The two initially absent M3 values remained absent,
consistent with M3's unsupported-missing contract. See
`docs/owner-mode-m3-physical-validation.md` for the authoritative release
identity and evidence, and
`docs/owner-mode-m3-current-user-cleanup-smoke-test.md` for the retained protocol.

See `docs/read-only-product-alpha.md` for the complete product and privacy
contract.

## Internal validation harness

The repository retains an explicit internal `mutation-alpha` feature
for exactly three closed current-user taskbar experiments. It is not part of the
normal Owner Mode authorization or presentation. Internal builds may
show a compact Experimental Apply & Undo workflow only when Rust build metadata
reports the mutation feature. The panel reads backend gate status, renders only
backend-authorized options, reviews one-use plans, requires the backend-provided
exact phrase, follows durable transaction state, records manual visual outcomes
through live evidence, and offers only transaction-bound exact rollback.

Historical engineering evidence remains separate from product authorization.
The first physical Owner Mode test later established a narrower product fact:
the Widgets write was denied unchanged for that machine/account, while a Task
View same-state write succeeded unelevated. This does not graduate the internal
harness or authorize any other operation.

Scoped approval version 2 introduced explicit operation and target-direction
binding. Under the committed explicit-target governance policy, it is now a
VM-only compatibility format; physical approval requires strict version 3
recovery, identity, management, expiry, final-plan, and final-disposition
evidence. The broker revalidates policy, denylist, approval, source evidence,
scope, and execution allowance at plan, execution, and pre-write boundaries.
This governance change grants no target or operation approval.

Those historical approval and deny-list rules apply only to the engineering
harness. They are not runtime prerequisites for the installed owner product.

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

The normal NSIS installer contains Owner Mode M3, needs no CLI flag, repository
working directory, validation file, debug build, or administrator rights for
these current-user operations, and uses Tauri's WebView2 downloaded-bootstrapper mode.
Windows 11 normally supplies WebView2; the installer can obtain it when missing.
The package is not code-signed and no public release is produced by this
milestone. Application binaries uninstall normally; local history under
`%LOCALAPPDATA%\Deslopper` is documented as user data and may remain unless the
user clears it in Settings or removes it deliberately.

Start with `AGENTS.md`, `ARCHITECTURE.md`, `SECURITY.md`, and
`docs/product-principles.md`.
