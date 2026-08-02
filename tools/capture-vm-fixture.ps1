param(
    [Parameter(Mandatory = $true)][ValidatePattern('^[a-z0-9-]+$')][string]$ScenarioId,
    [Parameter(Mandatory = $true)][string]$OutputPath
)

$ErrorActionPreference = 'Stop'
$queries = [ordered]@{}
$queryFailures = [ordered]@{}

function Invoke-AllowlistedReadOnlyQuery {
    param([string]$Id, [scriptblock]$Query)
    try {
        $queries[$Id] = & $Query
    } catch {
        $kind = if ($_.Exception.Message -match 'denied|privilege|administrator') { 'permission_denied' } else { 'non_zero_exit' }
        $queryFailures[$Id] = [ordered]@{ kind = $kind; message = 'The allowlisted query failed; sensitive exception detail was omitted.' }
        $queries[$Id] = $null
    }
}

Invoke-AllowlistedReadOnlyQuery 'platform_inventory' {
    $os = Get-CimInstance Win32_OperatingSystem
    $currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
    [ordered]@{
        ProductName = $currentVersion.ProductName
        Edition = $currentVersion.EditionID
        Build = [int]$currentVersion.CurrentBuildNumber
        DisplayVersion = $currentVersion.DisplayVersion
        UBR = $currentVersion.UBR
        Architecture = $os.OSArchitecture
        DeviceName = $null
        DomainJoined = [bool](Get-CimInstance Win32_ComputerSystem).PartOfDomain
        Windows11 = ([int]$currentVersion.CurrentBuildNumber -ge 22000)
    }
}

function Convert-SafeAppxRow {
    param($Package)
    [ordered]@{
        Name = $Package.Name
        PackageFamilyName = $Package.PackageFamilyName
        PackageFullName = $Package.PackageFullName
        Version = [string]$Package.Version
        Architecture = [string]$Package.Architecture
        PublisherId = $Package.PublisherId
        IsFramework = [bool]$Package.IsFramework
        IsResourcePackage = [bool]$Package.IsResourcePackage
        InstallLocation = if ($Package.InstallLocation) { '<present>' } else { $null }
        NonRemovable = [bool]$Package.NonRemovable
        Dependencies = @($Package.Dependencies | ForEach-Object { $_.Name })
    }
}

Invoke-AllowlistedReadOnlyQuery 'appx_current_user' { @(Get-AppxPackage | ForEach-Object { Convert-SafeAppxRow $_ }) }
Invoke-AllowlistedReadOnlyQuery 'appx_all_users' { @(Get-AppxPackage -AllUsers -ErrorAction Stop | ForEach-Object { Convert-SafeAppxRow $_ }) }
Invoke-AllowlistedReadOnlyQuery 'appx_provisioned' {
    @(Get-AppxProvisionedPackage -Online -ErrorAction Stop | ForEach-Object {
        [ordered]@{ DisplayName = $_.DisplayName; PackageName = $_.PackageName; Version = [string]$_.Version; Architecture = [string]$_.Architecture; PublisherId = $_.PublisherId }
    })
}

