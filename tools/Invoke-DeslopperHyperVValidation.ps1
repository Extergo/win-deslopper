param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Inventory', 'ConfirmCheckpoint', 'CreateCheckpoint', 'RestoreCheckpoint', 'CopyPayload', 'CopyManifest', 'CollectResults')]
    [string]$Action,

    [ValidatePattern('^[a-z0-9-]+$')]
    [string]$ScenarioId,

    [switch]$ConfirmVmStateChange
)

$ErrorActionPreference = 'Stop'
$repository = Resolve-Path (Join-Path $PSScriptRoot '..')
$approvalPath = Join-Path $repository '.deslopper\local\approved-validation-vms.json'
if (-not (Get-Command Get-VM -ErrorAction SilentlyContinue)) {
    throw 'Hyper-V management tooling is not installed or available in this session.'
}
if (-not (Test-Path -LiteralPath $approvalPath)) {
    throw 'Create the ignored .deslopper/local/approved-validation-vms.json inventory first.'
}
$approval = Get-Content -Raw $approvalPath | ConvertFrom-Json
if ([uint32]$approval.schemaVersion -ne 2) {
    throw 'The approved VM inventory must use schemaVersion 2.'
}
$approved = @($approval.vms)
$currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$fingerprintInput = "$($env:COMPUTERNAME.ToUpperInvariant()):$($currentVersion.EditionID):$($currentVersion.CurrentBuild)"
$sha = [Security.Cryptography.SHA256]::Create()
try {
    $developmentHostFingerprint = [BitConverter]::ToString(
        $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($fingerprintInput))
    ).Replace('-', '').ToLowerInvariant()
} finally {
    $sha.Dispose()
}

if ($Action -eq 'Inventory') {
    $approved | ForEach-Object {
        $vm = Get-VM -Name $_.vmName -ErrorAction Stop
        [pscustomobject]@{
            ScenarioId = $_.scenarioId
            VmName = $vm.Name
            State = $vm.State
            Generation = $vm.Generation
            ExpectedGeneration = $_.generation
            ApprovedCheckpoint = $_.checkpointName
            Disposable = $_.disposable -eq $true
            ApprovedOperations = @($_.approvedOperations) -join ', '
            EvidenceExportLocation = $_.evidenceExportLocation
            ExpectedMachineIsHost = [string]$_.expectedMachineId -eq $developmentHostFingerprint
            HostNameCollision = $vm.Name -eq $env:COMPUTERNAME
        }
    }
    return
}

$entry = $approved | Where-Object { $_.scenarioId -eq $ScenarioId }
if (-not $entry -or @($entry).Count -ne 1) {
    throw 'The scenario must map to exactly one explicitly approved VM.'
}
if ($entry.disposable -ne $true) {
    throw 'The approved scenario must explicitly declare the VM disposable.'
}
$requiredOperations = @('set_widgets_visibility', 'set_task_view_visibility', 'set_show_desktop_corner')
$operationList = @($entry.approvedOperations)
$missingOperation = $requiredOperations | Where-Object { $operationList -notcontains $_ }
$missingTargetState = $requiredOperations | Where-Object {
    $states = @($entry.approvedTargetStates.PSObject.Properties[$_].Value)
    $states.Count -ne 2 -or $states -notcontains 'enabled' -or $states -notcontains 'disabled'
}
if ([string]$entry.expectedMachineId -notmatch '^[a-f0-9]{64}$' -or
    [string]::IsNullOrWhiteSpace([string]$entry.expectedEdition) -or
    [string]$entry.expectedBuildFamily -notin @('24H2', '25H2') -or
    [string]$entry.accountClass -notin @('local-administrator', 'microsoft-account-administrator', 'domain-administrator', 'mdm-administrator', 'standard-user') -or
    $entry.domainJoined -isnot [bool] -or
    $entry.mdmEnrolled -isnot [bool] -or
    $operationList.Count -ne 3 -or
    @($missingOperation).Count -ne 0 -or
    @($missingTargetState).Count -ne 0 -or
    [string]::IsNullOrWhiteSpace([string]$entry.evidenceExportLocation) -or
    [string]::IsNullOrWhiteSpace([string]$entry.checkpointRestoreProcedure)) {
    throw 'The approved scenario is missing required identity, approval, evidence, or restore metadata.'
}
if ([string]$entry.expectedMachineId -eq $developmentHostFingerprint) {
    throw 'Refusing an approved VM entry whose expected machine identity is the development host.'
}
$vm = Get-VM -Name $entry.vmName -ErrorAction Stop
if ($vm.Name -eq $env:COMPUTERNAME) {
    throw 'Refusing to target the host identity.'
}
if ([uint32]$vm.Generation -ne [uint32]$entry.generation) {
    throw 'VM generation does not match the approved inventory.'
}

