param(
    [string]$OutputPath,
    [switch]$ParserSelfTest
)

$ErrorActionPreference = 'Stop'

function ConvertFrom-ReAgentCStatus {
    param([string[]]$Lines)
    $text = $Lines -join "`n"
    if ($text -match '(?im)^\s*Windows RE status\s*:\s*Enabled\s*$') { return 'enabled' }
    if ($text -match '(?im)^\s*Windows RE status\s*:\s*Disabled\s*$') { return 'disabled' }
    return 'unknown'
}

function ConvertFrom-ManageBdeStatus {
    param([string[]]$Lines)
    $text = $Lines -join "`n"
    $percentage = $null
    if ($text -match '(?im)^\s*Percentage Encrypted\s*:\s*([0-9.]+)%\s*$') {
        $percentage = [double]$Matches[1]
    }
    $conversion = if ($text -match '(?im)^\s*Conversion Status\s*:\s*([^\r\n]+)') {
        $Matches[1].Trim()
    } else { 'Unknown' }
    $protection = if ($text -match '(?im)^\s*Protection Status\s*:\s*([^\r\n]+)') {
        $Matches[1].Trim()
    } else { 'Unknown' }
    $method = if ($text -match '(?im)^\s*Encryption Method\s*:\s*([^\r\n]+)') {
        $Matches[1].Trim()
    } else { 'Unknown' }
    $classification = if ($protection -match '^Protection On$') {
        'on'
    } elseif ($protection -match '^Protection Off$' -and $conversion -notmatch 'Fully Decrypted') {
        'suspended'
    } elseif ($protection -match '^Protection Off$') {
        'off'
    } else {
        'unknown'
    }
    [pscustomobject]@{
        volumeStatus = $conversion
        encryptionPercentage = $percentage
        protectionState = $classification
        encryptionMethod = $method
        auditSucceeded = $classification -ne 'unknown'
    }
}

function Assert-Equal {
    param($Actual, $Expected, [string]$Label)
    if ($Actual -ne $Expected) { throw "$Label expected '$Expected', got '$Actual'." }
}

if ($ParserSelfTest) {
    Assert-Equal (ConvertFrom-ReAgentCStatus @('Windows RE status: Enabled')) 'enabled' 'WinRE enabled'
    Assert-Equal (ConvertFrom-ReAgentCStatus @('Windows RE status: Disabled')) 'disabled' 'WinRE disabled'

    $on = ConvertFrom-ManageBdeStatus @(
        'Conversion Status: Fully Encrypted',
        'Percentage Encrypted: 100.0%',
        'Encryption Method: XTS-AES 256',
        'Protection Status: Protection On'
    )
    Assert-Equal $on.protectionState 'on' 'BitLocker on'
    $off = ConvertFrom-ManageBdeStatus @(
        'Conversion Status: Fully Decrypted',
        'Percentage Encrypted: 0.0%',
        'Encryption Method: None',
        'Protection Status: Protection Off'
    )
    Assert-Equal $off.protectionState 'off' 'BitLocker off'
    $suspended = ConvertFrom-ManageBdeStatus @(
        'Conversion Status: Fully Encrypted',
        'Percentage Encrypted: 100.0%',
        'Encryption Method: XTS-AES 128',
        'Protection Status: Protection Off'
    )
    Assert-Equal $suspended.protectionState 'suspended' 'BitLocker suspended'
    $permissionFailure = ConvertFrom-ManageBdeStatus @('ERROR: Access is denied.')
    Assert-Equal $permissionFailure.auditSucceeded $false 'Audit permission failure'

    $unsafeInput = @(
        'Conversion Status: Fully Encrypted',
        'Percentage Encrypted: 100.0%',
        'Encryption Method: XTS-AES 128',
        'Protection Status: Protection On',
        'Numerical Password: 111111-222222-333333-444444-555555-666666-777777-888888'
    )
    $safe = ConvertFrom-ManageBdeStatus $unsafeInput | ConvertTo-Json -Compress
    if ($safe -match '111111|Numerical Password') {
        throw 'Recovery audit parser retained prohibited key material.'
    }
    [pscustomobject]@{ passed = 7; failed = 0; keyMaterialPersisted = $false }
    return
}

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    throw 'OutputPath is required outside parser self-test mode.'
}

