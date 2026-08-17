# Owner Mode M4.1 mutation-lock fix

## Physical evidence and scope

The first installed `0.4.0` Phone Link attempt stopped with “Another Deslopper
mutation transaction owns the process lock.” Independent current-user inventory
still showed the complete healthy `Microsoft.YourPhone` registration, and an
elevated read-only query showed provisioning unchanged. No package deployment
was physically exercised and this is not an AppX removal PASS.

This patch changes lock ownership only. Exact package identities, current-user
scope, provisioning observation, dependency checks, Restore classification,
app-data warnings, native PackageManager calls, and the no-elevation boundary
are unchanged.

## Root cause

Both mutation families resolved the same SQLite-derived path ending in
`mutation-alpha.lock`, but they interpreted it differently:

- M1-M3 `MutationProcessLock` opened or created the stable file with Windows
  `share_mode(0)`. The open file handle represented live ownership. The file was
  deliberately left on disk after the handle closed, so a later operation could
  reopen it safely.
- M4 `PackageProcessLock` used `OpenOptions::create_new(true)`. It treated file
  existence as ownership and deleted the file on drop.

Consequently, any prior setting mutation could leave the normal rendezvous file
behind. A later package operation failed at lock acquisition before inventory,
transaction creation, backend dispatch, or `RemovePackageAsync`. It was stale
file interpretation, not recursive acquisition, transaction self-comparison,
SQLite active-status ownership, or a leaked in-process mutex.

## Corrected ownership model

`OwnerMutationProcessLock` in `src/mutation/process_lock.rs` is now the sole
cross-process implementation used by setting Apply/Undo, engineering mutation,
package Remove, and package Restore. On Windows it opens the stable rendezvous
file with no sharing. Therefore:

- one live handle excludes every other Deslopper mutation, including another
  broker in the same process;
- dropping the handle releases ownership on every return path;
- an unopened historical file contains no ownership and can be reopened;
- process termination releases the kernel handle automatically; and
- transaction history/status remains durable audit and recovery state, not the
  cross-process lock primitive.

Each broker retains its own in-process mutex for duplicate calls within that
resource family. The shared exclusive Windows handle coordinates registry and
package brokers with each other and across processes. Startup recovery still
marks interrupted durable transactions `RecoveryRequired`; it does not retain a
dead process lock.

Genuine contention now uses the primary product message: “Another Deslopper
change is still in progress.”

## Regression coverage

Fake-backend tests prove:

- a single package Remove reaches the fake deployment backend exactly once;
- concurrent package brokers contend and the second backend is not reached;
- a live `owner-cleanup.3` registry mutation blocks package Remove;
- completion releases the shared lock and permits the package operation;
- a restored M3 transaction plus its persistent lock file does not block a new
  package operation;
- rejected unchanged, ambiguous/failed, restored, and completed package paths
  release ownership; and
- an interrupted package transaction is reconciled on relaunch without becoming
  lock ownership.

The lock primitive separately proves simultaneous ownership is rejected,
launches a second test process to prove process-to-process exclusion and release,
and, on Windows, proves an existing unopened lock file is stale-safe. All package
deployment tests use fake backends. No live AppX/MSIX mutation occurred while
diagnosing, implementing, or verifying `0.4.1`.
