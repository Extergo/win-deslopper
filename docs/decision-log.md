# Decision log

Each decision below is accepted for the current architecture. Replacements must preserve history and explain migration.

## D-001 — Rust core and Slint UI

- **Status:** Superseded by D-006
- **Context:** Deslopper needs a native, maintainable Windows application with a declarative UI.
- **Decision:** Use stable Rust for domain/orchestration and Slint for presentation.
- **Consequences:** Typed boundaries and small native deployment; toolkit-specific UI bridge is isolated.
- **Alternatives:** Web shells and other native toolkits were rejected for this milestone due to scope or architectural weight.

## D-002 — Material-inspired visual system

- **Status:** Accepted
- **Context:** The product needs an approachable identity distinct from dense administration tools.
- **Decision:** Use stock-Android-inspired Material principles with centralised tokens and no copied trademarks/assets.
- **Consequences:** UI review follows `docs/ui-design-system.md`; other visual languages require a new decision.
- **Alternatives:** Legacy control panels, generic desktop widgets, and mixed aesthetics were rejected.

## D-003 — Non-destructive UI-first milestone

- **Status:** Accepted
- **Context:** Safety architecture is not yet implemented.
- **Decision:** The first milestone implements mock domain data, local planning state, and UI only; no real Windows inspection or operation.
- **Consequences:** Every result is labelled preview; no inert destructive pathway may be pre-wired.
- **Alternatives:** Early PowerShell or registry integration was rejected as unsafe and premature.

## D-004 — Future trust boundaries

- **Status:** Accepted
- **Context:** Privileged operations, extensions, and online services will have different trust levels.
- **Decision:** Future elevation uses a narrow helper; plugins run out of process with capabilities; the backend supports accounts, commerce, entitlements, catalogue, and verified package delivery, never arbitrary remote control.
- **Consequences:** These process/protocol boundaries require later security design and explicit milestone approval.
- **Alternatives:** In-process privileged plugins and backend command channels were rejected.

## D-005 — Governance and central tokens

- **Status:** Superseded in part by D-006
- **Context:** Multiple contributors require durable, discoverable constraints.
- **Decision:** Mandatory repository governance documents own policies, and Slint theme tokens own shared visual values.
- **Consequences:** Relevant documentation changes with behaviour; token exceptions require design review.
- **Alternatives:** Convention-only governance and scattered styling were rejected as difficult to audit.

## D-006 — Tauri shell and SvelteKit presentation

- **Status:** Accepted
- **Date:** 2026-07-27
- **Context:** The maintainers requested that the UI preview move from Slint to Tauri and SvelteKit while preserving its non-destructive scope, Rust-owned domain rules, and Material-inspired visual language.
- **Decision:** Use Tauri 2 as the native Windows window and local IPC boundary, and SvelteKit 2 with static SPA output for presentation. Rust continues to own the catalogue, filters, plan, navigation, and review state. The webview receives serializable view models and sends a small typed action enum through two explicitly registered commands. Shared visual tokens move from Slint globals to a single CSS token file.
- **Consequences:** The application now includes the system WebView2-based Tauri runtime and a Bun-based frontend build toolchain. Static production assets are embedded locally; no server, remote origin, Tauri plugin, shell API, updater, filesystem API, or network client is introduced. Rust and frontend quality gates are both required. Dependency review is recorded in `docs/dependency-review-tauri-sveltekit.md`.
- **Security boundary:** Only the main local window receives the core Tauri capability. Custom IPC can read or change in-memory preview state only. It cannot inspect or modify Windows, execute commands, persist data, elevate, authenticate, make payments, load plugins, collect telemetry, or contact a backend.
- **Alternatives:** Keeping Slint did not satisfy the requested migration. A hosted or server-rendered frontend would introduce an unnecessary runtime network boundary. Exposing broad Tauri plugins or moving domain state into TypeScript would weaken the current safety and architecture boundaries.

## D-007 — Read-only inspection beta boundary

