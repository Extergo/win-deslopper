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
if ($denylist.developmentHostFingerprints -contains $fingerprint) {
    throw 'Refusing to create a live-validation scenario manifest on the recorded development host.'
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
