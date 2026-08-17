# Deslopper State Handoff

> Historical reconnaissance input for Owner Mode M1. This snapshot describes
> the pre-milestone repository and is retained for traceability; D-015 and the
> current Owner Mode documentation supersede its product recommendations and
> release-state claims.

Reconnaissance date: 2026-08-11  
Repository state inspected: `mutation-alpha-live-fixes` at `cacc8a5` (`fix: correct mutation prestate and no-op handling`)

## 1. Executive Summary

Deslopper currently contains two substantially different products in one repository:

1. A reasonably complete **read-only Windows inspection product**. It has a Svelte/Tauri interface, a bounded PowerShell inspection pipeline for 20 components, local SQLite history, desired-state comparisons, drift detection, diagnostics export, retention controls, and a normal NSIS installer.
2. A **feature-gated Mutation Broker Alpha**. It contains three real current-user registry mutations, durable plans and transactions, direct read-back verification, exact-state rollback, recovery marking, and an Apply/Undo UI. It is deliberately unavailable in the normal installer and is surrounded by debug, command-line, live-validation, target, approval, deny-list, phrase, nonce, and execution-limit gates.

The current NSIS installer is therefore usable only as a read-only inspector. It cannot perform any live mutation: `Cargo.toml` has `default = []`, `src/main.rs` excludes `mod mutation` without `mutation-alpha`, the normal `src/app.rs::run()` does not register mutation commands, and the frontend hides the experimental section when `get_product_info()` reports `buildMode = "normal read-only"`.

The repository is in a strong compile/test state. In this workspace, the standard Rust suite passed 67 tests, the all-feature Rust suite passed 128 tests, Clippy/format/check passed, the frontend passed 42 tests plus check/lint/build, normal and feature debug builds succeeded, the normal release NSIS bundle succeeded, and the mutation-boundary check passed. The current release artifacts are:

- `target/release/win-deslopper.exe`: 7,358,464 bytes, non-debug.
- `target/release/bundle/nsis/Deslopper_0.1.0-alpha.1_x64-setup.exe`: 2,354,384 bytes.

Passing tests do not make the mutation build main-rig ready. No checked-in evidence records a live mutation. `validation/report.md` reports zero live evidence records, and the physical-target governance history never reached an authorized execution. More importantly, source review found a concrete recovery defect: when a registry write succeeds but handler verification fails, `Broker::execute()` marks rollback available but does not capture `post_state`; `Broker::rollback()` then refuses with `No verified applied state exists.` The UI nevertheless treats `verification_failed` as undoable. There is also no automatic rollback after failed verification, and successful registry verification does not prove the visible taskbar changed because Explorer refresh is asynchronous and the app intentionally does not restart Explorer.

The scanner-to-mutation connection is much thinner than the operation count suggests. Only **Taskbar Widgets** has a detector that reads the same value the mutation writes (`TaskbarDa`). Task View and Show Desktop are broker-only subjects; the files containing them are misleadingly named after scanner components, but the scanner does not inspect `ShowTaskViewButton` or `TaskbarSd`.

The recommended direction is not a rewrite. Keep the closed operation enum, fixed handlers, policy/authority checks, exact pre-state capture, durable transaction journal, locks, direct verification, conflict-aware rollback, and recovery state. Remove the experimental authorization shell from the ordinary product: feature/debug/CLI/live-scenario/physical-target/manual-approval/typed-phrase requirements should not be part of owner mode. Generate any idempotency token internally. One deliberate Apply click should enter a transaction service that performs fresh preflight, capture, write, verification, commit, and automatic rollback when safe. Start with Widgets as the only actionable finding; do not expose Task View or Show Desktop as scanner actions until matching detectors and product cards exist.

Main-rig verdict: **YES, WITH SPECIFIC LIMITATIONS** for the current normal release strictly as a read-only inspector; **NO** for the current internal mutation build or as the desired Apply/Undo product. Details are in section 14.

## 2. Current Architecture

### Process composition

- `src/main.rs` declares the read-only modules unconditionally and declares `mod mutation` only under `#[cfg(feature = "mutation-alpha")]`. `main()` calls `app::run()`.
- `src/app.rs` contains two compile-selected `run()` implementations.
  - Normal: creates `ManagedAppState`, registers only read-only Tauri commands, and never constructs a mutation broker.
  - `mutation-alpha`: additionally creates `ManagedMutationState`, `Broker<WindowsSettingStore>`, runs interrupted-transaction recovery, and registers mutation commands.
- `Cargo.toml` has no default features. `mutation-alpha` enables optional `sha2` and Windows-only `winreg`.
- `tauri.conf.json` builds the static frontend from `frontend/build`, removes unused commands, uses the Wry WebView, applies a restrictive CSP, and bundles NSIS. The main-window capability only grants core event listen/unlisten.
- `frontend/src/routes/+page.svelte` is the current application shell. `frontend/src/lib/backend.ts` is the typed Tauri IPC adapter. There is no HTTP service and no remote backend.

### Read-only inspection path

The real read-only path is:

`+page.svelte::startInspection()`  
→ `backend.ts` invokes `start_inspection`  
→ `src/app.rs::start_inspection()` creates lifecycle state and a worker  
→ `src/inspection.rs::run_inspection()` executes a fixed query set and 20 detectors  
→ `src/app.rs` persists the completed snapshot and updates drift  
→ frontend reloads dashboard/history/drift and renders findings.

`src/inspection.rs` defines eight fixed PowerShell queries. Caller input is not interpolated into scripts. Each process uses `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass`, has a timeout/output limit, and emits progress through `deslopper://inspection-progress`:

| Query | Timeout | Output limit | Purpose |
|---|---:|---:|---|
| `PlatformInventory` | 10 s | 256 KiB | OS/build/edition/device/management context |
| `AppxCurrentUser` | 30 s | 16 MiB | `Get-AppxPackage` |
| `AppxAllUsers` | 45 s | 32 MiB | `Get-AppxPackage -AllUsers` |
| `AppxProvisioned` | 45 s | 16 MiB | `Get-AppxProvisionedPackage -Online` |
| `ManagementContext` | 12 s | 512 KiB | domain/MDM/management signals |
| `PolicyRegistry` | 12 s | 512 KiB | fixed HKLM policy values |
| `UserPreferences` | 12 s | 512 KiB | fixed HKCU preferences |
| `OneDriveMetadata` | 15 s | 1 MiB | client/account/KFM/Files On-Demand metadata, not file contents |

Shared AppX evidence is queried once, then exact package identities from `src/package_identity.rs` are evaluated. Applicability comes from `src/applicability.rs`, including edition/build rules and the Windows 11 24H2 `StartRecommendations` UBR prerequisite. `src/platform.rs` owns the domain types such as `ComponentId`, `DetectionResult`, `State`, `Authority`, `ApplicabilityResult`, and inspection lifecycle models.

### Persistence and presentation

- `src/persistence.rs` loads and writes `%LOCALAPPDATA%\Deslopper\deslopper.db`, falling back to `.deslopper/deslopper.db` if `LOCALAPPDATA` is unavailable.
- `src/app.rs` converts persisted state to dashboards, histories, comparisons, desired-state options, drift events, and diagnostics.
- `frontend/src/lib/product-ui.ts` contains UI filtering and status helpers.
- `src/privacy.rs` redacts sensitive platform data for persistence/diagnostics.
- `src/model.rs` and `src/presentation.rs` implement an older three-item mock/preview state path. Their `get_app_view`/`dispatch_app_action` commands remain registered, but the current page does not call them.

### Mutation architecture

The feature-on path adds these layers:

- `src/mutation/request.rs`: closed operation/target request types.
- `src/mutation/plan.rs`: `MutationPlan`, `CapturedState`, representation and hashing.
- `src/mutation/broker.rs`: orchestration, validation, state transitions, execution and rollback.
- `src/mutation/journal.rs`: durable SQLite plan/transaction/capture/rollback records and cross-process lock path.
- `src/mutation/transaction.rs`: transaction status machine and audit step models.
- `src/mutation/handlers/*`: one handler per operation, all backed by `WindowsSettingStore`.
- `src/mutation/governance.rs` and `src/mutation/live_validation.rs`: policy, target, approval, deny-list, scenario, machine, checkpoint, source, and evidence gates.
- `frontend/src/lib/MutationAlphaPanel.svelte` and `frontend/src/lib/mutation-alpha.ts`: experimental operation selection, plan review, approval phrase, execution, history, evidence, and Undo UI.

There is only one real Windows write backend: `src/mutation/handlers/windows_store.rs`. Desired states and preview plans do not execute writes.

## 3. Current User Journey

### Install

The NSIS installer installs Deslopper 0.1.0-alpha.1 and uses Tauri's `downloadBootstrapper` WebView2 mode. It is a normal no-feature release build. No code-signing configuration is present, so Windows reputation/signing prompts remain a distribution concern. Uninstall does not deliberately remove `%LOCALAPPDATA%\Deslopper`, so inspection history can remain after uninstall/reinstall.

### First launch

