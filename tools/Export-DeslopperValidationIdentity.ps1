param([switch]$ParserSelfTest)

$ErrorActionPreference = 'Stop'

function Get-ValidationIdentityHash {
    param(
        [Parameter(Mandatory = $true)][string]$ComputerName,
        [Parameter(Mandatory = $true)][string]$Edition,
        [Parameter(Mandatory = $true)][uint32]$Build
    )
    $inputText = "$($ComputerName.ToUpperInvariant())`:$Edition`:$Build"
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        [BitConverter]::ToString(
            $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($inputText))
        ).Replace('-', '').ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
}

if ($ParserSelfTest) {
    $first = Get-ValidationIdentityHash -ComputerName 'HOST-A' -Edition 'Professional' -Build 26200
    $second = Get-ValidationIdentityHash -ComputerName 'host-a' -Edition 'Professional' -Build 26200
    if ($first -ne $second -or $first -notmatch '^[a-f0-9]{64}$') {
        throw 'Validation identity hashing is not deterministic and closed.'
    }
    [pscustomobject]@{ passed = 2; failed = 0; identifyingSourcePersisted = $false }
    return
}

$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$localDirectory = Join-Path $repository '.deslopper\local'
$outputPath = Join-Path $localDirectory 'validation-identity-export.json'
if (Test-Path -LiteralPath $outputPath) {
    throw 'The fixed ignored identity export already exists. Review or move it before exporting again.'
}

$currentVersion = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion' -ErrorAction Stop
$fingerprint = Get-ValidationIdentityHash `
    -ComputerName $env:COMPUTERNAME `
    -Edition ([string]$currentVersion.EditionID) `
    -Build ([uint32]$currentVersion.CurrentBuild)
$record = [ordered]@{
    schemaVersion = 1
    developmentHostFingerprints = @($fingerprint)
}

[IO.Directory]::CreateDirectory($localDirectory) | Out-Null
[IO.File]::WriteAllText(
    $outputPath,
    ($record | ConvertTo-Json -Depth 3),
    [Text.UTF8Encoding]::new($false)
)

[pscustomobject]@{
    outputPath = $outputPath
    exportedFields = @('schemaVersion', 'developmentHostFingerprints')
    identifyingSourcePersisted = $false
}
