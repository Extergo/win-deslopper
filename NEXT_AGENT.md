# Read-Only Product Alpha handoff

The active product milestone is Deslopper `0.1.0-alpha.1`, a normal-feature,
read-only Windows application. The branch starts from readiness commit
`aad404e690175857a69085554e3223c078ded2e6`.

Read `AGENTS.md`, `.deslopper/policy.toml`, `README.md`, `ARCHITECTURE.md`,
`SECURITY.md`, `docs/read-only-product-alpha.md`, `docs/inspection-pipeline.md`,
and `docs/testing-strategy.md` before acting.

## Safety status

- The normal build registers only read-only inspection, local persistence,
  desired-state preview, snapshot comparison, diagnostics, and local-data
  commands.
- The main window capability remains exactly event listen/unlisten.
- Diagnostics are generated in Rust, exclude machine identity and validation
  files, and are saved only after user review through a system file picker.
- Product previews are explicitly non-executable and create no mutation
  transaction or approval nonce.
- The original development host remains permanently denied by an ignored,
  local-only fingerprint file.
- Mutation alpha remains separately compiled and synthetic-tested only. Zero
  live-validated handlers exist.

## Withdrawn physical target

The former physical laptop completed read-only validation but was withdrawn
because the hardware was sold. It was never approved for mutation, never
received the development-host identity, never used mutation flags, and never
performed a registry write. This was a target-availability decision, not a
mutation failure. A future disposable target or VM needs a new preparation and
approval record from scratch.

## Next safe work

Continue Product Alpha quality and real-world read-only compatibility work:

1. Expand read-only fixtures across Home, Pro, Enterprise, 24H2, 25H2, managed,
   and standard-user environments.
2. Review keyboard navigation, screen readers, and 125%/150%/200% Windows
   scaling on additional machines.
3. Exercise install/uninstall and database recovery with disposable local data.
4. Keep permission-limited AppX scopes and unsupported preview surfaces honest.
5. Do not begin live mutation without a new approved disposable target and a
   separately authorized milestone.