$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$localRoot = [IO.Path]::GetFullPath((Join-Path $repository '.deslopper\local'))
$resolvedOutput = [IO.Path]::GetFullPath($OutputPath)
if (-not $resolvedOutput.StartsWith($localRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Recovery audit output must remain under the ignored .deslopper/local directory.'
}

$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'This separate recovery audit requires an ordinary interactive UAC elevation.'
}

$reagentOutput = @(& reagentc.exe /info 2>&1 | ForEach-Object { $_.ToString() })
$reagentExitCode = $LASTEXITCODE
$winreState = ConvertFrom-ReAgentCStatus $reagentOutput

$bitLocker = $null
$bitLockerLimit = $null
try {
    $volume = Get-BitLockerVolume -MountPoint $env:SystemDrive -ErrorAction Stop |
        Select-Object -First 1 -Property MountPoint, VolumeStatus, EncryptionPercentage, ProtectionStatus, EncryptionMethod
    $protection = [string]$volume.ProtectionStatus
    $volumeStatus = [string]$volume.VolumeStatus
    $protectionState = if ($protection -eq 'On') {
        'on'
    } elseif ($protection -eq 'Off' -and $volumeStatus -notmatch 'FullyDecrypted') {
        'suspended'
    } elseif ($protection -eq 'Off') {
        'off'
    } else {
        'unknown'
    }
    $bitLocker = [pscustomobject]@{
        volumeMountPoint = [string]$volume.MountPoint
        volumeStatus = $volumeStatus
        encryptionPercentage = [double]$volume.EncryptionPercentage
        protectionState = $protectionState
        encryptionMethod = [string]$volume.EncryptionMethod
        auditSucceeded = $true
    }
} catch {
    $manageBdeOutput = @(& manage-bde.exe -status $env:SystemDrive 2>&1 | ForEach-Object { $_.ToString() })
    $manageBdeExitCode = $LASTEXITCODE
    $parsed = ConvertFrom-ManageBdeStatus $manageBdeOutput
    $bitLocker = [pscustomobject]@{
        volumeMountPoint = [string]$env:SystemDrive
        volumeStatus = $parsed.volumeStatus
        encryptionPercentage = $parsed.encryptionPercentage
        protectionState = $parsed.protectionState
        encryptionMethod = $parsed.encryptionMethod
        auditSucceeded = $parsed.auditSucceeded -and $manageBdeExitCode -eq 0
    }
    if (-not $bitLocker.auditSucceeded) { $bitLockerLimit = 'bitlocker_status_unavailable' }
}

$recoveryPartitionPresent = $false
try {
    $recoveryPartitionPresent = @(
        Get-Partition -ErrorAction Stop | Where-Object {
            $_.Type -eq 'Recovery' -or
            [string]$_.GptType -eq '{de94bba4-06d1-4d40-a16a-bfd50179d6ac}'
        }
    ).Count -gt 0
} catch {
    $recoveryPartitionLimit = 'recovery_partition_inventory_unavailable'
}

$currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$pending = [ordered]@{
    componentBasedServicing = Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending'
    windowsUpdate = Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired'
    pendingFileRename = $null -ne (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager' -Name PendingFileRenameOperations -ErrorAction SilentlyContinue)
}

$limitations = @($bitLockerLimit, $recoveryPartitionLimit) | Where-Object { $_ }
$record = [ordered]@{
    schemaVersion = 1
    auditType = 'elevated_read_only_recovery'
    auditedAtUtc = [DateTime]::UtcNow.ToString('o')
    windows = [ordered]@{
        edition = [string]$currentVersion.EditionID
        build = [uint32]$currentVersion.CurrentBuild
        ubr = [uint32]$currentVersion.UBR
    }
    winre = [ordered]@{
        state = $winreState
        querySucceeded = $reagentExitCode -eq 0
    }
    bitLocker = $bitLocker
    recoveryPartitionPresent = $recoveryPartitionPresent
    pendingRestart = $pending
    pendingRestartReported = $pending.componentBasedServicing -or $pending.windowsUpdate -or $pending.pendingFileRename
    builtInRecoveryRouteReported = $winreState -eq 'enabled' -or $recoveryPartitionPresent
    limitations = @($limitations)
    secretMaterialAccessed = $false
    protectorIdentifiersAccessed = $false
}

[IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($resolvedOutput)) | Out-Null
[IO.File]::WriteAllText(
    $resolvedOutput,
    ($record | ConvertTo-Json -Depth 8),
    [Text.UTF8Encoding]::new($false)
)
$record