`+page.svelte` loads product information, the component catalogue, dashboard, history, drift, and any running inspection. It also checks `localStorage` key `deslopper-onboarding-complete`. If onboarding has not been completed and no stored snapshot exists, the user sees an onboarding prompt. The user can start an inspection or skip. A scan is **not automatic** merely because the app launched.

### Initial scan

The scan is explicit and cancellable. Progress is surfaced from the Tauri event stream. The app runs the eight fixed PowerShell queries and the 20 detector evaluations. Cancellation kills the spawned child for the active query, but it is not a Windows job-object tree cancellation mechanism. On completion, the snapshot, observations, package evidence, and drift changes are saved to SQLite.

The scan reads system package inventories including all-user and provisioned packages. It reads fixed registry/policy values and OneDrive metadata, but it does not read personal file contents, uninstall packages, write registry values, restart processes, or contact a Deslopper service.

### Findings

The user can browse Overview and Components, search/filter components, open a detail view, inspect current state, authority, evidence, applicability, timeline, and package details, and save a desired state. Desired-state actions generate non-executable previews (`executorEnabled = false`). Drift and history pages compare snapshots and desired state. Diagnostics can be saved through the WebView File System Access API or browser download.

The normal build explicitly communicates that Apply is unavailable in Product Alpha. This is not a runtime gate the user can satisfy; the mutation commands and Rust module do not exist in the binary.

### Apply and Undo

- **Normal installed build:** no Experimental Apply & Undo navigation item, no mutation panel, no mutation Tauri commands, no Apply, and therefore no Undo.
- **Feature-on internal build:** the navigation adds Experimental Apply & Undo. Even there, the UI is blocked unless every compile/debug/CLI/live-validation/target/approval/warning/freshness/operation gate passes. The user must review a generated plan and type an exact approval phrase. Undo is shown for qualifying durable transaction states.

### Relaunch

Read-only state survives because SQLite is loaded on startup. Existing snapshots suppress first-run onboarding. Desired states, drift, inspection history, retention preference, and mutation audit records remain.

Mutation relaunch behavior is uneven:

- A `RollbackAvailable` transaction can be reloaded from mutation history and Undo can be attempted.
- An `AwaitingApproval` transaction cannot practically be resumed because the frontend does not persist/reconstruct the broker-issued raw nonce and `IssuedPlan`; only the nonce hash is durable. The plan expires after five minutes and leaves an audit row.
- The in-app mutation warning acknowledgment is an `AtomicBool` and resets each process launch.
- Startup calls `Broker::recover_interrupted()`, which marks transitional transactions `RecoveryRequired`; it does not replay a write or rollback automatically.

## 4. Inspection Components

The scanner has 20 catalogue components. None is a dead detector. Only one is actually connected to a mutation that reads/writes the same representation.

| Component ID | What is inspected | Classification | Actionability note |
|---|---|---|---|
| `onedrive` | Client presence/version, process/startup, account types, sync roots, known-folder redirection, KFM and Files On-Demand policy/metadata | Inspection only | No OneDrive mutation exists. No file contents are read. |
| `microsoft_365_copilot` | Exact AppX identities `Microsoft.MicrosoftOfficeHub` and, from build 26100, `Microsoft.Microsoft365Copilot` | Inspection only | No package removal/change path. |
| `phone_link` | Exact AppX identity `Microsoft.YourPhone` | Inspection only | No package mutation. |
| `consumer_copilot` | Exact AppX identity `Microsoft.Copilot` from build 22621 | Inspection only | No package mutation. |
| `widgets_platform` | Policy `HKLM\Software\Policies\Microsoft\Dsh\AllowNewsAndInterests` plus related context | Inspection only | Distinct from the current-user taskbar Widgets button. |
| `consumer_experiences` | CloudContent policy `DisableWindowsConsumerFeatures` | Inspection only | Policy support is partial on Home/Pro. |
| `welcome_experience` | CloudContent policy plus HKCU ContentDeliveryManager `SubscribedContent-310093Enabled` | Inspection only | No mutation. |
| `tips_suggestions` | CloudContent policy plus `SoftLandingEnabled` | Inspection only | No mutation. |
| `lock_screen_suggestions` | Spotlight policy plus `RotatingLockScreenOverlayEnabled` | Inspection only | No mutation. |
| `start_recommendations` | Start policy `HideRecommendedSection`; build 26100 requires UBR 4770+ | Inspection only | No mutation. |
| `notification_suggestions` | `DisableWindowsSpotlightOnActionCenter` plus `SubscribedContent-338389Enabled` | Inspection only | No mutation. |
| `settings_suggested_content` | `DisableWindowsSpotlightOnSettings` plus `SubscribedContent-338393Enabled` | Inspection only | No mutation. |
| `search_web_results` | Search policy `DisableWebSearch`/documented web-result authority | Inspection only | No mutation. |
| `search_highlights` | Search policy `EnableDynamicContentInWSB`/search-highlight authority | Partially wired | A broker handler file is named `search_highlights.rs`, but it actually changes Show Desktop `TaskbarSd`. There is no functional Apply for Search Highlights. |
| `taskbar_widgets` | HKCU Explorer Advanced `TaskbarDa` | Inspection + mutation exists | The only true scanner/action match. |
| `taskbar_search` | HKCU Search `SearchboxTaskbarMode` | Partially wired | A broker handler file is named `taskbar_search.rs`, but it actually changes Task View `ShowTaskViewButton`. There is no functional Apply for taskbar search. |
| `personal_teams_chat` | Exact legacy/successor AppX identities `MicrosoftTeamsClassic`/`MicrosoftTeams` | Inspection only | No package mutation. |
| `clipchamp` | Exact AppX identity `Clipchamp.Clipchamp` | Inspection only | No package mutation. |
| `news_weather` | Exact companion identities `Microsoft.BingWeather` and `Microsoft.BingNews` | Inspection only | No package mutation. |
| `solitaire` | Exact AppX identity `Microsoft.MicrosoftSolitaireCollection` | Inspection only | No package mutation. |

The broker-only subjects `taskbar_task_view` and `taskbar_show_desktop` are not members of the 20-component detector catalogue. Before presenting either as an actionable finding, add detectors for the exact values and give them honest product identities. Renaming the two existing handler files is also warranted.

## 5. Implemented Mutations

All implemented operations are closed, single-step, current-user operations. `PlanRequest` uses `deny_unknown_fields`, and `MutationOperationId::ALL` contains exactly three entries. No arbitrary registry path, PowerShell, executable, or command text can be supplied by the frontend.

All three target `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced`, write a DWORD `0` or `1`, directly read it back, capture the exact previous DWORD or value absence, and Undo by restoring that DWORD or deleting the value if it was previously absent. They require Windows build 22000+ and an edition string containing Core/Home/Professional/Pro/Enterprise/Education. They declare `current_user_unprivileged`; no elevation is requested.

| User-facing operation | Internal ID / subject | Read and change | Policy refusal | Apply / verify / Undo | Physical status / ordinary-build blocker |
|---|---|---|---|---|---|
| Show or hide Widgets taskbar button | `set_taskbar_widgets_visibility`; enum `WidgetsVisibility`; subject `taskbar_widgets` | `TaskbarWidgetsHandler` reads/writes `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDa` DWORD 0/1 | Refuses if `HKLM\Software\Policies\Microsoft\Dsh\AllowNewsAndInterests` is configured, or HKCU/HKLM `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer\NoSetTaskbar = 1` | Apply implemented; handler read-back implemented; full post-inspection context implemented; exact rollback implemented. Missing value has unknown effective state, so an explicit value is written. | This was the intended bounded physical validation operation, but no live execution evidence exists. Absent from normal binary; internal build also needs every alpha/live approval gate. |
| Show or hide Task View taskbar button | `set_taskbar_task_view_visibility`; enum `TaskViewVisibility`; subject `taskbar_task_view` | `TaskbarTaskViewHandler` in misleading file `handlers/taskbar_search.rs` reads/writes `...\Advanced\ShowTaskViewButton` DWORD 0/1 | Refuses if HKCU or HKLM `Software\Policies\Microsoft\Windows\Explorer\HideTaskViewButton` is configured at either 0 or 1, or `NoSetTaskbar = 1` | Apply, direct verify, and exact Undo implemented. Missing value is interpreted with documented default enabled. Full post-inspection does not inspect this value, so the second-stage verification mainly reconfirms platform/authority plus another direct handler read. | Designed for the same mutation-alpha machinery and synthetic execution; no physical evidence. Not compiled/registered in ordinary build and not represented by a matching scanner component. |
| Enable or disable far-corner Show Desktop | `set_taskbar_show_desktop_enabled`; enum `ShowDesktopEnabled`; subject `taskbar_show_desktop` | `TaskbarShowDesktopHandler` in misleading file `handlers/search_highlights.rs` reads/writes `...\Advanced\TaskbarSd` DWORD 0/1 | Refuses when HKCU/HKLM `Software\Microsoft\Windows\CurrentVersion\Policies\Explorer\NoSetTaskbar = 1` | Apply, direct verify, and exact Undo implemented. Missing value is interpreted with documented default enabled. Full post-inspection does not inspect this value. | Designed for mutation-alpha/synthetic paths; no physical evidence. Not compiled/registered in ordinary build and lacks a matching scanner component. |

