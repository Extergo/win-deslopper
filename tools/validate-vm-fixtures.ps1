$ErrorActionPreference = 'Stop'
$repository = Resolve-Path (Join-Path $PSScriptRoot '..')
Push-Location $repository
try {
    $started = Get-Date
    cargo test validation::tests::every_checked_in_vm_fixture_matches_current_detectors
    if ($LASTEXITCODE -ne 0) { throw 'VM fixture validation failed.' }
    cargo test --features mutation-alpha mutation::transaction::tests::mutation_fixtures_replay_state_machine_without_executing_windows_writes
    if ($LASTEXITCODE -ne 0) { throw 'Mutation fixture replay failed.' }
    cargo test --features mutation-alpha mutation::live_validation::tests::every_checked_in_live_evidence_bundle_is_unique_redacted_and_closed
    if ($LASTEXITCODE -ne 0) { throw 'Live mutation evidence validation failed.' }
    $fixtures = @(Get-ChildItem (Join-Path $repository 'validation\fixtures') -Filter '*.json')
    $mutationFixtures = @(Get-ChildItem (Join-Path $repository 'validation\mutation-fixtures') -Filter '*.json')
    $liveEvidence = @(Get-ChildItem (Join-Path $repository 'validation\live-evidence') -Filter '*.json')
    $matrix = Get-Content -Raw (Join-Path $repository 'validation\matrix.json') | ConvertFrom-Json
    $statusCounts = @{}
    foreach ($target in $matrix.targets) {
        $status = [string]$target.mutationStatus
        if (-not $statusCounts.ContainsKey($status)) { $statusCounts[$status] = 0 }
        $statusCounts[$status] += 1
    }
    foreach ($status in @('not_provisioned', 'provisioned_not_run', 'running', 'passed', 'passed_with_limitations', 'failed', 'blocked', 'handler_removed')) {
        if (-not $statusCounts.ContainsKey($status)) { $statusCounts[$status] = 0 }
    }
    $vmToolNames = @(
        @{ Name = 'Hyper-V'; Command = 'Get-VM' },
        @{ Name = 'VirtualBox'; Command = 'VBoxManage' },
        @{ Name = 'VMware'; Command = 'vmrun' },
        @{ Name = 'libvirt'; Command = 'virsh' },
        @{ Name = 'Multipass'; Command = 'multipass' }
    ) | Where-Object { Get-Command $_.Command -ErrorAction SilentlyContinue } | ForEach-Object { $_.Name }
    $vmInfrastructure = if ($vmToolNames.Count -eq 0) { 'None detected' } else { $vmToolNames -join ', ' }
    $hypervisorLayerPresent = $null -ne (Get-Service -Name 'hvhost' -ErrorAction SilentlyContinue)
    $windowsSandboxPresent = Test-Path -LiteralPath (Join-Path $env:SystemRoot 'System32\WindowsSandbox.exe')

    $localDirectory = Join-Path $repository '.deslopper\local'
    $denylistPath = Join-Path $localDirectory 'development-host-denylist.json'
    $liveScenarioPath = Join-Path $localDirectory 'live-validation-scenario.json'
    $approvalPath = Join-Path $localDirectory 'approved-validation-target.json'
    $preparationPath = Join-Path $localDirectory 'physical-laptop-validation-draft.json'
    $legacyApprovalPath = Join-Path $localDirectory 'approved-validation-vms.json'
    $denylistPresent = Test-Path -LiteralPath $denylistPath
    $liveScenarioPresent = Test-Path -LiteralPath $liveScenarioPath
    $approvedInventoryPresent = Test-Path -LiteralPath $approvalPath
    $preparationRecordPresent = Test-Path -LiteralPath $preparationPath
    $legacyApprovedInventoryPresent = Test-Path -LiteralPath $legacyApprovalPath
    $hostDenied = $false
    $liveScenarioTargetsHost = $false
    $approvedVmCount = 0
    $approvedTargetType = $null
    $preparedTargetProtected = $false

    $currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
    $fingerprintInput = "$($env:COMPUTERNAME.ToUpperInvariant()):$($currentVersion.EditionID):$($currentVersion.CurrentBuild)"
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        $hostFingerprint = [BitConverter]::ToString(
            $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($fingerprintInput))
        ).Replace('-', '').ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
    if ($denylistPresent) {
        $denylist = Get-Content -Raw $denylistPath | ConvertFrom-Json
        $hostDenied = @($denylist.developmentHostFingerprints) -contains $hostFingerprint
    }
    if ($liveScenarioPresent) {
        $liveScenario = Get-Content -Raw $liveScenarioPath | ConvertFrom-Json
        $liveScenarioTargetsHost = [string]$liveScenario.expectedMachineId -eq $hostFingerprint
    }
    if ($approvedInventoryPresent) {
        $approvedTarget = Get-Content -Raw $approvalPath | ConvertFrom-Json
        $approvedTargetType = [string]$approvedTarget.targetType
        $approvedVmCount = if (
            $approvedTarget.recordKind -eq 'approval' -and
            $approvedTarget.approvalStatus -eq 'approved'
        ) { 1 } else { 0 }
    } elseif ($legacyApprovedInventoryPresent) {
        $approvedInventory = Get-Content -Raw $legacyApprovalPath | ConvertFrom-Json
        $approvedTargetType = 'virtual_machine'
        $approvedVmCount = @($approvedInventory.vms).Count
    }
    if ($preparationRecordPresent -and $denylistPresent) {
        $preparationRecord = Get-Content -Raw $preparationPath | ConvertFrom-Json
        $preparedTargetProtected =
            $preparationRecord.recordKind -eq 'preparation' -and
            $preparationRecord.approvalStatus -eq 'pending' -and
            $preparationRecord.hashedMachineIdentity -eq $hostFingerprint -and
            $preparationRecord.developmentHostProtectionState -eq 'present_distinct' -and
            @($denylist.developmentHostFingerprints) -notcontains $hostFingerprint
    }
    $deslopperProcessCount = @(Get-Process -Name 'win-deslopper' -ErrorAction SilentlyContinue).Count
    $hostProtectionPassed =
        $denylistPresent -and
        ($hostDenied -or $preparedTargetProtected) -and
        -not $liveScenarioTargetsHost -and
        $deslopperProcessCount -eq 0
    $hostProtectionResult = if ($hostProtectionPassed) { 'Passed' } else { 'Failed closed / requires review' }
    $anyApprovedInventoryPresent = $approvedInventoryPresent -or $legacyApprovedInventoryPresent
    $liveValidationBlocker = if (-not $anyApprovedInventoryPresent) {
        'No ignored approved validation target exists; no target is authorized.'
    } elseif ($approvedVmCount -eq 0) {
        'The ignored approval record contains no approved target.'
    } elseif ($approvedTargetType -eq 'virtual_machine' -and $vmToolNames.Count -eq 0) {
        'Approved entries exist, but no supported VM management command is available.'
    } else {
        'No live evidence has been collected; complete the manual guest protocol.'
    }
    $lines = @(
        '# Deslopper VM fixture validation report',
        '',
        "Generated: $($started.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ'))",
        '',
        "- Result: Passed",
        "- Checked-in fixture bundles: $($fixtures.Count)",
        "- Synthetic mutation fixture bundles: $($mutationFixtures.Count)",
        "- Live mutation evidence bundles: $($liveEvidence.Count)",
        "- Defined matrix targets: $($matrix.targets.Count)",
        "- Physical preparation targets (not mutation evidence): $(@($matrix.preparationTargets).Count)",
        "- Historical retired-before-mutation events: $(@($matrix.preparationTargets.lifecycleEvents | Where-Object { $_.event -eq 'retired_before_mutation' }).Count)",
        "- Current approval-review-ready preparation targets: $(@($matrix.preparationTargets | Where-Object { $_.preparationStatus -eq 'ready_for_approval_review' }).Count)",
        "- Physical-target live mutation scenarios: $([int](($matrix.preparationTargets | Measure-Object -Property liveMutationScenarioCount -Sum).Sum))",
        '- Runner: Rust production parsers and detectors',
        '- Mutation boundary: Fixture replay only; no Windows mutation or query execution',
        "- Live mutation VM scenarios completed: $(@($matrix.targets | Where-Object { $_.mutationStatus -in @('passed','passed_with_limitations','failed','handler_removed') }).Count)",
        "- Not provisioned: $($statusCounts['not_provisioned'])",
        "- Provisioned but not run: $($statusCounts['provisioned_not_run'])",
        "- Running: $($statusCounts['running'])",
        "- Passed: $($statusCounts['passed'])",
        "- Passed with limitations: $($statusCounts['passed_with_limitations'])",
        "- Failed: $($statusCounts['failed'])",
        "- Blocked: $($statusCounts['blocked'])",
        "- Handler removed: $($statusCounts['handler_removed'])",
        "- Host protection audit: $hostProtectionResult",
        "- Development-host denylist present: $denylistPresent",
        "- Current host fingerprint denied: $hostDenied",
        "- Current host is protected prepared target: $preparedTargetProtected",
        "- Deslopper processes running: $deslopperProcessCount",
        "- Live scenario manifest present: $liveScenarioPresent",
        "- Live scenario targets development host: $liveScenarioTargetsHost",
        "- Approved generic target present: $approvedInventoryPresent",
        "- Legacy approved VM inventory present: $legacyApprovedInventoryPresent",
        "- Approved target entries: $approvedVmCount",
        "- Windows hypervisor layer present: $hypervisorLayerPresent",
        "- Windows Sandbox executable present: $windowsSandboxPresent",
        "- Host virtualisation tooling detected: $vmInfrastructure",
        "- Live validation blocker: $liveValidationBlocker",
        '- Manual provisioning handoff: `docs/manual-vm-provisioning.md`',
        '- Live infrastructure exercised: None',
        '- Live operation/state combinations exercised: None',
        '- Settings UI, visual behavior, refresh, lifecycle, rollback, drift, policy, recovery, and standard-user findings: Not tested',
        '',
        'Handler maturity:',
        '',
        '| Operation | Supported targets | Maturity | Live disposition |',
        '|---|---|---|---|',
        '| Widgets visibility (`TaskbarDa`) | enabled, disabled | synthetic tested | Internal alpha - not live validated |',
        '| Task View visibility (`ShowTaskViewButton`) | enabled, disabled | synthetic tested | Internal alpha - not live validated |',
        '| Show Desktop corner (`TaskbarSd`) | enabled, disabled | synthetic tested | Internal alpha - not live validated |',
        '',
        'Fixtures:',
        ''
    )
    $lines += $fixtures | Sort-Object Name | ForEach-Object { "- $($_.Name)" }
    $lines += ''
    $lines += 'Synthetic mutation fixtures (not live VM evidence):'
    $lines += ''
    $lines += $mutationFixtures | Sort-Object Name | ForEach-Object { "- $($_.Name)" }
    $lines += ''
    $lines += 'Live evidence bundles:'
    $lines += ''
    if ($liveEvidence.Count -eq 0) {
        $lines += '- None. No live mutation was performed.'
    } else {
        $lines += $liveEvidence | Sort-Object Name | ForEach-Object { "- $($_.Name)" }
    }
    $lines += ''
    $lines += 'Scenario status:'
    $lines += ''
    $lines += '| Scenario | Status |'
    $lines += '|---|---|'
    $lines += $matrix.targets | ForEach-Object { "| $($_.id) | $($_.mutationStatus.Replace('_', ' ')) |" }
    $lines += ''
    $lines += 'Read-only preparation targets (excluded from completed mutation coverage):'
    $lines += ''
    $lines += '| Target | Availability | Read-only | Preparation | Recovery | Approval | Mutation | Live scenarios | Completed credit | Failed credit | Approved credit | Live handler evidence |'
    $lines += '|---|---|---|---|---|---|---|---:|---|---|---|---|'
    $lines += $matrix.preparationTargets | ForEach-Object {
        "| $($_.id) | $($_.availabilityStatus.Replace('_', ' ')) | $($_.readOnlyStatus.Replace('_', ' ')) | $($_.preparationStatus.Replace('_', ' ')) | $($_.recoveryStatus.Replace('_', ' ')) | $($_.approvalStatus.Replace('_', ' ')) | $($_.mutationStatus.Replace('_', ' ')) | $($_.liveMutationScenarioCount) | $($_.countsAsCompletedMutationTarget) | $($_.countsAsFailedMutationTarget) | $($_.countsAsApprovedTarget) | $($_.countsAsLiveHandlerEvidence) |"
    }
    $lines += ''
    $lines += 'The physical target was retired before mutation because the hardware is being sold, then explicitly reactivated for bounded Widgets preparation before its final reset. Retirement remains in chronological history. Read-only Windows 11 Pro 25H2 inspection, Tauri event permission, OneDrive detection, MDM evidence, and permission-limited AppX findings remain useful; no handler has live evidence and zero mutation occurred.'
    $report = ($lines -join "`n") + "`n"
    [System.IO.File]::WriteAllText((Join-Path $repository 'validation\report.md'), $report, [System.Text.UTF8Encoding]::new($false))
} finally {
    Pop-Location
}
