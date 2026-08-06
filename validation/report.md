# Deslopper VM fixture validation report

Generated: 2026-08-05T22:16:56Z

- Result: Passed
- Checked-in fixture bundles: 1
- Synthetic mutation fixture bundles: 1
- Live mutation evidence bundles: 0
- Defined matrix targets: 13
- Physical preparation targets (not mutation evidence): 1
- Historical retired-before-mutation events: 1
- Current approval-review-ready preparation targets: 1
- Scoped approval-model correction events: 1
- Explicit physical-target governance events: 1
- Governed Widgets review integration events: 1
- Physical-target live mutation scenarios: 0
- Runner: Rust production parsers and detectors
- Mutation boundary: Fixture replay only; no Windows mutation or query execution
- Live mutation VM scenarios completed: 0
- Not provisioned: 13
- Provisioned but not run: 0
- Running: 0
- Passed: 0
- Passed with limitations: 0
- Failed: 0
- Blocked: 0
- Handler removed: 0
- Host protection audit: Passed
- Development-host denylist present: True
- Current host fingerprint denied: False
- Current host is protected prepared target: True
- Deslopper processes running: 0
- Live scenario manifest present: False
- Live scenario targets development host: False
- Approved generic target present: False
- Legacy approved VM inventory present: False
- Approved target entries: 0
- Windows hypervisor layer present: True
- Windows Sandbox executable present: False
- Host virtualisation tooling detected: None detected
- Live validation blocker: No ignored approved validation target exists; no target is authorized.
- Manual provisioning handoff: `docs/manual-vm-provisioning.md`
- Live infrastructure exercised: None
- Live operation/state combinations exercised: None
- Settings UI, visual behavior, refresh, lifecycle, rollback, drift, policy, recovery, and standard-user findings: Not tested

Handler maturity:

| Operation | Supported targets | Maturity | Live disposition |
|---|---|---|---|
| Widgets visibility (`TaskbarDa`) | enabled, disabled | synthetic tested | Internal alpha - not live validated |
| Task View visibility (`ShowTaskViewButton`) | enabled, disabled | synthetic tested | Internal alpha - not live validated |
| Show Desktop corner (`TaskbarSd`) | enabled, disabled | synthetic tested | Internal alpha - not live validated |

Fixtures:

- win11-pro-24h2-clean-local.json

Synthetic mutation fixtures (not live VM evidence):

- mutation-alpha-replay.json

Live evidence bundles:

- None. No live mutation was performed.

Scenario status:

| Scenario | Status |
|---|---|
| win11-home-24h2-clean-local | not provisioned |
| win11-pro-24h2-clean-local | not provisioned |
| win11-enterprise-24h2 | not provisioned |
| win11-home-25h2-microsoft-account | not provisioned |
| win11-pro-25h2 | not provisioned |
| win11-enterprise-25h2 | not provisioned |
| domain-joined-pro | not provisioned |
| mdm-enrolled-enterprise | not provisioned |
| post-feature-update | not provisioned |
| post-store-updates | not provisioned |
| package-current-user-removed | not provisioned |
| package-provisioning-removed | not provisioned |
| permission-limited-standard-user | not provisioned |

Read-only preparation targets (excluded from completed mutation coverage):

| Target | Availability | Read-only | Preparation | Recovery | Approval | Mutation | Live scenarios | Completed credit | Failed credit | Approved credit | Live handler evidence |
|---|---|---|---|---|---|---|---:|---|---|---|---|
| physical-laptop-win11-pro-25h2-read-only | available for approval review | read only validated | ready for approval review | recovery ready | not approved | mutation not attempted | 0 | False | False | False | False |

The physical target was retired before mutation because the hardware is being sold, then explicitly reactivated for bounded Widgets preparation before its final reset. Retirement remains in chronological history. The first execution handoff stopped at the overbroad version 1 approval boundary; scoped version 2 introduced Widgets-enabled-only scope, and the explicit-target governance change now requires strict version 3 for physical approval. The integrated schema-v3 review-draft path remains local, ignored, non-authorizing, limited to one Widgets-enabled plan and one execution, and bound to reset-before-sale disposition. The source change grants no approval: Widgets remains unexecuted, no handler has live evidence, and zero mutation occurred. Read-only Windows 11 Pro 25H2 inspection, Tauri event permission, OneDrive detection, MDM evidence, and permission-limited AppX findings remain useful.