Handler definitions are in `src/mutation/handlers/taskbar_widgets.rs`, `taskbar_search.rs`, and `search_highlights.rs`. All real Windows I/O is in `src/mutation/handlers/windows_store.rs`. No operation invokes PowerShell, restarts Explorer, calls WinRT, uninstalls AppX, or writes HKLM.

## 6. Apply / Verify / Undo Trace

### Widgets Apply trace

1. `frontend/src/routes/+page.svelte` adds the Experimental Apply & Undo navigation entry only when `isMutationAlphaBuild(productInfo.buildMode)` succeeds.
2. `MutationAlphaPanel.svelte` constructs `MutationAlphaWorkflow` from `frontend/src/lib/mutation-alpha.ts` and calls backend methods from `frontend/src/lib/backend.ts`.
3. The panel loads `get_mutation_alpha_status`, `get_mutation_operation_options`, and `get_mutation_history`. It requires the warning checkbox and calls `acknowledge_mutation_alpha_warning`.
4. The user selects `set_taskbar_widgets_visibility` plus `enabled` or `disabled`. `createPlan()` invokes Tauri command `generate_mutation_plan` with `PlanRequest { operation_id, target, source_inspection_id }`.
5. `src/app.rs::generate_mutation_plan()` builds a `BrokerContext` from the latest persisted observation through `mutation_context()`, then calls `Broker::generate_plan()`.
6. `Broker::generate_plan()` checks the alpha/live gate, target eligibility, operation authorization, fresh source inspection, applicability/authority/confidence, direct handler state, operation allowance, and execution allowance. It creates a five-minute `MutationPlan` bound to machine/build/edition/source observation/current representation/target/approval class, hashes the plan, stores only a hash of a random approval nonce, saves it through `MutationJournal::save_plan()`, and creates an `AwaitingApproval` transaction.
7. The raw nonce is returned only in the `IssuedPlan` response. The UI validates the issued plan and displays exact representation, expected effect, rollback, warnings, and an operation-specific approval phrase such as `APPROVE WIDGETS TEST`.
8. After the phrase matches, `execute()` invokes `approve_and_execute_mutation` with `ApprovalRequest { plan_id, approval_nonce, acknowledged: true }`.
9. `src/app.rs::approve_and_execute_mutation()` confirms the latest completed inspection is still the source, runs a fresh full inspection via `live_mutation_context()`, then calls `Broker::execute()`.
10. `Broker::execute()` rechecks the gates, obtains an in-process mutex and the SQLite-adjacent `.mutation-alpha.lock`, loads and validates the plan hash, expiry, unused state and nonce hash, checks scoped approval and usage limits, and transitions the durable transaction through `Validating` and `Approved`.
11. The handler captures exact pre-state. The broker refuses if authority changed or the actual representation/effective value differs from the reviewed plan. It hashes and saves pre-state before consuming the one-use plan.
12. If already compliant, it records `NoChangeNeeded` and does not write. Otherwise it marks rollback available, re-reads the full gate/authorization/target state immediately before mutation, and transitions to `Applying`.
13. `TaskbarWidgetsHandler::apply()` calls `WindowsSettingStore::write_widgets()`, which opens the existing HKCU Explorer Advanced key with `KEY_READ | KEY_WRITE` and writes `TaskbarDa` 0/1.
14. The transaction transitions to `Verifying`; `TaskbarWidgetsHandler::verify()` reads `TaskbarDa`, checks policy/authority, and proves representation/effective value match the target. The broker hashes/stores `post_state`, records `applied_and_verified`, and transitions to `Applied`.
15. `src/app.rs` runs another full inspection and passes a new context to `Broker::complete_effective_verification()`. That function verifies transaction integrity, machine/build/edition, authority/confidence/applicability, and another direct handler read. Success transitions to `RollbackAvailable`.
16. The final transaction is returned through Tauri. `MutationAlphaWorkflow` converts its status to a UI phase; the panel displays success/history and enables Undo.

Task View and Show Desktop follow the same route with different fixed handlers/values. Their full scan does not independently observe the changed value, so their effective-verification phase is not a true detector-level confirmation.

### Undo trace

1. The panel calls `MutationAlphaWorkflow.undo()`, which invokes `rollback_mutation` with the transaction ID, `acknowledged: true`, and `allowConflict: false`.
2. `src/app.rs::rollback_mutation()` performs a fresh full inspection/context and calls `Broker::rollback()`.
3. The broker requires the rollback environment, process locks, a valid durable transaction, an allowed status (`RollbackAvailable`, `VerificationFailed`, or `RecoveryRequired`), matching machine/build/edition, and acceptable current authority.
4. It loads exact `pre_state` and verified `post_state`, reads current state directly, and detects whether another actor changed the value after Apply. A conflict is refused unless `allow_conflict` is true.
5. The handler restores the exact prior DWORD or prior absence and reads it back. The broker stores `rollback_state`, hashes it, and transitions to `RolledBack`.
6. `src/app.rs` runs another full inspection and `complete_rollback_effective_verification()`, then returns the transaction to the UI.

The backend supports an explicit conflict override, but the UI always sends `allowConflict: false` and offers no second-step “restore anyway” flow.

### Critical failure path

In `Broker::execute()`, `transaction.rollback.available = true` is saved before the write. If the write succeeds and `handler.verify()` returns an error, the broker records `VerificationFailed` but never stores an observed `post_state`. `canUndo()` in `frontend/src/lib/mutation-alpha.ts` treats that status as undoable; `Broker::rollback()` then requires `post_state` and returns `rollback_unavailable: No verified applied state exists.` This must be fixed before owner-mode mutation. On verification failure, the broker should capture the actual current representation and attempt an automatic exact rollback when the current value is attributable to the attempted write; otherwise it should preserve a recoverable transaction with enough actual-state evidence for deliberate recovery.

## 7. Complete Mutation Gate Sequence

The effective path from source code to one registry write is:

1. **Compile inclusion:** Cargo feature `mutation-alpha` must include `src/mutation` and optional `winreg`/`sha2`.
2. **Command/UI composition:** feature-on `src/app.rs::run()` must register mutation commands; `get_product_info()` must return `internal mutation-alpha compile`; the frontend then shows the experimental section.
3. **Debug build:** `cfg!(debug_assertions)` must be true. A feature-on release still fails this gate.
4. **CLI alpha opt-in:** process arguments must contain `--enable-mutation-alpha`.
5. **CLI live-validation opt-in:** arguments must contain `--enable-live-validation`.
6. **CLI identity bindings:** exact values must be supplied for `--validation-scenario=...`, `--expected-machine-id=...`, `--expected-checkpoint-id=...`, and `--expected-source-commit=...`.
7. **Committed policy:** `.deslopper/policy.toml` must parse, allow mutation, permit the requested validation target type, and define the expected schema/constraints. It currently says `allow_windows_mutation = false`.
8. **Local scenario:** ignored local files under `.deslopper/local` must include a matching live-validation scenario/manifest. `local_validation_directory()` is based on current working directory.
9. **Local scoped approval:** the approval must normalize, be unexpired, have acceptable recovery fields, authorize the exact operation and direction, have execution allowance, and match scenario/machine/checkpoint/source/inspection/evidence/platform/build/UBR/management/target-type constraints.
10. **Development deny-list:** the deny-list must parse, be distinct from the target, and the current host must not match a permanently refused development identity.
11. **Physical/VM eligibility:** explicit target allow-list and target-type-specific checks must pass, including VM-detection/management assumptions and database machine identity binding.
12. **In-app warning:** `acknowledge_mutation_alpha_warning(true)` must set the process-local warning acknowledgment.
13. **Fresh completed inspection:** a completed source inspection/observation must exist and be less than ten minutes old.
14. **Operation maturity/definition:** the hardcoded operation must not be `Rejected`; currently synthetic-tested maturity can proceed if all external gates pass. Build/edition/target/privilege constraints must match.
15. **Direct authority/applicability check:** the fixed handler must read a known 0/1 or supported missing representation and must not find an external policy that owns/blocks the setting.
16. **Plan allowance:** the operation/direction must be allowed by the scoped approval, and remaining execution count must be positive.
17. **Plan integrity:** plan ID/hash/context/current state must be valid; plan must be younger than five minutes and unused.
18. **UI exact phrase:** the user must type the operation-specific approval phrase and send `acknowledged = true`.
19. **Nonce:** the raw one-time nonce returned with the plan must hash to the persisted nonce hash.
20. **Fresh execution context:** `src/app.rs` reruns the full inspection immediately before `Broker::execute()` and requires the source inspection still be latest.
21. **Execution locks:** the broker must obtain both its process mutex and `.mutation-alpha.lock` cross-process lock.
22. **Pre-state consistency:** the direct state/authority must still equal the reviewed plan; exact pre-state is persisted before write.
23. **Immediate reauthorization:** alpha/live gate, scoped authorization, target eligibility, policy, approval, deny-list, and management checks are reread immediately before write.
24. **Fixed write:** the operation-specific handler writes only its hardcoded HKCU DWORD.
25. **Direct verification:** the same handler reads back representation, effective value, and authority.
26. **Effective reinspection:** the app performs another full inspection/context and transitions the transaction to `RollbackAvailable` only if context and direct state still agree.

