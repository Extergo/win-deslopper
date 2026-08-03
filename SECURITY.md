# Security

Deslopper manages security-sensitive local state. Threats include compromised
frontend/dependencies, confused-deputy elevation, unsafe compatibility
assumptions, identifying evidence, stale/replayed plans, concurrent operations,
crashes, rollback conflict, and misleading success.

## Normal build

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
- Normal builds have no mutation module or mutation command registration, no
  elevation manifest, `runas`, helper, service, backend, telemetry, or plugin
  execution.

## Internal mutation alpha

The alpha requires the compile feature, debug/internal build, exact alpha and
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

SHA-256 records are integrity checks, not signatures against a same-user
attacker who can rewrite both the database and hashes. The alpha remains
internal until real disposable-VM coverage and security review are complete.
See `docs/mutation-threat-model.md` for the full analysis.

The environment banner and VM heuristic provide visibility only. Eligibility
comes from explicit scenario identity, platform, database, and denylist checks.
Live eligibility also requires a separate ignored, approved, unexpired generic
validation-target record. Physical targets fail closed unless stricter recovery
and user-approval fields are complete. Preparation records never authorize
mutation.
The threat boundary does not include a compromised same-user account with
arbitrary code execution. Live evidence contains typed manual assertions and
must be reviewed; it is not cryptographically attested visual proof.

## Reporting

Use a private repository security advisory or maintainers' private owner
channel. Never disclose suspected vulnerabilities publicly or include secrets.
