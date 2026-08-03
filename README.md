# Deslopper

**Windows without the slop.**

Deslopper is a safety-focused Rust/Tauri/SvelteKit Windows manager. Ordinary
builds provide a completed 20-component read-only inspection beta with bounded
queries, applicability/authority evidence, SQLite history, desired states,
drift, privacy redaction, and VM fixture replay.

## Internal mutation broker alpha

An optional internal feature proves the inspect → plan → revalidate → approve →
apply → verify → audit → exact rollback lifecycle for three current-user
taskbar presentation settings: Widgets button, Task View button, and Show
Desktop corner. Search mode and Search Highlights were not implemented because
official research did not establish suitable stable unelevated current-user
setters; see `docs/mutation-broker-alpha.md`.

Mutation requires all of:

```powershell
cargo run --features mutation-alpha -- --enable-mutation-alpha
```

It also requires a debug/internal build, in-app warning acknowledgement, a
fresh completed inspection, and a valid expiring machine-bound plan. Normal
builds compile without the broker, expose no mutation Tauri commands, and render
no mutation controls. There is no elevation, Apply All, automatic repair,
package removal, service/task management, arbitrary shell, or generic registry
setter.

Live execution additionally requires `--enable-live-validation`, a fixed local
scenario manifest, expected machine/checkpoint arguments, guest-owned database,
and development-host denylist. The UI labels all three handlers “Internal alpha
- not live validated.” No live evidence currently exists; all thirteen matrix
scenarios are not provisioned. See `docs/live-validation-environment.md`.

No handler is production-ready, no approved validation target currently exists,
and production mutation must remain disabled until the required live matrix
passes. The next intended validation environment is a spare laptop after it
clones the private checkpoint and is independently backed up, fingerprinted,
reviewed, and explicitly approved; it is not approved merely by being a laptop.

The first physical-laptop read-only preparation pass is documented in
`docs/physical-validation-target-readiness.md`. Live progress and persisted
history were verified after a minimal Tauri event-listen correction; OneDrive
detection now tolerates a missing startup value. Recovery remains not ready and
mutation remains prohibited.

## Development and validation

Prerequisites are stable Rust, Bun 1.3.14, and WebView2. Run the quality gates
from `.deslopper/policy.toml`. `tools/validate-vm-fixtures.ps1` replays redacted
read-only captures. `tools/scan-mutation-boundary.ps1` enforces the executable
mutation boundary. Live mutation validation is permitted only on an explicitly
approved, expendable validation target with complete recovery evidence; it is
not claimed by synthetic fixtures or read-only preparation.

Start with `AGENTS.md`; `ARCHITECTURE.md`, `SECURITY.md`, and the mutation threat
model are authoritative.