Invoke-AllowlistedReadOnlyQuery 'management_context' {
    $enrollments = @(Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\Enrollments' -ErrorAction SilentlyContinue | Where-Object { (Get-ItemProperty $_.PSPath).ProviderID })
    $providers = @(Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\PolicyManager\Providers' -ErrorAction SilentlyContinue)
    $domainHistory = @(Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Group Policy\History' -Recurse -ErrorAction SilentlyContinue | ForEach-Object { Get-ItemProperty $_.PSPath } | Where-Object { $_.DSPath -like 'LDAP://*' })
    [ordered]@{
        DomainJoined = [bool](Get-CimInstance Win32_ComputerSystem).PartOfDomain
        WorkplaceJoined = [bool](Get-ChildItem 'HKCU:\Software\Microsoft\Windows NT\CurrentVersion\WorkplaceJoin\JoinInfo' -ErrorAction SilentlyContinue)
        MdmEnrollmentCount = $enrollments.Count
        MdmPolicyProviderCount = $providers.Count
        DomainGpoHistoryCount = $domainHistory.Count
        LocalPolicyStorePresent = Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Group Policy'
    }
}

function Read-Value($Path, $Name) { try { Get-ItemPropertyValue -LiteralPath $Path -Name $Name -ErrorAction Stop } catch { $null } }
Invoke-AllowlistedReadOnlyQuery 'policy_registry' {
    [ordered]@{
        Widgets = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Dsh' 'AllowNewsAndInterests'
        ConsumerExperiences = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableWindowsConsumerFeatures'
        Welcome = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableWindowsSpotlightWindowsWelcomeExperience'
        Tips = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableSoftLanding'
        LockScreen = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'ConfigureWindowsSpotlight'
        StartRecommendations = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Explorer' 'HideRecommendedSection'
        NotificationSuggestions = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableWindowsSpotlightOnActionCenter'
        SettingsSuggestions = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableWindowsSpotlightOnSettings'
        SearchWeb = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Windows Search' 'DisableWebSearch'
        SearchHighlights = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Windows Search' 'EnableDynamicContentInWSB'
    }
}

Invoke-AllowlistedReadOnlyQuery 'user_preferences' {
    $delivery = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager'
    [ordered]@{
        Welcome = Read-Value $delivery 'SubscribedContent-310093Enabled'
        Tips = Read-Value $delivery 'SoftLandingEnabled'
        LockScreen = Read-Value $delivery 'RotatingLockScreenOverlayEnabled'
        NotificationSuggestions = Read-Value $delivery 'SubscribedContent-338389Enabled'
        SettingsSuggestions = Read-Value $delivery 'SubscribedContent-338393Enabled'
        TaskbarWidgets = Read-Value 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' 'TaskbarDa'
        TaskbarSearch = Read-Value 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Search' 'SearchboxTaskbarMode'
    }
}

Invoke-AllowlistedReadOnlyQuery 'one_drive_metadata' {
    $accounts = @(Get-ChildItem 'HKCU:\Software\Microsoft\OneDrive\Accounts' -ErrorAction SilentlyContinue)
    $accountProperties = @($accounts | ForEach-Object { Get-ItemProperty $_.PSPath })
    $roots = @($accountProperties.UserFolder | Where-Object { $_ })
    $filesOnDemand = @($accountProperties | ForEach-Object { $_.FilesOnDemandEnabled } | Where-Object { $_ -ne $null })
    [ordered]@{
        Installed = [bool](Get-Command OneDrive.exe -ErrorAction SilentlyContinue)
        Version = $null
        Running = [bool](Get-Process OneDrive -ErrorAction SilentlyContinue)
        Startup = [bool](Read-Value 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' 'OneDrive')
        Personal = [bool]($accounts | Where-Object { $_.PSChildName -eq 'Personal' })
        Work = [bool]($accounts | Where-Object { $_.PSChildName -like 'Business*' })
        SyncRootCount = $roots.Count
        DesktopRedirected = $false
        DocumentsRedirected = $false
        PicturesRedirected = $false
        KfmPolicy = Test-Path 'HKLM:\SOFTWARE\Policies\Microsoft\OneDrive'
        FilesOnDemandPolicy = Read-Value 'HKLM:\SOFTWARE\Policies\Microsoft\OneDrive' 'FilesOnDemandEnabled'
        FilesOnDemandValues = $filesOnDemand
        FilesOnDemandEvidenceComplete = ($accounts.Count -eq $filesOnDemand.Count)
    }
}

$platform = $queries.platform_inventory
$bundle = [ordered]@{
    fixtureSchemaVersion = 1
    scenarioId = $ScenarioId
    osEdition = $platform.Edition
    osBuild = $platform.Build
    queries = $queries
    queryFailures = $queryFailures
    expected = @()
    knownLimitations = @('Expected assertions and manual verification steps must be reviewed on the VM before check-in.')
}

$json = $bundle | ConvertTo-Json -Depth 10
[System.IO.File]::WriteAllText((Resolve-Path -LiteralPath (Split-Path -Parent $OutputPath)).Path + [System.IO.Path]::DirectorySeparatorChar + (Split-Path -Leaf $OutputPath), $json, [System.Text.UTF8Encoding]::new($false))
Write-Host "Captured redacted fixture: $OutputPath"
Write-Host 'Review expected assertions before committing. No token, email, filename, account identity, recovery material, or full install path is captured.'