There is no environment-variable shortcut. Environment reads are ordinary system inputs such as `COMPUTERNAME` and `LOCALAPPDATA`; Vite's `VITE_`/Tauri build environment is not a mutation authorization surface.

One binding gap remains: the live approval gate is constructed at application startup from the then-latest snapshot. A newer scan can later become the plan source, but plan generation does not bind the active approval's source inspection to that newer inspection with the same strictness as the frontend/source checks. Owner mode should remove approval objects, but the replacement transaction must still bind to the fresh observation it actually validated.

## 8. Safety & Governance Classification

Classification meanings are the requested A–D categories. Recommendations refer to owner mode, not changes authorized by this handoff.

| Mechanism | Current location | Class | Owner-mode recommendation |
|---|---|---:|---|
| Closed `MutationOperationId`, fixed targets, `deny_unknown_fields` | `request.rs`, `broker.rs` | A | Keep. This prevents arbitrary execution surfaces. |
| Fixed registry paths/value names; no caller-supplied commands | `handlers/windows_store.rs` | A | Keep. |
| Build/edition/applicability checks | `operation_definition()`, `validate_context()` | A | Keep, improve edition parsing and maintain an explicit tested support matrix. |
| External policy/authority detection | handlers/Windows store | A | Keep. Do not fight managed policy. |
| Direct pre-state capture and reviewed-state recheck | `Broker::execute()` | A | Keep. Bind it to the immediate Apply attempt, not a ceremonial approval. |
| Durable pre/post/rollback state and hashes | broker/journal/transaction | A | Keep. Fix incomplete capture on failures. |
| Direct read-back verification | handlers, `verification.rs` | A | Keep. Add matching detector/visible-state semantics where needed. |
| Exact rollback and conflict detection | `Broker::rollback()`, `rollback.rs` | A | Keep. Normal Undo is one click; a genuine post-Apply conflict may justify a second confirmation. |
| In-process and cross-process write locks | broker/journal lock | A | Keep invisibly. |
| Transaction recovery marking after interruption | `recover_interrupted()` | A | Keep; improve the recovery UX and evidence. |
| Machine/build/edition binding on an existing transaction | broker | A | Keep enough to prevent restoring one machine's state on another; replace weak identity derivation. |
| No elevation for these HKCU operations | operation definitions | A | Keep. Do not add UAC to the initial slice. Build a separate elevation boundary only when a future operation truly needs it. |
| Fresh inspection and fresh direct preflight | `app.rs`, broker | B | Keep automatically. Do not make the user understand freshness windows. |
| Five-minute plan and plan hash | plan/broker | B | Internalize or collapse into a short-lived transaction intent created by the Apply click. The user does not need a separate review artifact. |
| One-use/idempotency token | plan/journal | B | Keep as an internal generated request/transaction token if useful. Do not expose it as authorization. |
| Full post-mutation reinspection | `app.rs`, broker | B | Keep when bounded; ensure it observes the actual changed setting. A targeted detector may be enough for the immediate result, followed by background refresh. |
| Immediate policy/eligibility reread before write | broker/live validation | B | Keep the technical policy/authority part invisibly; remove target-governance rereads. |
| Mutation tables preserved by Clear History | persistence | B | Keep audit/undo records by default and provide a separate deliberate retention policy later. |
| Compile-time mutation feature separation | `Cargo.toml`, cfgs in `main.rs`/`app.rs` | C | Remove from the ordinary product composition or make mutation capability the product default. A separate developer no-write feature can remain only if it has a concrete test purpose. |
| Debug-only mutation requirement | gate construction | C | Remove. It makes an installable release incapable by design. |
| `--enable-mutation-alpha` | live gate | C | Remove from owner mode. |
| `--enable-live-validation` and scenario/checkpoint/source CLI bindings | `live_validation.rs` | C | Remove from normal product execution; retain only in an archived/explicit validation harness if still useful. |
| VM/physical target type and physical-target lifecycle governance | governance/live validation/policy/matrix | C | Remove from normal runtime. It was experiment authorization, not machine safety. |
| Machine allow-list and permanent development-host deny-list | local validation/governance | C | Remove from owner runtime. A development safety test can keep a no-write mock backend without refusing the installed owner's machine. |
| Local approval JSON, expiry, recovery reviewer fields, approval class, execution quota | live validation/journal | C | Remove from normal runtime and schema usage. Preserve old rows for migration/audit compatibility. |
| Evidence-export eligibility as a prerequisite | live validation/evidence UI | C | Decouple. Diagnostics/evidence export may remain a support tool, never a prerequisite to Apply. |
| Current-working-directory `.deslopper/local` discovery | `local_validation_directory()` | C | Remove from product. It is brittle for installed apps. |
| Process-local warning acknowledgment checkbox | broker/panel | D | Remove. The deliberate Apply click plus clear action copy is sufficient user authorization. |
| Exact typed approval phrase | `approval_phrase()`, panel | D | Remove. It adds effort but does not improve the registry transaction. |
| Separate “prepared but blocked” UI and detailed gate matrix | `MutationAlphaPanel.svelte` | D | Replace with concise actionable reasons such as “managed by your organization” or “run a new scan.” |
| Exposed plan ID, nonce ceremony, approval class, target/scenario/checkpoint terminology | mutation panel/workflow | D | Hide/remove from product UI. Internal IDs may remain in support details. |
| Multiple acknowledgments (`warning_acknowledged`, phrase, `ApprovalRequest.acknowledged`) | request/broker/panel | D | Collapse to the Apply click. Keep a separate conflict override only when the state changed after Apply. |

The strongest safety in the repository is the fixed executor plus transaction implementation, not the experimental target paperwork. Owner mode should retain the former and delete the latter from the normal path.

## 9. Build Modes and Release Behaviour

| Mode | Composition and behavior |
|---|---|
| Normal build | `default = []`; mutation module, broker, handlers and Tauri commands are not compiled. Product info says `normal read-only`; frontend does not add experimental navigation. |
| Mutation build | Requires `--features mutation-alpha`; compiles optional `sha2`/`winreg`, mutation modules and alternate `run()`, and reports `internal mutation-alpha compile`. This only exposes the gate/UI; it does not itself authorize writes. |
| Debug build | `cfg!(debug_assertions) = true`; required by the alpha gate. With the feature and exact runtime governance inputs it can reach handlers. Without the feature it remains read-only. |
| Release build | `cfg!(debug_assertions) = false`. A normal release is read-only. Even a release built with `mutation-alpha` is blocked by the debug gate unless code changes. |
| NSIS bundle | `bun run tauri build` uses default Cargo features because no feature argument/default enables mutation. The current installer is therefore normal read-only. |

Meaningful switches are:

- Cargo: `mutation-alpha` only.
- CLI: `--enable-mutation-alpha`, `--enable-live-validation`, `--validation-scenario=...`, `--expected-machine-id=...`, `--expected-checkpoint-id=...`, `--expected-source-commit=...`.
- Environment: no mutation opt-in. `COMPUTERNAME` participates in validation identity; `LOCALAPPDATA` chooses database location. Inspection scripts use standard Windows environment variables for fixed OneDrive paths.
- Runtime UI: process-local warning acknowledgment and exact typed approval phrase.

Why the current installer cannot mutate, in exact order:

1. The build command selected no Cargo feature.
2. `src/main.rs` did not compile `src/mutation`.
3. normal `src/app.rs::run()` did not construct `Broker<WindowsSettingStore>` or register mutation commands.
4. `get_product_info()` reported normal read-only.
5. `+page.svelte` omitted the Experimental Apply & Undo navigation.
6. Even direct IPC cannot call commands that were removed/not registered.

The existing compile boundary is effective: the boundary scanner passes and the normal product has no mutation symbol path. It is also the direct reason the release cannot meet the new product goal.

## 10. Persistence Model

### Location and database behavior

`src/persistence.rs` uses schema version 5 and SQLite at `%LOCALAPPDATA%\Deslopper\deslopper.db`, with a repository-relative fallback. It enables foreign keys, WAL journaling, `synchronous = NORMAL`, a five-second busy timeout, migration transactions, and `quick_check`. An older JSON store can be imported once.

### Read-only/product tables

| Table | Role |
|---|---|
| `database_metadata` | schema/import metadata |
| `machines` | local hashed machine ID and redacted platform summary |
| `inspections` | inspection lifecycle/snapshot/platform payload |
| `component_observations` | per-component current state, detector/applicability/authority/evidence |
| `package_observations` | exact package registration/provisioning evidence |
| `desired_states` | active desired value/note/validity per component |
| `desired_state_revisions` | revision history for desired state |
| `drift_events` | detected desired/current or snapshot changes and acknowledgments |
| `preview_plans` | non-executable desired-state preview JSON |
| `migration_log` | package identity migration observations |
| `product_preferences` | retention and related product preferences |