- **Status:** Accepted
- **Date:** 2026-08-01
- **Context:** The accepted inspection alpha needed lifecycle control, honest partiality, historical comparison, structured applicability, and repeatable Windows validation before privileged design could begin.
- **Decision:** Keep Windows discovery in fixed Rust-owned query definitions; execute shared queries once with timeout, output, parser, and cancellation contracts; persist normalized lifecycle, component, package, desired-revision, and drift records in versioned SQLite; expose typed Tauri APIs and progress events; keep VM capture developer-only.
- **Consequences:** The main process performs real read-only PowerShell inspection without elevation. Failed, cancelled, timed-out, permission-limited, unsupported, and absent states remain distinct. Svelte does not duplicate Windows rules. SQLite corruption is preserved and surfaced rather than recreated.
- **Security boundary:** There is no operation executor, Windows writer, arbitrary shell endpoint, elevation manifest, privileged helper, or hidden mutation command. Preview plans explicitly report `executorEnabled: false`.
- **Alternatives:** Polling the entire dashboard, Boolean absence, fuzzy package matching, frontend applicability tables, and unbounded child processes were rejected.

## D-008 — Feature-gated mutation broker alpha

- **Status:** Accepted for internal alpha only
- **Date:** 2026-08-02
- **Context:** Prove the safety lifecycle with a minimal reversible operation set without weakening the normal read-only release.
- **Decision:** Compile mutation only behind `mutation-alpha`; also require debug build, CLI opt-in, warning acknowledgement, fresh inspection, an expiring SHA-256-bound one-use plan, and explicit approval. Implement only fixed current-user Widgets, Task View, and Show Desktop handlers.
- **Substitution:** Search mode lacks a documented stable current-user setter, while Search Highlights is documented device policy. Task View and Show Desktop are documented unelevated substitutes.
- **Consequences:** SQLite v3 journals plans, transactions, steps, captures, rollback, and recovery. Optional `sha2` and `winreg` dependencies are reviewed. Real VM evidence remains mandatory.
- **Rejected:** Generic registry APIs, PowerShell actions, elevation, machine policy writes, VM detection as a security boundary, Apply All, automatic repair, and silent crash retry.

## D-009 - Live validation uses the normal broker with fail-closed guest identity

- **Status:** Accepted for internal validation
- **Date:** 2026-08-02
- **Context:** Live handler proof must never turn the development host or a merely detected VM into an eligible target.
- **Decision:** Keep the feature-gated application as the only guest runner. Add a separate live flag, fixed ignored manifest, exact scenario/machine/checkpoint, edition/build/UBR, guest-owned database, local development-host denylist, and visible environment banner. Keep VM detection informational. Host Hyper-V tooling reads an ignored explicit VM inventory, requires confirmation for state changes, and cannot invoke mutation. Evidence combines durable broker facts with closed manual visual states and label-only screenshots.
- **Consequences:** All three handlers remain `SyntheticTested` and visibly not live validated. No unavailable target counts as passed. Normal builds remain unchanged and read-only.
- **Rejected:** Direct handler test paths, VM detection as authorization, automatic VM creation/download, automatic acknowledgement, host mutation, arbitrary evidence paths, and treating registry reread as visual proof.

## D-010 - Validation targets separate preparation from approval

- **Status:** Accepted for validation readiness
- **Date:** 2026-08-03
- **Context:** The first physical-laptop read-only pass could not honestly use a disposable-VM approval schema, and a physical target needs stronger recovery evidence and explicit user confirmations.
- **Decision:** Add a generic local validation-target schema with `virtual_machine` and `physical_laptop` types and explicit `preparation`/`approval` record kinds. Keep the legacy VM inventory for Hyper-V compatibility. Require the live gate to load a separate ignored approved target and reject pending, incomplete, expired, recovery-unready, wrong-type/build/identity, empty-denylist, and development-host-equal states.
- **Consequences:** A physical preparation may record read-only inspection and recovery-audit facts without becoming an approval. Physical approval requires important-data, backup, reinstall, WinRE, BitLocker recovery, media, expendability, restore/reimage, timestamp, expiration, and development-host protection fields. The original development-host identity remains local-only and mandatory.
- **Rejected:** Claiming the laptop is a VM, treating a draft as approval, inventing or committing either machine identity, accepting an empty denylist, auto-approving from inspection/audit results, or counting read-only preparation as live mutation evidence.

