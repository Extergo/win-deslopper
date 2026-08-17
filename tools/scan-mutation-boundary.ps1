param()

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$files = Get-ChildItem -Path (Join-Path $repo 'src'), (Join-Path $repo 'frontend\src') -Recurse -File -Include *.rs,*.ts,*.svelte
$toolFiles = Get-ChildItem -Path (Join-Path $repo 'tools') -File -Filter '*.ps1' |
    Where-Object { $_.Name -ne 'scan-mutation-boundary.ps1' }
$failures = [System.Collections.Generic.List[string]]::new()
$sharedOwnerLockPath = Join-Path $repo 'src\mutation\process_lock.rs'
$sharedOwnerLockTestLine = (Select-String -LiteralPath $sharedOwnerLockPath -Pattern '#[cfg(test)]' -SimpleMatch).LineNumber

$prohibited = @(
    'Remove-AppxPackage',
    'Remove-AppxProvisionedPackage',
    'Add-AppxPackage',
    'Add-AppxProvisionedPackage',
    'dism.exe',
    'Enable-WindowsOptionalFeature',
    'Disable-WindowsOptionalFeature',
    'Set-Service',
    'Stop-Service',
    'Start-Service',
    'Register-ScheduledTask',
    'Unregister-ScheduledTask',
    'Remove-Item',
    'OneDrive.exe /unlink',
    'taskkill.exe',
    'explorer.exe',
    'shutdown.exe',
    'Restart-Computer',
    'runas.exe',
    'Verb RunAs',
    'cmd.exe',
    'RemoveForAllUsers',
    'RemovePackageWithOptionsAsync',
    'DeprovisionPackageForAllUsersAsync',
    'takeown.exe',
    'icacls.exe'
)

foreach ($match in ($files | Select-String -Pattern $prohibited -SimpleMatch)) {
    $failures.Add("$($match.Path):$($match.LineNumber): prohibited token '$($match.Pattern)'")
}

foreach ($match in ($files | Select-String -Pattern 'Command::new' -SimpleMatch)) {
    $isSharedLockProcessTest = $match.Path -eq $sharedOwnerLockPath -and
        $sharedOwnerLockTestLine -and $match.LineNumber -gt $sharedOwnerLockTestLine
    if ($match.Path -notlike '*\src\inspection.rs' -and -not $isSharedLockProcessTest) {
        $failures.Add("$($match.Path):$($match.LineNumber): process creation outside read-only inspection")
    }
}

foreach ($match in ($files | Select-String -Pattern '.set_value(', '.delete_value(' -SimpleMatch)) {
    if ($match.Path -notlike '*\src\mutation\handlers\windows_store.rs') {
        $failures.Add("$($match.Path):$($match.LineNumber): registry write outside the fixed mutation store")
    }
}

foreach ($match in ($files | Select-String -Pattern 'create_subkey', 'KEY_ALL_ACCESS' -SimpleMatch)) {
    $failures.Add("$($match.Path):$($match.LineNumber): forbidden broad or machine registry capability")
}

$fixedStorePath = Join-Path $repo 'src\mutation\handlers\windows_store.rs'
foreach ($match in ($files | Select-String -Pattern 'HKEY_LOCAL_MACHINE' -SimpleMatch)) {
    if ($match.Path -ne $fixedStorePath) {
        $failures.Add("$($match.Path):$($match.LineNumber): machine registry access outside fixed read-only policy checks")
    }
}

$fixedStore = Get-Content -Raw -LiteralPath $fixedStorePath
$fixedCleanupHandlerPath = Join-Path $repo 'src\mutation\handlers\current_user_cleanup.rs'
$fixedBoundarySources = $fixedStore + (Get-Content -Raw -LiteralPath $fixedCleanupHandlerPath)
$requiredFixedTokens = @(
    'Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced',
    'Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager',
    'Software\Policies\Microsoft\Windows\CloudContent',
    'TaskbarDa',
    'ShowTaskViewButton',
    'TaskbarSd',
    'SubscribedContent-310093Enabled',
    'SoftLandingEnabled',
    'SubscribedContent-338389Enabled',
    'SubscribedContent-338393Enabled',
    'DisableWindowsSpotlightWindowsWelcomeExperience',
    'DisableSoftLanding',
    'DisableWindowsSpotlightOnActionCenter',
    'DisableWindowsSpotlightOnSettings',
    'Software\Policies\Microsoft\Dsh',
    'AllowNewsAndInterests',
    'Software\Policies\Microsoft\Windows\Explorer',
    'HideTaskViewButton',
    'Software\Microsoft\Windows\CurrentVersion\Policies\Explorer',
    'NoSetTaskbar'
)
foreach ($token in $requiredFixedTokens) {
    if (-not $fixedBoundarySources.Contains($token)) {
        $failures.Add("fixed mutation sources: required closed registry token '$token' is missing")
    }
}
if ([regex]::Matches($fixedStore, '\.set_value\(').Count -ne 1 -or [regex]::Matches($fixedStore, '\.delete_value\(').Count -ne 1) {
    $failures.Add("${fixedStorePath}: fixed store must contain exactly one bounded set and one exact-absence restore call")
}

