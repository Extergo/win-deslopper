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
    $approvalPath = Join-Path $localDirectory 'approved-validation-vms.json'
    $denylistPresent = Test-Path -LiteralPath $denylistPath
    $liveScenarioPresent = Test-Path -LiteralPath $liveScenarioPath
    $approvedInventoryPresent = Test-Path -LiteralPath $approvalPath
    $hostDenied = $false
    $liveScenarioTargetsHost = $false
    $approvedVmCount = 0

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
        $approvedInventory = Get-Content -Raw $approvalPath | ConvertFrom-Json
        if ($null -ne $approvedInventory.vms) {
            $approvedVmCount = @($approvedInventory.vms).Count
        }
    }
    $deslopperProcessCount = @(Get-Process -Name 'win-deslopper' -ErrorAction SilentlyContinue).Count
    $hostProtectionPassed = $denylistPresent -and $hostDenied -and -not $liveScenarioTargetsHost -and $deslopperProcessCount -eq 0
    $hostProtectionResult = if ($hostProtectionPassed) { 'Passed' } else { 'Failed closed / requires review' }
    $liveValidationBlocker = if (-not $approvedInventoryPresent) {
        'No ignored approved-VM inventory exists; no guest is authorized.'
    } elseif ($approvedVmCount -eq 0) {
        'The ignored approved-VM inventory contains no guest entries.'
    } elseif ($vmToolNames.Count -eq 0) {
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
        "- Deslopper processes running: $deslopperProcessCount",
        "- Live scenario manifest present: $liveScenarioPresent",
        "- Live scenario targets development host: $liveScenarioTargetsHost",
        "- Approved VM inventory present: $approvedInventoryPresent",
        "- Approved VM entries: $approvedVmCount",
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
    $report = ($lines -join "`n") + "`n"
    [System.IO.File]::WriteAllText((Join-Path $repository 'validation\report.md'), $report, [System.Text.UTF8Encoding]::new($false))
} finally {
    Pop-Location
}