Inspections, observations, desired-state revisions and drift are fully local. `clear_local_history()` removes read-only history while preserving mutation audit tables.

### Mutation tables

| Table | Role |
|---|---|
| `mutation_plans` | plan JSON/hash, nonce hash, expiry and consumed timestamp |
| `mutation_transactions` | transaction identity/status/context, operation, target, hashes, result/error/recovery fields and transaction JSON |
| `mutation_steps` | ordered state-transition/audit steps with redacted evidence and errors |
| `mutation_state_captures` | pre/post/rollback captured-state JSON and hashes |
| `mutation_rollbacks` | availability, completion, attempt/result/verification/conflict fields and rollback-plan JSON |

These tables are created by migration even in normal builds. The previous value is `MutationTransaction.pre_state` and a `pre` capture row; the applied value is `post_state`; Undo information is exact pre-state plus `RollbackRecord`, not a generic inverse command. Verification strings and hashes are stored with the transaction/captures. Transitional statuses allow startup recovery marking.

The raw one-time nonce is **not** persisted; only `approval_nonce_hash` is. The typed phrase is not persisted. The approval class is inside plan/transaction data and usage is derived from journal records. Live scenario manifests, target approvals, and the development deny-list are ignored JSON files under `.deslopper/local`, not SQLite product data.

Machine identity has two implementations:

- Product/database `persistence::machine_identity()` uses Rust `DefaultHasher` over device name, architecture and product name, producing a short implementation-dependent hash. This is weak as a durable cross-version identity contract.
- Live validation derives a separate SHA-256 identity from computer name/edition/build and binds it to local approval material.

Platform payloads redact device name and user SID before normal persistence/diagnostics. The app still stores enough local inventory/history that the database should be treated as private local diagnostic data.

### Simplification opportunities

- Keep normalized mutation transactions, captures, rollback and steps. They are useful product mechanisms.
- Remove new usage of approval class/approval quota/nonce hash from owner authorization. Keep old columns/tables readable for migration compatibility rather than destructively rewriting schema 5 immediately.
- Replace separate user-visible `mutation_plans` with an internal short-lived transaction intent, or keep the table but create/consume it automatically in one Apply IPC.
- Consolidate duplicated full transaction JSON and normalized columns only after 0.1; changing both journal and migration now adds risk without unlocking the product.
- Retire `preview_plans` once the real actionable plan replaces non-executable preview, or keep preview only for inspection-only components with a clear name.
- Review vestigial desired-state flags such as persistent/approval-required semantics before expanding actionability.
- Replace the `DefaultHasher` machine ID with a versioned, stable local identifier before depending on it for long-lived rollback safety.

## 11. Windows Low-Level Implementation Review

### What is good

- The writable surface is extremely narrow: one existing HKCU key and three hardcoded DWORD names.
- No user-controlled path, value name, script, executable, or shell command reaches the writer.
- `winreg` performs direct registry access; PowerShell is inspection-only.
- Pre-state distinguishes a DWORD from value absence, allowing exact restoration.
- Values outside 0/1 are refused rather than normalized.
- Relevant fixed policy keys are checked before write and during direct verification.
- Every normal write is followed by direct read-back; Undo is also directly verified.
- No operation requests elevation, writes HKLM, restarts Explorer, logs off, reboots, kills a process, or uninstalls a package.

### Risks and assumptions

| Area | Finding | Consequence / required action |
|---|---|---|
| Hive/scope | All writes are HKCU and correctly match current-user taskbar preferences. HKLM is read only for policy. | Appropriate for initial owner mode; no UAC should appear. Clearly describe per-user scope. |
| Registry view | Code uses `KEY_READ`/`KEY_WRITE` without `KEY_WOW64_64KEY` or `KEY_WOW64_32KEY`. The x64 NSIS app will use its native view. | Explorer HKCU paths are normally appropriate, but registry-view intent should be documented and tested on the supported x64 build. Do not imply ARM64/32-bit support without tests. |
| Existing key | `open_subkey_with_flags()` does not create `...\Explorer\Advanced`. A missing key causes `MissingRepresentation`; only a missing value inside an existing key is handled. | Rare on a normal profile, but owner-mode error copy should be clear or safely create the known key after deciding that behavior. |
| Value type | Reading a non-DWORD as `u32` fails; it is not backed up as raw registry data. | Refusal is safer than coercion, but the transaction should report unsupported representation and perform no write. |
| Policy semantics | Task View treats either configured `HideTaskViewButton` value as external ownership. Widgets treats any configured `AllowNewsAndInterests` value as external ownership. `NoSetTaskbar` blocks only at value 1. | Conservative and appropriate. Surface “managed by policy,” not internal authority terminology. |
| Shell refresh | Definitions explicitly say no Explorer termination and “shell refresh may be asynchronous.” No broadcast or shell notification is sent. | Registry verification can succeed while the visible taskbar has not yet updated. Determine whether natural refresh is reliable on 24H2/25H2; otherwise add a non-destructive supported refresh mechanism or tell the user when sign-out/restart is needed. Never silently kill Explorer. |
| Process/logoff/reboot | No operation declares a required restart. | This is unproven physically. Manual validation must record immediate visual behavior and post-relaunch state. |
| Apply failure | A write error after transition marks `FailedAfterMutation`; the broker does not prove whether a partial/ambiguous write occurred. | Capture actual state after any write error when possible, then rollback only when attribution is safe. |
| Verification failure | `post_state` is absent, UI says Undo is available, rollback requires `post_state`. No automatic rollback. | Critical blocker. Fix and add fault-injection tests before real use. |
| Rollback conflict | Backend can override conflict with `allow_conflict`; UI cannot. | Add a clear second deliberate action only for a real conflict, showing current/applied/original values. |
| Crash recovery | Transitional transactions become `RecoveryRequired`; no write is replayed. | Good default, but recovery needs actual-state inspection and a guided restore/accept-current path. |
| Build support | Operations hardcode minimum build 22000, no maximum, and edition support uses substring matching. | Too broad to claim 25H2 confidence. Maintain tested build/UBR evidence and use the same normalized edition logic as applicability. |
| Scanner mismatch | Task View and Show Desktop have no matching detector. | Add exact read-only detectors before exposing them as findings. |
| Visible verification | No screenshot/UI Automation/shell API proves taskbar visibility. | For 0.1, direct registry + matching detector + documented manual visual check is acceptable; do not claim visible verification until established. |
| Machine identity | `DefaultHasher` identity is not a versioned stable security identifier. | Replace before relying on it to authorize recovery across upgrades. |

### Inspection command quality

The scanner's PowerShell scripts are fixed string literals and do not interpolate frontend data, so classic quoting/injection exposure is low. Timeouts and output limits bound execution. The AppX all-user/provisioned commands may fail or return partial evidence under permissions/servicing conditions; detector status models unknown/failed/cancelled states rather than assuming absence. The process cancellation implementation kills the direct PowerShell child but does not use a Windows job object, so descendants are not formally guaranteed to terminate.

### 24H2/25H2 confidence

The read-only applicability matrix explicitly handles one 24H2 servicing threshold: build 26100 UBR 4770 for `HideRecommendedSection`. The three mutation definitions only require build 22000 and have no UBR/max-build constraints. Documentation references 24H2/25H2 validation planning, but no live evidence exists. Therefore source correctness is plausible for these well-known HKCU values, but physical behavior on the developer's exact build remains an open validation item.

## 12. Current UI

### Normal product navigation

`frontend/src/routes/+page.svelte` provides:

- **Overview** — platform summary, scan status, scan/cancel controls, freshness and high-level findings.
- **Components** — searchable/filterable catalogue and detail view with current state, authority, applicability, evidence, package details and timeline.
- **Desired states** — save/clear a desired value and generate a read-only preview.
- **Drift** — filter, inspect, acknowledge and compare drift.
- **History** — inspection list/detail and snapshot comparison.
- **Settings & About** — retention, clear local history, diagnostics export, privacy/product/build information.

The normal build has solid inspection UX for an alpha: bounded progress, cancellation, unknown/error states, history and explanatory details. It does not automatically scan on every launch, and actionable outcomes are intentionally absent. Desired-state language can mislead a user into expecting execution even though previews say the executor is disabled.

### Mutation UI

Feature-on builds append **Experimental Apply & Undo** and mount `MutationAlphaPanel.svelte`. That panel exposes a gate checklist, target readiness, current validation identity, operation choices, target direction, plan representation, freshness, expected effect, rollback method, exact phrase input, Apply, transaction phase/history, live-evidence controls, and Undo.

The UI correctly avoids hiding mutation errors; `mutationError()` maps broker categories and the panel shows transaction/error/recovery text. It also validates that an issued plan matches the selected operation/target/source before enabling approval.

For an owner product, the panel is dominated by internal research terms: Mutation Alpha, gate, live validation, target, scenario, checkpoint, source commit, approval scope/class, execution allowance, evidence, operation maturity, plan ID, nonce lifecycle, exact approval phrase, recovery requirement, and “prepared but blocked.” Most should disappear from primary UI.