$cargo = Get-Content -Raw -LiteralPath (Join-Path $repo 'Cargo.toml')
if ($cargo -notmatch 'default\s*=\s*\["owner-mode"\]') {
    $failures.Add('Cargo.toml: normal production composition must default to owner-mode')
}

$appSource = Get-Content -Raw -LiteralPath (Join-Path $repo 'src\app.rs')
$ownerStart = $appSource.IndexOf('#[cfg(all(feature = "owner-mode", not(feature = "mutation-alpha")))]')
$alphaStart = $appSource.IndexOf('#[cfg(feature = "mutation-alpha")]', $ownerStart + 1)
if ($ownerStart -lt 0 -or $alphaStart -le $ownerStart) {
    $failures.Add('src\app.rs: distinct normal owner-mode composition is missing')
} else {
    $ownerComposition = $appSource.Substring($ownerStart, $alphaStart - $ownerStart)
    foreach ($command in 'get_owner_actionability', 'get_widgets_actionability', 'get_task_view_actionability', 'apply_owner_operation', 'undo_owner_operation', 'get_owner_change_history', 'get_owner_package_actionability', 'remove_owner_package', 'restore_owner_package', 'get_owner_package_history') {
        if (-not $ownerComposition.Contains($command)) {
            $failures.Add("src\app.rs: normal owner-mode composition is missing '$command'")
        }
    }
    foreach ($command in 'generate_mutation_plan', 'approve_and_execute_mutation', 'rollback_mutation', 'get_mutation_operation_options') {
        if ($ownerComposition.Contains($command)) {
            $failures.Add("src\app.rs: engineering command '$command' leaked into normal owner-mode composition")
        }
    }
}

