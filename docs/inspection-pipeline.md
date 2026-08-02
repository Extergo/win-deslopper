# Read-only inspection pipeline

## Lifecycle and progress protocol

Every run receives a unique `inspection-{timestamp}-{sequence}` ID and moves through Preparing, GatheringSharedContext, RunningDetectors, PersistingResults, CalculatingDrift, and one terminal phase: Completed, CompletedWithPartialFailures, Cancelled, or Failed. Cancelling is an explicit transitional phase. The final lifecycle record persists start/completion timestamps, detector totals and outcomes, warnings/errors, cancellation state, status text, and last update.

Tauri emits `deslopper://inspection-progress` payloads containing the inspection ID, phase, completed/total work, current component, detector status, warning/error counts, cancel availability, status text, and update timestamp. Svelte subscribes once; it does not poll the whole dashboard. Persisted SQLite state is authoritative after restart.

Cancellation is ID-scoped and idempotent at the token level. It propagates into the active child query, terminates that Deslopper-owned child, skips queued work, preserves completed detector results, and never reports success. Shutdown signals the same token. Drift comparison ignores failed and cancelled detector results.

## Query boundary and timeouts

`inspection.rs` contains the only production process boundary. Query IDs are fixed: platform inventory, current-user AppX, all-user AppX, provisioned AppX, management context, policy registry, user preferences, and OneDrive metadata. Each `QueryConfig` declares timeout, output limit, cancellation support, root format, parser ID, and whether shared context depends on it.

Errors are structured as spawn failure, timeout, cancellation, non-zero exit, output too large, invalid encoding, invalid JSON, schema mismatch, permission denied, unsupported command, or unexpected empty output. Timeout and cancellation terminate the child. Partial output is flagged but is never interpreted as a normal absence. Expensive inventories execute once; tests assert a query execution count of one across all 20 detectors.

## Permission-aware packages

Exact, build-aware identity rules represent canonical, legacy, successor, mutually exclusive, and multi-identity components. Fuzzy package names are never matched. Registration states are Present, Absent, Unknown, PermissionLimited, QueryFailed, or NotApplicable. Provisioning uses Provisioned, NotProvisioned, Unknown, PermissionLimited, QueryFailed, or NotApplicable.

Completeness is Complete, CurrentUserOnly, RegistrationCompleteProvisioningUnknown, ProvisioningCompleteAllUsersUnknown, PermissionLimited, Failed, Unknown, or NotApplicable. A successful empty scope means absence only in that scope. Access denied never means absence. Incomplete package scope blocks a future “fully absent” preview and lowers confidence.

`package_observations` contains one normalized row per exact observed identity and inspection: component and observation IDs, family/full/name, version, architecture, limited publisher ID, normalized registration/provisioning states, framework/resource/bundle/system flags, install-location presence only, dependency identities, completeness, permission status, and timestamp. Full install paths are not stored.

Known legacy-to-successor changes are package identity migration drift, not reinstallation. Uncertain successor applicability marks desired state for review.

## Authority and precedence

Policy values, user preferences, local policy evidence, domain GPO history, enrollment metadata, PolicyManager provider evidence, effective state, authority, confidence, alternatives, and exact-source proof are separate. Domain membership alone never proves Domain GPO. A work account alone never proves MDM. When the exact setting source cannot be tied to resultant policy, the UI says inferred and lists alternatives.

Authority confidence is Confirmed, Strong, Moderate, Weak, or Unknown. Conflicting policy/preference evidence remains visible; policy-derived state takes precedence only where the structured applicability rule supports that policy.

## Applicability and OneDrive

`applicability.rs` is a versioned data matrix separate from detector code. It records minimum/maximum builds, Windows generation, editions, policy/preference support, prerequisites, deprecation/removal, representation, official source, and review date. Results distinguish Applicable, PartiallyApplicable, NotApplicable, UnsupportedEdition, UnsupportedBuild, Deprecated, Removed, MissingPrerequisite, and Unknown. Rules were reviewed 2026-08-01 against Microsoft Learn and Windows release-health documentation.

OneDrive inspection does not enumerate or open personal files. It distinguishes not installed, installed/unlinked, linked roots, supported enabled/disabled, mixed/per-root uncertainty, policy enforcement, and incomplete detection from client, account-count, policy, and preference metadata. It never stores account identity or root paths. When no reliable global representation exists, the composite state remains partial.

## Desired state and history

Rust supplies finite allowed options for the current component and observation. Validation reports valid, valid-with-warnings, requires-review, invalid, externally managed, insufficient confidence, unsupported build/edition, missing rollback source, or incomplete package scope. Save requests contain only option key, allowed scope, persistence preference, approval preference, and a bounded note. Every save and clear creates a revision; applicability, authority, build, or detection changes mark records stale, invalid, or requiring review rather than deleting them.

Inspection history, inspection detail, component timelines, detailed package rows, and drift history are dedicated typed commands and UI views. Machine-identity changes are not ordinary comparisons. Cause inference persists primary cause, confidence, supporting facts, alternatives, and rule version 2.

## SQLite integrity

Schema v2 enables foreign keys, WAL, normal synchronous mode, a five-second busy timeout, immediate migration locking, transactional writes, history indexes, observation uniqueness, quick integrity checks, application-version metadata, and resumable migrations. Unsupported newer schemas and failed integrity checks preserve the original database and surface recovery status. Alpha JSON is renamed only after the import transaction commits; malformed or failed input remains in place. Backup-name collisions receive a numbered suffix.