## D-011 - Ship the useful product as a read-only alpha

- **Status:** Accepted
- **Date:** 2026-08-05
- **Context:** The complete inspection engine needed a coherent, useful product surface without weakening the mutation boundary or presenting internal research as a user feature.
- **Decision:** Expose the 20-component catalogue, honest dashboard, desired-state comparisons, non-executable previews, reviewed/resolved drift, history comparison and retention, and user-reviewed redacted diagnostics. Keep the normal binary feature-off, unelevated, local, and incapable of Windows mutation.
- **Consequences:** Internal mutation research stays behind its compile and live-validation gates and is absent from the normal product workflow. The former physical laptop is withdrawn because it was sold before approval or mutation; unavailability is not a failed operation and cannot count as mutation evidence.
- **Rejected:** Health scores that hide uncertainty, an Apply control with no released executor, generic diagnostics dumps, silent uploads, and treating an unavailable target as live validation.

## D-012 - Preserve retirement while allowing bounded reactivation preparation

- **Status:** Accepted for preparation only
- **Date:** 2026-08-05
- **Context:** The physical target was retired before mutation when sale and reset were planned. The user later allowed temporary preparation for one Widgets visibility validation before the final reset.
- **Decision:** Preserve the retirement event and add a chronological reactivation event limited to Widgets preparation. Keep approval `not_approved`, mutation `mutation_not_attempted`, and live scenarios at zero. Reactivation alone never authorizes a plan, transaction, or write.
- **Consequences:** Recovery, identity separation, a fresh read-only inspection, exact pre-state capture, and a new local approval-review draft must all pass before a separate approval decision. Task View and Show Desktop remain out of scope.
- **Rejected:** Rewriting retirement history, reusing an old approval, treating reactivation as approval, or granting any live-handler maturity before an independently approved apply-and-rollback session.

## D-013 - Scope physical live approval to operation and target direction

- **Status:** Accepted for internal validation
- **Date:** 2026-08-05
- **Context:** The first bounded execution handoff correctly stopped because version 1 required all three operations and both directions, while the user authorized preparation only for Widgets-to-enabled.
- **Decision:** Add explicit schema version 2 operation scopes with per-operation target states, handler/source/inspection/evidence/identity/denylist bindings, expiration, and bounded plan/execution counts. Require version 2 for physical live validation, filter broker options from validated scope, and independently revalidate the exact pair during plan generation, execution, and immediately before a write.
- **Consequences:** One Widgets-enabled plan and execution can be represented without authorizing Widgets-disabled, Task View, or Show Desktop. Exact rollback derives from the consumed durable transaction and captured pre-state, so it does not grant reusable reverse-direction authority. Version 1 remains a deliberate legacy VM path. The current physical test remains unexecuted, unapproved, and at zero live scenarios.
- **Rejected:** Treating compiled handlers as approved, global target-state inheritance, default-all scope, UI filtering as the security boundary, unlimited retries, requiring reverse-direction approval for transaction rollback, and silently promoting version 1 physical approval.

## D-014 - Explicitly allowlisted disposable validation targets

- **Status:** Accepted for internal validation governance
- **Date:** 2026-08-05
- **Context:** VM-only governance was the safest initial boundary while no physical recovery model existed. The later read-only laptop work established that an expendable physical target can be bounded more strictly than a VM, but the committed TOML still prohibited every physical execution and was not parsed by runtime.
- **Decision:** Replace the VM-only Boolean with live-validation policy schema version 2, explicit target allowlisting for `virtual_machine` and `physical_laptop`, and mandatory strict physical recovery enforcement. Parse the committed policy in internal mutation builds. Keep schema versions 1 and 2 as deliberate VM compatibility; require scoped approval version 3 for physical targets. Version 3 adds explicit important-data state, recovery route and media, alternate recovery device, clear restart state, safe BitLocker state, unmanaged domain/Entra/workplace/MDM state, disabled automatic repair, final-plan approval, no-more-than-30-minute authority, recovery/reimage procedure, and final disposition.
- **Consequences:** Policy, target approval, denylist, exact source/evidence/platform binding, scope, and limits are necessary together and are revalidated before a write. The original development PC remains permanently denied without an override. The withdrawn laptop's retirement and later reactivation remain separate audit events; Widgets remains ready for approval review, not approved or validated, and zero live mutations have occurred. This governance commit grants no operation or target approval.
- **Rejected:** Setting the old VM-only Boolean to false, default-all target types, reinterpreting old physical manifests, prompt or CLI overrides, hard-coded machine identity/build, managed physical targets, long-lived approval, automatic repair, and treating governance permission as approval.

