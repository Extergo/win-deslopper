param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[a-z0-9-]+$')]
    [string]$ScenarioId,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[a-zA-Z0-9._-]+$')]
    [string]$CheckpointId,

    [Parameter(Mandatory = $true)]
    [ValidateSet('local', 'microsoft', 'domain', 'work', 'standard-user')]
    [string]$AccountClass,

    [Parameter(Mandatory = $true)]
    [ValidateSet('none', 'local-policy', 'domain', 'mdm')]
    [string]$ManagementContext
)

$ErrorActionPreference = 'Stop'
$repository = Resolve-Path (Join-Path $PSScriptRoot '..')
$matrixPath = Join-Path $repository 'validation\matrix.json'
$matrix = Get-Content -Raw $matrixPath | ConvertFrom-Json
if (-not ($matrix.targets | Where-Object { $_.id -eq $ScenarioId })) {
    throw "Scenario '$ScenarioId' is not in validation/matrix.json."
}

$currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$computerName = $env:COMPUTERNAME.ToUpperInvariant()
$fingerprintInput = "$computerName`:$($currentVersion.EditionID):$($currentVersion.CurrentBuild)"
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
if (-not (Test-Path -LiteralPath $denylistPath)) {
    throw 'The local development-host denylist is required before creating a guest manifest.'
}
$denylist = Get-Content -Raw $denylistPath | ConvertFrom-Json
if ([uint32]$denylist.schemaVersion -ne 1 -or @($denylist.developmentHostFingerprints).Count -eq 0) {
    throw 'The development-host denylist must contain at least one valid local hash.'
}
if (@($denylist.developmentHostFingerprints | Where-Object { $_ -notmatch '^[a-f0-9]{64}$' }).Count -ne 0) {
    throw 'The development-host denylist contains an invalid hash.'
}
if ($denylist.developmentHostFingerprints -contains $fingerprint) {
    throw 'Refusing to create a live-validation scenario manifest on the recorded development host.'
}

$targetApprovalPath = Join-Path $localDirectory 'approved-validation-target.json'
if (-not (Test-Path -LiteralPath $targetApprovalPath)) {
    throw 'A separate, ignored approved-validation-target.json is required; a preparation draft cannot authorize mutation.'
}
$target = Get-Content -Raw $targetApprovalPath | ConvertFrom-Json
$requiredOperations = @(
    'set_taskbar_widgets_visibility',
    'set_taskbar_task_view_visibility',
    'set_taskbar_show_desktop_enabled'
)
$operations = @($target.approvedOperations)
$statesComplete = @($requiredOperations | Where-Object {
    $states = @($target.approvedTargetStates.PSObject.Properties[$_].Value)
    $states.Count -ne 2 -or $states -notcontains 'enabled' -or $states -notcontains 'disabled'
}).Count -eq 0
$recoveryComplete =
    $target.importantDataConfirmed -eq $true -and
    $target.backupConfirmed -eq $true -and
    $target.reinstallationAccepted -eq $true -and
    $target.winreVerified -eq $true -and
    $target.bitlockerRecoveryState -in @('not_applicable_unencrypted', 'recovery_material_confirmed') -and
    $target.recoveryMediaState -in @('available', 'built_in_verified')
$dispositionConfirmed =
    ($target.targetType -eq 'virtual_machine' -and $target.disposableConfirmed -eq $true) -or
    ($target.targetType -eq 'physical_laptop' -and $target.expendableConfirmed -eq $true)
if (
    [uint32]$target.schemaVersion -ne 1 -or
    $target.recordKind -ne 'approval' -or
    $target.approvalStatus -ne 'approved' -or
    $target.scenarioId -ne $ScenarioId -or
    $target.hashedMachineIdentity -ne $fingerprint -or
    $target.windowsEdition -ne $currentVersion.EditionID -or
    [uint32]$target.windowsBuild -ne [uint32]$currentVersion.CurrentBuild -or
    [uint32]$target.windowsUbr -ne [uint32]$currentVersion.UBR -or
    $target.sourceCheckpointCommit -notmatch '^(?:[a-f0-9]{40}|[a-f0-9]{64})$' -or
    $target.developmentHostProtectionState -ne 'present_distinct' -or
    $target.recoveryReadiness -notin @('ready', 'ready_with_warnings') -or
    [string]::IsNullOrWhiteSpace([string]$target.explicitUserApprovalTimestamp) -or
    [uint64]$target.expiresAtEpochMs -le [uint64][DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() -or
    $operations.Count -ne 3 -or
    @($requiredOperations | Where-Object { $operations -notcontains $_ }).Count -ne 0 -or
    -not $statesComplete -or
    -not $recoveryComplete -or
    -not $dispositionConfirmed
) {
    throw 'The local validation-target approval is incomplete, expired, or does not match this target.'
}

$manifest = [ordered]@{
    schemaVersion = 1
    scenarioId = $ScenarioId
    expectedMachineId = $fingerprint
    expectedEdition = [string]$currentVersion.EditionID
    expectedBuild = [uint32]$currentVersion.CurrentBuild
    expectedUpdateBuildRevision = [uint32]$currentVersion.UBR
    checkpointId = $CheckpointId
    accountClass = $AccountClass
    managementContext = $ManagementContext
}
$manifestPath = Join-Path $localDirectory 'live-validation-scenario.json'
[System.IO.Directory]::CreateDirectory($localDirectory) | Out-Null
[System.IO.File]::WriteAllText(
    $manifestPath,
    ($manifest | ConvertTo-Json -Depth 4),
    [System.Text.UTF8Encoding]::new($false)
)

[pscustomobject]@{
    ScenarioId = $ScenarioId
    MachineFingerprint = $fingerprint
    Edition = $currentVersion.EditionID
    Build = [uint32]$currentVersion.CurrentBuild
    UBR = [uint32]$currentVersion.UBR
    ManifestPath = $manifestPath
    LaunchArguments = "--enable-mutation-alpha --enable-live-validation --validation-scenario=$ScenarioId --expected-machine-id=$fingerprint --expected-checkpoint-id=$CheckpointId"
}