switch ($Action) {
    'ConfirmCheckpoint' {
        $snapshot = Get-VMSnapshot -VMName $vm.Name -Name $entry.checkpointName -ErrorAction Stop
        [pscustomobject]@{ ScenarioId = $ScenarioId; VmName = $vm.Name; Checkpoint = $snapshot.Name; Created = $snapshot.CreationTime }
    }
    'CreateCheckpoint' {
        if (-not $ConfirmVmStateChange) { throw 'CreateCheckpoint requires -ConfirmVmStateChange.' }
        Checkpoint-VM -Name $vm.Name -SnapshotName $entry.checkpointName -ErrorAction Stop
    }
    'RestoreCheckpoint' {
        if (-not $ConfirmVmStateChange) { throw 'RestoreCheckpoint requires -ConfirmVmStateChange.' }
        $snapshot = Get-VMSnapshot -VMName $vm.Name -Name $entry.checkpointName -ErrorAction Stop
        Restore-VMSnapshot -VMSnapshot $snapshot -Confirm:$false -ErrorAction Stop
    }
    'CopyPayload' {
        if (-not $ConfirmVmStateChange) { throw 'CopyPayload requires -ConfirmVmStateChange.' }
        $binary = Resolve-Path (Join-Path $repository 'target\debug\win-deslopper.exe')
        $denylist = Resolve-Path (Join-Path $repository '.deslopper\local\development-host-denylist.json')
        $manifestTool = Resolve-Path (Join-Path $repository 'tools\New-DeslopperLiveScenarioManifest.ps1')
        $matrix = Resolve-Path (Join-Path $repository 'validation\matrix.json')
        Copy-VMFile -Name $vm.Name -SourcePath $binary -DestinationPath 'C:\DeslopperValidation\win-deslopper.exe' -FileSource Host -CreateFullPath -Force
        Copy-VMFile -Name $vm.Name -SourcePath $denylist -DestinationPath 'C:\DeslopperValidation\.deslopper\local\development-host-denylist.json' -FileSource Host -CreateFullPath -Force
        Copy-VMFile -Name $vm.Name -SourcePath $manifestTool -DestinationPath 'C:\DeslopperValidation\tools\New-DeslopperLiveScenarioManifest.ps1' -FileSource Host -CreateFullPath -Force
        Copy-VMFile -Name $vm.Name -SourcePath $matrix -DestinationPath 'C:\DeslopperValidation\validation\matrix.json' -FileSource Host -CreateFullPath -Force
    }
    'CopyManifest' {
        if (-not $ConfirmVmStateChange) { throw 'CopyManifest requires -ConfirmVmStateChange.' }
        $manifest = Resolve-Path (Join-Path $repository '.deslopper\local\live-validation-scenario.json')
        Copy-VMFile -Name $vm.Name -SourcePath $manifest -DestinationPath 'C:\DeslopperValidation\.deslopper\local\live-validation-scenario.json' -FileSource Host -CreateFullPath -Force
    }
    'CollectResults' {
        $inbox = Join-Path $repository "validation\inbox\$ScenarioId"
        if (-not (Test-Path -LiteralPath $inbox)) {
            throw 'No manually transferred redacted evidence exists in the fixed scenario inbox.'
        }
        Get-ChildItem -LiteralPath $inbox -Filter '*.json' | Select-Object Name,Length,LastWriteTime
    }
}
