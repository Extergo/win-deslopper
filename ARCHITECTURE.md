# Architecture

The normal Deslopper build is the Read-Only Product Alpha. An internal
Cargo feature, `mutation-alpha`, adds a closed unelevated broker for exactly
three documented taskbar presentation settings.

## Layers

1. **Tauri/Svelte presentation.** Svelte renders onboarding, dashboard,
   component browsing, desired-state previews, drift/history comparison,
   diagnostics review, and local settings. The Product Alpha presentation has
   no mutation workflow and never owns Windows paths, values, scripts,
   applicability, authority, or redaction rules. The same frontend instantiates
   its internal Experimental Apply & Undo panel only when Rust's immutable build
   metadata reports a `mutation-alpha` compile. That panel is an untrusted
   transport client: Rust owns gate status, authorized operations and targets,
   exact representations, approval phrases, transaction state, and rollback.
2. **Application orchestration.** Rust owns inspection coordination, progress,
   desired-state validation, drift, feature-gated command transport, and fresh
   pre/post-operation inspection. Tauri handlers do not execute settings.
3. **Domain models.** The exact 20-component read-only catalogue remains intact.
   Mutation uses separate closed operation/subject/target enums and an immutable
   three-entry registry.
4. **Read-only Windows inspection.** The existing eight fixed PowerShell queries
   retain timeout, output, parser, cancellation, and evidence contracts. They
   run immediately before and after alpha apply where material.
5. **Planning.** Product previews are schema-versioned and non-executable. They
   create no nonce, approval, or transaction. Internal alpha plans are
   persisted, machine/build/edition/evidence bound, five-minute expiring,
   nonce-approved, one-use, and SHA-256 integrity checked.
6. **Mutation broker.** `src/mutation` owns gates, validation, locking, capture,
   dispatch, verification, journal, rollback, recovery, and test-only fault
   injection. Individual handlers own one fixed representation each. There is
   no generic registry setter or command runner.
7. **Windows setting store.** A safe registry wrapper exposes only named
   operation-specific preference reads/writes plus fixed read-only checks for
   Widgets policy, `HideTaskViewButton`, and `NoSetTaskbar`. No caller can
   provide a hive, path, name, type, or arbitrary value.
8. **Persistence.** SQLite v4 stores read-only history, desired revisions,
   reviewed/resolved drift, retention preferences, and internal feature-gated
   mutation journal tables. Local history clearing removes only Deslopper's
   read-only records and preserves internal audit tables. Persistence cannot
   dispatch operations.
9. **Privileged host, plugins, backend, and updater.** Absent. The alpha has no
   elevation, helper, service, network, authentication, telemetry, or remote
   operation boundary.
10. **Live-validation safety and evidence.** The existing application is the
   guest runner. A fixed local manifest and denylist add scenario, machine,
   checkpoint, source, platform, and guest-database gates. A separate ignored
   target approval distinguishes VMs from physical laptops and rejects pending,
   incomplete, expired, mismatched, or recovery-unready targets. A parsed,
   committed policy explicitly allowlists target types. Scoped version 3
   physical approvals add strict data, recovery-route, media, alternate-device,
   restart, encryption, management, final-plan, automatic-repair, expiry, and
   disposition requirements to operation-specific target states and exact
   source/evidence/identity bindings. Version 2 remains VM-only compatibility.
   Broker options are derived from scope, while plan, execution, and the
   immediate pre-write boundary independently revalidate policy, approval, and
   denylist. The frontend renders the
   identity banner and sends only closed visual-verification enums. Evidence
   facts come from the journal and export to a fixed ignored location. Optional
   Hyper-V tooling is allowlisted and cannot invoke broker operations.
11. **Diagnostics boundary.** Rust constructs a closed, redacted JSON value
    without machine identity, validation files, account data, or raw output.
    Svelte shows the included categories before using the WebView's user-driven
    save picker. No Tauri filesystem permission or network upload exists.

## Dependency direction and release boundary

Presentation sends typed IDs to Rust. Normal orchestration never calls the
broker; feature-on internal orchestration calls the broker, and the broker
selects a handler; the handler calls its fixed store method. Inspection never
imports mutation. Persistence stores data but cannot apply it. Normal builds do
not compile `src/mutation` or register its Tauri commands. Target-type evidence
must match the explicit approval and cannot override policy or identity gates.
Normal builds do not compile live-validation evidence commands.

The normal main-window capability contains only the Tauri listen and unlisten
commands required for Rust-emitted inspection progress. Frontend event emission
is not part of the presentation boundary.

Mutation execution emits no frontend event. The internal panel reads the durable
transaction history while an existing apply or rollback command is pending and
renders only journaled steps. Manual visual confirmation never triggers a second
write and is exported only through the existing evidence command after exact
transaction-bound rollback.

See `docs/read-only-product-alpha.md`, `docs/mutation-threat-model.md`,
`docs/mutation-broker-alpha.md`, and D-011.
