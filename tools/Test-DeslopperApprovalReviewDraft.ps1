param(
    [string]$DraftPath = (Join-Path $PSScriptRoot '..\.deslopper\local\widgets-approval-review-draft.json'),
    [switch]$SelfTest
)

$ErrorActionPreference = 'Stop'
$widgetsOperation = 'set_taskbar_widgets_visibility'
$knownOperations = @(
    $widgetsOperation,
    'set_taskbar_task_view_visibility',
    'set_taskbar_show_desktop_enabled'
)
$knownStates = @('enabled', 'disabled')

function Test-Hash([object]$Value) {
    return $Value -is [string] -and $Value -cmatch '^[a-f0-9]{64}$'
}

function Test-Representation([object]$Value) {
    if ($null -eq $Value) { return $false }
    $names = @($Value.PSObject.Properties.Name)
    if ($Value.kind -eq 'missing') { return $names.Count -eq 1 -and $names[0] -eq 'kind' }
    return $Value.kind -eq 'dword' -and $names.Count -eq 2 -and
        $names -contains 'kind' -and $names -contains 'value' -and [int]$Value.value -in @(0, 1)
}

function Assert-Scopes([object[]]$Scopes) {
    if (@($Scopes).Count -eq 0) { throw 'At least one approval scope is required.' }
    $operations = @{}
    foreach ($scope in $Scopes) {
        $fields = @($scope.PSObject.Properties.Name)
        if ($fields.Count -ne 3 -or $fields -notcontains 'operationId' -or
            $fields -notcontains 'allowedTargetStates' -or $fields -notcontains 'handlerVersion' -or
            $scope.operationId -notin $knownOperations -or $operations.ContainsKey($scope.operationId) -or
            $scope.handlerVersion -ne 'mutation-alpha.1') {
            throw 'An approval scope is unknown, duplicated, incomplete, or malformed.'
        }
        $states = @($scope.allowedTargetStates)
        if ($states.Count -eq 0 -or @($states | Select-Object -Unique).Count -ne $states.Count -or
            @($states | Where-Object { $_ -notin $knownStates }).Count -ne 0) {
            throw 'An approval scope has missing, duplicate, or unknown target states.'
        }
        $operations[$scope.operationId] = $true
    }
}

