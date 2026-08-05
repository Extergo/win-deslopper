param(
    [ValidatePattern('^[a-z0-9-]+$')]
    [string]$ScenarioId,

    [ValidatePattern('^[a-zA-Z0-9._-]+$')]
    [string]$CheckpointId,

    [ValidateSet('local', 'microsoft', 'domain', 'work', 'standard-user')]
    [string]$AccountClass,

    [ValidateSet('none', 'local-policy', 'domain', 'mdm')]
    [string]$ManagementContext,

    [string[]]$OperationScope,

    [switch]$ParserSelfTest
)

$ErrorActionPreference = 'Stop'
$knownOperations = @(
    'set_taskbar_widgets_visibility',
    'set_taskbar_task_view_visibility',
    'set_taskbar_show_desktop_enabled'
)
$knownStates = @('enabled', 'disabled')

function ConvertTo-OperationScopes([string[]]$Values) {
    if (@($Values).Count -eq 0) { throw 'At least one explicit operation scope is required.' }
    $seenOperations = @{}
    $result = @()
    foreach ($value in $Values) {
        if ($value -notmatch '^([^=]+)=([^=]+)$') {
            throw "Operation scope '$value' must use operation=state[,state]."
        }
        $operation = $Matches[1]
        $states = @($Matches[2].Split(',') | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        if ($operation -notin $knownOperations) { throw "Unknown operation '$operation'." }
        if ($seenOperations.ContainsKey($operation)) { throw "Duplicate operation '$operation'." }
        if ($states.Count -eq 0) { throw "Operation '$operation' has no target state." }
        if (@($states | Where-Object { $_ -notin $knownStates }).Count -ne 0) {
            throw "Operation '$operation' contains an unknown target state."
        }
        if (@($states | Select-Object -Unique).Count -ne $states.Count) {
            throw "Operation '$operation' contains a duplicate target state."
        }
        $seenOperations[$operation] = $true
        $result += [pscustomobject][ordered]@{
            operationId = $operation
            allowedTargetStates = $states
        }
    }
    return @($result)
}

if ($ParserSelfTest) {
    $valid = @(ConvertTo-OperationScopes @('set_taskbar_widgets_visibility=enabled'))
    if ($valid.Count -ne 1 -or $valid[0].allowedTargetStates[0] -ne 'enabled') {
        throw 'Scoped manifest parser did not preserve the exact Widgets-enabled scope.'
    }
    $rejected = 0
    foreach ($invalid in @(
        @(),
        @('unknown=enabled'),
        @('set_taskbar_widgets_visibility=unknown'),
        @('set_taskbar_widgets_visibility=enabled,enabled'),
        @('set_taskbar_widgets_visibility=enabled', 'set_taskbar_widgets_visibility=disabled')
    )) {
        try { [void](ConvertTo-OperationScopes $invalid) } catch { $rejected++ }
    }
    if ($rejected -ne 5) { throw "Scoped manifest parser rejected $rejected of 5 invalid inputs." }
    [pscustomobject]@{
        Passed = 6
        Failed = 0
        WidgetsScopes = $valid.Count
        WidgetsEnabledOnly = $true
        TaskViewAdded = $false
        ShowDesktopAdded = $false
        DisabledAdded = $false
    }
    return
}

if ([string]::IsNullOrWhiteSpace($ScenarioId) -or
    [string]::IsNullOrWhiteSpace($CheckpointId) -or
    [string]::IsNullOrWhiteSpace($AccountClass) -or
    [string]::IsNullOrWhiteSpace($ManagementContext)) {
    throw 'ScenarioId, CheckpointId, AccountClass, and ManagementContext are required.'
}
$requestedScopes = @(ConvertTo-OperationScopes $OperationScope)
$repository = Resolve-Path (Join-Path $PSScriptRoot '..')
$matrix = Get-Content -Raw (Join-Path $repository 'validation\matrix.json') | ConvertFrom-Json
if (-not (@($matrix.targets) + @($matrix.preparationTargets) | Where-Object { $_.id -eq $ScenarioId })) {
    throw "Scenario '$ScenarioId' is not in validation/matrix.json."
}

$currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$fingerprintInput = "$($env:COMPUTERNAME.ToUpperInvariant()):$($currentVersion.EditionID):$($currentVersion.CurrentBuild)"
$sha = [Security.Cryptography.SHA256]::Create()
try {
    $fingerprint = [BitConverter]::ToString(
        $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($fingerprintInput))
    ).Replace('-', '').ToLowerInvariant()
} finally {
    $sha.Dispose()
}

$localDirectory = Join-Path $repository '.deslopper\local'
$denylistPath = Join-Path $localDirectory 'development-host-denylist.json'
$targetApprovalPath = Join-Path $localDirectory 'approved-validation-target.json'
if (-not (Test-Path -LiteralPath $denylistPath)) { throw 'The local development-host denylist is required.' }
if (-not (Test-Path -LiteralPath $targetApprovalPath)) {
    throw 'A separate ignored version-2 target approval is required; a review draft cannot authorize mutation.'
}
$denylist = Get-Content -Raw $denylistPath | ConvertFrom-Json
$denylistHashes = @($denylist.developmentHostFingerprints)
if ([uint32]$denylist.schemaVersion -ne 1 -or $denylistHashes.Count -ne 1 -or
    $denylistHashes[0] -cnotmatch '^[a-f0-9]{64}$' -or $denylistHashes[0] -eq $fingerprint) {
    throw 'The development-host denylist is invalid or matches this target.'
}

$target = Get-Content -Raw $targetApprovalPath | ConvertFrom-Json
$approvalScopes = @($target.approvedOperationScopes)
$approvalOperations = @($approvalScopes | ForEach-Object { $_.operationId })
$scopeValid = $approvalScopes.Count -gt 0 -and
    @($approvalOperations | Select-Object -Unique).Count -eq $approvalOperations.Count