## D-015 - Owner Mode makes bounded Widgets mutation a normal product capability

- **Status:** Accepted
- **Date:** 2026-08-11
- **Context:** Mutation Alpha established a narrow executor, durable transaction journal, direct verification, exact rollback, concurrency control, and recovery states. Its experiment authorization stack made the ordinary installer structurally incapable of completing the product's Scan -> Apply -> Verify -> Undo journey.
- **Decision:** The normal Windows 11 x64 product may execute registered owner-mode operations. One deliberate UI Apply click authorizes one operation; one deliberate Undo click authorizes conflict-free exact restoration. Safety lives in the closed operation registry, fixed hardcoded handler, build/applicability and policy checks, fresh preflight, exact durable pre-state, actual post-attempt capture, direct and matching-detector verification, automatic safe rollback, stable versioned machine-plus-user scope, transaction integrity, locks, and recovery. M1 productizes only `set_taskbar_widgets_visibility` (`TaskbarDa`). Arbitrary registry paths, commands, PowerShell mutation, elevation, HKLM writes, AppX/service changes, Task View, and Show Desktop remain forbidden.
- **Supersedes:** D-008, D-009, D-010, D-012, D-013, and D-014 only as normal-product authorization. Their history and tooling remain valid for the optional engineering validation harness. D-011's read-only normal-build boundary is superseded; its inspection and privacy decisions remain accepted.
- **Consequences:** The default production build includes the owner broker and Widgets commands without debug flags, CLI opt-ins, repository-local validation files, machine/target approvals, warning checkbox, typed phrase, approval quota/class, or user nonce. Legacy validation commands are available only in an explicit engineering feature. Old database rows remain readable but legacy unversioned machine scopes cannot authorize owner rollback.
- **Rejected:** A second executor, generic registry or command API, running the whole app elevated, silently overwriting rollback conflicts, claiming visible shell repaint from registry verification, productizing all compiled handlers, deleting historical validation research, and weakening transaction tests.

## D-016 - Physical evidence pivots M2 to Task View and scopes Widgets denial

- **Status:** Accepted
- **Date:** 2026-08-11
- **Context:** The first physical M1 test found `TaskbarDa` DWORD 0. Deslopper and an independent unelevated write were both denied, with the exact state unchanged. On the same machine/account, `ShowTaskViewButton` DWORD 0 accepted an unelevated same-state write. `TaskbarSd` was absent and untested.
- **Decision:** Classify the Widgets result as write rejected unchanged, expose no Undo, and persist a Widgets-only **Direct change unavailable** capability for that stable machine/account scope. Do not elevate, modify ACLs, or infer a generic writer defect. Productize Task View in M2 with a distinct detector, fixed operation, fresh preflight, durable exact pre-state, direct and detector verification, and conflict-safe Undo. Keep Taskbar Search distinct. Do not touch or productize Show Desktop/`TaskbarSd`.
- **Consequences:** The catalogue has 21 components. Normal Owner Mode registers Widgets and Task View only. Failure receipts distinguish rejected unchanged, ambiguous write, changed verification failure, and verification failure rolled back. Historical `owner-widgets.1` receipts remain readable; new owner intents use `owner-taskbar.2`.
- **Rejected:** Elevation, registry ownership/ACL changes, brute-force retry, global Widgets suppression, representing unchanged denial as partial mutation, fake Undo, reusing the misleading `taskbar_search.rs` module name, and treating absent `TaskbarSd` as validation evidence.

## D-017 - Task View passes the complete physical Owner Mode lifecycle

