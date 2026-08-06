# Disposable-VM mutation protocol

No live mutation target is currently recorded as passed. The repository's
synthetic mutation fixture and read-only Windows fixture are not evidence of
apply/rollback behavior.

## Required targets before normal UI exposure

- Windows 11 Home, Pro, and Enterprise Evaluation 24H2
- At least one Windows 11 25H2 target
- Domain-joined or policy-managed target
- Permission-limited standard-user target

Use only disposable, licensed, explicitly approved VMs with checkpoints. Build
with `mutation-alpha`; supply `--enable-mutation-alpha`,
`--enable-live-validation`, scenario ID, expected machine fingerprint, and
checkpoint ID; verify the fixed local manifest and environment banner; then
acknowledge the warning. VM detection is never a security boundary. The local
development-host denylist and guest database identity are mandatory.

## Procedure per operation and target

1. Snapshot VM and capture a redacted read-only fixture.
2. Inspect, generate plan, and record build, edition, account, authority, and state.
3. Test every target through approval, apply, handler verification, and post inspection.
4. Restart Deslopper and verify audit/recovery history.
5. Approve rollback and verify the exact original representation/effective state.
6. Change the setting externally and verify drift requires a new plan/approval.
7. Test stale plan, policy conflict, standard-user access, failure, crash after
   write, crash during verification, and rollback conflict/failure.
8. Record handler version, mechanism, redacted evidence, screenshots, logs,
   verification, rollback, limitations, and failures.

Record representation, production detector, Settings UI, and user-visible
behavior separately. Determine the least refresh requirement without killing
Explorer. A sign-out/reboot result remains pending until the guest returns and
is reinspected. Export typed evidence through the normal application only.

Use `validation/mutation-fixture.schema.json`. `liveMutationPerformed` must be
truthful. CI parses state machines only; it never invokes production mutation.

## Current coverage

| Scenario | Status |
|---|---|
| Synthetic Widgets disable/rollback state machine | Passed; no live mutation |
| Home 24H2 | Not provisioned |
| Pro 24H2 | Not provisioned |
| Enterprise Evaluation 24H2 | Not provisioned |
| 25H2 | Not provisioned |
| Domain/MDM policy conflict | Not provisioned |
| Standard-user permission-limited | Not provisioned |

## Graduation and rejection

Maturity is separate from support: unvalidated, synthetic tested, live tested
single build, multi-build, multi-edition, policy-conflict tested,
recovery-tested, alpha-validated, or rejected. All three handlers currently
remain `SyntheticTested` and display “Internal alpha - not live validated.”

Reject a handler when the shell ignores it, Settings materially disagrees,
exact rollback fails, behavior is unstable across builds, policy ownership is
unsafe, state reverts inexplicably, only registry state can be proven, or
Explorer termination is required. Rejected handlers cannot generate plans.