Recommended product presentation:

- Put Apply on the matching finding/detail card, not in a separate experimental control room.
- Show concrete copy: “Hide Widgets button for this Windows account,” exact scope, policy ownership, and whether Undo is supported.
- One Apply click starts the transaction. A compact in-progress state covers checking, applying and verifying.
- Result states should be `Changed`, `Already set`, `Could not change`, `Restored`, or `Needs attention`, with technical details expandable.
- Show Undo beside the last successful change and in a unified change history.
- Reserve a second confirmation only for genuinely destructive/elevated future actions or overwriting a post-Apply conflict.
- Keep plan hashes, machine IDs, raw registry representation and support evidence in an expandable diagnostics view, not the normal flow.

`docs/screenshots/tauri-sveltekit-preview.png` depicts the old preview-era UI and is stale relative to the current multi-section page.

## 13. Legacy / Dead Weight

No code should be deleted solely from this inventory; these are candidates for deliberate retirement or archival.

### Clear runtime dead weight

- `src/model.rs` and `src/presentation.rs` implement an older three-item mock application (`ManagedAppState`, `AppAction`, `AppView`). `src/app.rs::get_app_view()` and `dispatch_app_action()` are registered but unused by current `+page.svelte`/`backend.ts` product calls. Their tests preserve obsolete preview behavior.
- `src/app.rs::get_migration_status()` and `get_vm_validation_metadata()` are registered but have no current frontend callers.
- `docs/screenshots/tauri-sveltekit-preview.png` documents the old UI.

### Validation/governance infrastructure

The following chiefly exists to prove mutations could not reach an unapproved development/physical machine:

- `src/mutation/governance.rs`
- much of `src/mutation/live_validation.rs`
- `.deslopper/policy.toml` mutation authorization fields
- `.deslopper/local/development-host-denylist.json`
- `validation/approved-validation-targets*.schema.json`
- `validation/approved-validation-vms.schema.json`
- `validation/live-validation-scenario*.schema.json`
- `validation/approval-review-draft.schema.json`
- `validation/live-evidence.schema.json` and live-evidence product flow
- physical lifecycle fields in `validation/matrix.json`
- `tools/Export-DeslopperValidationIdentity.ps1`
- `tools/Invoke-DeslopperHyperVValidation.ps1`
- `tools/Invoke-DeslopperRecoveryAudit.ps1`
- `tools/New-DeslopperApprovalReviewDraft.ps1`
- `tools/New-DeslopperLiveScenarioManifest.ps1`
- associated approval/deny-list/physical-readiness tests and most mutation-alpha gate UI tests
- governance-heavy documents including `docs/live-validation-environment.md`, `docs/manual-vm-provisioning.md`, `docs/physical-validation-target-readiness.md`, `docs/vm-mutation-protocol.md`, `docs/vm-validation.md`, and parts of `docs/mutation-broker-alpha.md`/threat model/decision log.

Archive useful research and retain the validation harness if it remains valuable, but decouple it from normal runtime. Do not retain a hidden alternate authorization path that can diverge from owner mode.

### Useful infrastructure that may look legacy but should remain

- `tools/scan-mutation-boundary.ps1` remains valuable if the project keeps a deliberate no-write build or as a packaging assertion.
- `validation/mutation-fixtures/mutation-alpha-replay.json` and transaction fault tests are useful regression assets; rename/reframe them around owner transactions rather than delete them.
- Old SQLite migrations must remain to open existing databases. Old schema code is compatibility code, not dead code.
- Exact package identity fixtures and scanner fixtures are useful.
- Broker state transition, hash/tamper, stale-prestate, policy, lock, verification, rollback, conflict and crash-recovery tests protect real safety behavior.

### Accidental complexity and naming

- `handlers/taskbar_search.rs` actually implements Task View; `handlers/search_highlights.rs` actually implements Show Desktop. This creates false scanner/action associations.
- Desired-state preview planning and mutation planning are parallel concepts. There is no duplicate Windows executor, but the user sees two “plan” systems, one permanently non-executable and one experimental.
- Mutation tables duplicate important normalized fields inside full JSON payloads. Do not optimize this before 0.1, but establish one canonical serialization contract later.
- Frontend tests that require alpha gate terminology, warning acknowledgment and exact phrase behavior should be replaced with tests for one-click authorization and technical preflight. Tests for disabled Apply under real policy/unknown state should remain.
- Documentation is extensive but describes multiple historical product identities. Keep decision history, then write a short current owner-mode architecture/security/product guide so new work does not follow obsolete prohibitions accidentally.

## 14. Main-Rig Readiness

**Answer: YES, WITH SPECIFIC LIMITATIONS.**

I would run the current **normal release installer** on a Windows PC containing important personal data as a read-only inspector, subject to these specific limitations:

- The binary is structurally incapable of registry/AppX mutation because mutation code and commands are absent. This is stronger than relying on a disabled button.
- The scan executes eight fixed local PowerShell queries, including all-user/provisioned AppX inventory and fixed registry/OneDrive metadata reads. I would expect antivirus/PowerShell policy or permissions to affect completeness, but not to change settings.
- Deslopper stores local inspection/history data under `%LOCALAPPDATA%\Deslopper`; the data is redacted for device name/SID but still contains useful system inventory and should be treated as private.
- The installer is not configured for code signing and may download/bootstrap WebView2. I would verify the artifact came from this build and expect Windows reputation prompts.
- I would not treat detector output as authoritative removal/tuning advice without reviewing unknown/partial applicability, especially across newer servicing builds.
- I would back up the local database before relying on history across upgrades because the product is still alpha, although current migrations are defensive.

I would **not** run the current feature-on internal mutation build against the main PC:

- There is no checked-in physical live-mutation evidence.
- A verification failure after a successful write can advertise Undo while lacking the `post_state` required to perform it.
- Failed verification does not trigger automatic rollback.
- Registry success does not prove immediate visible taskbar behavior, and no shell refresh behavior has been physically documented.
- Task View and Show Desktop lack matching scanner verification.
- Build/edition support is broad and not backed by exact 24H2/25H2 physical evidence.
- Recovery, approval discovery from installed current working directory, and process-relaunch approval behavior are brittle.

Thus the answer is “yes” only for the exact shipped read-only composition. As the requested Apply/Verify/Undo product, the current release is not ready because it cannot Apply at all.

## 15. Proposed Owner-Mode Architecture

Owner mode should make mutation an ordinary release capability while preserving the narrow executor and transaction protections.

### User contract

One deliberate Apply click on an actionable finding is the authorization. Before clicking, the card states the exact user-visible effect, scope (`this Windows account`), relevant restart/refresh expectation, and Undo availability. No typed phrase, CLI launch ritual, machine approval, scenario, checkpoint, target lifecycle, or debug build is involved.

### Internal execution contract

`Apply(operation, target, source_observation)` should perform one backend transaction:

1. Resolve a closed `MutationOperationId`; reject anything not registered.
2. Obtain process/cross-process lock.
3. Load a fresh matching observation or run the targeted detector/full scan automatically.
4. Normalize build/edition/applicability and check policy/authority.
5. Read the exact low-level representation immediately before write.
6. If already compliant, record `NoChangeNeeded` without writing.
7. Create and durably save a transaction intent with a generated internal idempotency ID, source observation, exact pre-state and handler version.
8. Recheck authority immediately before write.
9. Write through the fixed handler.
10. Capture the actual state even when the write reports an error.
11. Directly verify the exact representation and run the matching detector.
12. On success, commit a rollback-capable transaction and return a simple result.
13. On verification failure, automatically restore exact pre-state only if current state can safely be attributed to this attempt; verify the restore and record both failure and rollback outcome. Otherwise mark `NeedsAttention` with actual current/pre-state evidence.
14. Refresh the product snapshot/history and return the result to the card.

The existing `Broker`, handler traits, transaction status model and `MutationJournal` can be refactored into this service. Do not create a second owner executor beside the alpha broker.

### Undo contract

Undo takes a durable transaction ID and one deliberate click:

1. Load and verify transaction/capture integrity.
2. Confirm same stable local machine/account scope and compatible handler version.
3. Check policy/authority and read actual current state.
4. If current equals applied state, restore exact pre-state and verify.
5. If current differs, show a conflict with original/applied/current values. A second explicit “Restore original anyway” action may set `allow_conflict`; this is meaningful safety, not routine ceremony.
6. Persist and refresh the matching detector.

### Capability/elevation boundary

The initial owner build should expose only the three current HKCU operations at the Rust command layer, and only Widgets in finding UI until matching detectors exist for the others. No elevation is needed. Future HKLM/AppX operations should use separate operation definitions, preflight, backups where meaningful, and a narrowly scoped elevated helper or Tauri capability only when the operation actually requires it. Do not make the whole app run as administrator.

### Product/data model

- Keep one durable change history combining operation, previous value, result, verification and Undo.
- Internally generate/consume idempotency IDs; do not present nonce/plan ceremony.
- Keep support details exportable and redacted.
- Replace experimental `AlphaGateStatus` with per-operation `Actionability` (`ready`, `already_set`, `needs_scan`, `managed`, `unsupported`, `unknown`, `busy`) and concrete reasons.
- Bind an action to the exact detector representation. A similarly named component is not sufficient.

