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
$preparationPath = Join-Path $localDirectory 'physical-laptop-validation-draft.json'
$recoveryAuditPath = Join-Path $localDirectory 'execution-recovery-audit.json'

if (Test-Path -LiteralPath $approvalPath) {
    throw 'Refusing to create a review draft while a live approval manifest exists.'
}
if (-not (Test-Path -LiteralPath $denylistPath)) {
    throw 'The ignored development-host denylist is required.'
}
if (-not (Test-Path -LiteralPath $preparationPath) -or
    -not (Test-Path -LiteralPath $recoveryAuditPath)) {
    throw 'The ignored physical preparation record and fresh recovery audit are required.'
}

$policyText = Get-Content -Raw -LiteralPath (Join-Path $repository '.deslopper\policy.toml')
if ($policyText -notmatch '(?m)^\s*live_validation_policy_schema_version\s*=\s*2\s*$' -or
    $policyText -notmatch '(?m)^\s*allow_live_mutation_only_in_explicitly_approved_disposable_target\s*=\s*true\s*$' -or
    $policyText -notmatch '(?m)^\s*allowed_live_validation_target_types\s*=\s*\[[^\]]*"physical_laptop"[^\]]*\]\s*$' -or
    $policyText -notmatch '(?m)^\s*physical_target_requires_strict_recovery_readiness\s*=\s*true\s*$') {
    throw 'Committed policy schema 2 must explicitly allow strict physical-target review.'
}