foreach ($scope in $approvalScopes) {
    $states = @($scope.allowedTargetStates)
    $scopeValid = $scopeValid -and
        $scope.operationId -in $knownOperations -and
        $states.Count -gt 0 -and
        @($states | Select-Object -Unique).Count -eq $states.Count -and
        @($states | Where-Object { $_ -notin $knownStates }).Count -eq 0 -and
        $scope.handlerVersion -eq 'mutation-alpha.1'
}
$requestedMatchesApproval = $requestedScopes.Count -eq $approvalScopes.Count
foreach ($scope in $requestedScopes) {
    $approved = @($approvalScopes | Where-Object { $_.operationId -eq $scope.operationId })
    $requestedStates = @($scope.allowedTargetStates)
    $approvedStates = @($approved[0].allowedTargetStates)
    $requestedMatchesApproval = $requestedMatchesApproval -and $approved.Count -eq 1 -and
        $requestedStates.Count -eq $approvedStates.Count -and
        @($requestedStates | Where-Object { $_ -notin $approvedStates }).Count -eq 0
}
$recoveryComplete = $target.importantDataConfirmed -eq $true -and
    $target.backupConfirmed -eq $true -and $target.reinstallationAccepted -eq $true -and
    $target.winreVerified -eq $true -and
    $target.bitlockerRecoveryState -in @('not_applicable_unencrypted', 'recovery_material_confirmed') -and
    $target.recoveryMediaState -in @('available', 'built_in_verified')
$dispositionConfirmed = ($target.targetType -eq 'virtual_machine' -and $target.disposableConfirmed -eq $true) -or
    ($target.targetType -eq 'physical_laptop' -and $target.expendableConfirmed -eq $true)
$head = (& git -C $repository rev-parse HEAD).Trim()
if ([uint32]$target.schemaVersion -ne 2 -or $target.recordKind -ne 'approval' -or
    $target.approvalStatus -ne 'approved' -or $target.scenarioId -ne $ScenarioId -or
    $target.hashedMachineIdentity -ne $fingerprint -or
    $target.windowsEdition -ne $currentVersion.EditionID -or
    [uint32]$target.windowsBuild -ne [uint32]$currentVersion.CurrentBuild -or
    [uint32]$target.windowsUbr -ne [uint32]$currentVersion.UBR -or
    $target.sourceCheckpointCommit -ne $head -or
    $target.sourceInspectionId -cnotmatch '^inspection-[a-zA-Z0-9-]+$' -or
    $target.inspectionEvidenceSha256 -cnotmatch '^[a-f0-9]{64}$' -or
    $target.developmentHostDenylistIdentity -ne $denylistHashes[0] -or
    -not $scopeValid -or -not $requestedMatchesApproval -or
    [uint32]$target.maximumPlans -lt 1 -or [uint32]$target.maximumPlans -gt 32 -or
    [uint32]$target.maximumExecutions -lt 1 -or
    [uint32]$target.maximumExecutions -gt [uint32]$target.maximumPlans -or
    $target.developmentHostProtectionState -ne 'present_distinct' -or
    $target.recoveryReadiness -notin @('ready', 'ready_with_warnings') -or
    [string]::IsNullOrWhiteSpace([string]$target.explicitUserApprovalTimestamp) -or
    [uint64]$target.expiresAtEpochMs -le [uint64][DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() -or
    -not $recoveryComplete -or -not $dispositionConfirmed) {
    throw 'The scoped target approval is incomplete, expired, mismatched, or broader than requested.'
}

$manifestScopes = @($approvalScopes | ForEach-Object {
    [ordered]@{
        operationId = [string]$_.operationId
        allowedTargetStates = @($_.allowedTargetStates)
        handlerVersion = [string]$_.handlerVersion
    }
})
$manifest = [ordered]@{
    schemaVersion = 2
    scenarioId = $ScenarioId
    expectedMachineId = $fingerprint
    expectedEdition = [string]$currentVersion.EditionID
    expectedBuild = [uint32]$currentVersion.CurrentBuild
    expectedUpdateBuildRevision = [uint32]$currentVersion.UBR
    checkpointId = $CheckpointId
    accountClass = $AccountClass
    managementContext = $ManagementContext
    approvalId = [string]$target.approvalId
    sourceCommit = [string]$target.sourceCheckpointCommit
    sourceInspectionId = [string]$target.sourceInspectionId
    inspectionEvidenceSha256 = [string]$target.inspectionEvidenceSha256
    approvedOperationScopes = $manifestScopes
    maximumPlans = [uint32]$target.maximumPlans
    maximumExecutions = [uint32]$target.maximumExecutions
}
$manifestPath = Join-Path $localDirectory 'live-validation-scenario.json'
[IO.Directory]::CreateDirectory($localDirectory) | Out-Null
[IO.File]::WriteAllText(
    $manifestPath,
    (($manifest | ConvertTo-Json -Depth 8) + "`n"),
    [Text.UTF8Encoding]::new($false)
)

[pscustomobject]@{
    ScenarioId = $ScenarioId
    SchemaVersion = 2
    OperationScopeCount = $manifestScopes.Count
    Operations = @($manifestScopes | ForEach-Object { $_.operationId }) -join ', '
    MaximumPlans = $manifest.maximumPlans
    MaximumExecutions = $manifest.maximumExecutions
    ManifestPath = $manifestPath
    MachineFingerprintDisplayed = $false
    LaunchArguments = "--enable-mutation-alpha --enable-live-validation --validation-scenario=$ScenarioId --expected-machine-id=<local-target-hash> --expected-checkpoint-id=$CheckpointId --expected-source-commit=$($manifest.sourceCommit)"
}