## 16. Shortest Path to Deslopper 0.1

The shortest credible path is one productized vertical slice, not broad scanner actionability:

1. **Authorize the direction in repository governance.** Current `AGENTS.md`, `.deslopper/policy.toml`, security docs and decision log explicitly prohibit normal mutation. Record owner mode and its retained safety invariants before changing code so implementation is not simultaneously violating project policy.
2. **Extract experimental authorization from transaction safety.** Refactor `Broker` so execution depends on fixed operation/actionability/pre-state/lock/journal/verification—not live target approval objects. Retain the old validation harness outside the normal product path if desired.
3. **Make the ordinary release mutation-capable.** Compile/register the closed mutation service in normal release and remove debug/CLI prerequisites. Keep packaging assertions that only registered operations can write.
4. **Ship Widgets as the first and only actionable detector.** It already has a correct scanner/writer representation match. Fix verification-failure recovery, automatic rollback, stable identity, relaunch recovery, and result mapping before enabling its Apply button.
5. **Move Apply/Undo into normal findings/history UI.** Retire the alpha control-room flow and terminology. Preserve technical details behind disclosure.
6. **Perform a bounded main-rig smoke test.** First inspect and manually record/export exact `TaskbarDa` pre-state, then run enable/disable/no-op/Undo/relaunch/conflict/failure scenarios on the exact installer. Do not test Task View/Show Desktop until Widgets passes.
7. **Add Task View and Show Desktop honestly.** Create matching detectors/catalogue entries for `ShowTaskViewButton` and `TaskbarSd`, validate shell behavior on supported builds, rename handlers, then expose one at a time.
8. **Expand from the remaining 19 findings only when each has a documented fixed implementation, exact backup/rollback semantics, authority rules, and physical validation.** Package removals and HKLM policies are materially higher-risk phases, not part of the shortest 0.1 path.

This gets to an ordinary installer with Scan → finding → Apply → verified result → Undo without replacing Tauri, Svelte, SQLite, the scanner, handlers, or transaction journal.

## 17. Exact Implementation Phases

### Phase 0 — Record the owner-mode decision

**Likely files:** `AGENTS.md`, `.deslopper/policy.toml`, `ARCHITECTURE.md`, `SECURITY.md`, `docs/decision-log.md`, `docs/product-principles.md`, `docs/windows-safety-model.md`, `README.md`, `ROADMAP.md`, `NEXT_AGENT.md`.

**Behavior:** no runtime change. Replace “normal product can never mutate” with explicit owner-mode invariants: closed operations, exact pre-state, durable transaction, verification, rollback, no arbitrary command execution, per-operation elevation.

**Tests:** keep the existing suites unchanged. Update policy/document consistency tests or boundary scripts only to express the new composition; do not weaken the closed-write boundary.

**Risk:** Low technically, high governance importance.

**Manual validation:** review the decision against this handoff; confirm Widgets-only initial scope and whether internal validation tools are archived or retained.

### Phase 1 — Separate owner authorization from alpha governance

**Likely files:** `src/mutation/broker.rs`, `src/mutation/request.rs`, `src/mutation/plan.rs`, `src/mutation/governance.rs`, `src/mutation/live_validation.rs`, `src/mutation/mod.rs`, `src/app.rs`, `src/mutation/journal.rs`, `src/mutation/transaction.rs`.

**Behavior:** introduce a normal owner execution entry point where one Apply request creates/consumes its internal transaction intent. Remove `AlphaGateStatus`, warning acknowledgment, scoped approval, target scenario, approval class/quota, phrase and externally supplied nonce from the product path. Keep technical actionability, lock, stale-state, journal, verification and rollback checks. The old live-validation harness may call the same service using a mock/fault backend.

**Tests to retain:** closed request shape, fixed operation registry, stale source/pre-state, external policy, unsupported build/edition, one-active-write lock, hash/tamper, no-op, pre-state durability, fault injection, rollback exactness/conflict, interrupted recovery.

**Tests to modify/remove:** debug/CLI/live-scenario/physical target/approval expiry/approval phrase/execution quota as runtime prerequisites; frontend `mutation-alpha` tests that encode those ceremonies. Keep validation-tool tests only if that harness remains explicitly separate.

**Risk:** Medium-high because authorization and safety are intertwined in `Broker`.

**Manual validation:** use an in-memory/fake `MutationBackend`; prove every failure before write performs zero writes, repeated request is idempotent, and no arbitrary operation can be deserialized.

### Phase 2 — Make normal release composition mutation-capable

**Likely files:** `Cargo.toml`, `src/main.rs`, both `src/app.rs::run()` blocks, `get_product_info()`, `tauri.conf.json` if feature arguments remain, `frontend/src/lib/backend.ts`, `frontend/src/routes/+page.svelte`, `tools/scan-mutation-boundary.ps1`, build docs.

**Behavior:** compile the closed mutation service and `winreg` into the ordinary release; register owner Apply/Undo/history/actionability commands; remove release/debug/CLI opt-in distinctions. Prefer one production composition. If a read-only build remains, make it an explicit testing/support flavor rather than the default product.

**Tests to retain:** normal and release build, Tauri command allow-list/CSP tests, mutation boundary tests adjusted to assert the only write path is `WindowsSettingStore`, frontend backend contract, all-feature/normal tests until feature removal is complete.

**Tests to modify:** `isMutationAlphaBuild`, product-info build-mode assertions, command registration snapshots, scripts that currently require mutation symbols to be absent from release.

**Risk:** Medium. A packaging error now affects real machines, although operations remain closed.

**Manual validation:** inspect release symbols/registered commands, install NSIS on a disposable Windows user profile, confirm no mutation happens at launch/scan, and confirm Apply is disabled until a fresh actionable finding exists.

### Phase 3 — Complete the Widgets Apply/Verify/Undo slice

**Likely files:** `src/mutation/broker.rs`, `src/mutation/handlers/taskbar_widgets.rs`, `src/mutation/handlers/windows_store.rs`, `src/mutation/verification.rs`, `src/mutation/rollback.rs`, `src/mutation/transaction.rs`, `src/mutation/journal.rs`, `src/persistence.rs`, `src/app.rs`, `frontend/src/lib/backend.ts`, `frontend/src/routes/+page.svelte` or a new small owner-action component.

**Behavior:** connect `ComponentId::TaskbarWidgets` directly to `WidgetsVisibility`; capture actual state on every post-write path; automatically rollback safe verification failures; make recovery and transaction identity stable across relaunch; persist a clear result; add one-click Undo and a separate conflict resolution path. Decide/document shell-refresh behavior without killing Explorer.

**Tests to retain/add:** all Widgets handler policy/value tests; missing value and 0/1 cases; write failure before/after ambiguous state; verification failure with successful automatic rollback; rollback failure; crash at every fault point; relaunch recovery; no-op; conflict detection/override; matching detector result; database migration/open of existing schema 5; UI ready/applying/changed/failed/restored states.

**Tests to modify:** `canUndo()` must not promise rollback without sufficient actual-state evidence. Replace phrase/nonce workflow tests with one-click transaction tests.

**Risk:** High. This is the first product-authorized real write.

**Manual validation:** on a disposable profile first, export/read `TaskbarDa` type/value or absence; run enable, disable, already-compliant, Undo-to-DWORD, Undo-to-absence, app relaunch before Undo, policy-managed refusal, external-change conflict, and simulated verification failure. Confirm registry and visible taskbar state after each step, after app restart, and after sign-out/restart if immediate refresh is absent.

### Phase 4 — Productize the UI

**Likely files:** `frontend/src/routes/+page.svelte`, `frontend/src/lib/MutationAlphaPanel.svelte`, `frontend/src/lib/mutation-alpha.ts`, `frontend/src/lib/backend.ts`, `frontend/src/lib/product-ui.ts`, `frontend/src/lib/theme.css`, associated frontend tests.

**Behavior:** remove Experimental Apply & Undo as a primary section; place a Widgets Apply control on the finding/detail; replace alpha/gate/target/approval/nonce/evidence terminology with actionability and concrete results; integrate Undo into finding/change history; keep expandable support details. Decide whether first launch should start a scan automatically or keep one explicit Scan button.

**Tests to retain:** component filtering, product safety copy for unsupported/managed/unknown states, inspection progress/cancel, backend error mapping, history/retention/diagnostics.

**Tests to modify:** alpha panel, exact phrase, warning checkbox and build-mode navigation tests. Add accessible button/state/error tests and ensure Apply cannot double-submit.

**Risk:** Medium; errors here can misrepresent backend state even if the writer is safe.

**Manual validation:** keyboard-only and screen-size pass; scan-to-Apply journey; double-click/rapid navigation; backend timeout/error; restart during Apply; transaction result and Undo after relaunch; confirm internal identifiers are absent from primary UI.

### Phase 5 — Main-rig release smoke test

**Likely files:** test protocol/evidence documentation only unless defects are found; release artifact from the productized branch.