- **Status:** Accepted physical validation evidence
- **Date:** 2026-08-11
- **Context:** Deslopper `0.2.0` from commit `d4cd3390633be20f8ce13cd26f41af739340f298` was installed and run unelevated on a Windows 11 x64 owner machine. Fresh inspection observed `ShowTaskViewButton` DWORD 0 and offered Show Task View button.
- **Evidence:** One Apply reported `changed`, target enabled, exact pre-state DWORD 0, rollback available, and `direct_and_detector_verified`. Independent `reg.exe` read DWORD 1 and Explorer displayed Task View without an Explorer restart. After Deslopper fully exited, no process remained and DWORD 1 persisted. Relaunch preserved the durable Undo. One Undo reported exact restoration, and independent verification read DWORD 0.
- **Decision:** Mark Owner Mode M2 / Task View physical validation PASS for Detect -> Apply -> real Windows write -> direct verification -> detector verification -> durable transaction -> process exit -> relaunch -> Undo -> exact restoration -> verification. Preserve the earlier Widgets PermissionDenied/unchanged result as scoped evidence, not a generic writer failure.
- **Consequences:** Task View's released current-user Apply/Undo lifecycle now has physical evidence. This does not expand the M2 operation set or authorize elevation, ACL changes, Show Desktop/`TaskbarSd`, AppX, services, HKLM writes, or generic execution.

## D-018 - M3 ships four exact current-user cleanup settings

- **Date:** 2026-08-11
- **Status:** Accepted
- **Context:** M2 physically validated the established transaction architecture. Five existing suggestion/content detectors were audited for a coherent current-user cleanup pack.
- **Decision:** Productize welcome experience (`SubscribedContent-310093Enabled`), tips and suggestions (`SoftLandingEnabled`), notification suggestions (`SubscribedContent-338389Enabled`), and suggested content in Settings (`SubscribedContent-338393Enabled`) as fixed DWORD 0/1 operations under the current-user Content Delivery Manager key. Their matching Cloud Content `Disable…` policies remain authoritative and cause refusal. Missing and non-binary values remain unknown/unsupported. Skip lock-screen suggestions because `RotatingLockScreenOverlayEnabled` is paired with broader `ConfigureWindowsSpotlight` policy semantics and exact detector/handler parity cannot be proven. Reuse the M2 transaction path and generalize direct-change-unavailable per operation and owner scope.
- **Consequences:** Owner Mode `0.3.0` has exactly six product operations. Actionable component pages hide the legacy desired-state preview, show concise normal change history, and retain expandable technical receipts. No elevation, ACL change, HKLM write, arbitrary registry input, AppX, service, Explorer lifecycle, Taskbar Search, or `TaskbarSd` capability is added. M2 Task View evidence and the scoped Widgets access-denied result remain regression contracts.

## D-019 - M3 passes physical validation for two present cleanup DWORDs

- **Date:** 2026-08-12
- **Status:** Accepted physical validation evidence
- **Context:** The installed Deslopper `0.3.0` NSIS product from commit `2216c8eaac7f7efa961e04251f6b7eb670b8f83d` was run unelevated on the Windows 11 x64 owner machine. Tips and Notification suggestion values were exact DWORD 1; Welcome and Settings suggested-content values were absent. Independent non-elevated same-state writes succeeded only for the two present DWORDs. No absent value was created for testing.
- **Evidence:** Tips `SoftLandingEnabled` and Notification `SubscribedContent-338389Enabled` each changed from exact DWORD 1 to 0 through one Deslopper Apply. Independent registry reads verified DWORD 0. Deslopper then fully exited, relaunch preserved both durable transactions and Undo, and independent final reads verified exact restoration to DWORD 1. The two initially absent M3 values remained absent after both sequences.
- **Decision:** Mark Owner Mode M3 physical validation PASS for Tips and suggestions and Notification suggestions, including real write, verification, process-boundary durability, relaunch Undo, and exact restoration. Mark absence preservation PASS. Treat the unmutated absent Welcome and Settings representations as the intended unsupported-missing behavior, not failures.
- **Consequences:** Owner Mode now has successful physical Apply/Undo evidence across Task View and two independent M3 cleanup operations. Widgets remains a separate operation-scoped access-denied/unchanged result, not a generic store failure. This evidence does not claim present-DWORD live coverage for Welcome or Settings suggested content and grants no broader mutation authority.