$frontendBackend = Get-Content -Raw -LiteralPath (Join-Path $repo 'frontend\src\lib\backend.ts')
foreach ($operation in 'set_taskbar_widgets_visibility', 'set_taskbar_task_view_visibility', 'set_welcome_experience_enabled', 'set_tips_suggestions_enabled', 'set_notification_suggestions_enabled', 'set_settings_suggested_content_enabled') {
    if (-not $frontendBackend.Contains("| '$operation'")) {
        $failures.Add("frontend\src\lib\backend.ts: closed Owner Mode operation '$operation' is missing")
    }
}
$packageSourcePath = Join-Path $repo 'src\mutation\package.rs'
$packageSource = Get-Content -Raw -LiteralPath $packageSourcePath
$brokerSource = Get-Content -Raw -LiteralPath (Join-Path $repo 'src\mutation\broker.rs')
$ownerLockSourcePath = Join-Path $repo 'src\mutation\process_lock.rs'
$ownerLockSource = Get-Content -Raw -LiteralPath $ownerLockSourcePath
foreach ($token in 'FindPackagesByUserSecurityId', 'RemovePackageAsync', 'RegisterPackageByFullNameAsync', 'Windows.Management.Deployment.PackageManager/', 'owner-appx.4', 'Microsoft.Copilot', 'Microsoft.YourPhone', 'Clipchamp.Clipchamp', 'Microsoft.MicrosoftSolitaireCollection') {
    if (-not $packageSource.Contains($token)) {
        $failures.Add("${packageSourcePath}: required bounded deployment token '$token' is missing")
    }
}
if ($packageSource.Contains('.FindPackages()')) {
    $failures.Add("${packageSourcePath}: parameterless cross-user package inventory is forbidden")
}
if (-not $packageSource.Contains('process_lock::OwnerMutationProcessLock') -or -not $brokerSource.Contains('process_lock::OwnerMutationProcessLock')) {
    $failures.Add('Owner setting and package brokers must use the same cross-process lock type')
}
foreach ($token in 'share_mode(0)', 'Another Deslopper change is still in progress.') {
    if (-not $ownerLockSource.Contains($token)) {
        $failures.Add("${ownerLockSourcePath}: required shared-lock token '$token' is missing")
    }
}
foreach ($token in 'PackageProcessLock', 'create_new(true)') {
    if ($packageSource.Contains($token)) {
        $failures.Add("${packageSourcePath}: stale package-specific lock behavior '$token' returned")
    }
}
foreach ($operation in 'remove_consumer_copilot_current_user', 'remove_phone_link_current_user', 'remove_clipchamp_current_user', 'remove_solitaire_current_user') {
    if (-not $frontendBackend.Contains("| '$operation'") -and -not $frontendBackend.Contains("= '$operation'")) {
        $failures.Add("frontend\src\lib\backend.ts: closed package operation '$operation' is missing")
    }
}
$removeRequestStart = $packageSource.IndexOf('pub struct OwnerPackageRemoveRequest')
$restoreRequestStart = $packageSource.IndexOf('pub struct OwnerPackageRestoreRequest')
if ($removeRequestStart -lt 0 -or $restoreRequestStart -le $removeRequestStart) {
    $failures.Add("${packageSourcePath}: closed package removal request is missing")
} else {
    $removeRequest = $packageSource.Substring($removeRequestStart, $restoreRequestStart - $removeRequestStart)
    foreach ($token in 'package_name', 'package_full_name', 'script', 'command', 'all_users', 'provisioned') {
        if ($removeRequest.Contains($token)) {
            $failures.Add("${packageSourcePath}: frontend removal request exposes forbidden field '$token'")
        }
    }
}
$productPage = Get-Content -Raw -LiteralPath (Join-Path $repo 'frontend\src\routes\+page.svelte')
foreach ($operation in 'set_taskbar_show_desktop_enabled') {
    if ($productPage.Contains($operation)) {
        $failures.Add("frontend\src\routes\+page.svelte: non-product operation '$operation' is exposed")
    }
}
foreach ($token in 'TaskbarDa', 'ShowTaskViewButton', 'SubscribedContent', 'SoftLandingEnabled', 'CurrentVersion\Explorer\Advanced', 'ContentDeliveryManager') {
    if ($productPage.Contains($token)) {
        $failures.Add("frontend\src\routes\+page.svelte: raw registry token '$token' leaked into product input/UI")
    }
}

$prohibitedTooling = @(
    'Invoke-Expression',
    'Invoke-Command',
    'Start-Process',
    'New-VM',
    'Remove-VM',
    'Start-VM',
    'Stop-VM',
    'powershell.exe',
    'cmd.exe',
    'reg.exe'
)
foreach ($match in ($toolFiles | Select-String -Pattern $prohibitedTooling -SimpleMatch)) {
    $failures.Add("$($match.Path):$($match.LineNumber): prohibited generic or lifecycle VM tooling '$($match.Pattern)'")
}

foreach ($command in 'Checkpoint-VM', 'Restore-VMSnapshot', 'Copy-VMFile') {
    foreach ($match in ($toolFiles | Select-String -Pattern $command -SimpleMatch)) {
        if ($match.Path -notlike '*\tools\Invoke-DeslopperHyperVValidation.ps1') {
            $failures.Add("$($match.Path):$($match.LineNumber): Hyper-V state command outside the closed validation tool")
        }
    }
}

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Host 'Mutation boundary scan passed.'
Write-Host 'Normal product writes: six fixed HKCU values for Widgets, Task View, and four current-user cleanup operations only.'
Write-Host 'Normal package deployment: four exact current-user PackageManager operations; no all-user or provisioning removal.'
Write-Host 'Engineering harness only: the closed TaskbarSd handler remains compiled/testable but is not registered by normal launch.'
Write-Host 'Allowed machine reads: fixed Widgets, Task View, taskbar-lock, and CloudContent policy checks only.'
Write-Host 'Normal launch and Scan reach no write call; registry writes remain isolated behind explicit owner commands.'
Write-Host 'Allowed process boundary: fixed read-only inspection PowerShell only; package deployment is native WinRT.'
Write-Host 'Allowed VM tooling: explicit approved-VM checkpoint/restore/payload copy in Invoke-DeslopperHyperVValidation.ps1 only; it cannot launch mutation.'
