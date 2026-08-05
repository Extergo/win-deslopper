param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[a-f0-9]{40}$')]
    [string]$SourceCheckpointCommit,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^inspection-[a-zA-Z0-9-]+$')]
    [string]$InspectionId,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[a-f0-9]{64}$')]
    [string]$InspectionEvidenceSha256,

    [Parameter(Mandatory = $true)]
    [ValidateSet('visible', 'not_visible')]
    [string]$UserObservedState,

    [Parameter(Mandatory = $true)]
    [long]$EvidenceTimestampEpochMs
)

$ErrorActionPreference = 'Stop'
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$localDirectory = Join-Path $repository '.deslopper\local'
$denylistPath = Join-Path $localDirectory 'development-host-denylist.json'
$approvalPath = Join-Path $localDirectory 'approved-validation-target.json'
$draftPath = Join-Path $localDirectory 'widgets-approval-review-draft.json'

if (Test-Path -LiteralPath $approvalPath) {
    throw 'Refusing to create a review draft while a live approval manifest exists.'
}
if (-not (Test-Path -LiteralPath $denylistPath)) {
    throw 'The ignored development-host denylist is required.'
}

$denylist = Get-Content -Raw -LiteralPath $denylistPath | ConvertFrom-Json
$developmentHostHashes = @($denylist.developmentHostFingerprints)
if ([uint32]$denylist.schemaVersion -ne 1 -or
    $developmentHostHashes.Count -ne 1 -or
    $developmentHostHashes[0] -cnotmatch '^[a-f0-9]{64}$') {
    throw 'The development-host denylist must contain exactly one valid identity.'
}

$currentVersion = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$fingerprintInput = "$($env:COMPUTERNAME.ToUpperInvariant()):$($currentVersion.EditionID):$($currentVersion.CurrentBuild)"
$sha = [Security.Cryptography.SHA256]::Create()
try {
    $targetFingerprint = [BitConverter]::ToString(
        $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($fingerprintInput))
    ).Replace('-', '').ToLowerInvariant()
} finally {
    $sha.Dispose()
}
if ($developmentHostHashes -contains $targetFingerprint) {
    throw 'The development-host identity matches this validation target.'
}

& git -C $repository merge-base --is-ancestor $SourceCheckpointCommit HEAD
if ($LASTEXITCODE -ne 0) {
    throw 'The source checkpoint is not an ancestor of the current repository state.'
}

$widgetsPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced'
$widgetsValue = 'TaskbarDa'
$widgetsKey = Get-Item -LiteralPath $widgetsPath -ErrorAction Stop
if ($widgetsKey.GetValueNames() -contains $widgetsValue) {
    $kind = $widgetsKey.GetValueKind($widgetsValue).ToString()
    $value = $widgetsKey.GetValue($widgetsValue, $null, 'DoNotExpandEnvironmentNames')
    if ($kind -ne 'DWord' -or [int]$value -notin @(0, 1)) {
        throw 'The current Widgets representation is not a supported DWORD 0/1 or value absence.'
    }
    $representation = [ordered]@{ kind = 'dword'; value = [int]$value }
} else {
    $representation = [ordered]@{ kind = 'missing' }
}

$policyPath = 'HKLM:\SOFTWARE\Policies\Microsoft\Dsh'
$policyValue = 'AllowNewsAndInterests'
$policyConfigured = $false
if (Test-Path -LiteralPath $policyPath) {
    $policyKey = Get-Item -LiteralPath $policyPath
    $policyConfigured = $policyKey.GetValueNames() -contains $policyValue
}
if ($policyConfigured) {
    throw 'A configured Widgets policy blocks approval-review preparation.'
}

$proposedTarget = if ($UserObservedState -eq 'visible') { 'disabled' } else { 'enabled' }
$created = [DateTimeOffset]::UtcNow
$draft = [ordered]@{
    schemaVersion = 1
    recordKind = 'approval_review_draft'
    approvalStatus = 'ready_for_approval_review'
    mutationAllowed = $false
    executed = $false
    scenarioId = 'physical-laptop-widgets-visibility-review'
    targetType = 'physical_laptop'
    hashedMachineIdentity = $targetFingerprint
    windowsEdition = [string]$currentVersion.EditionID
    windowsBuild = [uint32]$currentVersion.CurrentBuild
    windowsUbr = [uint32]$currentVersion.UBR
    sourceCheckpointCommit = $SourceCheckpointCommit
    handlerVersion = 'mutation-alpha.1'
    inspectionId = $InspectionId
    inspectionEvidenceSha256 = $InspectionEvidenceSha256
    developmentHostDenylistIdentity = $developmentHostHashes[0]
    operation = 'set_taskbar_widgets_visibility'
    originalRepresentation = $representation
    detectorState = [ordered]@{
        effectiveState = 'unknown'
        authority = 'unknown'
        policyState = 'not_configured'
        conflict = $false
        evidenceTimestampEpochMs = $EvidenceTimestampEpochMs
    }
    userObservedState = $UserObservedState
    proposedTarget = $proposedTarget
    rollbackRepresentation = $representation
    executionLimit = 1
    createdAtUtc = $created.ToString('o')
    expiresAtEpochMs = $created.AddHours(2).ToUnixTimeMilliseconds()
    automaticRepair = $false
    elevation = $false
    explorerTermination = $false
    genericRegistryPath = $false
    allowedHandlers = @('set_taskbar_widgets_visibility')
}

[IO.Directory]::CreateDirectory($localDirectory) | Out-Null
[IO.File]::WriteAllText(
    $draftPath,
    (($draft | ConvertTo-Json -Depth 8) + "`n"),
    [Text.UTF8Encoding]::new($false)
)

[pscustomobject]@{
    DraftCreated = $true
    ApprovalStatus = $draft.approvalStatus
    MutationAllowed = $draft.mutationAllowed
    Operation = $draft.operation
    ProposedTarget = $draft.proposedTarget
    OriginalRepresentation = $draft.originalRepresentation.kind
    ExecutionLimit = $draft.executionLimit
    ValidityMinutes = 120
    IdentityDisplayed = $false
}
