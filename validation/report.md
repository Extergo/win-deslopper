# Deslopper VM fixture validation report

Generated: 2026-08-05T15:26:25Z

- Result: Passed
- Checked-in fixture bundles: 1
- Synthetic mutation fixture bundles: 1
- Live mutation evidence bundles: 0
- Defined matrix targets: 13
- Physical retired/read-only targets (not mutation evidence): 1
- Retired before mutation: 1
- Target withdrawn: 1
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
- Current host fingerprint denied: True
- Deslopper processes running: 0
- Live scenario manifest present: False
- Live scenario targets development host: False
- Approved generic target present: False
- Legacy approved VM inventory present: False
- Approved target entries: 0
- Windows hypervisor layer present: True
- Windows Sandbox executable present: False
- Host virtualisation tooling detected: None detected
- Live validation blocker: The physical target was withdrawn before mutation; no target is approved or authorized.
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

Retired read-only physical targets (excluded from completed, failed, approved, and handler-evidence mutation coverage):

| Target | Target state | Retirement | Read-only | Approval | Mutation | Live scenarios | Completed credit | Failed credit | Approved credit | Live handler evidence |
|---|---|---|---|---|---|---:|---|---|---|---|
| physical-laptop-win11-pro-25h2-read-only | target withdrawn | retired before mutation | read only validated | not approved | mutation not attempted | 0 | False | False | False | False |

The target was withdrawn because the hardware is being sold. Its read-only
findings remain useful, but they do not demonstrate that any mutation handler
works live. Zero mutation occurred.
