# Spare-laptop validation handoff

This checkpoint contains the completed read-only beta and the internal-only
Mutation Broker Alpha. Normal builds remain read-only. The alpha contains
exactly three closed current-user operations: Widgets taskbar visibility, Task
View visibility, and Show Desktop corner. All mutation testing is synthetic;
zero live scenarios are complete, all 13 validation targets remain
`not_provisioned`, and no handler is production-ready.

The original development host is recorded in the ignored local denylist and
must never be used for live mutation. No spare laptop, VM, or other machine is
currently approved. Do not copy a machine identity, approval manifest, or live
evidence between computers.

## Required reading

Before acting, read `AGENTS.md`, `.deslopper/policy.toml`, `README.md`,
`ROADMAP.md`, `ARCHITECTURE.md`, `SECURITY.md`,
`docs/mutation-threat-model.md`, `docs/mutation-broker-alpha.md`,
`docs/live-validation-environment.md`, `docs/manual-vm-provisioning.md`,
`docs/vm-mutation-protocol.md`, and `validation/report.md`.

## Next exact phase

1. Clone the private GitHub repository on the spare laptop.
2. Check out the `mutation-alpha` branch.
3. Verify the annotated `mutation-alpha-synthetic-v1` tag before proceeding.
4. Install the documented Rust, Bun, Node/Tauri, WebView2, and Windows build
   prerequisites without adding project dependencies.
5. Build the normal read-only application first and run all repository quality
   and mutation-boundary gates.
6. Run a complete read-only inspection and confirm the resulting SQLite
   database is owned by the spare laptop environment.
7. Confirm the laptop has a current backup or can be cleanly reinstalled, and
   document the recovery procedure before considering it disposable.
8. Generate the laptop's machine fingerprint locally. Do not commit or transmit
   the unhashed identity inputs.
9. Create the ignored approval manifest for that exact laptop using
   `validation/approved-validation-vms.schema.json` and the template in
   `docs/manual-vm-provisioning.md`.
10. Record the exact Windows edition, build, UBR, account class, management
    context, approved target states, evidence location, checkpoint/recovery
    equivalent, and restoration procedure in the ignored manifest.
11. Copy or recreate the original development-host denylist through a secure
    local handoff and confirm the original host remains denied. Do not replace
    it with the laptop fingerprint.
12. Only after every identity, recovery, database, CLI, build, scenario, and UI
    gate passes may the internal live-validation gates be considered.
13. Validate only Widgets, Task View, and Show Desktop corner. Do not add
    operations or generic registry/script execution.
14. Capture truthful redacted evidence for apply, representation, detector and
    visible state, least disruptive refresh, reboot persistence, exact rollback,
    drift, conflict, and crash recovery.
15. Keep production mutation disabled until reviewed live evidence passes the
    required build, edition, policy-managed, standard-user, lifecycle, rollback,
    and recovery matrix.

Do not claim the spare laptop is approved or disposable until steps 6 through
11 have been completed on that machine. If exact rollback fails, visible state
disagrees, policy ownership is unsafe, or Explorer termination is required,
record the failure and reject the affected handler rather than weakening a
gate.
