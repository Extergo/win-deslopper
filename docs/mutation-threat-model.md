# Mutation alpha threat model

Reviewed 2026-08-02. This model applies only to internal debug builds compiled
with `mutation-alpha` and launched with `--enable-mutation-alpha`.

This is a pre-live static hardening review. No disposable VM was available, so
the required post-live review remains open and no handler graduated. The review
covered plan canonicalisation, nonce lifecycle, database tampering, same-user
attackers, the cross-process lock, crash consistency, TOCTOU, frontend command
exposure, feature-off registration, handler closure, fixed value mappings,
fault isolation, runner isolation, and evidence redaction.

## Assets and trust boundaries

Protected assets are Windows user state, exact rollback state, plan and
transaction integrity, machine identity, authority/applicability evidence, and
the guarantee that ordinary builds are read-only. Svelte is untrusted input.
Tauri handlers are transport adapters, not authorization. Rust's closed broker,
three isolated handlers, SQLite journal, fixed registry store, and fresh
read-only inspector form the trusted local boundary. There is no network,
elevation, service, generic helper, or remote principal.

## Threats and controls

| Threat | Control |
|---|---|
| Compromised frontend, component substitution, parameter tampering | `deny_unknown_fields`, closed operation/target enums, immutable operation-to-subject mapping; execution accepts only plan ID, one-time nonce, and acknowledgement. |
| Arbitrary commands, scripts, arguments, paths, environment variables, registry paths/names/values | No action process exists. The registry store has operation-specific preference and policy methods with compile-time hives, paths, and names. Callers never supply a hive, path, name, type, or data mapping. |
| Stale plan or changed source/build/edition/machine/authority/policy | Five-minute expiry, SHA-256 plan hash, machine/build/edition/evidence binding, latest-source check, fresh full inspection before apply, and immediate handler pre-state capture. |
| Approval replay | The nonce is stored only as SHA-256; plans are atomically consumed before the write and map one-to-one to a transaction. |
| Inspection/execution race | In-process mutex, cross-process share-deny lock, fresh inspection, immediate capture, and source-state equality. Post-apply reinspection detects a material policy race. |
| Domain, MDM, or local policy takeover | Widgets policy, `HideTaskViewButton`, and enabled `NoSetTaskbar` representations are read through fixed methods and block planning. They are re-read at immediate pre-state capture, verification, and rollback, while material inspection context is re-evaluated before and after apply. Domain/workplace membership alone is not proof. |
| Privilege-escalation abuse, host targeting, or malicious local IPC | All writes are HKCU and declare no elevation. Compile feature, debug build, alpha CLI opt-in, separate live-validation opt-in, fixed manifest, exact machine/scenario/checkpoint, edition/build/UBR, guest database identity, local development-host denylist, warning acknowledgement, fresh inspection, nonce, and plan binding are required. There is no helper or admin path. Same-user IPC remains constrained to issued unconsumed plans. |
| Partial write/process failure | Atomic setting writes finish; cancellation exists only before mutation. Durable states distinguish pre-write failure, post-write failure, verification failure, and recovery required. |
| Crash or power loss | Approved, CapturingPreState, Applying, Verifying, Applied-pending-full-detection, and RollingBack are recovered by comparing current, pre-, and expected post-state. Apply is never replayed. WAL reduces but cannot eliminate last-write loss on abrupt power failure. |
| Rollback failure or later user change | Exact DWORD/value absence is captured and hashed. Rollback rechecks context and compares current with verified post-state. Conflict requires separate approval; restoration is verified exactly. |
| Concurrent operations/windows | Nonblocking process mutex, Windows share-deny file lock, plan uniqueness, and one-use consumption. There is no Apply All. |
| Database tampering | Before history, recovery, or rollback, the broker revalidates the canonical plan, immutable plan/transaction mapping, operation/subject mapping, and every captured-state hash. Constraints and foreign keys protect topology. Hashes are not signatures and cannot defeat a same-user attacker able to rewrite both data and hashes. |
| Future generic-runner regression | Three-entry immutable registry, per-operation handler files, fixed store, feature-gated Tauri registration, boundary scan, tests, dependency review, and future-operation checklist. |
| Falsified visual evidence or identifying exports | Representation, detector, and visual states use closed enums; transaction facts come from the journal; output uses a fixed ignored directory and create-new identity; screenshot input is label-only; secret/profile/email markers are rejected. Manual visual claims remain human assertions and require review. |
| Validation tooling targets an unintended VM | Hyper-V tooling reads an ignored explicit allowlist, requires one scenario-to-VM match, checks generation/checkpoint, refuses a VM name colliding with the host name, and requires a separate confirmation switch for VM state changes. The guest application independently refuses the development-host fingerprint. Tooling never launches or acknowledges mutation. |

## Review disposition

- Plan canonicalisation is deterministic and every material binding is tested;
  schema or serialization changes still require fixture review.
- One-time nonces and atomic plan consumption prevent accidental replay. They
  are not credentials against arbitrary code already running as the user.
- The process and file locks close ordinary concurrent-app races, while fresh
  inspection, immediate capture, and post-write inspection narrow rather than
  eliminate TOCTOU.
- Mutation Tauri commands are compiled and registered only with
  `mutation-alpha`; normal-build compilation is a release gate. The frontend is
  untrusted even in the internal build.
- Fault injection is test-only, handlers remain a three-entry closed registry,
  and no registry hive/path/name/value or shell command is caller-controlled.
- The guest runner is the normal application/broker path. Live evidence cannot
  upgrade handler maturity automatically and manual visual claims remain
  reviewable human assertions.
- The post-live review must revisit servicing behavior, Settings disagreement,
  lifecycle persistence, policy races, and recovery using actual evidence.

## Residual risks

- SHA-256 provides integrity detection, not authenticity against a same-user
  attacker with database and process access.
- Shell servicing can change when a taskbar presentation refresh becomes
  visible even when the representation verifies.
- Real apply/rollback behavior is unproven until the disposable-VM matrix runs.
- The cross-process lock coordinates local instances; it is not a standalone
  security boundary.
- No operation is enabled in a normal build; this alpha is not production-ready.
- Deslopper protects against accidental misuse, frontend parameter tampering,
  stale plans, and implementation errors. It is not a security boundary against
  an already-compromised user account with arbitrary same-user code execution.