function Assert-DraftShape([object]$Draft, [long]$NowEpochMs) {
    $required = @(
        'schemaVersion', 'recordKind', 'approvalStatus', 'mutationAllowed', 'executed',
        'scenarioId', 'targetType', 'hashedMachineIdentity', 'windowsEdition', 'windowsBuild',
        'windowsUbr', 'sourceCheckpointCommit', 'inspectionId', 'inspectionEvidenceSha256',
        'developmentHostDenylistIdentity', 'approvedOperationScopes', 'maximumPlans',
        'maximumExecutions', 'originalRepresentation', 'detectorState', 'userObservedState',
        'rollbackRepresentation', 'rollbackAuthority', 'createdAtUtc', 'expiresAtEpochMs',
        'automaticRepair', 'elevation', 'explorerTermination', 'restart', 'genericRegistryPath'
    )
    $actual = @($Draft.PSObject.Properties.Name)
    if (@($required | Where-Object { $actual -notcontains $_ }).Count -ne 0 -or
        @($actual | Where-Object { $required -notcontains $_ }).Count -ne 0) {
        throw 'The approval-review draft fields do not match the closed version-2 schema.'
    }
    if ([uint32]$Draft.schemaVersion -ne 2 -or $Draft.recordKind -ne 'approval_review_draft' -or
        $Draft.approvalStatus -ne 'ready_for_approval_review' -or $Draft.mutationAllowed -ne $false -or
        $Draft.executed -ne $false -or $Draft.targetType -ne 'physical_laptop' -or
        $Draft.automaticRepair -ne $false -or $Draft.elevation -ne $false -or
        $Draft.explorerTermination -ne $false -or $Draft.restart -ne $false -or
        $Draft.genericRegistryPath -ne $false) {
        throw 'The draft would broaden or authorize the preparation boundary.'
    }
    if (-not (Test-Hash $Draft.hashedMachineIdentity) -or
        -not (Test-Hash $Draft.inspectionEvidenceSha256) -or
        -not (Test-Hash $Draft.developmentHostDenylistIdentity) -or
        $Draft.hashedMachineIdentity -eq $Draft.developmentHostDenylistIdentity) {
        throw 'The draft identity or evidence binding is invalid.'
    }
    if ($Draft.sourceCheckpointCommit -cnotmatch '^[a-f0-9]{40}$' -or
        $Draft.inspectionId -cnotmatch '^inspection-[a-zA-Z0-9-]+$' -or
        [uint32]$Draft.windowsBuild -lt 22000 -or
        [string]::IsNullOrWhiteSpace([string]$Draft.windowsEdition)) {
        throw 'The draft source, inspection, or platform binding is invalid.'
    }
    if (-not (Test-Representation $Draft.originalRepresentation) -or
        -not (Test-Representation $Draft.rollbackRepresentation) -or
        ($Draft.originalRepresentation | ConvertTo-Json -Compress) -cne
            ($Draft.rollbackRepresentation | ConvertTo-Json -Compress) -or
        $Draft.rollbackAuthority -ne 'transaction_bound_exact_pre_state') {
        throw 'Rollback must remain transaction-bound to the exact original representation.'
    }
    Assert-Scopes @($Draft.approvedOperationScopes)
    $scopes = @($Draft.approvedOperationScopes)
    $expectedTarget = if ($Draft.userObservedState -eq 'visible') { 'disabled' } elseif ($Draft.userObservedState -eq 'not_visible') { 'enabled' } else { $null }
    if ($scopes.Count -ne 1 -or $scopes[0].operationId -ne $widgetsOperation -or
        @($scopes[0].allowedTargetStates).Count -ne 1 -or
        $scopes[0].allowedTargetStates[0] -ne $expectedTarget) {
        throw 'This bounded draft must contain Widgets and only the opposite user-confirmed target.'
    }
    if ([uint32]$Draft.maximumPlans -ne 1 -or [uint32]$Draft.maximumExecutions -ne 1) {
        throw 'This bounded draft must allow exactly one plan and one execution.'
    }
    $detectorFields = @('effectiveState', 'authority', 'policyState', 'conflict', 'evidenceTimestampEpochMs')
    $actualDetectorFields = @($Draft.detectorState.PSObject.Properties.Name)
    if (@($detectorFields | Where-Object { $actualDetectorFields -notcontains $_ }).Count -ne 0 -or
        @($actualDetectorFields | Where-Object { $detectorFields -notcontains $_ }).Count -ne 0 -or
        $Draft.detectorState.conflict -ne $false -or
        [long]$Draft.detectorState.evidenceTimestampEpochMs -lt 1) {
        throw 'The Widgets detector state is incomplete or conflicting.'
    }
    $created = [DateTimeOffset]::Parse([string]$Draft.createdAtUtc).ToUnixTimeMilliseconds()
    $expires = [long]$Draft.expiresAtEpochMs
    if ($created -gt $NowEpochMs -or $expires -le $NowEpochMs -or $expires -gt ($created + 1800000)) {
        throw 'The approval-review draft is expired, future-dated, or valid for longer than 30 minutes.'
    }
}

