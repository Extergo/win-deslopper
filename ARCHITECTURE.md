# Architecture

The normal Deslopper build is Owner Mode. It combines the read-only
21-component inspector with six closed, unelevated operations: current-user
Taskbar Widgets and Task View visibility plus four suggestion/content preferences. The historical `mutation-alpha`
feature remains an explicit engineering validation flavor; it is not the
authorization path for the installed product.

## Layers

1. **Tauri/Svelte presentation.** Svelte renders onboarding, dashboard,
   component browsing, desired-state previews, drift/history comparison,
   diagnostics review, local settings, and taskbar actionability on the normal
   component detail. Presentation never owns Windows paths, values, scripts,
   applicability, authority, or redaction rules. It is an untrusted transport
   client: Rust owns the registered product operation, exact representation,
   transaction state, verification, and rollback.
2. **Application orchestration.** Rust owns inspection coordination, progress,
   desired-state validation, drift, feature-gated command transport, and fresh
   pre/post-operation inspection. Tauri handlers do not execute settings.
3. **Domain models.** The 21-component read-only catalogue includes a distinct
   Task View detector; Taskbar Search remains a separate component.
   Mutation uses separate closed operation/subject/target enums and an immutable
   three-entry engineering-alpha registry plus a separate six-entry Owner Mode
   product allowlist.
4. **Read-only Windows inspection.** The existing eight fixed PowerShell queries
   retain timeout, output, parser, cancellation, and evidence contracts. They
   run immediately before and after alpha apply where material.
5. **Owner operation intent.** One Apply command creates and consumes an
   internal durable intent. There is no separately reviewed five-minute plan,
   typed phrase, user nonce, machine approval, or repository-local runtime
   manifest. Legacy previews remain non-executable for inspection-only items.
6. **Mutation broker.** `src/mutation` owns actionability, validation, locking,
   exact capture, dispatch, actual post-attempt capture, verification,
   automatic safe rollback, journal, one-click Undo, recovery, and test-only
   fault injection. There is no generic registry setter or command runner. Only
   Widgets, Task View, welcome experience, tips and suggestions, notification
   suggestions, and suggested content in Settings are in the Owner Mode M3
   product registry. Show Desktop remains internal and `TaskbarSd` is not productized.
7. **Windows setting store.** A safe registry wrapper exposes only named
   operation-specific preference reads/writes plus fixed read-only checks for
   Widgets policy, `HideTaskViewButton`, `NoSetTaskbar`, and four matching Cloud
   Content policies. The generic internal DWORD helper accepts only a closed
   setting enum. No caller can
   provide a hive, path, name, type, or arbitrary value.
8. **Persistence.** SQLite v5 stores read-only history, desired revisions,
   reviewed/resolved drift, retention preferences, and internal feature-gated
   mutation journal tables. Owner transactions bind to a versioned SHA-256
   digest of machine identity plus current-user SID; raw values are not
   persisted. Local history clearing removes only Deslopper's
   read-only records and preserves internal audit tables. Persistence cannot
   dispatch operations.
9. **Privileged host, plugins, backend, and updater.** Absent. The alpha has no
   elevation, helper, service, network, authentication, telemetry, or remote
   operation boundary.
10. **Engineering validation harness.** The historical application path is the
   guest runner. A fixed local manifest and denylist add scenario, machine,
   checkpoint, source, platform, and guest-database gates. A separate ignored
   target approval distinguishes VMs from physical laptops and rejects pending,
   incomplete, expired, mismatched, or recovery-unready targets. A parsed,
   committed policy explicitly allowlists target types. Scoped version 3
   physical approvals add strict data, recovery-route, media, alternate-device,
   restart, encryption, management, final-plan, automatic-repair, expiry, and
   disposition requirements to operation-specific target states and exact
   source/evidence/identity bindings. Version 2 remains VM-only compatibility.
   Harness options are derived from scope, while plan, execution, and the
   immediate pre-write boundary independently revalidate policy, approval, and
   denylist. The frontend renders the
   identity banner and sends only closed visual-verification enums. This
   infrastructure is not registered in the normal product. Evidence
   facts come from the journal and export to a fixed ignored location. Optional
   Hyper-V tooling is allowlisted and cannot invoke broker operations.
11. **Diagnostics boundary.** Rust constructs a closed, redacted JSON value
    without machine identity, validation files, account data, or raw output.
    Svelte shows the included categories before using the WebView's user-driven
    save picker. No Tauri filesystem permission or network upload exists.

## Dependency direction and release boundary

Presentation sends typed IDs to Rust. Normal orchestration calls only closed
owner APIs; the broker selects one of six fixed product handlers;
the handler calls its fixed store method. Inspection never imports mutation. Persistence stores data
but cannot apply it. Normal builds compile the closed owner broker and register
owner actionability/Apply/Undo/history commands. They do not register legacy
live-validation approval or evidence commands. The explicit `mutation-alpha`
engineering flavor may register those commands separately.

The normal main-window capability contains only the Tauri listen and unlisten
commands required for Rust-emitted inspection progress. Frontend event emission
is not part of the presentation boundary.

Owner execution emits no frontend event. The component UI prevents duplicate
submission and renders the command result plus durable transaction history.
Manual visible-shell observation is not claimed by automated verification and
never triggers a second write.

See `docs/owner-mode-m3-current-user-cleanup-smoke-test.md`,
`docs/owner-mode-widgets-smoke-test.md`, `docs/mutation-threat-model.md`,
`docs/mutation-broker-alpha.md`, and D-018.
