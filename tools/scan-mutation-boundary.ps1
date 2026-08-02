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
        $failures.Add("$($match.Path):$($match.LineNumber): registry write outside fixed alpha store")
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
Write-Host 'Allowed writes: TaskbarDa, ShowTaskViewButton, and TaskbarSd in the fixed HKCU taskbar handler only.'
Write-Host 'Allowed machine reads: fixed Widgets, HideTaskViewButton, and NoSetTaskbar policy checks only.'
Write-Host 'Allowed process boundary: fixed read-only inspection PowerShell only.'
Write-Host 'Allowed VM tooling: explicit approved-VM checkpoint/restore/payload copy in Invoke-DeslopperHyperVValidation.ps1 only; it cannot launch mutation.'
