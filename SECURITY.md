# Security

Deslopper manages security-sensitive local state. Threats include compromised
frontend/dependencies, confused-deputy elevation, unsafe compatibility
assumptions, identifying evidence, stale/replayed plans, concurrent operations,
crashes, rollback conflict, and misleading success.

## Normal owner build

- The main UI runs without administrator privileges.
- Inspection uses fixed read-only PowerShell only; frontend input cannot supply
  scripts, commands, arguments, paths, or environment variables.
- Queries remain bounded, cancellable, output-limited, strict, typed, and
  permission-aware. Deslopper may terminate only its own read-only child.
- SQLite evidence is redacted. It excludes profile paths, SIDs, identities,
  emails, filenames, OneDrive roots, tokens, and recovery material.
- Tauri grants the normal window only the event listen/unlisten lifecycle used
  for backend inspection progress. It grants no frontend event emission,
  shell, filesystem, network, updater, or external-URL plugin.
- Normal builds register only closed Owner Mode M3 actionability, Apply,
  Undo, and history commands. There is no elevation manifest, `runas`, helper,
  service, backend, telemetry, plugin execution, arbitrary registry API, or
  mutation shell command.
- Desired-state previews remain non-executable. One deliberate owner Apply
  click creates an internal durable transaction intent; no phrase, approval
  file, validation target, CLI flag, or user-visible nonce is involved.
- Diagnostics are built from an allowlisted schema and exclude machine
  identity, hostname, account identifiers, development-host protection,
  approval manifests, raw profile paths, OneDrive identities, credentials,
  recovery data, and unredacted command output. The user reviews categories and
  chooses the destination; there is no automatic upload.
- Retention and clear-history commands operate only on Deslopper's local
  read-only database records. They do not alter Windows or erase internal
  mutation audit tables.

### Owner operation boundary

- The six M3 product operations are Widgets and Task View visibility plus
  welcome experience, tips and suggestions, notification suggestions, and
  suggested content in Settings. Rust maps them to fixed HKCU DWORD 0/1 values;
  no registry coordinates or integer come from the frontend.
- The four cleanup operations use exact Content Delivery Manager values and
  matching read-only Cloud Content policy checks. Missing/non-binary preference
  representations are unsupported, and any configured matching policy causes
  refusal. Lock-screen suggestions remain inspection-only because their broad
  Spotlight policy does not prove exact parity.
- The app runs unelevated and affects only the current Windows account. It
  refuses unsupported/non-Windows-11 contexts, unknown representations, and
  fixed external policy ownership.
- The broker locks execution, persists exact pre-state before writing, captures
  actual state after every attempted write where readable, verifies direct
  representation plus the matching detector, and records exact rollback data.
- A verification failure automatically restores exact pre-state only when the
  attempted target is directly observed and can safely be attributed to the
  attempt. Ambiguous or policy-owned state becomes recovery-required and never
  advertises unusable Undo.
- Undo requires an intact transaction, the same versioned machine-plus-user
  scope, acceptable authority, and current state equal to the recorded applied
  state. A conflict is refused; normal UI has no force override.
- Startup never replays a mutation. Interrupted transactions are reconciled
  against actual state and otherwise surfaced as needing attention.
- Automated tests and build commands use fake backends and never touch the live
  registry.
- A rejected write whose exact pre-state remains present is classified as
  unchanged, has no Undo, and is not a recovery alarm. A proven Widgets
  PermissionDenied result suppresses only Widgets direct change for that stable
  machine/account scope. Deslopper never elevates or changes registry ACLs.
- Show Desktop, `TaskbarSd`, and Taskbar Search are outside Owner Mode M3.

## Separate internal mutation-alpha harness

The optional engineering harness requires the compile feature, debug/internal build, exact alpha and
live-validation CLI opt-ins, fixed local scenario manifest, expected machine
and checkpoint identities, guest-owned database, development-host denylist,
in-app acknowledgement, fresh inspection, and valid plan. It exposes three
current-user HKCU operations through closed IDs and fixed methods. It has no
arbitrary command/path/name/value input and never elevates.

The three handlers independently re-read their fixed policy representations:
Widgets policy, Task View's user/device `HideTaskViewButton`, and the user/device
`NoSetTaskbar` block. A newly configured policy fails the plan, apply, verify,
or rollback stage closed instead of relying only on catalogue-level authority.

Plans are machine/build/edition/evidence bound, expiring, SHA-256 checked,
nonce-approved, and one-use. Apply re-inspects, captures exact pre-state,
finishes the atomic write, verifies the target, re-inspects material context,
journals each transition, and offers separately approved exact rollback.
Cross-process locking prevents concurrent app instances. Interrupted operations
become recovery-required and are never silently replayed.

Live-validation policy schema version 2 explicitly allowlists disposable target
types. Physical live validation additionally requires scoped approval schema
version 3. Each approved operation lists its own non-empty target-state set and
matching handler version; the approval also binds source commit, fresh
inspection evidence, development-host denylist identity, short expiration, and
bounded plan/execution counts. Physical approval also requires no active
domain, Entra, workplace, or MDM management, complete recovery and restart
evidence, disabled automatic repair, final-plan approval, and a recorded final
disposition. Broker option filtering is informational defense in depth: plan
generation, execution, and the immediate pre-write boundary independently
revalidate committed policy, the local denylist, and the exact operation and
target against the unchanged local approval.
Rollback authority comes only from the durable transaction and captured exact
pre-state, never from an inferred reverse-direction approval.

SHA-256 records are integrity checks, not signatures against a same-user
attacker who can rewrite both the database and hashes. The alpha remains
internal until real explicitly approved disposable-target coverage and security
review are complete.
See `docs/mutation-threat-model.md` for the full analysis.

The environment banner and VM heuristic provide visibility only. Eligibility
comes from explicit scenario identity, platform, database, and denylist checks.
Live eligibility also requires a separate ignored, approved, unexpired generic
validation-target record. Physical targets fail closed unless stricter recovery
and user-approval fields are complete. Preparation records never authorize
mutation.
The original development PC is permanently denied regardless of an otherwise
valid manifest or approval. It has no CLI, prompt, or maintainer override.
The threat boundary does not include a compromised same-user account with
arbitrary code execution. Live evidence contains typed manual assertions and
must be reviewed; it is not cryptographically attested visual proof.

The engineering frontend entry point is selected only by Rust-provided build
metadata. It does not call mutation status in a normal build, and normal command
registration remains the hard boundary. The panel cannot infer handler scope:
it renders only returned operation/target options, rejects unsafe or expired
one-operation plans as defense in depth, requires an exact backend-provided
phrase, redacts plan and nonce display, and sends only the existing closed
requests. Undo supplies only a durable transaction ID. Machine identities,
approval hashes, and raw nonces are not rendered in history.

The harness gates neither authorize nor block the normal owner runtime.

## Reporting

Use a private repository security advisory or maintainers' private owner
channel. Never disclose suspected vulnerabilities publicly or include secrets.
