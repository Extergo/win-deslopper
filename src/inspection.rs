//! Bounded, cancellable, read-only Windows inspection. Every command and parser
//! is allowlisted here; no caller-provided command text or arguments are accepted.

use crate::{
    applicability, package_identity,
    platform::{
        ApplicabilityStatus, Authority, AuthorityAttribution, AuthorityConfidence, ComponentId,
        ControlPrecedence, DetectionResult, DetectorStatus, Evidence, FilesOnDemandState,
        PackageCompleteness, PackageObservation, PackageProvisioningState,
        PackageRegistrationState, PlatformInfo, State,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub const PROGRESS_EVENT: &str = "deslopper://inspection-progress";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryId {
    PlatformInventory,
    AppxCurrentUser,
    AppxAllUsers,
    AppxProvisioned,
    ManagementContext,
    PolicyRegistry,
    UserPreferences,
    OneDriveMetadata,
}

pub const SHARED_QUERY_IDS: [QueryId; 8] = [
    QueryId::PlatformInventory,
    QueryId::AppxCurrentUser,
    QueryId::AppxAllUsers,
    QueryId::AppxProvisioned,
    QueryId::ManagementContext,
    QueryId::PolicyRegistry,
    QueryId::UserPreferences,
    QueryId::OneDriveMetadata,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedOutput {
    JsonObject,
    JsonArray,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserId {
    PlatformV1,
    AppxV2,
    ProvisionedV1,
    ManagementV1,
    PolicyV2,
    PreferencesV2,
    OneDriveV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryConfig {
    pub id: QueryId,
    pub default_timeout_ms: u64,
    pub maximum_output_bytes: usize,
    pub cancellation_supported: bool,
    pub expected_output: ExpectedOutput,
    pub parser_id: ParserId,
    pub essential: bool,
}

pub fn query_config(id: QueryId) -> QueryConfig {
    let (timeout, maximum, output, parser, essential) = match id {
        QueryId::PlatformInventory => (
            10_000,
            256 * 1024,
            ExpectedOutput::JsonObject,
            ParserId::PlatformV1,
            true,
        ),
        QueryId::AppxCurrentUser => (
            30_000,
            16 * 1024 * 1024,
            ExpectedOutput::JsonArray,
            ParserId::AppxV2,
            false,
        ),
        QueryId::AppxAllUsers => (
            45_000,
            32 * 1024 * 1024,
            ExpectedOutput::JsonArray,
            ParserId::AppxV2,
            false,
        ),
        QueryId::AppxProvisioned => (
            45_000,
            16 * 1024 * 1024,
            ExpectedOutput::JsonArray,
            ParserId::ProvisionedV1,
            false,
        ),
        QueryId::ManagementContext => (
            12_000,
            512 * 1024,
            ExpectedOutput::JsonObject,
            ParserId::ManagementV1,
            false,
        ),
        QueryId::PolicyRegistry => (
            12_000,
            512 * 1024,
            ExpectedOutput::JsonObject,
            ParserId::PolicyV2,
            false,
        ),
        QueryId::UserPreferences => (
            12_000,
            512 * 1024,
            ExpectedOutput::JsonObject,
            ParserId::PreferencesV2,
            false,
        ),
        QueryId::OneDriveMetadata => (
            15_000,
            1024 * 1024,
            ExpectedOutput::JsonObject,
            ParserId::OneDriveV2,
            false,
        ),
    };
    QueryConfig {
        id,
        default_timeout_ms: timeout,
        maximum_output_bytes: maximum,
        cancellation_supported: true,
        expected_output: output,
        parser_id: parser,
        essential,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryErrorKind {
    SpawnFailure,
    Timeout,
    Cancellation,
    NonZeroExit,
    OutputTooLarge,
    InvalidEncoding,
    InvalidJson,
    SchemaMismatch,
    PermissionDenied,
    UnsupportedCommand,
    UnexpectedEmptyOutput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryFailure {
    pub query_id: QueryId,
    pub kind: QueryErrorKind,
    pub message: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub partial_output_available: bool,
}

#[derive(Clone, Debug)]
pub struct CommandResult {
    pub query_id: QueryId,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub duration_ms: u128,
}

#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn cancel(&self) -> bool {
        !self.0.swap(true, Ordering::SeqCst)
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub trait ReadOnlyRunner: Send + Sync {
    fn run(
        &self,
        query_id: QueryId,
        cancellation: &CancellationToken,
    ) -> Result<CommandResult, QueryFailure>;
}

pub struct WindowsPowerShellRunner;

impl ReadOnlyRunner for WindowsPowerShellRunner {
    fn run(
        &self,
        query_id: QueryId,
        cancellation: &CancellationToken,
    ) -> Result<CommandResult, QueryFailure> {
        #[cfg(not(windows))]
        {
            let _ = cancellation;
            Err(failure(
                query_id,
                QueryErrorKind::UnsupportedCommand,
                "Windows PowerShell inspection is unavailable on this platform",
                0,
            ))
        }
        #[cfg(windows)]
        {
            run_bounded_powershell(query_id, cancellation)
        }
    }
}

fn failure(
    query_id: QueryId,
    kind: QueryErrorKind,
    message: impl Into<String>,
    duration_ms: u128,
) -> QueryFailure {
    QueryFailure {
        query_id,
        kind,
        message: message.into(),
        exit_code: None,
        duration_ms,
        partial_output_available: false,
    }
}

#[cfg(windows)]
fn powershell_script(query_id: QueryId) -> &'static str {
    match query_id {
        QueryId::PlatformInventory => {
            r#"$ErrorActionPreference='Stop';$os=Get-CimInstance Win32_OperatingSystem;$cs=Get-CimInstance Win32_ComputerSystem;$cv=Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion';$sid=[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value;$machineGuid=Get-ItemPropertyValue 'HKLM:\SOFTWARE\Microsoft\Cryptography' 'MachineGuid';[pscustomobject]@{ProductName=$cv.ProductName;Edition=$cv.EditionID;Build=[int]$cv.CurrentBuildNumber;DisplayVersion=$cv.DisplayVersion;UBR=$cv.UBR;Architecture=$os.OSArchitecture;DeviceName=$cs.Name;Manufacturer=$cs.Manufacturer;Model=$cs.Model;UserSid=$sid;MachineGuid=$machineGuid;DomainJoined=[bool]$cs.PartOfDomain;Windows11=([int]$cv.CurrentBuildNumber -ge 22000)}|ConvertTo-Json -Compress"#
        }
        QueryId::AppxCurrentUser => {
            r#"$ErrorActionPreference='Stop';@(Get-AppxPackage|Select-Object Name,PackageFamilyName,PackageFullName,Version,Architecture,PublisherId,IsFramework,IsResourcePackage,PackageUserInformation,InstallLocation,NonRemovable,Dependencies,SignatureKind)|ConvertTo-Json -Depth 6 -Compress"#
        }
        QueryId::AppxAllUsers => {
            r#"$ErrorActionPreference='Stop';@(Get-AppxPackage -AllUsers|Select-Object Name,PackageFamilyName,PackageFullName,Version,Architecture,PublisherId,IsFramework,IsResourcePackage,PackageUserInformation,InstallLocation,NonRemovable,Dependencies,SignatureKind)|ConvertTo-Json -Depth 6 -Compress"#
        }
        QueryId::AppxProvisioned => {
            r#"$ErrorActionPreference='Stop';@(Get-AppxProvisionedPackage -Online|Select-Object DisplayName,PackageName,Version,Architecture,PublisherId,ResourceId)|ConvertTo-Json -Depth 5 -Compress"#
        }
        QueryId::ManagementContext => {
            r#"$ErrorActionPreference='SilentlyContinue';$cs=Get-CimInstance Win32_ComputerSystem;$entra=@(Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Control\CloudDomainJoin\JoinInfo');$work=@(Get-ChildItem 'HKCU:\Software\Microsoft\Windows NT\CurrentVersion\WorkplaceJoin\JoinInfo');$metadata=@(Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\Enrollments'|ForEach-Object{$p=Get-ItemProperty $_.PSPath;if($p.ProviderID){[pscustomobject]@{Id=$_.PSChildName;Properties=$p}}});$enroll=@($metadata|Where-Object{$id=$_.Id;$p=$_.Properties;$account=Test-Path "HKLM:\SOFTWARE\Microsoft\Provisioning\OMADM\Accounts\$id";$task=Test-Path "$env:windir\System32\Tasks\Microsoft\Windows\EnterpriseMgmt\$id";$certificate=-not[string]::IsNullOrWhiteSpace([string]$p.DMPCertThumbPrint);$account -or $task -or $certificate});$providers=@(Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\PolicyManager\Providers');$history=@(Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Group Policy\History' -Recurse|ForEach-Object{Get-ItemProperty $_.PSPath}|Where-Object{$_.DSPath -like 'LDAP://*'});[pscustomobject]@{DomainJoined=[bool]$cs.PartOfDomain;EntraJoined=($entra.Count -gt 0);WorkplaceJoined=($work.Count -gt 0);MdmEnrollmentCount=$enroll.Count;MdmEnrollmentMetadataCount=$metadata.Count;MdmPolicyProviderCount=$providers.Count;DomainGpoHistoryCount=$history.Count;LocalPolicyStorePresent=(Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Group Policy')}|ConvertTo-Json -Compress"#
        }
        QueryId::PolicyRegistry => {
            r#"$ErrorActionPreference='SilentlyContinue';function V($p,$n){try{Get-ItemPropertyValue -LiteralPath $p -Name $n -ErrorAction Stop}catch{$null}};[pscustomobject]@{Widgets=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Dsh' 'AllowNewsAndInterests');ConsumerExperiences=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableWindowsConsumerFeatures');Welcome=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableWindowsSpotlightWindowsWelcomeExperience');Tips=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableSoftLanding');LockScreen=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'ConfigureWindowsSpotlight');StartRecommendations=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Explorer' 'HideRecommendedSection');NotificationSuggestions=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableWindowsSpotlightOnActionCenter');SettingsSuggestions=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent' 'DisableWindowsSpotlightOnSettings');SearchWeb=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Windows Search' 'DisableWebSearch');SearchHighlights=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Windows Search' 'EnableDynamicContentInWSB');TaskViewMachineHide=(V 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\Explorer' 'HideTaskViewButton');TaskViewUserHide=(V 'HKCU:\SOFTWARE\Policies\Microsoft\Windows\Explorer' 'HideTaskViewButton');TaskbarMachineLocked=(V 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer' 'NoSetTaskbar');TaskbarUserLocked=(V 'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer' 'NoSetTaskbar')}|ConvertTo-Json -Compress"#
        }
        QueryId::UserPreferences => {
            r#"$ErrorActionPreference='SilentlyContinue';function V($p,$n){try{Get-ItemPropertyValue -LiteralPath $p -Name $n -ErrorAction Stop}catch{$null}};$cd='HKCU:\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager';[pscustomobject]@{Welcome=(V $cd 'SubscribedContent-310093Enabled');Tips=(V $cd 'SoftLandingEnabled');LockScreen=(V $cd 'RotatingLockScreenOverlayEnabled');NotificationSuggestions=(V $cd 'SubscribedContent-338389Enabled');SettingsSuggestions=(V $cd 'SubscribedContent-338393Enabled');TaskbarWidgets=(V 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' 'TaskbarDa');TaskbarTaskView=(V 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' 'ShowTaskViewButton');TaskbarSearch=(V 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Search' 'SearchboxTaskbarMode')}|ConvertTo-Json -Compress"#
        }
        QueryId::OneDriveMetadata => {
            r#"$ErrorActionPreference='SilentlyContinue';$paths=@("$env:LOCALAPPDATA\Microsoft\OneDrive\OneDrive.exe","$env:ProgramFiles\Microsoft OneDrive\OneDrive.exe","${env:ProgramFiles(x86)}\Microsoft OneDrive\OneDrive.exe");$exe=$paths|Where-Object{Test-Path -LiteralPath $_}|Select-Object -First 1;$accounts=@(Get-ChildItem 'HKCU:\Software\Microsoft\OneDrive\Accounts');$props=@($accounts|ForEach-Object{Get-ItemProperty $_.PSPath});$roots=@($props.UserFolder|Where-Object{$_});$shell=Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders';$run=Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue;function Inside($p){if(!$p){return $false};@($roots|Where-Object{$p.StartsWith($_,[StringComparison]::OrdinalIgnoreCase)}).Count -gt 0};$fod=@($props|ForEach-Object{$_.FilesOnDemandEnabled}|Where-Object{$_ -ne $null});$fodPolicy=Get-ItemPropertyValue 'HKLM:\SOFTWARE\Policies\Microsoft\OneDrive' 'FilesOnDemandEnabled' -ErrorAction SilentlyContinue;$kfm=Get-ItemProperty 'HKLM:\SOFTWARE\Policies\Microsoft\OneDrive' -ErrorAction SilentlyContinue;[pscustomobject]@{Installed=[bool]$exe;Version=$(if($exe){(Get-Item -LiteralPath $exe).VersionInfo.FileVersion});Running=[bool](Get-Process OneDrive);Startup=[bool]$run.OneDrive;Personal=[bool]($accounts|Where-Object{$_.PSChildName -eq 'Personal'});Work=[bool]($accounts|Where-Object{$_.PSChildName -like 'Business*'});SyncRootCount=$roots.Count;DesktopRedirected=(Inside $shell.Desktop);DocumentsRedirected=(Inside $shell.Personal);PicturesRedirected=(Inside $shell.'My Pictures');KfmPolicy=[bool]$kfm;FilesOnDemandPolicy=$fodPolicy;FilesOnDemandValues=@($fod);FilesOnDemandEvidenceComplete=($accounts.Count -eq $fod.Count)}|ConvertTo-Json -Depth 4 -Compress"#
        }
    }
}

#[cfg(windows)]
fn read_limited<R: Read>(mut reader: R, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut kept = Vec::new();
    let mut too_large = false;
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(kept.len());
        kept.extend_from_slice(&buffer[..count.min(remaining)]);
        too_large |= count > remaining;
    }
    Ok((kept, too_large))
}

#[cfg(windows)]
fn run_bounded_powershell(
    query_id: QueryId,
    cancellation: &CancellationToken,
) -> Result<CommandResult, QueryFailure> {
    let config = query_config(query_id);
    let started = Instant::now();
    if cancellation.is_cancelled() {
        return Err(failure(
            query_id,
            QueryErrorKind::Cancellation,
            "Inspection cancellation was requested",
            0,
        ));
    }
    let mut child = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            powershell_script(query_id),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            failure(
                query_id,
                QueryErrorKind::SpawnFailure,
                error.to_string(),
                started.elapsed().as_millis(),
            )
        })?;
    let stdout = child.stdout.take().ok_or_else(|| {
        failure(
            query_id,
            QueryErrorKind::SpawnFailure,
            "PowerShell stdout was unavailable",
            started.elapsed().as_millis(),
        )
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        failure(
            query_id,
            QueryErrorKind::SpawnFailure,
            "PowerShell stderr was unavailable",
            started.elapsed().as_millis(),
        )
    })?;
    let stdout_thread = thread::spawn(move || read_limited(stdout, config.maximum_output_bytes));
    let stderr_thread = thread::spawn(move || read_limited(stderr, 1024 * 1024));
    let termination_kind = loop {
        if cancellation.is_cancelled() {
            let _ = child.kill();
            break Some(QueryErrorKind::Cancellation);
        }
        if started.elapsed() >= Duration::from_millis(config.default_timeout_ms) {
            let _ = child.kill();
            break Some(QueryErrorKind::Timeout);
        }
        match child.try_wait() {
            Ok(Some(_)) => break None,
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(failure(
                    query_id,
                    QueryErrorKind::SpawnFailure,
                    error.to_string(),
                    started.elapsed().as_millis(),
                ));
            }
        }
    };
    let status = child.wait().map_err(|error| {
        failure(
            query_id,
            QueryErrorKind::SpawnFailure,
            error.to_string(),
            started.elapsed().as_millis(),
        )
    })?;
    let (stdout_bytes, stdout_too_large) = stdout_thread
        .join()
        .map_err(|_| {
            failure(
                query_id,
                QueryErrorKind::SpawnFailure,
                "stdout reader stopped unexpectedly",
                started.elapsed().as_millis(),
            )
        })?
        .map_err(|error| {
            failure(
                query_id,
                QueryErrorKind::SpawnFailure,
                error.to_string(),
                started.elapsed().as_millis(),
            )
        })?;
    let (stderr_bytes, _) = stderr_thread
        .join()
        .map_err(|_| {
            failure(
                query_id,
                QueryErrorKind::SpawnFailure,
                "stderr reader stopped unexpectedly",
                started.elapsed().as_millis(),
            )
        })?
        .map_err(|error| {
            failure(
                query_id,
                QueryErrorKind::SpawnFailure,
                error.to_string(),
                started.elapsed().as_millis(),
            )
        })?;
    let duration = started.elapsed().as_millis();
    let stdout = String::from_utf8(stdout_bytes).map_err(|_| {
        failure(
            query_id,
            QueryErrorKind::InvalidEncoding,
            "PowerShell stdout was not valid UTF-8",
            duration,
        )
    })?;
    let stderr = String::from_utf8(stderr_bytes).map_err(|_| {
        failure(
            query_id,
            QueryErrorKind::InvalidEncoding,
            "PowerShell stderr was not valid UTF-8",
            duration,
        )
    })?;
    if let Some(kind) = termination_kind {
        let mut value = failure(
            query_id,
            kind,
            if kind == QueryErrorKind::Timeout {
                "Read-only query exceeded its bounded timeout"
            } else {
                "Read-only query was cancelled and its child process was terminated"
            },
            duration,
        );
        value.partial_output_available = !stdout.trim().is_empty();
        value.exit_code = status.code();
        return Err(value);
    }
    if stdout_too_large {
        let mut value = failure(
            query_id,
            QueryErrorKind::OutputTooLarge,
            "Read-only query exceeded its configured output limit",
            duration,
        );
        value.partial_output_available = true;
        return Err(value);
    }
    if !status.success() {
        let lower = stderr.to_ascii_lowercase();
        let kind = if lower.contains("access is denied")
            || lower.contains("permissiondenied")
            || lower.contains("administrator privileges")
        {
            QueryErrorKind::PermissionDenied
        } else {
            QueryErrorKind::NonZeroExit
        };
        let message = if stderr.trim().is_empty() {
            "PowerShell returned a non-zero exit code".into()
        } else {
            crate::privacy::redact(stderr.trim())
        };
        let mut value = failure(query_id, kind, message, duration);
        value.exit_code = status.code();
        return Err(value);
    }
    if stdout.trim().is_empty() {
        return Err(failure(
            query_id,
            QueryErrorKind::UnexpectedEmptyOutput,
            "Read-only query returned no structured output",
            duration,
        ));
    }
    Ok(CommandResult {
        query_id,
        exit_code: status.code(),
        stdout,
        duration_ms: duration,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionPhase {
    Idle,
    Preparing,
    GatheringSharedContext,
    RunningDetectors,
    PersistingResults,
    CalculatingDrift,
    Completed,
    CompletedWithPartialFailures,
    Cancelling,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionLifecycle {
    pub inspection_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub phase: InspectionPhase,
    pub total_detector_count: usize,
    pub completed_detector_count: usize,
    pub successful_detector_count: usize,
    pub unknown_detector_count: usize,
    pub failed_detector_count: usize,
    pub cancelled_detector_count: usize,
    pub current_detector_ids: Vec<String>,
    pub status: String,
    pub last_progress_update: String,
    pub cancellation_requested: bool,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionProgress {
    pub inspection_id: String,
    pub phase: InspectionPhase,
    pub completed_work: usize,
    pub total_work: usize,
    pub current_component: Option<String>,
    pub detector_result_status: Option<DetectorStatus>,
    pub warning_count: usize,
    pub error_count: usize,
    pub cancellation_available: bool,
    pub status: String,
    pub updated_at: String,
}

impl InspectionLifecycle {
    pub fn new(inspection_id: String, total: usize) -> Self {
        let now = timestamp();
        Self {
            inspection_id,
            started_at: now.clone(),
            completed_at: None,
            phase: InspectionPhase::Preparing,
            total_detector_count: total,
            completed_detector_count: 0,
            successful_detector_count: 0,
            unknown_detector_count: 0,
            failed_detector_count: 0,
            cancelled_detector_count: 0,
            current_detector_ids: Vec::new(),
            status: "Preparing the read-only inspection".into(),
            last_progress_update: now,
            cancellation_requested: false,
            warning_count: 0,
            error_count: 0,
        }
    }

    pub fn progress(
        &self,
        current_component: Option<String>,
        detector_result_status: Option<DetectorStatus>,
    ) -> InspectionProgress {
        InspectionProgress {
            inspection_id: self.inspection_id.clone(),
            phase: self.phase,
            completed_work: self.completed_detector_count,
            total_work: self.total_detector_count,
            current_component,
            detector_result_status,
            warning_count: self.warning_count,
            error_count: self.error_count,
            cancellation_available: !matches!(
                self.phase,
                InspectionPhase::Completed
                    | InspectionPhase::CompletedWithPartialFailures
                    | InspectionPhase::Cancelled
                    | InspectionPhase::Failed
            ),
            status: self.status.clone(),
            updated_at: self.last_progress_update.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InspectionOutcome {
    pub lifecycle: InspectionLifecycle,
    pub platform: PlatformInfo,
    pub observations: Vec<DetectionResult>,
    pub query_failures: Vec<QueryFailure>,
    pub query_execution_counts: BTreeMap<QueryId, usize>,
}

#[derive(Default)]
struct SharedContext {
    values: BTreeMap<QueryId, Result<Value, QueryFailure>>,
    executions: BTreeMap<QueryId, usize>,
}

fn parse_query(result: CommandResult) -> Result<Value, QueryFailure> {
    let config = query_config(result.query_id);
    let value: Value =
        serde_json::from_str(result.stdout.trim()).map_err(|error| QueryFailure {
            query_id: result.query_id,
            kind: QueryErrorKind::InvalidJson,
            message: error.to_string(),
            exit_code: result.exit_code,
            duration_ms: result.duration_ms,
            partial_output_available: true,
        })?;
    let valid = match config.expected_output {
        ExpectedOutput::JsonObject => value.is_object(),
        ExpectedOutput::JsonArray => value.is_array() || value.is_object() || value.is_null(),
    };
    if !valid {
        return Err(QueryFailure {
            query_id: result.query_id,
            kind: QueryErrorKind::SchemaMismatch,
            message: "Structured output did not match the configured root schema".into(),
            exit_code: result.exit_code,
            duration_ms: result.duration_ms,
            partial_output_available: true,
        });
    }
    Ok(value)
}

fn parse_inventory(value: Option<&Value>) -> PlatformInfo {
    let Some(value) = value else {
        return default_platform();
    };
    PlatformInfo {
        product_name: value["ProductName"].as_str().unwrap_or("Unknown").into(),
        edition: value["Edition"].as_str().unwrap_or("Unknown").into(),
        build: value["Build"]
            .as_u64()
            .or_else(|| {
                value["Build"]
                    .as_str()
                    .and_then(|number| number.parse().ok())
            })
            .unwrap_or(0) as u32,
        display_version: value["DisplayVersion"].as_str().unwrap_or("Unknown").into(),
        update_build_revision: value["UBR"].as_u64().map(|number| number as u32),
        architecture: value["Architecture"].as_str().unwrap_or("Unknown").into(),
        device_name: value["DeviceName"].as_str().map(str::to_owned),
        manufacturer: value["Manufacturer"].as_str().map(str::to_owned),
        model: value["Model"].as_str().map(str::to_owned),
        user_sid: None,
        owner_scope_id: {
            #[cfg(feature = "owner-mode")]
            {
                value["MachineGuid"]
                    .as_str()
                    .zip(value["UserSid"].as_str())
                    .map(|(machine, user)| crate::owner_scope::from_stable_ids(machine, user))
            }
            #[cfg(not(feature = "owner-mode"))]
            {
                None
            }
        },
        elevated: false,
        domain_joined: value["DomainJoined"].as_bool(),
        entra_joined: None,
        workplace_joined: None,
        mdm_enrolled: None,
        is_windows_11: value["Windows11"].as_bool(),
    }
}

pub fn timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}

pub fn default_platform() -> PlatformInfo {
    PlatformInfo {
        product_name: "Unknown".into(),
        edition: "Unknown".into(),
        build: 0,
        display_version: "Unknown".into(),
        update_build_revision: None,
        architecture: "Unknown".into(),
        device_name: None,
        manufacturer: None,
        model: None,
        user_sid: None,
        owner_scope_id: None,
        elevated: false,
        domain_joined: None,
        entra_joined: None,
        workplace_joined: None,
        mdm_enrolled: None,
        is_windows_11: None,
    }
}

fn evidence(
    query_id: QueryId,
    source: &str,
    detail: impl Into<String>,
    confidence: u8,
) -> Evidence {
    Evidence {
        query_id: format!("{query_id:?}"),
        source: source.into(),
        detail: detail.into(),
        confidence,
    }
}

fn cancelled(id: ComponentId, info: &PlatformInfo) -> DetectionResult {
    base_unknown(
        id,
        info,
        DetectorStatus::Cancelled,
        "Detector was not run because inspection cancellation was requested",
        QueryId::PlatformInventory,
        QueryErrorKind::Cancellation,
    )
}

fn base_unknown(
    id: ComponentId,
    info: &PlatformInfo,
    status: DetectorStatus,
    reason: &str,
    query_id: QueryId,
    kind: QueryErrorKind,
) -> DetectionResult {
    let applicability = applicability::evaluate(id, info);
    let package_completeness = if package_identity::rules(id).is_empty() {
        PackageCompleteness::NotApplicable
    } else {
        PackageCompleteness::Unknown
    };
    DetectionResult {
        component_id: id,
        current: State::Unknown {
            error: reason.into(),
        },
        authority: Authority::Unknown,
        platform: info.clone(),
        applicable: false,
        applicability,
        evidence: vec![evidence(
            query_id,
            "Read-only inspection",
            format!("{kind:?}: {reason}"),
            0,
        )],
        detected_at: timestamp(),
        error: if status == DetectorStatus::Failed {
            Some(reason.into())
        } else {
            None
        },
        warnings: vec![reason.into()],
        package_identities: Vec::new(),
        policy_state: None,
        preference_state: None,
        provisioning_state: None,
        detector_status: status,
        authority_attribution: AuthorityAttribution::default(),
        control_precedence: ControlPrecedence::default(),
        package_completeness,
        packages: Vec::new(),
    }
}

fn value_array(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(values) => values.iter().collect(),
        Value::Object(_) => vec![value],
        Value::Null => Vec::new(),
        _ => Vec::new(),
    }
}

fn query_state(context: &SharedContext, id: QueryId) -> Result<&Value, QueryFailure> {
    match context.values.get(&id) {
        Some(Ok(value)) => Ok(value),
        Some(Err(error)) => Err(error.clone()),
        None => Err(failure(
            id,
            QueryErrorKind::UnsupportedCommand,
            "Shared query was not executed",
            0,
        )),
    }
}

fn registration_from_query(
    result: Result<&Value, QueryFailure>,
    present: bool,
) -> PackageRegistrationState {
    match result {
        Ok(_) if present => PackageRegistrationState::Present,
        Ok(_) => PackageRegistrationState::Absent,
        Err(error) if error.kind == QueryErrorKind::PermissionDenied => {
            PackageRegistrationState::PermissionLimited
        }
        Err(_) => PackageRegistrationState::QueryFailed,
    }
}

fn provisioning_from_query(
    result: Result<&Value, QueryFailure>,
    present: bool,
) -> PackageProvisioningState {
    match result {
        Ok(_) if present => PackageProvisioningState::Provisioned,
        Ok(_) => PackageProvisioningState::NotProvisioned,
        Err(error) if error.kind == QueryErrorKind::PermissionDenied => {
            PackageProvisioningState::PermissionLimited
        }
        Err(_) => PackageProvisioningState::QueryFailed,
    }
}

fn package_completeness(
    current: &Result<&Value, QueryFailure>,
    all: &Result<&Value, QueryFailure>,
    provisioned: &Result<&Value, QueryFailure>,
) -> PackageCompleteness {
    match (current, all, provisioned) {
        (Ok(_), Ok(_), Ok(_)) => PackageCompleteness::Complete,
        (Ok(_), Err(a), Ok(_)) if a.kind == QueryErrorKind::PermissionDenied => {
            PackageCompleteness::ProvisioningCompleteAllUsersUnknown
        }
        (Ok(_), Ok(_), Err(_)) => PackageCompleteness::RegistrationCompleteProvisioningUnknown,
        (Ok(_), Err(a), Err(p))
            if a.kind == QueryErrorKind::PermissionDenied
                || p.kind == QueryErrorKind::PermissionDenied =>
        {
            PackageCompleteness::PermissionLimited
        }
        (Ok(_), Err(_), Err(_)) => PackageCompleteness::CurrentUserOnly,
        _ => PackageCompleteness::Failed,
    }
}

fn package_detection(
    id: ComponentId,
    info: &PlatformInfo,
    context: &SharedContext,
) -> DetectionResult {
    let applicability = applicability::evaluate(id, info);
    if matches!(
        applicability.status,
        ApplicabilityStatus::UnsupportedBuild
            | ApplicabilityStatus::UnsupportedEdition
            | ApplicabilityStatus::Removed
            | ApplicabilityStatus::MissingPrerequisite
    ) {
        return unsupported(id, info, applicability);
    }
    let current_query = query_state(context, QueryId::AppxCurrentUser);
    let all_query = query_state(context, QueryId::AppxAllUsers);
    let provisioned_query = query_state(context, QueryId::AppxProvisioned);
    let completeness = package_completeness(&current_query, &all_query, &provisioned_query);
    let current_matches: Vec<&Value> = current_query
        .as_ref()
        .map(|value| {
            value_array(value)
                .into_iter()
                .filter(|item| {
                    package_identity::exact_match(
                        id,
                        item["Name"].as_str().unwrap_or(""),
                        info.build,
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let all_matches: Vec<&Value> = all_query
        .as_ref()
        .map(|value| {
            value_array(value)
                .into_iter()
                .filter(|item| {
                    package_identity::exact_match(
                        id,
                        item["Name"].as_str().unwrap_or(""),
                        info.build,
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let provisioned_matches: Vec<&Value> = provisioned_query
        .as_ref()
        .map(|value| {
            value_array(value)
                .into_iter()
                .filter(|item| {
                    package_identity::exact_match(
                        id,
                        item["DisplayName"].as_str().unwrap_or(""),
                        info.build,
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let current_state = registration_from_query(current_query.clone(), !current_matches.is_empty());
    let all_state = registration_from_query(all_query.clone(), !all_matches.is_empty());
    let provisioning_state =
        provisioning_from_query(provisioned_query.clone(), !provisioned_matches.is_empty());
    let mut names = BTreeSet::new();
    for item in current_matches.iter().chain(all_matches.iter()) {
        if let Some(name) = item["Name"].as_str() {
            names.insert(name.to_owned());
        }
    }
    for item in &provisioned_matches {
        if let Some(name) = item["DisplayName"].as_str() {
            names.insert(name.to_owned());
        }
    }
    let mut packages = Vec::new();
    for name in &names {
        let current_item = current_matches.iter().copied().find(|item| {
            item["Name"]
                .as_str()
                .is_some_and(|value| value.eq_ignore_ascii_case(name))
        });
        let all_item = all_matches.iter().copied().find(|item| {
            item["Name"]
                .as_str()
                .is_some_and(|value| value.eq_ignore_ascii_case(name))
        });
        let provisioned_item = provisioned_matches.iter().copied().find(|item| {
            item["DisplayName"]
                .as_str()
                .is_some_and(|value| value.eq_ignore_ascii_case(name))
        });
        let item = current_item.or(all_item).or(provisioned_item);
        let dependencies = item
            .and_then(|value| value["Dependencies"].as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|dependency| {
                        dependency["Name"]
                            .as_str()
                            .map(str::to_owned)
                            .or_else(|| dependency.as_str().map(str::to_owned))
                    })
                    .collect()
            })
            .unwrap_or_default();
        packages.push(PackageObservation {
            component_id: id,
            package_family_name: item
                .and_then(|value| value["PackageFamilyName"].as_str())
                .map(str::to_owned),
            package_full_name: item
                .and_then(|value| {
                    value["PackageFullName"]
                        .as_str()
                        .or_else(|| value["PackageName"].as_str())
                })
                .map(str::to_owned),
            package_name: name.clone(),
            version: item
                .and_then(|value| value["Version"].as_str())
                .map(str::to_owned),
            architecture: item
                .and_then(|value| value["Architecture"].as_str())
                .map(str::to_owned),
            publisher_id: item
                .and_then(|value| value["PublisherId"].as_str())
                .map(str::to_owned),
            current_user: registration_from_query(
                query_state(context, QueryId::AppxCurrentUser),
                current_item.is_some(),
            ),
            other_users: registration_from_query(
                query_state(context, QueryId::AppxAllUsers),
                all_item.is_some(),
            ),
            provisioning: provisioning_from_query(
                query_state(context, QueryId::AppxProvisioned),
                provisioned_item.is_some(),
            ),
            framework: item
                .and_then(|value| value["IsFramework"].as_bool())
                .unwrap_or(false),
            resource_package: item
                .and_then(|value| value["IsResourcePackage"].as_bool())
                .unwrap_or(false),
            bundle: item
                .and_then(|value| value["PackageFullName"].as_str())
                .is_some_and(|value| value.to_ascii_lowercase().contains("bundle")),
            non_removable: item
                .and_then(|value| value["NonRemovable"].as_bool())
                .unwrap_or(false),
            install_location_present: item.map(|value| {
                value["InstallLocation"]
                    .as_str()
                    .is_some_and(|path| !path.is_empty())
            }),
            dependencies,
            source_query_completeness: completeness,
            permission_status: if completeness == PackageCompleteness::Complete {
                "complete".into()
            } else {
                format!("{completeness:?}")
            },
            observed_at: timestamp(),
        });
    }
    let warnings = if completeness == PackageCompleteness::Complete {
        Vec::new()
    } else {
        vec!["Package scope is incomplete; Deslopper will not interpret unavailable machine-wide scope as absence or preview full absence".into()]
    };
    let status = if completeness == PackageCompleteness::Failed {
        DetectorStatus::Failed
    } else if completeness == PackageCompleteness::Complete {
        DetectorStatus::Successful
    } else {
        DetectorStatus::Unknown
    };
    let authority_attribution = AuthorityAttribution {
        authority: Authority::WindowsServicing,
        confidence: AuthorityConfidence::Moderate,
        exact_source_proven: false,
        evidence: vec!["Exact package identity observed in one or more AppX inventories".into()],
        alternatives: vec![Authority::User, Authority::DomainPolicy, Authority::Mdm],
    };
    DetectionResult {
        component_id: id,
        current: State::Package {
            current_user: current_state,
            all_users: all_state,
            provisioned: provisioning_state,
            version: current_matches
                .first()
                .or(all_matches.first())
                .and_then(|item| item["Version"].as_str())
                .map(str::to_owned),
        },
        authority: authority_attribution.authority,
        platform: info.clone(),
        applicable: matches!(
            applicability.status,
            ApplicabilityStatus::Applicable | ApplicabilityStatus::PartiallyApplicable
        ),
        applicability,
        evidence: vec![evidence(
            QueryId::AppxCurrentUser,
            "Shared AppX inventories",
            format!(
                "{} current-user, {} all-user, and {} provisioned exact identity record(s) matched; completeness is {completeness:?}",
                current_matches.len(),
                all_matches.len(),
                provisioned_matches.len()
            ),
            if completeness == PackageCompleteness::Complete {
                95
            } else {
                55
            },
        )],
        detected_at: timestamp(),
        error: if status == DetectorStatus::Failed {
            Some("All relevant AppX inventory scopes failed".into())
        } else {
            None
        },
        warnings,
        package_identities: names.into_iter().collect(),
        policy_state: None,
        preference_state: None,
        provisioning_state: Some(format!("{provisioning_state:?}")),
        detector_status: status,
        authority_attribution,
        control_precedence: ControlPrecedence::default(),
        package_completeness: completeness,
        packages,
    }
}

fn unsupported(
    id: ComponentId,
    info: &PlatformInfo,
    applicability: crate::platform::ApplicabilityResult,
) -> DetectionResult {
    DetectionResult {
        component_id: id,
        current: State::Unsupported {
            reason: applicability.reason.clone(),
        },
        authority: Authority::Unknown,
        platform: info.clone(),
        applicable: false,
        evidence: vec![Evidence {
            query_id: "applicability_matrix".into(),
            source: applicability.source.clone(),
            detail: applicability.reason.clone(),
            confidence: 95,
        }],
        detected_at: timestamp(),
        error: None,
        warnings: vec![applicability.reason.clone()],
        package_identities: Vec::new(),
        policy_state: None,
        preference_state: None,
        provisioning_state: None,
        detector_status: DetectorStatus::Successful,
        authority_attribution: AuthorityAttribution::default(),
        control_precedence: ControlPrecedence::default(),
        package_completeness: PackageCompleteness::NotApplicable,
        packages: Vec::new(),
        applicability,
    }
}

fn management_attribution(
    context: &SharedContext,
    policy_present: bool,
    preference_present: bool,
) -> AuthorityAttribution {
    let management = query_state(context, QueryId::ManagementContext).ok();
    let domain = management
        .and_then(|value| value["DomainGpoHistoryCount"].as_u64())
        .unwrap_or(0)
        > 0;
    let mdm = management
        .and_then(|value| value["MdmPolicyProviderCount"].as_u64())
        .unwrap_or(0)
        > 0
        && management
            .and_then(|value| value["MdmEnrollmentCount"].as_u64())
            .unwrap_or(0)
            > 0;
    let local = management
        .and_then(|value| value["LocalPolicyStorePresent"].as_bool())
        .unwrap_or(false);
    if policy_present && domain && mdm {
        AuthorityAttribution { authority: Authority::Unknown, confidence: AuthorityConfidence::Moderate, exact_source_proven: false, evidence: vec!["Both domain Group Policy history and an enrolled MDM policy provider are present; the exact setting source is not proven".into()], alternatives: vec![Authority::DomainPolicy, Authority::Mdm, Authority::LocalPolicy] }
    } else if policy_present && domain {
        AuthorityAttribution { authority: Authority::DomainPolicy, confidence: AuthorityConfidence::Strong, exact_source_proven: false, evidence: vec!["Applied domain Group Policy history exists, but the registry value cannot be tied to a specific GPO without supported resultant-set detail".into()], alternatives: if local { vec![Authority::LocalPolicy] } else { Vec::new() } }
    } else if policy_present && mdm {
        AuthorityAttribution { authority: Authority::Mdm, confidence: AuthorityConfidence::Strong, exact_source_proven: false, evidence: vec!["Documented enrollment metadata and a PolicyManager provider are both present; the exact CSP setting source remains inferred".into()], alternatives: if local { vec![Authority::LocalPolicy] } else { Vec::new() } }
    } else if policy_present {
        AuthorityAttribution { authority: Authority::LocalPolicy, confidence: if local { AuthorityConfidence::Moderate } else { AuthorityConfidence::Weak }, exact_source_proven: false, evidence: vec!["A policy value is present; neither domain resultant history nor MDM provider evidence proves its source".into()], alternatives: vec![Authority::DomainPolicy, Authority::Mdm] }
    } else if preference_present {
        AuthorityAttribution {
            authority: Authority::User,
            confidence: AuthorityConfidence::Confirmed,
            exact_source_proven: true,
            evidence: vec![
                "A current-user preference value was read from its documented representation"
                    .into(),
            ],
            alternatives: Vec::new(),
        }
    } else {
        AuthorityAttribution::default()
    }
}

fn field_map(id: ComponentId) -> Option<(&'static str, Option<&'static str>, bool)> {
    use ComponentId::*;
    match id {
        WidgetsPlatform => Some(("Widgets", None, false)),
        ConsumerExperiences => Some(("ConsumerExperiences", None, true)),
        WelcomeExperience => Some(("Welcome", Some("Welcome"), true)),
        TipsSuggestions => Some(("Tips", Some("Tips"), true)),
        LockScreenSuggestions => Some(("LockScreen", Some("LockScreen"), false)),
        StartRecommendations => Some(("StartRecommendations", None, true)),
        NotificationSuggestions => Some((
            "NotificationSuggestions",
            Some("NotificationSuggestions"),
            true,
        )),
        SettingsSuggestedContent => {
            Some(("SettingsSuggestions", Some("SettingsSuggestions"), true))
        }
        SearchWebResults => Some(("SearchWeb", None, true)),
        SearchHighlights => Some(("SearchHighlights", None, false)),
        TaskbarWidgets => Some(("", Some("TaskbarWidgets"), false)),
        TaskbarSearch => Some(("", Some("TaskbarSearch"), false)),
        _ => None,
    }
}

fn setting_detection(
    id: ComponentId,
    info: &PlatformInfo,
    context: &SharedContext,
) -> DetectionResult {
    if id == ComponentId::TaskbarTaskView {
        return task_view_detection(info, context);
    }
    let applicability = applicability::evaluate(id, info);
    if matches!(
        applicability.status,
        ApplicabilityStatus::UnsupportedBuild
            | ApplicabilityStatus::UnsupportedEdition
            | ApplicabilityStatus::Removed
            | ApplicabilityStatus::MissingPrerequisite
    ) {
        return unsupported(id, info, applicability);
    }
    let Some((policy_field, preference_field, inverted)) = field_map(id) else {
        return base_unknown(
            id,
            info,
            DetectorStatus::Unknown,
            "No verified setting representation exists",
            QueryId::PolicyRegistry,
            QueryErrorKind::UnsupportedCommand,
        );
    };
    let policy_raw = if policy_field.is_empty() {
        None
    } else {
        query_state(context, QueryId::PolicyRegistry)
            .ok()
            .and_then(|value| value.get(policy_field).cloned())
    };
    let policy_value = policy_raw.as_ref().and_then(serde_json::Value::as_i64);
    let preference_raw = preference_field.and_then(|field| {
        query_state(context, QueryId::UserPreferences)
            .ok()
            .and_then(|value| value.get(field).cloned())
    });
    let preference_value = preference_raw.as_ref().and_then(serde_json::Value::as_i64);
    let exact_binary_contract = matches!(
        id,
        ComponentId::WelcomeExperience
            | ComponentId::TipsSuggestions
            | ComponentId::LockScreenSuggestions
            | ComponentId::NotificationSuggestions
            | ComponentId::SettingsSuggestedContent
            | ComponentId::TaskbarWidgets
    );
    let preference_inverted = inverted
        && !matches!(
            id,
            ComponentId::WelcomeExperience
                | ComponentId::TipsSuggestions
                | ComponentId::NotificationSuggestions
                | ComponentId::SettingsSuggestedContent
        );
    let invalid_policy = policy_raw
        .as_ref()
        .is_some_and(|value| !value.is_null() && !matches!(value.as_i64(), Some(0 | 1)));
    let invalid_preference = preference_raw
        .as_ref()
        .is_some_and(|value| !value.is_null() && !matches!(value.as_i64(), Some(0 | 1)));
    if exact_binary_contract && (invalid_policy || invalid_preference) {
        return base_unknown(
            id,
            info,
            DetectorStatus::Unknown,
            "A fixed setting representation was not DWORD 0 or 1; no effective state was inferred",
            if invalid_policy {
                QueryId::PolicyRegistry
            } else {
                QueryId::UserPreferences
            },
            QueryErrorKind::SchemaMismatch,
        );
    }
    if policy_value.is_none() && preference_value.is_none() {
        let query_failed = query_state(context, QueryId::PolicyRegistry).is_err()
            && query_state(context, QueryId::UserPreferences).is_err();
        return base_unknown(
            id,
            info,
            if query_failed {
                DetectorStatus::Failed
            } else {
                DetectorStatus::Unknown
            },
            if query_failed {
                "Policy and preference queries failed; no normal state was inferred"
            } else {
                "No configured value was observed and a documented effective default cannot be proven"
            },
            QueryId::PolicyRegistry,
            if query_failed {
                QueryErrorKind::NonZeroExit
            } else {
                QueryErrorKind::UnexpectedEmptyOutput
            },
        );
    }
    let render_policy = |value: i64| {
        if if inverted { value == 0 } else { value != 0 } {
            "Enabled".to_owned()
        } else {
            "Disabled".to_owned()
        }
    };
    let render_preference = |value: i64| {
        if if preference_inverted {
            value == 0
        } else {
            value != 0
        } {
            "Enabled".to_owned()
        } else {
            "Disabled".to_owned()
        }
    };
    let policy_state = policy_value.map(render_policy);
    let preference_state = preference_value.map(render_preference);
    let effective_state = policy_state.clone().or_else(|| preference_state.clone());
    let attribution =
        management_attribution(context, policy_value.is_some(), preference_value.is_some());
    let conflict =
        policy_state.is_some() && preference_state.is_some() && policy_state != preference_state;
    let current = if let Some(value) = policy_value {
        State::Policy {
            configured: true,
            enabled: if inverted { value == 0 } else { value != 0 },
        }
    } else {
        State::UserPreference {
            enabled: preference_value.is_some_and(|value| {
                if preference_inverted {
                    value == 0
                } else {
                    value != 0
                }
            }),
        }
    };
    let control_precedence = ControlPrecedence {
        documented_default: None,
        user_preference: preference_state.clone(),
        local_policy: if attribution.authority == Authority::LocalPolicy {
            policy_state.clone()
        } else {
            None
        },
        domain_policy: if attribution.authority == Authority::DomainPolicy {
            policy_state.clone()
        } else {
            None
        },
        mdm_policy: if attribution.authority == Authority::Mdm {
            policy_state.clone()
        } else {
            None
        },
        effective_state,
        effective_authority: attribution.clone(),
        conflicting_evidence: conflict,
    };
    let mut warnings = Vec::new();
    if conflict {
        warnings.push(
            "User preference conflicts with policy; the policy-derived state is effective".into(),
        );
    }
    if policy_value.is_some() && !attribution.exact_source_proven {
        warnings.push("The policy value is effective evidence, but its exact management source is inferred rather than proven".into());
    }
    if !applicability.policy_supported && policy_value.is_some() {
        warnings.push("A policy path exists on an edition where the documented policy is unsupported; the value is not treated as effective".into());
    }
    DetectionResult {
        component_id: id,
        current,
        authority: attribution.authority,
        platform: info.clone(),
        applicable: matches!(
            applicability.status,
            ApplicabilityStatus::Applicable | ApplicabilityStatus::PartiallyApplicable
        ),
        applicability,
        evidence: vec![evidence(
            QueryId::PolicyRegistry,
            "Policy and current-user preference shared context",
            format!(
                "policy={policy_state:?}; preference={preference_state:?}; exact source proven={}",
                attribution.exact_source_proven
            ),
            if attribution.exact_source_proven {
                95
            } else {
                70
            },
        )],
        detected_at: timestamp(),
        error: None,
        warnings,
        package_identities: Vec::new(),
        policy_state,
        preference_state,
        provisioning_state: None,
        detector_status: DetectorStatus::Successful,
        authority_attribution: attribution,
        control_precedence,
        package_completeness: PackageCompleteness::NotApplicable,
        packages: Vec::new(),
    }
}

fn task_view_detection(info: &PlatformInfo, context: &SharedContext) -> DetectionResult {
    let id = ComponentId::TaskbarTaskView;
    let applicability = applicability::evaluate(id, info);
    if matches!(
        applicability.status,
        ApplicabilityStatus::UnsupportedBuild
            | ApplicabilityStatus::UnsupportedEdition
            | ApplicabilityStatus::Removed
            | ApplicabilityStatus::MissingPrerequisite
    ) {
        return unsupported(id, info, applicability);
    }
    let preferences = match query_state(context, QueryId::UserPreferences) {
        Ok(value) => value,
        Err(error) => {
            return base_unknown(
                id,
                info,
                DetectorStatus::Failed,
                &error.message,
                QueryId::UserPreferences,
                error.kind,
            );
        }
    };
    let policies = match query_state(context, QueryId::PolicyRegistry) {
        Ok(value) => value,
        Err(error) => {
            return base_unknown(
                id,
                info,
                DetectorStatus::Failed,
                &error.message,
                QueryId::PolicyRegistry,
                error.kind,
            );
        }
    };
    let read_optional_dword = |value: &serde_json::Value, field: &str| {
        let raw = value.get(field).unwrap_or(&serde_json::Value::Null);
        if raw.is_null() {
            Ok(None)
        } else if matches!(raw.as_i64(), Some(0 | 1)) {
            Ok(raw.as_i64())
        } else {
            Err(())
        }
    };
    let preference = match read_optional_dword(preferences, "TaskbarTaskView") {
        Ok(value) => value,
        Err(()) => {
            return base_unknown(
                id,
                info,
                DetectorStatus::Unknown,
                "ShowTaskViewButton was not a supported DWORD value; no effective state was inferred",
                QueryId::UserPreferences,
                QueryErrorKind::SchemaMismatch,
            );
        }
    };
    let machine_hide = read_optional_dword(policies, "TaskViewMachineHide");
    let user_hide = read_optional_dword(policies, "TaskViewUserHide");
    let machine_locked = read_optional_dword(policies, "TaskbarMachineLocked");
    let user_locked = read_optional_dword(policies, "TaskbarUserLocked");
    if machine_hide.is_err()
        || user_hide.is_err()
        || machine_locked.is_err()
        || user_locked.is_err()
    {
        return base_unknown(
            id,
            info,
            DetectorStatus::Unknown,
            "A Task View policy control was not a supported DWORD value; authority is unknown",
            QueryId::PolicyRegistry,
            QueryErrorKind::SchemaMismatch,
        );
    }
    let machine_hide = machine_hide.ok().flatten();
    let user_hide = user_hide.ok().flatten();
    let taskbar_locked =
        machine_locked.ok().flatten() == Some(1) || user_locked.ok().flatten() == Some(1);
    let hide_policy = machine_hide.or(user_hide);
    let policy_present = hide_policy.is_some() || taskbar_locked;
    let preference_enabled = preference.is_none_or(|value| value == 1);
    let effective_enabled = hide_policy.map_or(preference_enabled, |hide| hide == 0);
    let preference_state = Some(
        if preference_enabled {
            "Shown"
        } else {
            "Hidden"
        }
        .into(),
    );
    let policy_state = hide_policy
        .map(|hide| if hide == 1 { "Hidden" } else { "Shown" }.into())
        .or_else(|| taskbar_locked.then(|| "Taskbar changes locked".into()));
    let attribution = if !policy_present && preference.is_none() {
        AuthorityAttribution {
            authority: Authority::User,
            confidence: AuthorityConfidence::Confirmed,
            exact_source_proven: true,
            evidence: vec![
                "The current-user value is absent and the documented shown default applies".into(),
            ],
            alternatives: Vec::new(),
        }
    } else {
        management_attribution(context, policy_present, preference.is_some())
    };
    let conflict = hide_policy.is_some() && effective_enabled != preference_enabled;
    let current = if let Some(hide) = hide_policy {
        State::Policy {
            configured: true,
            enabled: hide == 0,
        }
    } else {
        State::UserPreference {
            enabled: preference_enabled,
        }
    };
    let mut warnings = Vec::new();
    if conflict {
        warnings.push(
            "The current-user Task View preference conflicts with policy; policy is effective"
                .into(),
        );
    }
    if taskbar_locked {
        warnings.push("NoSetTaskbar blocks direct Task View changes for this account".into());
    }
    if machine_hide.is_some() && user_hide.is_some() && machine_hide != user_hide {
        warnings.push("Machine and user HideTaskViewButton policy values conflict; machine policy is treated as effective".into());
    }
    DetectionResult {
        component_id: id,
        current,
        authority: attribution.authority,
        platform: info.clone(),
        applicable: matches!(
            applicability.status,
            ApplicabilityStatus::Applicable | ApplicabilityStatus::PartiallyApplicable
        ),
        applicability,
        evidence: vec![evidence(
            QueryId::UserPreferences,
            "Task View current-user preference and fixed policy controls",
            format!(
                "preference={preference_state:?}; policy={policy_state:?}; documented default=Shown"
            ),
            if policy_present { 85 } else { 95 },
        )],
        detected_at: timestamp(),
        error: None,
        warnings,
        package_identities: Vec::new(),
        policy_state: policy_state.clone(),
        preference_state: preference_state.clone(),
        provisioning_state: None,
        detector_status: DetectorStatus::Successful,
        authority_attribution: attribution.clone(),
        control_precedence: ControlPrecedence {
            documented_default: Some("Shown".into()),
            user_preference: preference_state,
            local_policy: if attribution.authority == Authority::LocalPolicy {
                policy_state.clone()
            } else {
                None
            },
            domain_policy: if attribution.authority == Authority::DomainPolicy {
                policy_state.clone()
            } else {
                None
            },
            mdm_policy: if attribution.authority == Authority::Mdm {
                policy_state.clone()
            } else {
                None
            },
            effective_state: Some(if effective_enabled { "Shown" } else { "Hidden" }.into()),
            effective_authority: attribution,
            conflicting_evidence: conflict,
        },
        package_completeness: PackageCompleteness::NotApplicable,
        packages: Vec::new(),
    }
}

fn onedrive_detection(info: &PlatformInfo, context: &SharedContext) -> DetectionResult {
    let applicability = applicability::evaluate(ComponentId::Onedrive, info);
    let value = match query_state(context, QueryId::OneDriveMetadata) {
        Ok(value) => value,
        Err(error) => {
            return base_unknown(
                ComponentId::Onedrive,
                info,
                if error.kind == QueryErrorKind::Cancellation {
                    DetectorStatus::Cancelled
                } else {
                    DetectorStatus::Failed
                },
                &error.message,
                QueryId::OneDriveMetadata,
                error.kind,
            );
        }
    };
    let Some(installed) = value["Installed"].as_bool() else {
        return base_unknown(
            ComponentId::Onedrive,
            info,
            DetectorStatus::Failed,
            "OneDrive query returned an incomplete object",
            QueryId::OneDriveMetadata,
            QueryErrorKind::SchemaMismatch,
        );
    };
    let root_count = value["SyncRootCount"].as_u64().unwrap_or(0) as u32;
    let policy_value = value["FilesOnDemandPolicy"].as_i64();
    let account_values: Vec<i64> = value["FilesOnDemandValues"]
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default();
    let evidence_complete = value["FilesOnDemandEvidenceComplete"]
        .as_bool()
        .unwrap_or(false);
    let files_on_demand = if !installed {
        FilesOnDemandState::NotInstalled
    } else if root_count == 0 {
        FilesOnDemandState::InstalledUnlinked
    } else if policy_value.is_some() {
        FilesOnDemandState::PolicyEnforced
    } else if !evidence_complete {
        FilesOnDemandState::DetectionIncomplete
    } else if account_values.iter().all(|value| *value != 0) {
        FilesOnDemandState::SupportedEnabled
    } else if account_values.iter().all(|value| *value == 0) {
        FilesOnDemandState::SupportedDisabled
    } else {
        FilesOnDemandState::MixedOrPerRootUnknown
    };
    let attribution = management_attribution(
        context,
        value["KfmPolicy"].as_bool().unwrap_or(false) || policy_value.is_some(),
        true,
    );
    let partial = matches!(
        files_on_demand,
        FilesOnDemandState::DetectionIncomplete | FilesOnDemandState::MixedOrPerRootUnknown
    );
    DetectionResult {
        component_id: ComponentId::Onedrive,
        current: State::OneDrive {
            installed,
            version: value["Version"].as_str().map(str::to_owned),
            running: value["Running"].as_bool().unwrap_or(false),
            startup: value["Startup"].as_bool().unwrap_or(false),
            personal_account: value["Personal"].as_bool().unwrap_or(false),
            work_account: value["Work"].as_bool().unwrap_or(false),
            sync_root_count: root_count,
            desktop_redirected: value["DesktopRedirected"].as_bool().unwrap_or(false),
            documents_redirected: value["DocumentsRedirected"].as_bool().unwrap_or(false),
            pictures_redirected: value["PicturesRedirected"].as_bool().unwrap_or(false),
            kfm_policy_configured: value["KfmPolicy"].as_bool().unwrap_or(false),
            files_on_demand,
            partial,
        },
        authority: attribution.authority,
        platform: info.clone(),
        applicable: true,
        applicability,
        evidence: vec![evidence(
            QueryId::OneDriveMetadata,
            "OneDrive client, account metadata, shell redirection and policy",
            format!(
                "linked roots={root_count}; Files On-Demand={files_on_demand:?}; no account identity, path, or filename was retained"
            ),
            if partial { 60 } else { 90 },
        )],
        detected_at: timestamp(),
        error: None,
        warnings: if partial {
            vec!["Files On-Demand has no reliable single global representation for every linked root; a composite state is shown".into()]
        } else {
            Vec::new()
        },
        package_identities: Vec::new(),
        policy_state: policy_value.map(|value| format!("Files On-Demand policy value {value}")),
        preference_state: Some(format!("{files_on_demand:?}")),
        provisioning_state: None,
        detector_status: if partial {
            DetectorStatus::Unknown
        } else {
            DetectorStatus::Successful
        },
        authority_attribution: attribution.clone(),
        control_precedence: ControlPrecedence {
            documented_default: None,
            user_preference: Some(format!("{files_on_demand:?}")),
            local_policy: None,
            domain_policy: None,
            mdm_policy: None,
            effective_state: Some(format!("{files_on_demand:?}")),
            effective_authority: attribution,
            conflicting_evidence: false,
        },
        package_completeness: PackageCompleteness::NotApplicable,
        packages: Vec::new(),
    }
}

pub fn run_inspection<F: FnMut(InspectionProgress)>(
    runner: &dyn ReadOnlyRunner,
    inspection_id: String,
    cancellation: &CancellationToken,
    mut progress: F,
) -> InspectionOutcome {
    let catalogue = crate::platform::v1_catalogue();
    let mut lifecycle = InspectionLifecycle::new(inspection_id, catalogue.len());
    progress(lifecycle.progress(None, None));
    lifecycle.phase = InspectionPhase::GatheringSharedContext;
    lifecycle.status = "Gathering shared read-only Windows context".into();
    lifecycle.last_progress_update = timestamp();
    progress(lifecycle.progress(None, None));
    let mut context = SharedContext::default();
    for query_id in SHARED_QUERY_IDS {
        *context.executions.entry(query_id).or_default() += 1;
        let result = if cancellation.is_cancelled() {
            Err(failure(
                query_id,
                QueryErrorKind::Cancellation,
                "Inspection cancellation was requested",
                0,
            ))
        } else {
            runner.run(query_id, cancellation).and_then(parse_query)
        };
        let essential_failure = result
            .as_ref()
            .err()
            .is_some_and(|_| query_config(query_id).essential);
        context.values.insert(query_id, result);
        if essential_failure {
            break;
        }
        if cancellation.is_cancelled() {
            break;
        }
    }
    let inventory = context
        .values
        .get(&QueryId::PlatformInventory)
        .and_then(|result| result.as_ref().ok());
    let mut platform = parse_inventory(inventory);
    if let Ok(management) = query_state(&context, QueryId::ManagementContext) {
        platform.domain_joined = management["DomainJoined"].as_bool();
        platform.entra_joined = management["EntraJoined"].as_bool();
        platform.workplace_joined = management["WorkplaceJoined"].as_bool();
        platform.mdm_enrolled = Some(management["MdmEnrollmentCount"].as_u64().unwrap_or(0) > 0);
    }
    lifecycle.phase = if cancellation.is_cancelled() {
        InspectionPhase::Cancelling
    } else {
        InspectionPhase::RunningDetectors
    };
    lifecycle.status = if cancellation.is_cancelled() {
        "Cancelling queued detectors".into()
    } else {
        "Running component detectors from shared context".into()
    };
    lifecycle.last_progress_update = timestamp();
    progress(lifecycle.progress(None, None));
    let mut observations = Vec::with_capacity(catalogue.len());
    for definition in catalogue {
        lifecycle.current_detector_ids = vec![definition.id.key().into()];
        let observation = if cancellation.is_cancelled() {
            cancelled(definition.id, &platform)
        } else if definition.is_package {
            package_detection(definition.id, &platform, &context)
        } else if definition.id == ComponentId::Onedrive {
            onedrive_detection(&platform, &context)
        } else {
            setting_detection(definition.id, &platform, &context)
        };
        lifecycle.completed_detector_count += 1;
        lifecycle.warning_count += observation.warnings.len();
        lifecycle.error_count += usize::from(observation.error.is_some());
        match observation.detector_status {
            DetectorStatus::Successful => lifecycle.successful_detector_count += 1,
            DetectorStatus::Unknown | DetectorStatus::NotRun => {
                lifecycle.unknown_detector_count += 1
            }
            DetectorStatus::Failed => lifecycle.failed_detector_count += 1,
            DetectorStatus::Cancelled => lifecycle.cancelled_detector_count += 1,
        }
        lifecycle.status = format!("Completed detector {}", definition.id.key());
        lifecycle.last_progress_update = timestamp();
        progress(lifecycle.progress(
            Some(definition.id.key().into()),
            Some(observation.detector_status),
        ));
        observations.push(observation);
    }
    lifecycle.current_detector_ids.clear();
    lifecycle.cancellation_requested = cancellation.is_cancelled();
    let query_failures = context
        .values
        .values()
        .filter_map(|result| result.as_ref().err().cloned())
        .collect();
    InspectionOutcome {
        lifecycle,
        platform,
        observations,
        query_failures,
        query_execution_counts: context.executions,
    }
}

pub fn complete_lifecycle(lifecycle: &mut InspectionLifecycle, persistence_failed: bool) {
    lifecycle.completed_at = Some(timestamp());
    lifecycle.last_progress_update = timestamp();
    lifecycle.phase = if persistence_failed {
        InspectionPhase::Failed
    } else if lifecycle.cancellation_requested || lifecycle.cancelled_detector_count > 0 {
        InspectionPhase::Cancelled
    } else if lifecycle.failed_detector_count > 0 || lifecycle.unknown_detector_count > 0 {
        InspectionPhase::CompletedWithPartialFailures
    } else {
        InspectionPhase::Completed
    };
    lifecycle.status = match lifecycle.phase {
        InspectionPhase::Completed => "Inspection completed successfully",
        InspectionPhase::CompletedWithPartialFailures => {
            "Inspection completed with partial failures or incomplete evidence"
        }
        InspectionPhase::Cancelled => {
            "Inspection cancelled; completed detector evidence was preserved"
        }
        InspectionPhase::Failed => "Inspection failed while preserving the previous database state",
        _ => "Inspection finished",
    }
    .into();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn widgets_detection(preference: serde_json::Value) -> DetectionResult {
        let mut values = BTreeMap::new();
        values.insert(
            QueryId::PolicyRegistry,
            Ok(serde_json::json!({ "Widgets": null })),
        );
        values.insert(
            QueryId::UserPreferences,
            Ok(serde_json::json!({ "TaskbarWidgets": preference })),
        );
        setting_detection(
            ComponentId::TaskbarWidgets,
            &default_platform(),
            &SharedContext {
                values,
                executions: BTreeMap::new(),
            },
        )
    }

    fn cleanup_detection(
        component_id: ComponentId,
        policy_field: &str,
        policy: serde_json::Value,
        preference_field: &str,
        preference: serde_json::Value,
    ) -> DetectionResult {
        let mut policy_values = serde_json::Map::new();
        policy_values.insert(policy_field.into(), policy);
        let mut preference_values = serde_json::Map::new();
        preference_values.insert(preference_field.into(), preference);
        let mut values = BTreeMap::new();
        values.insert(
            QueryId::PolicyRegistry,
            Ok(serde_json::Value::Object(policy_values)),
        );
        values.insert(
            QueryId::UserPreferences,
            Ok(serde_json::Value::Object(preference_values)),
        );
        setting_detection(
            component_id,
            &default_platform(),
            &SharedContext {
                values,
                executions: BTreeMap::new(),
            },
        )
    }

    #[test]
    fn m3_cleanup_detectors_enforce_exact_binary_policy_and_preference_contracts() {
        let contracts = [
            (ComponentId::WelcomeExperience, "Welcome", "Welcome"),
            (ComponentId::TipsSuggestions, "Tips", "Tips"),
            (
                ComponentId::NotificationSuggestions,
                "NotificationSuggestions",
                "NotificationSuggestions",
            ),
            (
                ComponentId::SettingsSuggestedContent,
                "SettingsSuggestions",
                "SettingsSuggestions",
            ),
        ];
        for (component_id, policy_field, preference_field) in contracts {
            let enabled = cleanup_detection(
                component_id,
                policy_field,
                serde_json::Value::Null,
                preference_field,
                serde_json::json!(1),
            );
            assert!(matches!(
                enabled.current,
                State::UserPreference { enabled: true }
            ));
            let disabled = cleanup_detection(
                component_id,
                policy_field,
                serde_json::Value::Null,
                preference_field,
                serde_json::json!(0),
            );
            assert!(matches!(
                disabled.current,
                State::UserPreference { enabled: false }
            ));
            let missing = cleanup_detection(
                component_id,
                policy_field,
                serde_json::Value::Null,
                preference_field,
                serde_json::Value::Null,
            );
            assert_eq!(missing.detector_status, DetectorStatus::Unknown);
            for invalid in [serde_json::json!(2), serde_json::json!("1")] {
                let invalid_preference = cleanup_detection(
                    component_id,
                    policy_field,
                    serde_json::Value::Null,
                    preference_field,
                    invalid.clone(),
                );
                assert_eq!(invalid_preference.detector_status, DetectorStatus::Unknown);
                let invalid_policy = cleanup_detection(
                    component_id,
                    policy_field,
                    invalid,
                    preference_field,
                    serde_json::json!(1),
                );
                assert_eq!(invalid_policy.detector_status, DetectorStatus::Unknown);
            }
            let disabled_by_policy = cleanup_detection(
                component_id,
                policy_field,
                serde_json::json!(1),
                preference_field,
                serde_json::json!(1),
            );
            assert!(matches!(
                disabled_by_policy.current,
                State::Policy {
                    configured: true,
                    enabled: false
                }
            ));
        }
    }

    #[cfg(windows)]
    #[test]
    fn m3_cleanup_detector_queries_use_the_same_fixed_handler_values() {
        let preferences = powershell_script(QueryId::UserPreferences);
        let policies = powershell_script(QueryId::PolicyRegistry);
        for value in [
            "SubscribedContent-310093Enabled",
            "SoftLandingEnabled",
            "SubscribedContent-338389Enabled",
            "SubscribedContent-338393Enabled",
        ] {
            assert!(preferences.contains(value), "missing preference {value}");
        }
        for value in [
            "DisableWindowsSpotlightWindowsWelcomeExperience",
            "DisableSoftLanding",
            "DisableWindowsSpotlightOnActionCenter",
            "DisableWindowsSpotlightOnSettings",
        ] {
            assert!(policies.contains(value), "missing policy {value}");
        }
    }

    fn task_view_fixture(
        preference: serde_json::Value,
        policies: serde_json::Value,
    ) -> DetectionResult {
        let mut values = BTreeMap::new();
        values.insert(QueryId::PolicyRegistry, Ok(policies));
        values.insert(
            QueryId::UserPreferences,
            Ok(serde_json::json!({ "TaskbarTaskView": preference })),
        );
        task_view_detection(
            &default_platform(),
            &SharedContext {
                values,
                executions: BTreeMap::new(),
            },
        )
    }

    #[test]
    fn task_view_detector_matches_fixed_preference_default_and_policy_contract() {
        let no_policy = serde_json::json!({
            "TaskViewMachineHide": null,
            "TaskViewUserHide": null,
            "TaskbarMachineLocked": null,
            "TaskbarUserLocked": null
        });
        for (preference, enabled) in [
            (serde_json::json!(0), false),
            (serde_json::json!(1), true),
            (serde_json::Value::Null, true),
        ] {
            let result = task_view_fixture(preference, no_policy.clone());
            assert_eq!(result.detector_status, DetectorStatus::Successful);
            assert!(
                matches!(result.current, State::UserPreference { enabled: value } if value == enabled)
            );
            assert_eq!(
                result.control_precedence.documented_default.as_deref(),
                Some("Shown")
            );
        }
        for invalid in [serde_json::json!(2), serde_json::json!("0")] {
            let result = task_view_fixture(invalid, no_policy.clone());
            assert_eq!(result.detector_status, DetectorStatus::Unknown);
            assert!(matches!(result.current, State::Unknown { .. }));
        }
        let hidden_by_policy = task_view_fixture(
            serde_json::json!(1),
            serde_json::json!({
                "TaskViewMachineHide": 1,
                "TaskViewUserHide": null,
                "TaskbarMachineLocked": null,
                "TaskbarUserLocked": null
            }),
        );
        assert!(matches!(
            hidden_by_policy.current,
            State::Policy {
                configured: true,
                enabled: false
            }
        ));
        assert_eq!(hidden_by_policy.authority, Authority::LocalPolicy);
    }

    #[test]
    fn taskbar_widgets_detector_does_not_infer_absence_or_invalid_values_as_compliance() {
        let absent = widgets_detection(serde_json::Value::Null);
        assert_eq!(absent.detector_status, DetectorStatus::Unknown);
        assert!(matches!(absent.current, State::Unknown { .. }));

        let disabled = widgets_detection(serde_json::json!(0));
        assert_eq!(disabled.detector_status, DetectorStatus::Successful);
        assert!(matches!(
            disabled.current,
            State::UserPreference { enabled: false }
        ));

        let enabled = widgets_detection(serde_json::json!(1));
        assert_eq!(enabled.detector_status, DetectorStatus::Successful);
        assert!(matches!(
            enabled.current,
            State::UserPreference { enabled: true }
        ));

        for invalid in [serde_json::json!(2), serde_json::json!("1")] {
            let detection = widgets_detection(invalid);
            assert_eq!(detection.detector_status, DetectorStatus::Unknown);
            assert!(matches!(detection.current, State::Unknown { .. }));
        }
    }

    struct FixtureRunner {
        counts: Mutex<BTreeMap<QueryId, usize>>,
        cancel_on: Option<QueryId>,
        token: CancellationToken,
    }
    impl FixtureRunner {
        fn new(cancel_on: Option<QueryId>, token: CancellationToken) -> Self {
            Self {
                counts: Mutex::new(BTreeMap::new()),
                cancel_on,
                token,
            }
        }
    }
    impl ReadOnlyRunner for FixtureRunner {
        fn run(
            &self,
            query_id: QueryId,
            _: &CancellationToken,
        ) -> Result<CommandResult, QueryFailure> {
            *self
                .counts
                .lock()
                .map_err(|_| failure(query_id, QueryErrorKind::SpawnFailure, "test lock", 0))?
                .entry(query_id)
                .or_default() += 1;
            if self.cancel_on == Some(query_id) {
                self.token.cancel();
                return Err(failure(
                    query_id,
                    QueryErrorKind::Cancellation,
                    "cancelled fixture query",
                    1,
                ));
            }
            let stdout = match query_id {
                QueryId::PlatformInventory => r#"{"ProductName":"Windows 11 Pro","Edition":"Professional","Build":26100,"DisplayVersion":"24H2","UBR":1,"Architecture":"64-bit","DeviceName":"TEST-PC","DomainJoined":false,"Windows11":true}"#,
                QueryId::AppxCurrentUser | QueryId::AppxAllUsers => r#"[{"Name":"Microsoft.Copilot","PackageFamilyName":"Microsoft.Copilot_8wekyb3d8bbwe","PackageFullName":"Microsoft.Copilot_1.0.0.0_x64__8wekyb3d8bbwe","Version":"1.0.0.0","Architecture":"X64","PublisherId":"8wekyb3d8bbwe","InstallLocation":"C:\\Program Files\\WindowsApps\\redacted"}]"#,
                QueryId::AppxProvisioned => "[]", QueryId::ManagementContext => r#"{"DomainJoined":false,"WorkplaceJoined":true,"MdmEnrollmentCount":0,"MdmPolicyProviderCount":0,"DomainGpoHistoryCount":0,"LocalPolicyStorePresent":false}"#,
                QueryId::PolicyRegistry => r#"{"Widgets":1,"SearchWeb":1}"#, QueryId::UserPreferences => r#"{"TaskbarWidgets":1,"TaskbarSearch":0}"#,
                QueryId::OneDriveMetadata => r#"{"Installed":true,"Version":"1","Running":false,"Startup":true,"Personal":true,"Work":false,"SyncRootCount":1,"DesktopRedirected":false,"DocumentsRedirected":false,"PicturesRedirected":false,"KfmPolicy":false,"FilesOnDemandPolicy":null,"FilesOnDemandValues":[1],"FilesOnDemandEvidenceComplete":true}"#,
            }.to_owned();
            Ok(CommandResult {
                query_id,
                exit_code: Some(0),
                stdout,
                duration_ms: 1,
            })
        }
    }

    #[test]
    fn shared_queries_execute_once_for_twenty_one_detectors() {
        let token = CancellationToken::default();
        let runner = FixtureRunner::new(None, token.clone());
        let outcome = run_inspection(&runner, "test".into(), &token, |_| {});
        assert_eq!(outcome.observations.len(), 21);
        assert!(
            outcome
                .query_execution_counts
                .values()
                .all(|count| *count == 1)
        );
        assert_eq!(
            outcome
                .observations
                .iter()
                .find(|result| result.component_id == ComponentId::ConsumerCopilot)
                .map(|result| result.package_completeness),
            Some(PackageCompleteness::Complete)
        );
    }

    #[test]
    fn progress_ordering_and_counts_are_monotonic() {
        let token = CancellationToken::default();
        let runner = FixtureRunner::new(None, token.clone());
        let events = Mutex::new(Vec::new());
        let mut outcome = run_inspection(&runner, "progress".into(), &token, |event| {
            events.lock().unwrap().push(event)
        });
        complete_lifecycle(&mut outcome.lifecycle, false);
        let events = events.into_inner().unwrap();
        assert_eq!(
            events.first().map(|event| event.phase),
            Some(InspectionPhase::Preparing)
        );
        assert!(
            events
                .windows(2)
                .all(|pair| pair[0].completed_work <= pair[1].completed_work)
        );
        assert_eq!(outcome.lifecycle.completed_detector_count, 21);
    }

    #[test]
    fn cancel_during_appx_preserves_cancelled_result_status() {
        let token = CancellationToken::default();
        let runner = FixtureRunner::new(Some(QueryId::AppxAllUsers), token.clone());
        let mut outcome = run_inspection(&runner, "cancel".into(), &token, |_| {});
        complete_lifecycle(&mut outcome.lifecycle, false);
        assert_eq!(outcome.lifecycle.phase, InspectionPhase::Cancelled);
        assert_eq!(outcome.lifecycle.cancelled_detector_count, 21);
    }

    #[test]
    fn double_cancellation_is_idempotent() {
        let token = CancellationToken::default();
        assert!(token.cancel());
        assert!(!token.cancel());
    }

    #[test]
    fn all_user_permission_denied_is_not_absence() {
        struct PermissionRunner(FixtureRunner);
        impl ReadOnlyRunner for PermissionRunner {
            fn run(
                &self,
                id: QueryId,
                token: &CancellationToken,
            ) -> Result<CommandResult, QueryFailure> {
                if id == QueryId::AppxAllUsers {
                    Err(failure(
                        id,
                        QueryErrorKind::PermissionDenied,
                        "Access is denied",
                        1,
                    ))
                } else {
                    self.0.run(id, token)
                }
            }
        }
        let token = CancellationToken::default();
        let runner = PermissionRunner(FixtureRunner::new(None, token.clone()));
        let outcome = run_inspection(&runner, "permission".into(), &token, |_| {});
        let package = outcome
            .observations
            .iter()
            .find(|result| result.component_id == ComponentId::ConsumerCopilot)
            .unwrap();
        assert!(matches!(
            package.current,
            State::Package {
                all_users: PackageRegistrationState::PermissionLimited,
                ..
            }
        ));
        assert_ne!(package.package_completeness, PackageCompleteness::Complete);
    }

    #[test]
    fn provisioning_permission_denied_is_separate_from_all_user_state() {
        struct ProvisioningPermissionRunner(FixtureRunner);
        impl ReadOnlyRunner for ProvisioningPermissionRunner {
            fn run(
                &self,
                id: QueryId,
                token: &CancellationToken,
            ) -> Result<CommandResult, QueryFailure> {
                if id == QueryId::AppxProvisioned {
                    Err(failure(
                        id,
                        QueryErrorKind::PermissionDenied,
                        "Access is denied",
                        1,
                    ))
                } else {
                    self.0.run(id, token)
                }
            }
        }
        let token = CancellationToken::default();
        let outcome = run_inspection(
            &ProvisioningPermissionRunner(FixtureRunner::new(None, token.clone())),
            "provisioning-permission".into(),
            &token,
            |_| {},
        );
        let package = outcome
            .observations
            .iter()
            .find(|result| result.component_id == ComponentId::ConsumerCopilot)
            .unwrap();
        assert!(matches!(
            package.current,
            State::Package {
                all_users: PackageRegistrationState::Present,
                provisioned: PackageProvisioningState::PermissionLimited,
                ..
            }
        ));
        assert_eq!(
            package.package_completeness,
            PackageCompleteness::RegistrationCompleteProvisioningUnknown
        );
    }

    #[test]
    fn cancellation_token_can_start_a_new_independent_inspection() {
        let cancelled = CancellationToken::default();
        cancelled.cancel();
        let fresh = CancellationToken::default();
        let runner = FixtureRunner::new(None, fresh.clone());
        let outcome = run_inspection(&runner, "fresh".into(), &fresh, |_| {});
        assert_eq!(outcome.lifecycle.cancelled_detector_count, 0);
    }

    #[test]
    fn cancel_before_detectors_marks_every_detector_cancelled() {
        let token = CancellationToken::default();
        token.cancel();
        let runner = FixtureRunner::new(None, token.clone());
        let mut outcome = run_inspection(&runner, "cancel-before".into(), &token, |_| {});
        complete_lifecycle(&mut outcome.lifecycle, false);
        assert_eq!(outcome.lifecycle.cancelled_detector_count, 21);
        assert_eq!(outcome.lifecycle.phase, InspectionPhase::Cancelled);
    }

    #[test]
    fn cancel_between_detector_results_cancels_queued_work() {
        let token = CancellationToken::default();
        let runner = FixtureRunner::new(None, token.clone());
        let callback_token = token.clone();
        let outcome = run_inspection(&runner, "cancel-between".into(), &token, move |event| {
            if event.completed_work == 5 {
                callback_token.cancel();
            }
        });
        assert_eq!(outcome.lifecycle.completed_detector_count, 21);
        assert!(outcome.lifecycle.cancelled_detector_count >= 15);
        assert!(outcome.lifecycle.successful_detector_count > 0);
    }

    #[test]
    fn timeout_is_structured_and_never_package_absence() {
        struct TimeoutRunner(FixtureRunner);
        impl ReadOnlyRunner for TimeoutRunner {
            fn run(
                &self,
                id: QueryId,
                token: &CancellationToken,
            ) -> Result<CommandResult, QueryFailure> {
                if id == QueryId::AppxAllUsers {
                    Err(failure(
                        id,
                        QueryErrorKind::Timeout,
                        "bounded timeout",
                        45_000,
                    ))
                } else {
                    self.0.run(id, token)
                }
            }
        }
        let token = CancellationToken::default();
        let outcome = run_inspection(
            &TimeoutRunner(FixtureRunner::new(None, token.clone())),
            "timeout".into(),
            &token,
            |_| {},
        );
        let package = outcome
            .observations
            .iter()
            .find(|value| value.component_id == ComponentId::ConsumerCopilot)
            .unwrap();
        assert!(matches!(
            package.current,
            State::Package {
                all_users: PackageRegistrationState::QueryFailed,
                ..
            }
        ));
        assert!(
            outcome
                .query_failures
                .iter()
                .any(|failure| failure.kind == QueryErrorKind::Timeout)
        );
    }

    fn authority_context(management: Value) -> SharedContext {
        let mut context = SharedContext::default();
        context
            .values
            .insert(QueryId::ManagementContext, Ok(management));
        context
    }

    #[test]
    fn authority_distinguishes_local_domain_mdm_and_user_evidence() {
        let local = management_attribution(
            &authority_context(
                serde_json::json!({"LocalPolicyStorePresent":true,"DomainGpoHistoryCount":0,"MdmEnrollmentCount":0,"MdmPolicyProviderCount":0}),
            ),
            true,
            false,
        );
        assert_eq!(local.authority, Authority::LocalPolicy);
        let domain = management_attribution(
            &authority_context(
                serde_json::json!({"LocalPolicyStorePresent":true,"DomainGpoHistoryCount":2,"MdmEnrollmentCount":0,"MdmPolicyProviderCount":0}),
            ),
            true,
            false,
        );
        assert_eq!(domain.authority, Authority::DomainPolicy);
        assert!(!domain.exact_source_proven);
        let mdm = management_attribution(
            &authority_context(
                serde_json::json!({"LocalPolicyStorePresent":false,"DomainGpoHistoryCount":0,"MdmEnrollmentCount":1,"MdmPolicyProviderCount":1}),
            ),
            true,
            false,
        );
        assert_eq!(mdm.authority, Authority::Mdm);
        let user = management_attribution(
            &authority_context(
                serde_json::json!({"WorkplaceJoined":true,"DomainGpoHistoryCount":0,"MdmEnrollmentCount":0,"MdmPolicyProviderCount":0}),
            ),
            false,
            true,
        );
        assert_eq!(user.authority, Authority::User);
        assert_eq!(user.confidence, AuthorityConfidence::Confirmed);
    }

    #[test]
    fn domain_membership_or_work_account_alone_never_claims_management() {
        let domain_only = management_attribution(
            &authority_context(
                serde_json::json!({"DomainJoined":true,"DomainGpoHistoryCount":0,"MdmEnrollmentCount":0,"MdmPolicyProviderCount":0}),
            ),
            true,
            false,
        );
        assert_ne!(domain_only.authority, Authority::DomainPolicy);
        let work_only = management_attribution(
            &authority_context(
                serde_json::json!({"WorkplaceJoined":true,"DomainGpoHistoryCount":0,"MdmEnrollmentCount":0,"MdmPolicyProviderCount":0}),
            ),
            true,
            false,
        );
        assert_ne!(work_only.authority, Authority::Mdm);
    }

    #[test]
    fn bare_enrollment_metadata_without_active_corroboration_is_not_mdm() {
        let attribution = management_attribution(
            &authority_context(serde_json::json!({
                "LocalPolicyStorePresent": false,
                "DomainGpoHistoryCount": 0,
                "MdmEnrollmentCount": 0,
                "MdmEnrollmentMetadataCount": 3,
                "MdmPolicyProviderCount": 15
            })),
            true,
            false,
        );
        assert_eq!(attribution.authority, Authority::LocalPolicy);
        assert_eq!(attribution.confidence, AuthorityConfidence::Weak);
        assert!(!attribution.exact_source_proven);
        assert!(attribution.alternatives.contains(&Authority::Mdm));
    }

    #[cfg(windows)]
    #[test]
    fn management_query_requires_active_mdm_corroboration() {
        let script = powershell_script(QueryId::ManagementContext);
        assert!(script.contains("MdmEnrollmentMetadataCount=$metadata.Count"));
        assert!(script.contains("Provisioning\\OMADM\\Accounts"));
        assert!(script.contains("Windows\\EnterpriseMgmt"));
        assert!(script.contains("DMPCertThumbPrint"));
        assert!(script.contains("CloudDomainJoin\\JoinInfo"));
        assert!(script.contains("EntraJoined=($entra.Count -gt 0)"));
        assert!(!script.contains("MdmEnrollmentCount=$metadata.Count"));
    }

    #[test]
    fn conflicting_domain_and_mdm_evidence_remains_explicitly_uncertain() {
        let conflict = management_attribution(
            &authority_context(
                serde_json::json!({"DomainGpoHistoryCount":1,"MdmEnrollmentCount":1,"MdmPolicyProviderCount":1}),
            ),
            true,
            false,
        );
        assert_eq!(conflict.authority, Authority::Unknown);
        assert!(conflict.alternatives.contains(&Authority::DomainPolicy));
        assert!(conflict.alternatives.contains(&Authority::Mdm));
    }

    fn onedrive_fixture(value: Value, build: u32) -> DetectionResult {
        let info = parse_inventory(Some(&serde_json::json!({
            "ProductName": "Windows 11 Pro",
            "Edition": "Professional",
            "Build": build,
            "DisplayVersion": "25H2",
            "UBR": 8875,
            "Architecture": "64-bit",
            "DomainJoined": false,
            "Windows11": true
        })));
        let mut context = SharedContext::default();
        context.values.insert(QueryId::OneDriveMetadata, Ok(value));
        context.values.insert(
            QueryId::ManagementContext,
            Ok(serde_json::json!({
                "DomainJoined": false,
                "WorkplaceJoined": false,
                "MdmEnrollmentCount": 0,
                "MdmPolicyProviderCount": 0,
                "DomainGpoHistoryCount": 0,
                "LocalPolicyStorePresent": false
            })),
        );
        onedrive_detection(&info, &context)
    }

    fn onedrive_value(installed: bool, roots: u64) -> Value {
        serde_json::json!({
            "Installed": installed,
            "Version": null,
            "Running": false,
            "Startup": false,
            "Personal": false,
            "Work": false,
            "SyncRootCount": roots,
            "DesktopRedirected": false,
            "DocumentsRedirected": false,
            "PicturesRedirected": false,
            "KfmPolicy": false,
            "FilesOnDemandPolicy": null,
            "FilesOnDemandValues": [],
            "FilesOnDemandEvidenceComplete": roots == 0
        })
    }

    #[test]
    fn onedrive_missing_registry_branches_is_honest_not_installed_state() {
        let result = onedrive_fixture(onedrive_value(false, 0), 26_200);
        assert!(matches!(
            result.current,
            State::OneDrive {
                installed: false,
                files_on_demand: FilesOnDemandState::NotInstalled,
                ..
            }
        ));
        assert_eq!(result.detector_status, DetectorStatus::Successful);
    }

    #[test]
    fn onedrive_installed_unlinked_or_without_active_account_remains_distinct() {
        let result = onedrive_fixture(onedrive_value(true, 0), 26_200);
        assert!(matches!(
            result.current,
            State::OneDrive {
                installed: true,
                personal_account: false,
                work_account: false,
                files_on_demand: FilesOnDemandState::InstalledUnlinked,
                ..
            }
        ));
    }

    #[test]
    fn onedrive_null_values_under_strict_mode_remain_incomplete() {
        let mut value = onedrive_value(true, 1);
        value["Startup"] = Value::Null;
        value["FilesOnDemandValues"] = serde_json::json!([null]);
        let result = onedrive_fixture(value, 26_200);
        assert!(matches!(
            result.current,
            State::OneDrive {
                startup: false,
                files_on_demand: FilesOnDemandState::DetectionIncomplete,
                partial: true,
                ..
            }
        ));
        assert_eq!(result.detector_status, DetectorStatus::Unknown);
    }

    #[test]
    fn onedrive_non_zero_query_is_failure_not_absence() {
        let info = parse_inventory(Some(&serde_json::json!({
            "ProductName": "Windows 11 Pro",
            "Edition": "Professional",
            "Build": 26200,
            "Windows11": true
        })));
        let mut context = SharedContext::default();
        context.values.insert(
            QueryId::OneDriveMetadata,
            Err(QueryFailure {
                query_id: QueryId::OneDriveMetadata,
                kind: QueryErrorKind::NonZeroExit,
                message: "PowerShell returned a non-zero exit code".into(),
                exit_code: Some(1),
                duration_ms: 1,
                partial_output_available: false,
            }),
        );
        let result = onedrive_detection(&info, &context);
        assert_eq!(result.detector_status, DetectorStatus::Failed);
        assert!(matches!(result.current, State::Unknown { .. }));
    }

    #[cfg(windows)]
    #[test]
    fn onedrive_startup_query_tolerates_a_missing_run_value() {
        let script = powershell_script(QueryId::OneDriveMetadata);
        assert!(script.contains("$run=Get-ItemProperty"));
        assert!(script.contains("Startup=[bool]$run.OneDrive"));
        assert!(!script.contains("Startup=[bool](Get-ItemPropertyValue"));
    }
}