if ($SelfTest) {
    $now = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $valid = [pscustomobject][ordered]@{
        schemaVersion = 2; recordKind = 'approval_review_draft'; approvalStatus = 'ready_for_approval_review'
        mutationAllowed = $false; executed = $false; scenarioId = 'widgets-review'; targetType = 'physical_laptop'
        hashedMachineIdentity = ('a' * 64); windowsEdition = 'Professional'; windowsBuild = 26200; windowsUbr = 1
        sourceCheckpointCommit = ('c' * 40); inspectionId = 'inspection-1'; inspectionEvidenceSha256 = ('d' * 64)
        developmentHostDenylistIdentity = ('b' * 64)
        approvedOperationScopes = @([pscustomobject][ordered]@{ operationId = $widgetsOperation; allowedTargetStates = @('enabled'); handlerVersion = 'mutation-alpha.1' })
        maximumPlans = 1; maximumExecutions = 1; originalRepresentation = [pscustomobject]@{ kind = 'missing' }
        detectorState = [pscustomobject]@{ effectiveState = 'unknown'; authority = 'unknown'; policyState = 'not_configured'; conflict = $false; evidenceTimestampEpochMs = $now }
        userObservedState = 'not_visible'; rollbackRepresentation = [pscustomobject]@{ kind = 'missing' }
        rollbackAuthority = 'transaction_bound_exact_pre_state'; createdAtUtc = [DateTimeOffset]::FromUnixTimeMilliseconds($now - 1000).ToString('o')
        expiresAtEpochMs = $now + 1000; automaticRepair = $false; elevation = $false
        explorerTermination = $false; restart = $false; genericRegistryPath = $false
    }
    Assert-DraftShape $valid $now
    $rejections = 0
    foreach ($change in @(
        @{ mutationAllowed = $true },
        @{ approvedOperationScopes = @() },
        @{ approvedOperationScopes = @([pscustomobject][ordered]@{ operationId = $widgetsOperation; allowedTargetStates = @('enabled', 'enabled'); handlerVersion = 'mutation-alpha.1' }) },
        @{ approvedOperationScopes = @([pscustomobject][ordered]@{ operationId = 'set_taskbar_task_view_visibility'; allowedTargetStates = @('enabled'); handlerVersion = 'mutation-alpha.1' }) },
        @{ maximumPlans = 2 },
        @{ expiresAtEpochMs = $now + 1800001 }
    )) {
        $candidate = $valid | ConvertTo-Json -Depth 8 | ConvertFrom-Json
        foreach ($entry in $change.GetEnumerator()) { $candidate.($entry.Key) = $entry.Value }
        try { Assert-DraftShape $candidate $now } catch { $rejections++ }
    }
    if ($rejections -ne 6) { throw "Approval-review self-test rejected $rejections of 6 invalid drafts." }
    Write-Host 'Scoped approval-review validator self-test passed (valid=1, rejected=6).'
    return
}

if (-not (Test-Path -LiteralPath $DraftPath)) { throw "Approval-review draft is missing: $DraftPath" }
$draft = Get-Content -Raw -LiteralPath $DraftPath | ConvertFrom-Json
Assert-DraftShape $draft ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())

$repository = Resolve-Path (Join-Path $PSScriptRoot '..')
$currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$fingerprintInput = "$($env:COMPUTERNAME.ToUpperInvariant()):$($currentVersion.EditionID):$($currentVersion.CurrentBuild)"
$sha = [Security.Cryptography.SHA256]::Create()
try {
    $fingerprint = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($fingerprintInput))).Replace('-', '').ToLowerInvariant()
} finally { $sha.Dispose() }
$denylist = Get-Content -Raw (Join-Path $repository '.deslopper\local\development-host-denylist.json') | ConvertFrom-Json
$denylistHashes = @($denylist.developmentHostFingerprints)
if ($draft.hashedMachineIdentity -ne $fingerprint -or
    $draft.developmentHostDenylistIdentity -notin $denylistHashes -or
    $draft.developmentHostDenylistIdentity -eq $fingerprint -or
    $draft.windowsEdition -ne [string]$currentVersion.EditionID -or
    [uint32]$draft.windowsBuild -ne [uint32]$currentVersion.CurrentBuild -or
    [uint32]$draft.windowsUbr -ne [uint32]$currentVersion.UBR) {
    throw 'The approval-review draft does not match this target or its denylist.'
}
$head = (& git -C $repository rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $draft.sourceCheckpointCommit -ne $head) {
    throw 'The draft source checkpoint does not equal the current repository commit.'
}
Write-Host 'Scoped approval-review draft is valid, current, target-bound, Widgets-enabled-only, and non-authorizing.'
