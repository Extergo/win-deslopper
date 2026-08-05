param(
    [string]$DraftPath = (Join-Path $PSScriptRoot '..\.deslopper\local\widgets-approval-review-draft.json'),
    [switch]$SelfTest
)

$ErrorActionPreference = 'Stop'

function Test-Hash([object]$Value) {
    return $Value -is [string] -and $Value -cmatch '^[a-f0-9]{64}$'
}

function Test-Representation([object]$Value) {
    if ($null -eq $Value) { return $false }
    $names = @($Value.PSObject.Properties.Name)
    if ($Value.kind -eq 'missing') {
        return $names.Count -eq 1 -and $names[0] -eq 'kind'
    }
    return $Value.kind -eq 'dword' -and
        $names.Count -eq 2 -and
        $names -contains 'kind' -and
        $names -contains 'value' -and
        [int]$Value.value -in @(0, 1)
}

function Assert-DraftShape([object]$Draft, [long]$NowEpochMs) {
    $required = @(
        'schemaVersion', 'recordKind', 'approvalStatus', 'mutationAllowed', 'executed',
        'scenarioId', 'targetType', 'hashedMachineIdentity', 'windowsEdition', 'windowsBuild',
        'windowsUbr', 'sourceCheckpointCommit', 'handlerVersion', 'inspectionId',
        'inspectionEvidenceSha256', 'developmentHostDenylistIdentity', 'operation',
        'originalRepresentation', 'detectorState', 'userObservedState', 'proposedTarget',
        'rollbackRepresentation', 'executionLimit', 'createdAtUtc', 'expiresAtEpochMs',
        'automaticRepair', 'elevation', 'explorerTermination', 'genericRegistryPath',
        'allowedHandlers'
    )
    $actual = @($Draft.PSObject.Properties.Name)
    if (@($required | Where-Object { $actual -notcontains $_ }).Count -ne 0 -or
        @($actual | Where-Object { $required -notcontains $_ }).Count -ne 0) {
        throw 'The approval-review draft fields do not match the closed schema.'
    }
    if ([uint32]$Draft.schemaVersion -ne 1 -or
        $Draft.recordKind -ne 'approval_review_draft' -or
        $Draft.approvalStatus -ne 'ready_for_approval_review' -or
        $Draft.mutationAllowed -ne $false -or
        $Draft.executed -ne $false -or
        $Draft.targetType -ne 'physical_laptop' -or
        $Draft.operation -ne 'set_taskbar_widgets_visibility' -or
        [uint32]$Draft.executionLimit -ne 1 -or
        $Draft.automaticRepair -ne $false -or
        $Draft.elevation -ne $false -or
        $Draft.explorerTermination -ne $false -or
        $Draft.genericRegistryPath -ne $false) {
        throw 'The approval-review draft would broaden or authorize the preparation boundary.'
    }
    if (-not (Test-Hash $Draft.hashedMachineIdentity) -or
        -not (Test-Hash $Draft.inspectionEvidenceSha256) -or
        -not (Test-Hash $Draft.developmentHostDenylistIdentity) -or
        $Draft.hashedMachineIdentity -eq $Draft.developmentHostDenylistIdentity) {
        throw 'The approval-review draft identity or evidence binding is invalid.'
    }
    if ($Draft.sourceCheckpointCommit -cnotmatch '^[a-f0-9]{40}$' -or
        $Draft.inspectionId -cnotmatch '^inspection-[a-zA-Z0-9-]+$' -or
        [uint32]$Draft.windowsBuild -lt 22000 -or
        [uint32]$Draft.windowsUbr -lt 0 -or
        [string]::IsNullOrWhiteSpace([string]$Draft.windowsEdition) -or
        [string]::IsNullOrWhiteSpace([string]$Draft.handlerVersion)) {
        throw 'The approval-review draft target, source, handler, or inspection binding is invalid.'
    }
    if (-not (Test-Representation $Draft.originalRepresentation) -or
        -not (Test-Representation $Draft.rollbackRepresentation) -or
        ($Draft.originalRepresentation | ConvertTo-Json -Compress) -cne
            ($Draft.rollbackRepresentation | ConvertTo-Json -Compress)) {
        throw 'Rollback must preserve the exact original Widgets representation.'
    }
    $handlers = @($Draft.allowedHandlers)
    if ($handlers.Count -ne 1 -or $handlers[0] -ne 'set_taskbar_widgets_visibility') {
        throw 'Only the fixed Widgets visibility handler may appear in this draft.'
    }
    $detectorFields = @('effectiveState', 'authority', 'policyState', 'conflict', 'evidenceTimestampEpochMs')
    $actualDetectorFields = @($Draft.detectorState.PSObject.Properties.Name)
    if (@($detectorFields | Where-Object { $actualDetectorFields -notcontains $_ }).Count -ne 0 -or
        @($actualDetectorFields | Where-Object { $detectorFields -notcontains $_ }).Count -ne 0 -or
        $Draft.detectorState.effectiveState -notin @('enabled', 'disabled', 'unknown') -or
        $Draft.detectorState.authority -notin @('user', 'local_policy', 'domain', 'mdm', 'unknown') -or
        $Draft.detectorState.policyState -notin @('not_configured', 'enabled', 'disabled', 'unknown') -or
        $Draft.detectorState.conflict -ne $false -or
        [long]$Draft.detectorState.evidenceTimestampEpochMs -lt 1) {
        throw 'The Widgets detector state is incomplete or conflicting.'
    }
    $expectedTarget = if ($Draft.userObservedState -eq 'visible') { 'disabled' } elseif ($Draft.userObservedState -eq 'not_visible') { 'enabled' } else { $null }
    if ($null -eq $expectedTarget -or $Draft.proposedTarget -ne $expectedTarget) {
        throw 'The proposed Widgets target is not the opposite of the user-confirmed visible state.'
    }
    $created = [DateTimeOffset]::Parse([string]$Draft.createdAtUtc).ToUnixTimeMilliseconds()
    $expires = [long]$Draft.expiresAtEpochMs
    if ($created -gt $NowEpochMs -or $expires -le $NowEpochMs -or $expires -gt ($created + 7200000)) {
        throw 'The approval-review draft is expired, future-dated, or valid for longer than two hours.'
    }
}

