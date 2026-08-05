param([switch]$ParserSelfTest)

$ErrorActionPreference = 'Stop'

function Get-ValidationIdentityHash {
    param([string]$ComputerName, [string]$Edition, [uint32]$Build)
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        [BitConverter]::ToString(
            $sha.ComputeHash(
                [Text.Encoding]::UTF8.GetBytes("$($ComputerName.ToUpperInvariant())`:$Edition`:$Build")
            )
        ).Replace('-', '').ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
}

function Test-DenylistRecord {
    param($Record, [string]$CurrentTargetHash)
    $propertyNames = @($Record.PSObject.Properties.Name)
    $allowedProperties = @('schemaVersion', 'developmentHostFingerprints')
    if (@($propertyNames | Where-Object { $_ -notin $allowedProperties }).Count -ne 0) {
        throw 'The denylist contains unsupported properties.'
    }
    $hashes = @($Record.developmentHostFingerprints)
    if ([uint32]$Record.schemaVersion -ne 1 -or $hashes.Count -ne 1) {
        throw 'The denylist must contain exactly one development-host identity.'
    }
    if (@($hashes | Where-Object { $_ -notmatch '^[a-f0-9]{64}$' }).Count -ne 0) {
        throw 'The denylist contains an invalid validation identity hash.'
    }
    if (@($hashes | Select-Object -Unique).Count -ne $hashes.Count) {
        throw 'The denylist contains duplicate validation identity hashes.'
    }
    if ($hashes -contains $CurrentTargetHash) {
        throw 'The development-host identity equals the current validation target.'
    }
    [pscustomobject]@{
        valid = $true
        hashCount = $hashes.Count
        distinctFromCurrentTarget = $true
    }
}

if ($ParserSelfTest) {
    $target = ('b' * 64) -join ''
    $valid = [pscustomobject]@{
        schemaVersion = 1
        developmentHostFingerprints = @((('a' * 64) -join ''))
    }
    [void](Test-DenylistRecord -Record $valid -CurrentTargetHash $target)
    $failed = 0
    foreach ($invalid in @(
        [pscustomobject]@{ schemaVersion = 1; developmentHostFingerprints = @() },
        [pscustomobject]@{ schemaVersion = 1; developmentHostFingerprints = @($target) },
        [pscustomobject]@{ schemaVersion = 1; developmentHostFingerprints = @('invalid') },
        [pscustomobject]@{ schemaVersion = 1; developmentHostFingerprints = @((('a' * 64) -join ''), (('c' * 64) -join '')) }
    )) {
        try { [void](Test-DenylistRecord -Record $invalid -CurrentTargetHash $target) } catch { $failed++ }
    }
    if ($failed -ne 4) { throw 'Denylist rejection self-test did not fail closed.' }
    [pscustomobject]@{ passed = 5; failed = 0; identityHashDisplayed = $false }
    return
}

$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$denylistPath = Join-Path $repository '.deslopper\local\development-host-denylist.json'
if (-not (Test-Path -LiteralPath $denylistPath)) {
    throw 'The fixed ignored development-host denylist is missing.'
}
$currentVersion = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion' -ErrorAction Stop
$targetHash = Get-ValidationIdentityHash `
    -ComputerName $env:COMPUTERNAME `
    -Edition ([string]$currentVersion.EditionID) `
    -Build ([uint32]$currentVersion.CurrentBuild)
$record = Get-Content -Raw -LiteralPath $denylistPath | ConvertFrom-Json
Test-DenylistRecord -Record $record -CurrentTargetHash $targetHash
