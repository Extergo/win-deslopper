# Windows VM validation

## Mutation status vocabulary

Every matrix row independently records `not_provisioned`,
`provisioned_not_run`, `running`, `passed`, `passed_with_limitations`, `failed`,
`blocked`, or `handler_removed`. Synthetic fixtures never change a live status.
The current matrix has thirteen `not_provisioned` rows and no live evidence.

`preparationTargets` is a separate inventory for read-only physical-target
readiness. Its Windows 11 Pro 25H2 laptop row is
`withdrawn_sold`, `read_only_validated`, `preparation_incomplete`, `recovery_not_ready`,
`not_approved`, and `mutation_not_attempted`. It is not aggregated with the
thirteen mutation targets and cannot change a live status. The laptop was sold
before approval or mutation; unavailable does not mean failed.

`validation/matrix.json` is the target inventory. Each checked-in scenario must record OS edition/build/UBR, account type, domain/MDM evidence, OneDrive state, expected component applicability, package registration and provisioning, policy/preference precedence, authority/confidence, limitations, and manual verification steps.

## Capture workflow

1. Create or revert a legally licensed Windows test VM to a named matrix state.
2. Run the application non-elevated and save an inspection.
3. Run `tools\capture-vm-fixture.ps1 -ScenarioId <matrix-id> -OutputPath <new-json>` from a developer checkout in the VM.
4. Review the output. It must contain no email, token, filename, recovery material, device name, account identity, root path, or full install path.
5. Add expected assertions and manual checks based on direct Windows Settings, documented policy tools, and package inventory.
6. Run `tools\validate-vm-fixtures.ps1`. Commit the fixture and updated report only after it passes.

Capture uses only fixed read-only queries. It does not recursively inspect files, hydrate cloud placeholders, trigger sync, change attributes, elevate, or call production mutation code. It is developer-only and absent from Tauri commands.

## Manual minimum

Verify progress/cancel responsiveness, standard-user permission messaging, correct edition/build applicability, exact package rows, Group Policy versus domain/MDM wording, OneDrive composite state, restart/reopen history, component timeline, drift filters, and database recovery messaging. After Store or feature updates, capture before and after fixtures and confirm normal servicing or applicability drift is informational.

The current repository includes the framework and a synthetic/redacted 24H2 Pro fixture. Completion of the full matrix requires access to the listed Windows VMs and, for the authority cases, a lawful test domain and MDM tenant.

See `live-validation-environment.md`, `live-evidence-format.md`, and
`vm-mutation-protocol.md` for the hardened live runner and evidence rules.