$denylist = Get-Content -Raw -LiteralPath $denylistPath | ConvertFrom-Json
$developmentHostHashes = @($denylist.developmentHostFingerprints)
$denylistFields = @($denylist.PSObject.Properties.Name)
if (@($denylistFields | Where-Object { $_ -notin @('schemaVersion', 'developmentHostFingerprints') }).Count -ne 0 -or
    [uint32]$denylist.schemaVersion -ne 1 -or
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

$preparation = Get-Content -Raw -LiteralPath $preparationPath | ConvertFrom-Json
$confirmations = $preparation.preparationStatus.userConfirmations
$recoverySummary = $preparation.preparationStatus.recoveryAuditSummary
if ($preparation.recordKind -ne 'preparation' -or
    $preparation.targetType -ne 'physical_laptop' -or
    $preparation.hashedMachineIdentity -ne $targetFingerprint -or
    $preparation.approvalStatus -ne 'pending' -or
    $preparation.expendableConfirmed -ne $true -or
    $preparation.importantDataConfirmed -ne $true -or
    $preparation.reinstallationAccepted -ne $true -or
    $preparation.winreVerified -ne $true -or
    $preparation.bitlockerRecoveryState -ne 'not_applicable_unencrypted' -or
    $preparation.recoveryMediaState -ne 'available' -or
    $preparation.developmentHostProtectionState -ne 'present_distinct' -or
    $confirmations.importantPersonalData -ne 'no' -or
    $confirmations.allImportantDataBackedUp -ne 'not_applicable' -or
    $confirmations.recoveryMediaAvailable -ne $true -or
    $confirmations.alternateComputerAvailable -ne $true -or
    $confirmations.externalDriveAvailable -ne $true -or
    $recoverySummary.recoveryRouteAvailable -ne $true) {
    throw 'The ignored physical preparation evidence is incomplete or no longer target-bound.'
}

$recoveryAudit = Get-Content -Raw -LiteralPath $recoveryAuditPath | ConvertFrom-Json
$auditAge = [DateTimeOffset]::UtcNow - [DateTimeOffset]::Parse([string]$recoveryAudit.auditedAtUtc)
if ($auditAge.TotalMinutes -lt 0 -or $auditAge.TotalHours -gt 24 -or
    $recoveryAudit.auditType -ne 'elevated_read_only_recovery' -or
    [string]$recoveryAudit.windows.edition -ne [string]$currentVersion.EditionID -or
    [uint32]$recoveryAudit.windows.build -ne [uint32]$currentVersion.CurrentBuild -or
    [uint32]$recoveryAudit.windows.ubr -ne [uint32]$currentVersion.UBR -or
    $recoveryAudit.winre.state -ne 'enabled' -or
    $recoveryAudit.winre.querySucceeded -ne $true -or
    $recoveryAudit.recoveryPartitionPresent -ne $true -or
    $recoveryAudit.bitLocker.auditSucceeded -ne $true -or
    $recoveryAudit.bitLocker.volumeStatus -ne 'FullyDecrypted' -or
    [uint32]$recoveryAudit.bitLocker.encryptionPercentage -ne 0 -or
    $recoveryAudit.bitLocker.protectionState -ne 'off' -or
    $recoveryAudit.pendingRestart.componentBasedServicing -ne $false -or
    $recoveryAudit.pendingRestart.windowsUpdate -ne $false -or
    $recoveryAudit.pendingRestart.pendingFileRename -ne $false -or
    $recoveryAudit.pendingRestartReported -ne $false -or
    @($recoveryAudit.limitations).Count -ne 0 -or
    $recoveryAudit.secretMaterialAccessed -ne $false -or
    $recoveryAudit.protectorIdentifiersAccessed -ne $false) {
    throw 'The fresh recovery audit is missing, stale, restart-pending, encrypted, or recovery-unready.'
}

$sessionManager = Get-ItemProperty -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager'
$pendingRenameProperty = $sessionManager.PSObject.Properties['PendingFileRenameOperations']
$pendingRenameEntries = if ($null -eq $pendingRenameProperty) {
    @()
} else {
    @($pendingRenameProperty.Value | Where-Object { -not [string]::IsNullOrWhiteSpace([string]$_) })
}
$activeComputerName = (Get-ItemProperty -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Control\ComputerName\ActiveComputerName').ComputerName
$configuredComputerName = (Get-ItemProperty -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Control\ComputerName\ComputerName').ComputerName
$restartPending =
    (Test-Path -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending') -or
    (Test-Path -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired') -or
    $pendingRenameEntries.Count -gt 0 -or
    $activeComputerName -cne $configuredComputerName -or
    (Test-Path -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Services\Netlogon\JoinDomain') -or
    (Test-Path -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Services\Netlogon\AvoidSpnSet')
$recoveryPartitionPresent = @(
    Get-Partition -ErrorAction Stop | Where-Object {
        $_.Type -eq 'Recovery' -or
        [string]$_.GptType -eq '{de94bba4-06d1-4d40-a16a-bfd50179d6ac}'
    }
).Count -gt 0
if ($restartPending -or -not $recoveryPartitionPresent) {
    throw 'Current restart or recovery-partition state blocks physical approval review.'
}

$computerSystem = Get-CimInstance Win32_ComputerSystem
$domainJoined = [bool]$computerSystem.PartOfDomain
$entraJoined = @(Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Control\CloudDomainJoin\JoinInfo' -ErrorAction SilentlyContinue).Count -gt 0
$workplaceJoined = @(Get-ChildItem 'HKCU:\Software\Microsoft\Windows NT\CurrentVersion\WorkplaceJoin\JoinInfo' -ErrorAction SilentlyContinue).Count -gt 0
$metadata = @(Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\Enrollments' -ErrorAction SilentlyContinue | ForEach-Object {
        $properties = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
        if ($properties.ProviderID) {
            [pscustomobject]@{ Id = $_.PSChildName; Properties = $properties }
        }
    })
$activeMdm = @($metadata | Where-Object {
        $id = $_.Id
        $properties = $_.Properties
        (Test-Path "HKLM:\SOFTWARE\Microsoft\Provisioning\OMADM\Accounts\$id") -or
        (Test-Path "$env:windir\System32\Tasks\Microsoft\Windows\EnterpriseMgmt\$id") -or
        -not [string]::IsNullOrWhiteSpace([string]$properties.DMPCertThumbPrint)
    }).Count -gt 0
if ($domainJoined -or $entraJoined -or $workplaceJoined -or $activeMdm) {
    throw 'Active domain, Entra, workplace, or MDM management blocks physical approval review.'
}

$head = (& git -C $repository rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $SourceCheckpointCommit -ne $head) {
    throw 'The source checkpoint must equal the current repository commit.'
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
$taskbarSettingsBlocked = $false
foreach ($taskbarPolicyPath in @(
        'HKCU:\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer',
        'HKLM:\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer'
    )) {
    if (Test-Path -LiteralPath $taskbarPolicyPath) {
        $taskbarPolicyKey = Get-Item -LiteralPath $taskbarPolicyPath
        if ($taskbarPolicyKey.GetValueNames() -contains 'NoSetTaskbar' -and
            [int]$taskbarPolicyKey.GetValue('NoSetTaskbar', 0, 'DoNotExpandEnvironmentNames') -eq 1) {
            $taskbarSettingsBlocked = $true
        }
    }
}
if ($taskbarSettingsBlocked) {
    throw 'An enabled taskbar-settings policy blocks approval-review preparation.'
}

$proposedTarget = if ($UserObservedState -eq 'visible') { 'disabled' } else { 'enabled' }
$created = [DateTimeOffset]::UtcNow
$draft = [ordered]@{
    schemaVersion = 3
    policySchemaVersion = 2
    recordKind = 'approval_review_draft'
    approvalStatus = 'ready_for_approval_review'
    authorizing = $false
    executionAuthorized = $false
    mutationAllowed = $false
    executed = $false
    scenarioId = 'physical-laptop-widgets-visibility-review'
    targetType = 'physical_laptop'
    hashedMachineIdentity = $targetFingerprint
    windowsEdition = [string]$currentVersion.EditionID
    windowsBuild = [uint32]$currentVersion.CurrentBuild
    windowsUbr = [uint32]$currentVersion.UBR
    managementState = [ordered]@{
        domainJoined = $false
        entraJoined = $false
        workplaceJoined = $false
        mdmEnrolled = $false
    }
    sourceCheckpointCommit = $SourceCheckpointCommit
    inspectionId = $InspectionId
    inspectionEvidenceSha256 = $InspectionEvidenceSha256
    developmentHostDenylistIdentity = $developmentHostHashes[0]
    approvedOperationScopes = @(
        [ordered]@{
            operationId = 'set_taskbar_widgets_visibility'
            allowedTargetStates = @($proposedTarget)
            handlerVersion = 'mutation-alpha.1'
        }
    )
    maximumPlans = 1
    maximumExecutions = 1
    recoveryReadiness = 'ready'
    importantDataConfirmed = $true
    backupConfirmed = $false
    reinstallationAccepted = $true
    winreVerified = $true
    bitlockerRecoveryState = 'not_applicable_unencrypted'
    recoveryMediaState = 'available'
    developmentHostProtectionState = 'present_distinct'
    expendableConfirmed = $true
    importantDataState = 'absent'
    recoveryRouteState = 'recovery_partition_verified'
    alternateRecoveryDeviceAvailable = $true
    restartPendingState = 'clear'
    automaticRepairDisabled = $true
    finalPlanApprovalRequired = $true
    restoreOrReimageProcedure = [string]$preparation.restoreOrReimageProcedure
    finalDisposition = 'reset_before_sale'
    originalRepresentation = $representation
    detectorState = [ordered]@{
        effectiveState = 'unknown'
        authority = 'unknown'
        policyState = 'not_configured'
        conflict = $false
        evidenceTimestampEpochMs = $EvidenceTimestampEpochMs
    }
    userObservedState = $UserObservedState
    rollbackRepresentation = $representation
    rollbackAuthority = 'transaction_bound_exact_pre_state'
    createdAtUtc = $created.ToString("yyyy-MM-dd'T'HH:mm:ss.fff'Z'", [Globalization.CultureInfo]::InvariantCulture)
    expiresAtEpochMs = $created.AddMinutes(30).ToUnixTimeMilliseconds()
    elevation = $false
    explorerTermination = $false
    restart = $false
    genericRegistryPath = $false
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
    Authorizing = $draft.authorizing
    ExecutionAuthorized = $draft.executionAuthorized
    MutationAllowed = $draft.mutationAllowed
    Operation = $draft.approvedOperationScopes[0].operationId
    ProposedTarget = $draft.approvedOperationScopes[0].allowedTargetStates[0]
    OriginalRepresentation = $draft.originalRepresentation.kind
    MaximumPlans = $draft.maximumPlans
    MaximumExecutions = $draft.maximumExecutions
    ValidityMinutes = 30
    IdentityDisplayed = $false
}