**Behavior:** validate the exact signed-or-hashed NSIS artifact on the developer's current Windows build. No operation beyond Widgets.

**Tests:** rerun all automated gates once for the release candidate, including release packaging and mutation-boundary/closed-operation checks. Record artifact hash, app version, Windows edition/build/UBR, handler version and scenarios.

**Risk:** High because this is the first important-data machine, though the scoped DWORD is low impact and exact rollback is expected.

**Manual validation:** create a restore/reference point appropriate to the operation (at minimum export the exact Explorer Advanced key/value and record absence/type); close unrelated installers; Scan; Apply opposite state; verify direct registry, detector, visible shell and durable transaction; Undo; verify exact original representation; relaunch and repeat no-op; induce a benign external-change conflict; verify uninstall leaves no changed setting. Stop on any mismatch—do not test the next operation.

### Phase 6 — Add exact Task View and Show Desktop product slices

**Likely files:** `src/platform.rs`, inspection catalogue in `src/inspection.rs`/`src/app.rs`, `src/applicability.rs`, `src/mutation/handlers/taskbar_search.rs`, `src/mutation/handlers/search_highlights.rs`, handler module names, frontend component metadata/tests, detector matrix/docs.

**Behavior:** rename handlers to Task View/Show Desktop, add actual detector components reading `ShowTaskViewButton` and `TaskbarSd`, define missing-value semantics and policy attribution, then expose each only after its independent physical validation. Do not map either to `taskbar_search` or `search_highlights`.

**Tests:** exact detector/handler representation parity, policy cases, default/missing semantics, build matrix, full Apply/Verify/Undo/failure/relaunch suite per operation.

**Risk:** Medium-high per operation.

**Manual validation:** repeat the Phase 5 protocol separately for each operation, including immediate shell visual behavior and exact restoration.

### Phase 7 — Expand actionability deliberately

**Likely files:** operation-specific; likely `src/platform.rs`, `inspection.rs`, `applicability.rs`, new handlers, broker operation registry, UI catalogue, persistence only if backup types expand, security docs/tests.

**Behavior:** choose one of the remaining inspection-only components at a time. Specify exact Windows representation, authority, privilege, backup, verification, rollback, servicing/build support and failure semantics before adding Apply. Package removal, provisioning and HKLM changes should not reuse the low-risk HKCU assumptions.

**Tests:** operation contract plus fixture and physical validation; keep closed execution and no-arbitrary-command guarantees.

**Risk:** operation-dependent; package/provisioning/policy changes are high.

**Manual validation:** operation-specific disposable VM/profile first, then owner machine only after evidence and recovery procedure are complete.

## 18. Open Questions / Unknowns

These could not be answered without product decisions or live mutation, which this reconnaissance intentionally did not perform:

1. What exact Windows edition, build, UBR, architecture and management state is the developer's main PC at first owner-mode release?
2. Do `TaskbarDa`, `ShowTaskViewButton`, and `TaskbarSd` update the visible shell immediately on that exact 24H2/25H2 build, or only after Explorer refresh/sign-out? No physical evidence is checked in.
3. Should first launch automatically scan, or is one explicit Scan click preferred for predictable PowerShell activity?
4. Is owner mode permanently single-user/current-user, or must transaction identity include Windows user SID so another account cannot see/undo the first account's HKCU changes?
5. What stable machine/account identifier should replace `DefaultHasher`, and what migration behavior should existing transaction rows use?
6. Should a missing Explorer Advanced key be created, or should that remain a safe refusal? The current handlers require the key to exist.
7. Should a genuine rollback conflict offer “restore original anyway,” or only show manual guidance for 0.1?
8. What code-signing certificate/distribution channel will be used? The repository config does not sign the NSIS artifact.
9. Should uninstall retain the SQLite audit/history by design, and should it ever offer cleanup without losing rollback evidence?
10. Should Task View and Show Desktop become new catalogue components, or should the first release contain only Widgets until a broader information architecture is chosen?
11. Is the old live-validation harness worth preserving as an optional engineering tool after owner mode, or should only its fault/fixture tests survive?
12. What is the intended support policy for ARM64, Windows 10, Home/Pro/Enterprise, managed devices and future 25H2 servicing? Current mutation definitions are x64-tested only in this workspace and broadly match editions by substring.
13. What counts as verification in product copy: registry persistence, detector agreement, visible shell change, or all three? Current code proves the first and, only for Widgets, meaningfully supports the second.
14. For future AppX/HKLM operations, what backup/elevation architecture is acceptable? The initial HKCU slice does not answer this and should not preemptively add administrator execution.
15. The local physical-target documents contain a historical retired/reactivated sequence but no valid execution evidence. If any uncommitted evidence exists outside the repository, it must be reviewed separately rather than assumed.

## 19. Important Files Map

| Path | Importance |
|---|---|
| `AGENTS.md` | Current repository-wide prohibition/governance rules; must be revised before implementation. |
| `.deslopper/policy.toml` | Current committed mutation policy (`allow_windows_mutation = false`) and validation constraints. |
| `Cargo.toml` | `default = []`, `mutation-alpha`, optional `sha2`/`winreg`. |
| `tauri.conf.json` | Tauri build, CSP, command pruning, NSIS and WebView2 bootstrapper configuration. |
| `capabilities/main-window.json` | Narrow frontend capability grant. |
| `src/main.rs` | Compile-time inclusion boundary. |
| `src/app.rs` | Tauri commands, scan orchestration, desired state/drift/history, product info, feature-selected app composition, and mutation command bridge. |
| `src/inspection.rs` | Eight PowerShell queries, time/output bounds, shared query execution and 20 detectors. |
| `src/platform.rs` | Component, state, applicability, authority, evidence and lifecycle domain types/catalogue definitions. |
| `src/applicability.rs` | Versioned build/edition rules and 24H2 UBR exception. |
| `src/package_identity.rs` | Exact AppX identity rules; no fuzzy matching. |
| `src/persistence.rs` | SQLite path, schema v5, migrations, read-only persistence, retention and machine identity. |
| `src/privacy.rs` | Persistence/diagnostic redaction. |
| `src/model.rs`, `src/presentation.rs` | Legacy three-item mock/preview architecture, currently unused by the main page. |
| `src/mutation/request.rs` | Closed operation IDs and plan/apply/rollback request shapes. |
| `src/mutation/broker.rs` | Core planning, gate checks, execution, verification completion, rollback, recovery and operation definitions. |
| `src/mutation/plan.rs` | Plan/captured representation, hashes and expiry. |
| `src/mutation/transaction.rs` | Durable transaction states, steps and rollback record. |
| `src/mutation/journal.rs` | SQLite mutation records and cross-process lock path. |
| `src/mutation/verification.rs` | Apply result classification. |
| `src/mutation/rollback.rs` | Applied-state conflict comparison. |
| `src/mutation/handlers/windows_store.rs` | The only real write backend; fixed HKCU DWORDs and fixed policy reads. |
| `src/mutation/handlers/taskbar_widgets.rs` | Correctly named Widgets operation handler. |
| `src/mutation/handlers/taskbar_search.rs` | Misnamed Task View handler. |
| `src/mutation/handlers/search_highlights.rs` | Misnamed Show Desktop handler. |
| `src/mutation/governance.rs` | Committed policy/target governance. |
| `src/mutation/live_validation.rs` | Debug/CLI/scenario/approval/deny-list/identity/live evidence gates. |
| `frontend/src/routes/+page.svelte` | Current normal product shell, navigation, scan/findings/desired/drift/history/settings. |
| `frontend/src/lib/backend.ts` | Typed Tauri IPC contract and error descriptions. |
| `frontend/src/lib/MutationAlphaPanel.svelte` | Experimental gate/plan/phrase/Apply/Undo/history/evidence UI. |
| `frontend/src/lib/mutation-alpha.ts` | Mutation workflow state, issued-plan validation, phase/error mapping and `canUndo`. |
| `frontend/src/lib/product-ui.ts` | Normal product filtering/status helpers. |
| `validation/report.md` | Current evidence summary: synthetic fixtures, no live mutation evidence. |
| `validation/matrix.json` | Validation target/scenario lifecycle inventory. |
| `validation/mutation-fixtures/mutation-alpha-replay.json` | Synthetic transaction replay fixture worth retaining. |
| `tools/scan-mutation-boundary.ps1` | Compile/boundary safety assertion; should evolve with product composition. |
| `ARCHITECTURE.md`, `SECURITY.md` | Current read-only/alpha architecture and security model. |
| `docs/decision-log.md` | Historical D-008 through D-014 decisions explaining the safety/governance expansion. |
| `docs/detector-matrix.md`, `docs/inspection-pipeline.md` | Scanner coverage and query design. |
| `docs/mutation-broker-alpha.md`, `docs/mutation-threat-model.md` | Broker/state/gate design and threats. |
| `README.md`, `ROADMAP.md`, `NEXT_AGENT.md` | Current product/release narrative; will need owner-mode alignment. |

HANDOFF COMPLETE
SOURCE CHANGES: NONE
LIVE MUTATIONS PERFORMED: NONE