if ($SelfTest) {
    $now = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $hashA = 'a' * 64
    $hashB = 'b' * 64
    $valid = [pscustomobject][ordered]@{
        schemaVersion = 1; recordKind = 'approval_review_draft'; approvalStatus = 'ready_for_approval_review'
        mutationAllowed = $false; executed = $false; scenarioId = 'widgets-review'; targetType = 'physical_laptop'
        hashedMachineIdentity = $hashA; windowsEdition = 'Professional'; windowsBuild = 26200; windowsUbr = 1
        sourceCheckpointCommit = ('c' * 40); handlerVersion = 'mutation-alpha.1'; inspectionId = 'inspection-1'
        inspectionEvidenceSha256 = ('d' * 64); developmentHostDenylistIdentity = $hashB
        operation = 'set_taskbar_widgets_visibility'; originalRepresentation = [pscustomobject]@{ kind = 'missing' }
        detectorState = [pscustomobject]@{ effectiveState = 'unknown'; authority = 'unknown'; policyState = 'not_configured'; conflict = $false; evidenceTimestampEpochMs = $now }
        userObservedState = 'visible'; proposedTarget = 'disabled'; rollbackRepresentation = [pscustomobject]@{ kind = 'missing' }
        executionLimit = 1; createdAtUtc = [DateTimeOffset]::FromUnixTimeMilliseconds($now - 1000).ToString('o')
        expiresAtEpochMs = $now + 1000; automaticRepair = $false; elevation = $false; explorerTermination = $false
        genericRegistryPath = $false; allowedHandlers = @('set_taskbar_widgets_visibility')
    }
    Assert-DraftShape $valid $now
    $rejections = 0
    foreach ($change in @(
        @{ mutationAllowed = $true },
        @{ allowedHandlers = @('set_taskbar_widgets_visibility', 'set_taskbar_task_view_visibility') },
        @{ expiresAtEpochMs = $now + 7200001 },
        @{ proposedTarget = 'enabled' }
    )) {
        $candidate = $valid | ConvertTo-Json -Depth 8 | ConvertFrom-Json
        foreach ($entry in $change.GetEnumerator()) { $candidate.($entry.Key) = $entry.Value }
        try { Assert-DraftShape $candidate $now } catch { $rejections++ }
    }
    if ($rejections -ne 4) { throw "Approval-review self-test rejected $rejections of 4 invalid drafts." }
    Write-Host 'Approval-review draft validator self-test passed (valid=1, rejected=4).'
    return
}

if (-not (Test-Path -LiteralPath $DraftPath)) {
    throw "Approval-review draft is missing: $DraftPath"
}
$draft = Get-Content -Raw -LiteralPath $DraftPath | ConvertFrom-Json
Assert-DraftShape $draft ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())

$repository = Resolve-Path (Join-Path $PSScriptRoot '..')
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
$denylistPath = Join-Path $repository '.deslopper\local\development-host-denylist.json'
if (-not (Test-Path -LiteralPath $denylistPath)) { throw 'The local development-host denylist is missing.' }
$denylist = Get-Content -Raw -LiteralPath $denylistPath | ConvertFrom-Json
$denylistHashes = @($denylist.developmentHostFingerprints)
if ($draft.hashedMachineIdentity -ne $fingerprint -or
    $draft.developmentHostDenylistIdentity -notin $denylistHashes -or
    $draft.developmentHostDenylistIdentity -eq $fingerprint -or
    $draft.windowsEdition -ne [string]$currentVersion.EditionID -or
    [uint32]$draft.windowsBuild -ne [uint32]$currentVersion.CurrentBuild -or
    [uint32]$draft.windowsUbr -ne [uint32]$currentVersion.UBR) {
    throw 'The approval-review draft does not match this target or its local development-host denylist.'
}
& git -C $repository merge-base --is-ancestor $draft.sourceCheckpointCommit HEAD
if ($LASTEXITCODE -ne 0) { throw 'The draft source checkpoint is not an ancestor of the current repository state.' }
Write-Host 'Approval-review draft is valid, current, target-bound, Widgets-only, and non-authorizing.'
