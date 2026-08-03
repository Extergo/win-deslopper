# Architecture

The normal Deslopper build remains the completed read-only beta. An internal
Cargo feature, `mutation-alpha`, adds a closed unelevated broker for exactly
three documented taskbar presentation settings.

## Layers

1. **Tauri/Svelte presentation.** Svelte renders read-only evidence and, only
   when the feature-only status command exists, the alpha warning, operation
   options, exact review, progress, audit history, recovery, and rollback. It
   never owns Windows paths, values, scripts, applicability, or authority.
2. **Application orchestration.** Rust owns inspection coordination, progress,
   desired-state validation, drift, feature-gated command transport, and fresh
   pre/post-operation inspection. Tauri handlers do not execute settings.
3. **Domain models.** The exact 20-component read-only catalogue remains intact.
   Mutation uses separate closed operation/subject/target enums and an immutable
   three-entry registry.
4. **Read-only Windows inspection.** The existing eight fixed PowerShell queries
   retain timeout, output, parser, cancellation, and evidence contracts. They
   run immediately before and after alpha apply where material.
5. **Planning.** Ordinary previews stay non-executable. Alpha plans are
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
8. **Persistence.** SQLite v3 stores read-only history plus mutation plans,
   transactions, steps, captures, and rollback records. Persistence cannot
   dispatch operations.
9. **Privileged host, plugins, backend, and updater.** Absent. The alpha has no
   elevation, helper, service, network, authentication, telemetry, or remote
   operation boundary.
10. **Live-validation safety and evidence.** The existing application is the
   guest runner. A fixed local manifest and denylist add scenario, machine,
   checkpoint, platform, and guest-database gates. A separate ignored generic
   target approval distinguishes VMs from physical laptops and rejects pending,
   incomplete, expired, mismatched, or recovery-unready targets. The frontend renders the
   identity banner and sends only closed visual-verification enums. Evidence
   facts come from the journal and export to a fixed ignored location. Optional
   Hyper-V tooling is allowlisted and cannot invoke broker operations.

## Dependency direction and release boundary

Presentation sends typed IDs to Rust. Orchestration calls the broker; the broker
selects a handler; the handler calls its fixed store method. Inspection never
imports mutation. Persistence stores data but cannot apply it. Normal builds do
not compile `src/mutation` or register its Tauri commands. Runtime VM detection
is not a security gate. Normal builds do not compile live-validation evidence
commands.

The normal main-window capability contains only the Tauri listen and unlisten
commands required for Rust-emitted inspection progress. Frontend event emission
is not part of the presentation boundary.

See `docs/mutation-threat-model.md`, `docs/mutation-broker-alpha.md`, and D-010.
