# Owner Mode M1 handoff

The active product milestone is Deslopper Owner Mode M1: one normal-release
Widgets Apply/Verify/Undo vertical slice. D-015 supersedes the read-only normal
product boundary while preserving the historical Mutation Alpha record.

Read `AGENTS.md`, `.deslopper/policy.toml`, `README.md`, `ARCHITECTURE.md`,
`SECURITY.md`, `docs/read-only-product-alpha.md`, `docs/inspection-pipeline.md`,
and `docs/testing-strategy.md` before acting.

## Safety contract

- The normal build registers the read-only product plus closed owner Widgets
  actionability, Apply, Undo, and history commands.
- The main window capability remains exactly event listen/unlisten.
- Diagnostics are generated in Rust, exclude machine identity and validation
  files, and are saved only after user review through a system file picker.
- Product previews are explicitly non-executable and create no mutation
  transaction or approval nonce.
- Mutation Alpha governance remains an optional engineering harness and is not
  a product runtime gate.

## Withdrawn physical target

The former physical laptop completed read-only validation but was withdrawn
because the hardware was sold. It was never approved for mutation, never
received the development-host identity, never used mutation flags, and never
performed a registry write. This was a target-availability decision, not a
mutation failure. A future disposable target or VM needs a new preparation and
approval record from scratch.

## M1 status

Owner Mode M1 is implemented and packaged as version `0.1.0`. The default
release composition contains only Widgets actionability, Apply, Undo, and owner
history; Task View and Show Desktop remain engineering-harness-only handlers.
Automated verification uses fake backends and performed no live registry write.

The remaining release activity is the human Windows smoke test in
`docs/owner-mode-widgets-smoke-test.md`. Do not productize Task View, Show
Desktop, AppX, HKLM, services, elevation, or generic execution as a continuation
of M1.
