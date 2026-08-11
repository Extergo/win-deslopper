param()

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$files = Get-ChildItem -Path (Join-Path $repo 'src'), (Join-Path $repo 'frontend\src') -Recurse -File -Include *.rs,*.ts,*.svelte
$toolFiles = Get-ChildItem -Path (Join-Path $repo 'tools') -File -Filter '*.ps1' |
    Where-Object { $_.Name -ne 'scan-mutation-boundary.ps1' }
$failures = [System.Collections.Generic.List[string]]::new()

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
    'cmd.exe'
)

foreach ($match in ($files | Select-String -Pattern $prohibited -SimpleMatch)) {
    $failures.Add("$($match.Path):$($match.LineNumber): prohibited token '$($match.Pattern)'")
}

foreach ($match in ($files | Select-String -Pattern 'Command::new' -SimpleMatch)) {
    if ($match.Path -notlike '*\src\inspection.rs') {
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
$requiredFixedTokens = @(
    'Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced',
    'TaskbarDa',
    'ShowTaskViewButton',
    'TaskbarSd',
    'Software\Policies\Microsoft\Dsh',
    'AllowNewsAndInterests',
    'Software\Policies\Microsoft\Windows\Explorer',
    'HideTaskViewButton',
    'Software\Microsoft\Windows\CurrentVersion\Policies\Explorer',
    'NoSetTaskbar'
)
foreach ($token in $requiredFixedTokens) {
    if (-not $fixedStore.Contains($token)) {
        $failures.Add("${fixedStorePath}: required closed registry token '$token' is missing")
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
    foreach ($command in 'get_widgets_actionability', 'apply_owner_operation', 'undo_owner_operation', 'get_owner_change_history') {
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
if ($frontendBackend -notmatch "applyWidgets:[\s\S]*operationId: 'set_taskbar_widgets_visibility'") {
    $failures.Add('frontend\src\lib\backend.ts: owner Apply must hardcode the closed Widgets operation ID')
}
$productPage = Get-Content -Raw -LiteralPath (Join-Path $repo 'frontend\src\routes\+page.svelte')
foreach ($operation in 'set_taskbar_task_view_visibility', 'set_taskbar_show_desktop_enabled') {
    if ($productPage.Contains($operation)) {
        $failures.Add("frontend\src\routes\+page.svelte: non-product operation '$operation' is exposed")
    }
}
foreach ($token in 'TaskbarDa', 'CurrentVersion\Explorer\Advanced') {
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
Write-Host 'Normal product write: TaskbarDa in the fixed HKCU Widgets operation only.'
Write-Host 'Engineering harness only: closed ShowTaskViewButton and TaskbarSd handlers remain compiled/testable but are not registered by normal launch.'
Write-Host 'Allowed machine reads: fixed Widgets, HideTaskViewButton, and NoSetTaskbar policy checks only.'
Write-Host 'Normal launch and Scan reach no write call; registry writes remain isolated behind explicit owner commands.'
Write-Host 'Allowed process boundary: fixed read-only inspection PowerShell only.'
Write-Host 'Allowed VM tooling: explicit approved-VM checkpoint/restore/payload copy in Invoke-DeslopperHyperVValidation.ps1 only; it cannot launch mutation.'
