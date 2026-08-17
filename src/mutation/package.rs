//! Closed, current-user-only AppX/MSIX deployment operations.
//!
//! This module intentionally does not reuse the DWORD captured-state model. Package registration,
//! dependency inventory, deployment verification, and restoration capability are represented as a
//! separate mutation family with its own durable transaction records.

use std::{
    collections::BTreeSet,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::platform::{
    ComponentId, PackageCompleteness, PackageProvisioningState, PackageRegistrationState,
};

use super::{plan::hash_serializable, process_lock::OwnerMutationProcessLock};

pub const PACKAGE_HANDLER_VERSION: &str = "owner-appx.4";
static PACKAGE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)] // The full closed operation names are deliberate audit vocabulary.
pub enum PackageOperationId {
    RemoveConsumerCopilotCurrentUser,
    RemovePhoneLinkCurrentUser,
    RemoveClipchampCurrentUser,
    RemoveSolitaireCurrentUser,
}

impl PackageOperationId {
    #[cfg(test)]
    pub const ALL: [Self; 4] = [
        Self::RemoveConsumerCopilotCurrentUser,
        Self::RemovePhoneLinkCurrentUser,
        Self::RemoveClipchampCurrentUser,
        Self::RemoveSolitaireCurrentUser,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::RemoveConsumerCopilotCurrentUser => "remove_consumer_copilot_current_user",
            Self::RemovePhoneLinkCurrentUser => "remove_phone_link_current_user",
            Self::RemoveClipchampCurrentUser => "remove_clipchamp_current_user",
            Self::RemoveSolitaireCurrentUser => "remove_solitaire_current_user",
        }
    }

    pub const fn component_id(self) -> ComponentId {
        match self {
            Self::RemoveConsumerCopilotCurrentUser => ComponentId::ConsumerCopilot,
            Self::RemovePhoneLinkCurrentUser => ComponentId::PhoneLink,
            Self::RemoveClipchampCurrentUser => ComponentId::Clipchamp,
            Self::RemoveSolitaireCurrentUser => ComponentId::Solitaire,
        }
    }

    pub const fn package_name(self) -> &'static str {
        match self {
            Self::RemoveConsumerCopilotCurrentUser => "Microsoft.Copilot",
            Self::RemovePhoneLinkCurrentUser => "Microsoft.YourPhone",
            Self::RemoveClipchampCurrentUser => "Clipchamp.Clipchamp",
            Self::RemoveSolitaireCurrentUser => "Microsoft.MicrosoftSolitaireCollection",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::RemoveConsumerCopilotCurrentUser => "Microsoft Copilot app",
            Self::RemovePhoneLinkCurrentUser => "Phone Link",
            Self::RemoveClipchampCurrentUser => "Clipchamp",
            Self::RemoveSolitaireCurrentUser => "Microsoft Solitaire Collection",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnerPackageRemoveRequest {
    pub operation_id: PackageOperationId,
    pub source_inspection_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnerPackageRestoreRequest {
    pub transaction_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageRestoreCapability {
    RestoreAvailable,
    ReinstallRequired,
    RemoveUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageActionabilityStatus {
    Ready,
    AlreadyAbsent,
    RemoveUnavailable,
    NeedsScan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDependencyState {
    pub name: String,
    pub package_full_name: String,
    pub package_family_name: String,
    pub version: String,
    pub architecture: String,
    pub framework: bool,
    pub resource_package: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserPackage {
    pub name: String,
    pub package_full_name: String,
    pub package_family_name: String,
    pub version: String,
    pub architecture: String,
    pub publisher: String,
    pub publisher_id: String,
    pub resource_id: Option<String>,
    pub install_location_present: bool,
    pub status: String,
    pub framework: bool,
    pub resource_package: bool,
    pub bundle: bool,
    pub optional: bool,
    pub dependencies: Vec<PackageDependencyState>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetectorEvidence {
    pub component_id: ComponentId,
    pub detector_status: String,
    pub completeness: PackageCompleteness,
    pub package_name: String,
    pub package_full_name: Option<String>,
    pub package_family_name: Option<String>,
    pub current_user: PackageRegistrationState,
    pub other_users: PackageRegistrationState,
    pub provisioning: PackageProvisioningState,
    pub provisioned_package_full_name: Option<String>,
    pub non_removable: bool,
    pub framework: bool,
    pub resource_package: bool,
    pub bundle: bool,
    pub install_location_present: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct PackageMutationContext {
    pub machine_id: String,
    pub inspection_id: String,
    pub source_observation_id: String,
    pub windows_build: u32,
    pub edition: String,
    pub detector: Option<PackageDetectorEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageCapturedState {
    pub component_id: ComponentId,
    pub operation_id: PackageOperationId,
    pub target: Option<CurrentUserPackage>,
    pub current_user_package_full_names: Vec<String>,
    pub current_user_registration_present: bool,
    pub detector_registration: PackageRegistrationState,
    pub other_user_registration: PackageRegistrationState,
    pub provisioning: PackageProvisioningState,
    pub provisioned_package_full_name: Option<String>,
    pub deployment_handler: String,
    pub restore_capability: PackageRestoreCapability,
    pub captured_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageActionability {
    pub operation_id: PackageOperationId,
    pub component_id: ComponentId,
    pub title: &'static str,
    pub exact_package_name: &'static str,
    pub status: PackageActionabilityStatus,
    pub restore_capability: PackageRestoreCapability,
    pub reason: String,
    pub current_user_present: bool,
    pub provisioned: bool,
    pub may_remove_app_data: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageTransactionStatus {
    CapturingPreState,
    Removing,
    Verifying,
    Removed,
    AlreadyAbsent,
    RejectedUnchanged,
    ResultAmbiguous,
    UnexpectedCollateralChange,
    NeedsAttention,
    Restoring,
    Restored,
    RestoredNewerVersion,
    RestoreFailed,
    RecoveryRequired,
}

impl PackageTransactionStatus {
    pub const fn key(self) -> &'static str {
        match self {
            Self::CapturingPreState => "capturing_pre_state",
            Self::Removing => "removing",
            Self::Verifying => "verifying",
            Self::Removed => "removed",
            Self::AlreadyAbsent => "already_absent",
            Self::RejectedUnchanged => "rejected_unchanged",
            Self::ResultAmbiguous => "result_ambiguous",
            Self::UnexpectedCollateralChange => "unexpected_collateral_change",
            Self::NeedsAttention => "needs_attention",
            Self::Restoring => "restoring",
            Self::Restored => "restored",
            Self::RestoredNewerVersion => "restored_newer_version",
            Self::RestoreFailed => "restore_failed",
            Self::RecoveryRequired => "recovery_required",
        }
    }

    pub const fn is_transitional(self) -> bool {
        matches!(
            self,
            Self::CapturingPreState | Self::Removing | Self::Verifying | Self::Restoring
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageMutationStep {
    pub sequence: u32,
    pub step_type: String,
    pub at: String,
    pub status: String,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageMutationTransaction {
    pub transaction_id: String,
    pub machine_id: String,
    pub component_id: ComponentId,
    pub operation_id: PackageOperationId,
    pub source_inspection_id: String,
    pub source_observation_id: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub status: PackageTransactionStatus,
    pub application_version: String,
    pub handler_version: String,
    pub windows_build: u32,
    pub edition: String,
    pub intent_hash: String,
    pub pre_state_hash: Option<String>,
    pub post_state_hash: Option<String>,
    pub restore_state_hash: Option<String>,
    pub pre_state: Option<PackageCapturedState>,
    pub post_state: Option<PackageCapturedState>,
    pub restore_state: Option<PackageCapturedState>,
    pub restore_capability: PackageRestoreCapability,
    pub disappeared_package_full_names: Vec<String>,
    pub appeared_package_full_names: Vec<String>,
    pub restore_disappeared_package_full_names: Vec<String>,
    pub restore_appeared_package_full_names: Vec<String>,
    pub detector_verified: bool,
    pub provisioning_unchanged: bool,
    pub result: String,
    pub error_category: Option<String>,
    pub error_summary: Option<String>,
    pub recovery_requirement: Option<String>,
    pub steps: Vec<PackageMutationStep>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageOperationResult {
    pub transaction: PackageMutationTransaction,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackageBackendErrorKind {
    InventoryFailed,
    DeploymentRejected,
    DeploymentAmbiguous,
    #[cfg(not(windows))]
    UnsupportedPlatform,
}

#[derive(Clone, Debug)]
pub struct PackageBackendError {
    pub kind: PackageBackendErrorKind,
    pub summary: String,
}

impl PackageBackendError {
    fn new(kind: PackageBackendErrorKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
        }
    }
}

pub trait PackageDeploymentBackend: Send + Sync {
    fn current_user_inventory(&self) -> Result<Vec<CurrentUserPackage>, PackageBackendError>;
    fn remove_current_user(
        &self,
        operation: PackageOperationId,
        package_full_name: &str,
    ) -> Result<(), PackageBackendError>;
    fn register_current_user(
        &self,
        operation: PackageOperationId,
        package_full_name: &str,
        dependency_full_names: &[String],
    ) -> Result<(), PackageBackendError>;
}

#[derive(Default)]
pub struct WindowsPackageDeploymentBackend;

#[cfg(windows)]
impl PackageDeploymentBackend for WindowsPackageDeploymentBackend {
    fn current_user_inventory(&self) -> Result<Vec<CurrentUserPackage>, PackageBackendError> {
        use windows::{Management::Deployment::PackageManager, core::HSTRING};

        let manager = PackageManager::new().map_err(win_inventory_error)?;
        // An empty user SID explicitly scopes enumeration to the current user. The parameterless
        // FindPackages overload is cross-user and requires elevation, so it is intentionally not
        // part of the M4 deployment boundary.
        let packages = manager
            .FindPackagesByUserSecurityId(&HSTRING::from(""))
            .map_err(win_inventory_error)?;
        let mut result = Vec::new();
        for package in packages {
            result.push(read_windows_package(&package)?);
        }
        result.sort_by(|left, right| left.package_full_name.cmp(&right.package_full_name));
        Ok(result)
    }

    fn remove_current_user(
        &self,
        operation: PackageOperationId,
        package_full_name: &str,
    ) -> Result<(), PackageBackendError> {
        validate_deployment_identity(operation, package_full_name)?;
        use windows::{Management::Deployment::PackageManager, core::HSTRING};
        let manager = PackageManager::new().map_err(win_deployment_error)?;
        let result = manager
            .RemovePackageAsync(&HSTRING::from(package_full_name))
            .and_then(|operation| operation.join())
            .map_err(win_deployment_error)?;
        deployment_result(result)
    }

    fn register_current_user(
        &self,
        operation: PackageOperationId,
        package_full_name: &str,
        dependency_full_names: &[String],
    ) -> Result<(), PackageBackendError> {
        validate_deployment_identity(operation, package_full_name)?;
        use windows::{
            Management::Deployment::{DeploymentOptions, PackageManager},
            core::HSTRING,
        };
        use windows_collections::IIterable;

        let manager = PackageManager::new().map_err(win_deployment_error)?;
        let dependencies: IIterable<HSTRING> = dependency_full_names
            .iter()
            .map(|value| HSTRING::from(value.as_str()))
            .collect::<Vec<_>>()
            .into();
        let result = manager
            .RegisterPackageByFullNameAsync(
                &HSTRING::from(package_full_name),
                &dependencies,
                DeploymentOptions::None,
            )
            .and_then(|operation| operation.join())
            .map_err(win_deployment_error)?;
        deployment_result(result)
    }
}

#[cfg(not(windows))]
impl PackageDeploymentBackend for WindowsPackageDeploymentBackend {
    fn current_user_inventory(&self) -> Result<Vec<CurrentUserPackage>, PackageBackendError> {
        Err(PackageBackendError::new(
            PackageBackendErrorKind::UnsupportedPlatform,
            "Windows package deployment is unavailable on this platform.",
        ))
    }

    fn remove_current_user(
        &self,
        _operation: PackageOperationId,
        _package_full_name: &str,
    ) -> Result<(), PackageBackendError> {
        self.current_user_inventory().map(|_| ())
    }

    fn register_current_user(
        &self,
        _operation: PackageOperationId,
        _package_full_name: &str,
        _dependency_full_names: &[String],
    ) -> Result<(), PackageBackendError> {
        self.current_user_inventory().map(|_| ())
    }
}

#[cfg(windows)]
fn read_windows_package(
    package: &windows::ApplicationModel::Package,
) -> Result<CurrentUserPackage, PackageBackendError> {
    let id = package.Id().map_err(win_inventory_error)?;
    let version = id.Version().map_err(win_inventory_error)?;
    let dependencies = package.Dependencies().map_err(win_inventory_error)?;
    let mut dependency_states = Vec::new();
    for dependency in dependencies {
        dependency_states.push(read_windows_dependency(&dependency)?);
    }
    dependency_states.sort_by(|left, right| left.package_full_name.cmp(&right.package_full_name));
    let status = package.Status().map_err(win_inventory_error)?;
    Ok(CurrentUserPackage {
        name: id.Name().map_err(win_inventory_error)?.to_string(),
        package_full_name: id.FullName().map_err(win_inventory_error)?.to_string(),
        package_family_name: id.FamilyName().map_err(win_inventory_error)?.to_string(),
        version: format!(
            "{}.{}.{}.{}",
            version.Major, version.Minor, version.Build, version.Revision
        ),
        architecture: architecture_name(id.Architecture().map_err(win_inventory_error)?).into(),
        publisher: id.Publisher().map_err(win_inventory_error)?.to_string(),
        publisher_id: id.PublisherId().map_err(win_inventory_error)?.to_string(),
        resource_id: nonempty(id.ResourceId().map_err(win_inventory_error)?.to_string()),
        install_location_present: package
            .InstalledLocation()
            .and_then(|folder| folder.Path())
            .is_ok_and(|path| !path.is_empty()),
        status: package_status(&status),
        framework: package.IsFramework().map_err(win_inventory_error)?,
        resource_package: package.IsResourcePackage().map_err(win_inventory_error)?,
        bundle: package.IsBundle().map_err(win_inventory_error)?,
        optional: package.IsOptional().map_err(win_inventory_error)?,
        dependencies: dependency_states,
    })
}

#[cfg(windows)]
fn read_windows_dependency(
    package: &windows::ApplicationModel::Package,
) -> Result<PackageDependencyState, PackageBackendError> {
    let id = package.Id().map_err(win_inventory_error)?;
    let version = id.Version().map_err(win_inventory_error)?;
    Ok(PackageDependencyState {
        name: id.Name().map_err(win_inventory_error)?.to_string(),
        package_full_name: id.FullName().map_err(win_inventory_error)?.to_string(),
        package_family_name: id.FamilyName().map_err(win_inventory_error)?.to_string(),
        version: format!(
            "{}.{}.{}.{}",
            version.Major, version.Minor, version.Build, version.Revision
        ),
        architecture: architecture_name(id.Architecture().map_err(win_inventory_error)?).into(),
        framework: package.IsFramework().map_err(win_inventory_error)?,
        resource_package: package.IsResourcePackage().map_err(win_inventory_error)?,
    })
}

#[cfg(windows)]
fn architecture_name(value: windows::System::ProcessorArchitecture) -> &'static str {
    use windows::System::ProcessorArchitecture;
    match value {
        ProcessorArchitecture::X86 => "x86",
        ProcessorArchitecture::X64 => "x64",
        ProcessorArchitecture::Arm => "arm",
        ProcessorArchitecture::Arm64 => "arm64",
        ProcessorArchitecture::Neutral => "neutral",
        ProcessorArchitecture::X86OnArm64 => "x86_on_arm64",
        _ => "unknown",
    }
}

#[cfg(windows)]
fn package_status(status: &windows::ApplicationModel::PackageStatus) -> String {
    let mut flags = Vec::new();
    if status.VerifyIsOK().unwrap_or(false) {
        flags.push("ok");
    }
    for (name, set) in [
        ("not_available", status.NotAvailable().unwrap_or(false)),
        ("package_offline", status.PackageOffline().unwrap_or(false)),
        ("data_offline", status.DataOffline().unwrap_or(false)),
        ("disabled", status.Disabled().unwrap_or(false)),
        (
            "needs_remediation",
            status.NeedsRemediation().unwrap_or(false),
        ),
        ("license_issue", status.LicenseIssue().unwrap_or(false)),
        ("modified", status.Modified().unwrap_or(false)),
        ("tampered", status.Tampered().unwrap_or(false)),
        (
            "dependency_issue",
            status.DependencyIssue().unwrap_or(false),
        ),
        ("servicing", status.Servicing().unwrap_or(false)),
        (
            "deployment_in_progress",
            status.DeploymentInProgress().unwrap_or(false),
        ),
        (
            "partially_staged",
            status.IsPartiallyStaged().unwrap_or(false),
        ),
    ] {
        if set {
            flags.push(name);
        }
    }
    if flags.is_empty() {
        "unknown".into()
    } else {
        flags.join(",")
    }
}

#[cfg(windows)]
fn deployment_result(
    result: windows::Management::Deployment::DeploymentResult,
) -> Result<(), PackageBackendError> {
    let code = result.ExtendedErrorCode().map_err(win_deployment_error)?;
    if code.is_ok() {
        Ok(())
    } else {
        let text = result
            .ErrorText()
            .map(|value| value.to_string())
            .unwrap_or_default();
        Err(PackageBackendError::new(
            PackageBackendErrorKind::DeploymentRejected,
            format!("Windows deployment rejected the fixed operation ({code:?}): {text}"),
        ))
    }
}

#[cfg(windows)]
fn win_inventory_error(error: windows::core::Error) -> PackageBackendError {
    PackageBackendError::new(PackageBackendErrorKind::InventoryFailed, error.to_string())
}

#[cfg(windows)]
fn win_deployment_error(error: windows::core::Error) -> PackageBackendError {
    PackageBackendError::new(
        PackageBackendErrorKind::DeploymentAmbiguous,
        error.to_string(),
    )
}

fn nonempty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn validate_deployment_identity(
    operation: PackageOperationId,
    package_full_name: &str,
) -> Result<(), PackageBackendError> {
    let exact_prefix = format!("{}_", operation.package_name());
    if package_full_name.starts_with(&exact_prefix)
        && !package_full_name.contains('*')
        && !package_full_name.contains('?')
    {
        Ok(())
    } else {
        Err(PackageBackendError::new(
            PackageBackendErrorKind::DeploymentRejected,
            "The resolved PackageFullName did not belong to the closed package operation.",
        ))
    }
}

#[derive(Clone, Default)]
struct PackageJournal {
    database_path: Option<std::path::PathBuf>,
}

impl PackageJournal {
    #[cfg(test)]
    fn at(path: std::path::PathBuf) -> Self {
        Self {
            database_path: Some(path),
        }
    }

    fn connection(&self) -> Result<rusqlite::Connection, String> {
        match &self.database_path {
            Some(path) => crate::persistence::open_at(path).map_err(|error| error.to_string()),
            None => crate::persistence::open().map_err(|error| error.to_string()),
        }
    }

    fn lock_path(&self) -> std::path::PathBuf {
        self.database_path
            .clone()
            .unwrap_or_else(crate::persistence::path)
            .with_extension("mutation-alpha.lock")
    }

    fn save(&self, transaction: &PackageMutationTransaction) -> Result<(), String> {
        let mut connection = self.connection()?;
        let tx = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        tx.execute(
            "INSERT INTO package_mutation_transactions(id,machine_id,component_id,operation_id,source_inspection_id,created_at,completed_at,status,transaction_json)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)
             ON CONFLICT(id) DO UPDATE SET completed_at=excluded.completed_at,status=excluded.status,transaction_json=excluded.transaction_json",
            params![
                transaction.transaction_id,
                transaction.machine_id,
                transaction.component_id.key(),
                transaction.operation_id.key(),
                transaction.source_inspection_id,
                transaction.created_at,
                transaction.completed_at,
                transaction.status.key(),
                serde_json::to_string(transaction).map_err(|error| error.to_string())?,
            ],
        ).map_err(|error| error.to_string())?;
        for (capture_type, state) in [
            ("pre_state", transaction.pre_state.as_ref()),
            ("post_state", transaction.post_state.as_ref()),
            ("restore_state", transaction.restore_state.as_ref()),
        ] {
            if let Some(state) = state {
                tx.execute(
                    "INSERT INTO package_mutation_state_captures(transaction_id,capture_type,state_json,captured_at,integrity_hash)
                     VALUES(?1,?2,?3,?4,?5)
                     ON CONFLICT(transaction_id,capture_type) DO UPDATE SET state_json=excluded.state_json,captured_at=excluded.captured_at,integrity_hash=excluded.integrity_hash",
                    params![
                        transaction.transaction_id,
                        capture_type,
                        serde_json::to_string(state).map_err(|error| error.to_string())?,
                        state.captured_at,
                        hash_serializable(state).map_err(|error| error.to_string())?,
                    ],
                ).map_err(|error| error.to_string())?;
            }
        }
        tx.commit().map_err(|error| error.to_string())
    }

    fn load(&self, id: &str) -> Result<Option<PackageMutationTransaction>, String> {
        let connection = self.connection()?;
        let raw: Option<String> = connection
            .query_row(
                "SELECT transaction_json FROM package_mutation_transactions WHERE id=?1",
                [id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        raw.map(|value| serde_json::from_str(&value).map_err(|error| error.to_string()))
            .transpose()
    }

    fn history(&self) -> Result<Vec<PackageMutationTransaction>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare("SELECT transaction_json FROM package_mutation_transactions ORDER BY created_at DESC")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        rows.map(|row| {
            serde_json::from_str(&row.map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())
        })
        .collect()
    }
}

pub struct PackageBroker {
    backend: Arc<dyn PackageDeploymentBackend>,
    journal: PackageJournal,
    execution_lock: Mutex<()>,
}

impl PackageBroker {
    pub fn owner(backend: Arc<dyn PackageDeploymentBackend>) -> Self {
        Self {
            backend,
            journal: PackageJournal::default(),
            execution_lock: Mutex::new(()),
        }
    }

    #[cfg(test)]
    fn at(backend: Arc<dyn PackageDeploymentBackend>, path: std::path::PathBuf) -> Self {
        Self {
            backend,
            journal: PackageJournal::at(path),
            execution_lock: Mutex::new(()),
        }
    }

    pub fn actionability(
        &self,
        operation: PackageOperationId,
        context: &PackageMutationContext,
    ) -> PackageActionability {
        match self.backend.current_user_inventory() {
            Ok(inventory) => actionability(operation, context, &inventory),
            Err(error) => unavailable_actionability(
                operation,
                PackageActionabilityStatus::NeedsScan,
                format!(
                    "Current-user package inventory could not be read: {}",
                    error.summary
                ),
            ),
        }
    }

    pub fn remove<F>(
        &self,
        operation: PackageOperationId,
        context: &PackageMutationContext,
        detector_verification: F,
    ) -> Result<PackageOperationResult, String>
    where
        F: FnOnce() -> Result<PackageMutationContext, String>,
    {
        let _process = self
            .execution_lock
            .try_lock()
            .map_err(|_| "Another owner operation is active.".to_string())?;
        let _cross_process = OwnerMutationProcessLock::acquire(self.journal.lock_path())?;
        if !crate::owner_scope::is_valid(&context.machine_id) {
            return Err("The current owner machine/account scope is unavailable.".into());
        }
        let inventory = self
            .backend
            .current_user_inventory()
            .map_err(|error| error.summary)?;
        let readiness = actionability(operation, context, &inventory);
        let now = crate::inspection::timestamp();
        let mut transaction = new_transaction(operation, context, &now)?;

        if readiness.status == PackageActionabilityStatus::AlreadyAbsent {
            transaction.status = PackageTransactionStatus::AlreadyAbsent;
            transaction.result = "already_absent".into();
            transaction.completed_at = Some(now);
            self.journal.save(&transaction)?;
            return Ok(PackageOperationResult {
                transaction,
                message: format!("{} is already absent from this account.", operation.title()),
            });
        }
        if readiness.status != PackageActionabilityStatus::Ready {
            return Err(readiness.reason);
        }

        let pre_state = capture_state(operation, context, &inventory)?;
        transaction.restore_capability = pre_state.restore_capability;
        transaction.pre_state_hash =
            Some(hash_serializable(&pre_state).map_err(|error| error.to_string())?);
        transaction.pre_state = Some(pre_state.clone());
        transition(
            &self.journal,
            &mut transaction,
            PackageTransactionStatus::CapturingPreState,
            "pre_state_durable",
        )?;

        let immediate = self
            .backend
            .current_user_inventory()
            .map_err(|error| error.summary)?;
        let immediate_target = resolve_target(operation, &immediate)?;
        if immediate_target
            .as_ref()
            .map(|item| item.package_full_name.as_str())
            != pre_state
                .target
                .as_ref()
                .map(|item| item.package_full_name.as_str())
            || inventory_names(&immediate) != pre_state.current_user_package_full_names
        {
            transaction.status = PackageTransactionStatus::NeedsAttention;
            transaction.result = "state_changed_before_remove".into();
            transaction.recovery_requirement =
                Some("Run a fresh inspection before retrying package removal.".into());
            transaction.completed_at = Some(crate::inspection::timestamp());
            self.journal.save(&transaction)?;
            return Ok(PackageOperationResult {
                transaction,
                message: "Package state changed after review; nothing was removed.".into(),
            });
        }

        transition(
            &self.journal,
            &mut transaction,
            PackageTransactionStatus::Removing,
            "remove_current_user",
        )?;
        let full_name = pre_state
            .target
            .as_ref()
            .expect("ready target")
            .package_full_name
            .clone();
        let deployment = self.backend.remove_current_user(operation, &full_name);
        transition(
            &self.journal,
            &mut transaction,
            PackageTransactionStatus::Verifying,
            "verify_inventory_and_detector",
        )?;
        let post_inventory = self
            .backend
            .current_user_inventory()
            .map_err(|error| error.summary)?;
        let post_context = detector_verification()?;
        if post_context.machine_id != context.machine_id {
            return Err("Detector verification returned a different owner scope.".into());
        }
        let post_state = capture_state(operation, &post_context, &post_inventory)?;
        transaction.post_state_hash =
            Some(hash_serializable(&post_state).map_err(|error| error.to_string())?);
        transaction.post_state = Some(post_state.clone());
        let pre_names: BTreeSet<_> = pre_state
            .current_user_package_full_names
            .iter()
            .cloned()
            .collect();
        let post_names: BTreeSet<_> = post_state
            .current_user_package_full_names
            .iter()
            .cloned()
            .collect();
        transaction.disappeared_package_full_names = pre_names
            .difference(&post_names)
            .filter(|name| !name.eq_ignore_ascii_case(&full_name))
            .cloned()
            .collect();
        transaction.appeared_package_full_names =
            post_names.difference(&pre_names).cloned().collect();
        transaction.detector_verified = post_state.target.is_none()
            && post_state.detector_registration == PackageRegistrationState::Absent;
        transaction.provisioning_unchanged = pre_state.provisioning == post_state.provisioning
            && pre_state.provisioned_package_full_name == post_state.provisioned_package_full_name;

        let message = classify_remove_result(&mut transaction, deployment);
        transaction.completed_at = Some(crate::inspection::timestamp());
        self.journal.save(&transaction)?;
        Ok(PackageOperationResult {
            transaction,
            message,
        })
    }

    pub fn restore<F>(
        &self,
        transaction_id: &str,
        context: &PackageMutationContext,
        detector_verification: F,
    ) -> Result<PackageOperationResult, String>
    where
        F: FnOnce() -> Result<PackageMutationContext, String>,
    {
        let _process = self
            .execution_lock
            .try_lock()
            .map_err(|_| "Another owner operation is active.".to_string())?;
        let _cross_process = OwnerMutationProcessLock::acquire(self.journal.lock_path())?;
        let mut transaction = self
            .journal
            .load(transaction_id)?
            .ok_or_else(|| "The package transaction was not found.".to_string())?;
        validate_transaction(&transaction)?;
        if transaction.machine_id != context.machine_id
            || !crate::owner_scope::is_valid(&context.machine_id)
        {
            return Err("Restore is bound to the original owner machine/account.".into());
        }
        if transaction.restore_capability != PackageRestoreCapability::RestoreAvailable {
            return Err("Automatic Restore is unavailable; reinstall is required.".into());
        }
        let pre_state = transaction
            .pre_state
            .clone()
            .ok_or_else(|| "The captured package pre-state is missing.".to_string())?;
        let original = pre_state
            .target
            .as_ref()
            .ok_or_else(|| "The original package identity is missing.".to_string())?;
        let inventory = self
            .backend
            .current_user_inventory()
            .map_err(|error| error.summary)?;
        let same_family: Vec<_> = inventory
            .iter()
            .filter(|item| {
                item.package_family_name
                    .eq_ignore_ascii_case(&original.package_family_name)
            })
            .collect();
        if same_family.first().is_some_and(|existing| {
            existing.package_full_name != original.package_full_name
                && !compare_versions(&existing.version, &original.version).is_gt()
        }) {
            return Err(
                "A conflicting package version is registered; automatic Restore was refused."
                    .into(),
            );
        }

        transition(
            &self.journal,
            &mut transaction,
            PackageTransactionStatus::Restoring,
            "register_current_user",
        )?;
        let dependencies: Vec<String> = original
            .dependencies
            .iter()
            .map(|item| item.package_full_name.clone())
            .collect();
        let (deployment, restored_inventory) = if same_family.is_empty() {
            let deployment = self.backend.register_current_user(
                transaction.operation_id,
                &original.package_full_name,
                &dependencies,
            );
            let inventory = self
                .backend
                .current_user_inventory()
                .map_err(|error| error.summary)?;
            (deployment, inventory)
        } else {
            (Ok(()), inventory)
        };
        let restored_context = detector_verification()?;
        let restored_state = capture_state(
            transaction.operation_id,
            &restored_context,
            &restored_inventory,
        )?;
        transaction.restore_state_hash =
            Some(hash_serializable(&restored_state).map_err(|error| error.to_string())?);
        transaction.restore_state = Some(restored_state.clone());
        let pre_names: BTreeSet<_> = pre_state
            .current_user_package_full_names
            .iter()
            .cloned()
            .collect();
        let restored_names: BTreeSet<_> = restored_state
            .current_user_package_full_names
            .iter()
            .cloned()
            .collect();
        transaction.restore_disappeared_package_full_names =
            pre_names.difference(&restored_names).cloned().collect();
        transaction.restore_appeared_package_full_names =
            restored_names.difference(&pre_names).cloned().collect();
        let restored = restored_state.target.as_ref();
        let exact =
            restored.is_some_and(|item| item.package_full_name == original.package_full_name);
        let newer = restored.is_some_and(|item| {
            item.package_family_name
                .eq_ignore_ascii_case(&original.package_family_name)
                && compare_versions(&item.version, &original.version).is_gt()
        });
        let expected_missing = if newer {
            transaction
                .restore_disappeared_package_full_names
                .iter()
                .all(|name| name == &original.package_full_name)
        } else {
            transaction
                .restore_disappeared_package_full_names
                .is_empty()
        };
        let expected_appeared = if newer {
            restored.is_some_and(|item| {
                transaction.restore_appeared_package_full_names.len() == 1
                    && transaction.restore_appeared_package_full_names[0] == item.package_full_name
            })
        } else {
            transaction.restore_appeared_package_full_names.is_empty()
        };
        transaction.provisioning_unchanged = pre_state.provisioning == restored_state.provisioning
            && pre_state.provisioned_package_full_name
                == restored_state.provisioned_package_full_name;
        if deployment.is_ok()
            && restored_context.machine_id == context.machine_id
            && restored_state.detector_registration == PackageRegistrationState::Present
            && (exact || newer)
            && expected_missing
            && expected_appeared
            && transaction.provisioning_unchanged
        {
            transaction.status = if newer {
                PackageTransactionStatus::RestoredNewerVersion
            } else {
                PackageTransactionStatus::Restored
            };
            transaction.result = if newer {
                "same_family_newer_version_restored"
            } else {
                "exact_version_restored"
            }
            .into();
            transaction.error_category = None;
            transaction.error_summary = None;
            transaction.recovery_requirement = None;
        } else {
            transaction.status = PackageTransactionStatus::RestoreFailed;
            transaction.result = "restore_not_verified".into();
            transaction.recovery_requirement =
                Some("Reinstall the app through Microsoft Store or Windows servicing.".into());
            if let Err(error) = deployment {
                transaction.error_category = Some(format!("{:?}", error.kind));
                transaction.error_summary = Some(error.summary);
            }
        }
        transaction.completed_at = Some(crate::inspection::timestamp());
        let message = if matches!(
            transaction.status,
            PackageTransactionStatus::Restored | PackageTransactionStatus::RestoredNewerVersion
        ) {
            "The package was restored for this account and verified.".into()
        } else {
            "Restore could not be verified and needs attention.".into()
        };
        self.journal.save(&transaction)?;
        Ok(PackageOperationResult {
            transaction,
            message,
        })
    }

    pub fn history(&self, owner_scope: &str) -> Result<Vec<PackageMutationTransaction>, String> {
        if !crate::owner_scope::is_valid(owner_scope) {
            return Ok(Vec::new());
        }
        let transactions: Vec<_> = self
            .journal
            .history()?
            .into_iter()
            .filter(|transaction| transaction.machine_id == owner_scope)
            .collect();
        for transaction in &transactions {
            validate_transaction(transaction)?;
        }
        Ok(transactions)
    }

    pub fn recover_interrupted(
        &self,
        owner_scope: &str,
    ) -> Result<Vec<PackageMutationTransaction>, String> {
        let mut recovered = Vec::new();
        for mut transaction in self.history(owner_scope)? {
            if transaction.status.is_transitional() {
                transaction.status = PackageTransactionStatus::RecoveryRequired;
                transaction.result = "interrupted_operation".into();
                transaction.recovery_requirement =
                    Some("Inspect current package registration before another action.".into());
                transaction.completed_at = Some(crate::inspection::timestamp());
                self.journal.save(&transaction)?;
                recovered.push(transaction);
            }
        }
        Ok(recovered)
    }
}

fn actionability(
    operation: PackageOperationId,
    context: &PackageMutationContext,
    inventory: &[CurrentUserPackage],
) -> PackageActionability {
    let Some(detector) = context.detector.as_ref() else {
        return unavailable_actionability(
            operation,
            PackageActionabilityStatus::NeedsScan,
            "The exact package detector has no current evidence.",
        );
    };
    if detector.component_id != operation.component_id()
        || !detector
            .package_name
            .eq_ignore_ascii_case(operation.package_name())
    {
        return unavailable_actionability(
            operation,
            PackageActionabilityStatus::RemoveUnavailable,
            "Detector identity did not match the closed operation.",
        );
    }
    let matches: Vec<_> = inventory
        .iter()
        .filter(|item| item.name.eq_ignore_ascii_case(operation.package_name()))
        .collect();
    if detector.current_user == PackageRegistrationState::Absent && matches.is_empty() {
        return PackageActionability {
            operation_id: operation,
            component_id: operation.component_id(),
            title: operation.title(),
            exact_package_name: operation.package_name(),
            status: PackageActionabilityStatus::AlreadyAbsent,
            restore_capability: PackageRestoreCapability::RemoveUnavailable,
            reason: "The package is not registered for this account.".into(),
            current_user_present: false,
            provisioned: detector.provisioning == PackageProvisioningState::Provisioned,
            may_remove_app_data: true,
        };
    }
    if detector.current_user != PackageRegistrationState::Present || matches.len() != 1 {
        return unavailable_actionability(
            operation,
            PackageActionabilityStatus::RemoveUnavailable,
            "Deslopper could not resolve exactly one current-user package registration.",
        );
    }
    let target = matches[0];
    if detector.package_full_name.as_deref() != Some(target.package_full_name.as_str()) {
        return unavailable_actionability(
            operation,
            PackageActionabilityStatus::RemoveUnavailable,
            "Direct package inventory and the exact detector disagreed.",
        );
    }
    if detector.non_removable
        || detector.framework
        || detector.resource_package
        || detector.bundle
        || target.framework
        || target.resource_package
        || target.bundle
    {
        return unavailable_actionability(
            operation,
            PackageActionabilityStatus::RemoveUnavailable,
            "Windows reports this package state as protected, framework, resource, or bundle-backed.",
        );
    }
    if target.status != "ok" {
        return unavailable_actionability(
            operation,
            PackageActionabilityStatus::RemoveUnavailable,
            format!(
                "Package health is not ready for bounded removal: {}",
                target.status
            ),
        );
    }
    let capability = classify_restore(detector, target, inventory);
    PackageActionability {
        operation_id: operation, component_id: operation.component_id(), title: operation.title(),
        exact_package_name: operation.package_name(), status: PackageActionabilityStatus::Ready,
        restore_capability: capability,
        reason: match capability {
            PackageRestoreCapability::RestoreAvailable => "Windows retains an exact provisioned/staged identity and all dependency identities needed for local registration.".into(),
            _ => "Automatic Restore is not guaranteed; reinstalling may require Microsoft Store or Windows servicing.".into(),
        },
        current_user_present: true,
        provisioned: detector.provisioning == PackageProvisioningState::Provisioned,
        may_remove_app_data: true,
    }
}

fn unavailable_actionability(
    operation: PackageOperationId,
    status: PackageActionabilityStatus,
    reason: impl Into<String>,
) -> PackageActionability {
    PackageActionability {
        operation_id: operation,
        component_id: operation.component_id(),
        title: operation.title(),
        exact_package_name: operation.package_name(),
        status,
        restore_capability: PackageRestoreCapability::RemoveUnavailable,
        reason: reason.into(),
        current_user_present: false,
        provisioned: false,
        may_remove_app_data: true,
    }
}

fn classify_restore(
    detector: &PackageDetectorEvidence,
    target: &CurrentUserPackage,
    inventory: &[CurrentUserPackage],
) -> PackageRestoreCapability {
    let inventory_names: BTreeSet<_> = inventory
        .iter()
        .map(|item| item.package_full_name.as_str())
        .collect();
    let dependencies_captured = target.dependencies.iter().all(|dependency| {
        !dependency.package_full_name.is_empty()
            && inventory_names.contains(dependency.package_full_name.as_str())
    });
    if detector.provisioning == PackageProvisioningState::Provisioned
        && detector.provisioned_package_full_name.as_deref()
            == Some(target.package_full_name.as_str())
        && target.install_location_present
        && dependencies_captured
    {
        PackageRestoreCapability::RestoreAvailable
    } else {
        PackageRestoreCapability::ReinstallRequired
    }
}

fn resolve_target(
    operation: PackageOperationId,
    inventory: &[CurrentUserPackage],
) -> Result<Option<CurrentUserPackage>, String> {
    let matches: Vec<_> = inventory
        .iter()
        .filter(|item| item.name.eq_ignore_ascii_case(operation.package_name()))
        .cloned()
        .collect();
    if matches.len() > 1 {
        Err(
            "Multiple current-user versions matched the exact package name; removal was refused."
                .into(),
        )
    } else {
        Ok(matches.into_iter().next())
    }
}

fn capture_state(
    operation: PackageOperationId,
    context: &PackageMutationContext,
    inventory: &[CurrentUserPackage],
) -> Result<PackageCapturedState, String> {
    let target = resolve_target(operation, inventory)?;
    let detector = context.detector.as_ref();
    let restore_capability = match (detector, target.as_ref()) {
        (Some(detector), Some(target)) => classify_restore(detector, target, inventory),
        _ => PackageRestoreCapability::RemoveUnavailable,
    };
    Ok(PackageCapturedState {
        component_id: operation.component_id(),
        operation_id: operation,
        target,
        current_user_package_full_names: inventory_names(inventory),
        current_user_registration_present: detector
            .is_some_and(|item| item.current_user == PackageRegistrationState::Present),
        detector_registration: detector
            .map_or(PackageRegistrationState::Unknown, |item| item.current_user),
        other_user_registration: detector
            .map_or(PackageRegistrationState::Unknown, |item| item.other_users),
        provisioning: detector.map_or(PackageProvisioningState::Unknown, |item| item.provisioning),
        provisioned_package_full_name: detector
            .and_then(|item| item.provisioned_package_full_name.clone()),
        deployment_handler: format!(
            "Windows.Management.Deployment.PackageManager/{PACKAGE_HANDLER_VERSION}"
        ),
        restore_capability,
        captured_at: crate::inspection::timestamp(),
    })
}

fn inventory_names(inventory: &[CurrentUserPackage]) -> Vec<String> {
    let mut names: Vec<_> = inventory
        .iter()
        .map(|item| item.package_full_name.clone())
        .collect();
    names.sort();
    names.dedup();
    names
}

fn classify_remove_result(
    transaction: &mut PackageMutationTransaction,
    deployment: Result<(), PackageBackendError>,
) -> String {
    let target_absent = transaction
        .post_state
        .as_ref()
        .is_some_and(|state| state.target.is_none());
    if !transaction.disappeared_package_full_names.is_empty() {
        transaction.status = PackageTransactionStatus::UnexpectedCollateralChange;
        transaction.result = "unexpected_collateral_package_removal".into();
        transaction.recovery_requirement = Some(
            "Review the recorded package identities before any further package action.".into(),
        );
        return "The target was processed, but unexpected package registrations also disappeared. This needs attention.".into();
    }
    if !transaction.provisioning_unchanged {
        transaction.status = PackageTransactionStatus::NeedsAttention;
        transaction.result = "provisioning_evidence_changed".into();
        transaction.recovery_requirement = Some(
            "Provisioning evidence changed unexpectedly; inspect Windows package state.".into(),
        );
        return "Current-user removal could not prove that provisioning stayed unchanged.".into();
    }
    match deployment {
        Ok(()) if target_absent && transaction.detector_verified => {
            transaction.status = PackageTransactionStatus::Removed;
            transaction.result = "removed_current_user_verified".into();
            format!(
                "{} was removed from this account and verified.",
                transaction.operation_id.title()
            )
        }
        Ok(()) if !target_absent => {
            transaction.status = PackageTransactionStatus::RejectedUnchanged;
            transaction.result = "deployment_completed_target_still_present".into();
            "Windows left the package registered; no removal was verified.".into()
        }
        Ok(()) => {
            transaction.status = PackageTransactionStatus::NeedsAttention;
            transaction.result = "direct_and_detector_disagreed".into();
            transaction.recovery_requirement =
                Some("Direct inventory and the Deslopper detector disagreed after removal.".into());
            "Package removal needs attention because verification sources disagreed.".into()
        }
        Err(error) if !target_absent => {
            transaction.status = PackageTransactionStatus::RejectedUnchanged;
            transaction.result = "remove_rejected_unchanged".into();
            transaction.error_category = Some(format!("{:?}", error.kind));
            transaction.error_summary = Some(error.summary);
            "Windows could not remove the package and it remains registered.".into()
        }
        Err(error) => {
            transaction.status = PackageTransactionStatus::ResultAmbiguous;
            transaction.result = "deployment_error_target_absent".into();
            transaction.error_category = Some(format!("{:?}", error.kind));
            transaction.error_summary = Some(error.summary);
            transaction.recovery_requirement =
                Some("The package is absent, but the deployment result was ambiguous.".into());
            "The package is absent, but Windows returned an ambiguous deployment result. This needs attention.".into()
        }
    }
}

fn new_transaction(
    operation: PackageOperationId,
    context: &PackageMutationContext,
    created_at: &str,
) -> Result<PackageMutationTransaction, String> {
    let sequence = PACKAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut transaction = PackageMutationTransaction {
        transaction_id: format!("package-transaction-{created_at}-{sequence}"),
        machine_id: context.machine_id.clone(),
        component_id: operation.component_id(),
        operation_id: operation,
        source_inspection_id: context.inspection_id.clone(),
        source_observation_id: context.source_observation_id.clone(),
        created_at: created_at.into(),
        completed_at: None,
        status: PackageTransactionStatus::CapturingPreState,
        application_version: env!("CARGO_PKG_VERSION").into(),
        handler_version: PACKAGE_HANDLER_VERSION.into(),
        windows_build: context.windows_build,
        edition: context.edition.clone(),
        intent_hash: String::new(),
        pre_state_hash: None,
        post_state_hash: None,
        restore_state_hash: None,
        pre_state: None,
        post_state: None,
        restore_state: None,
        restore_capability: PackageRestoreCapability::RemoveUnavailable,
        disappeared_package_full_names: Vec::new(),
        appeared_package_full_names: Vec::new(),
        restore_disappeared_package_full_names: Vec::new(),
        restore_appeared_package_full_names: Vec::new(),
        detector_verified: false,
        provisioning_unchanged: true,
        result: "created".into(),
        error_category: None,
        error_summary: None,
        recovery_requirement: None,
        steps: vec![PackageMutationStep {
            sequence: 1,
            step_type: "created".into(),
            at: created_at.into(),
            status: "created".into(),
            evidence: vec!["Closed current-user package operation created.".into()],
        }],
    };
    transaction.intent_hash = transaction_integrity_hash(&transaction)?;
    Ok(transaction)
}

fn transition(
    journal: &PackageJournal,
    transaction: &mut PackageMutationTransaction,
    status: PackageTransactionStatus,
    step_type: &str,
) -> Result<(), String> {
    transaction.status = status;
    transaction.steps.push(PackageMutationStep {
        sequence: transaction.steps.len() as u32 + 1,
        step_type: step_type.into(),
        at: crate::inspection::timestamp(),
        status: status.key().into(),
        evidence: vec!["Package transaction transition persisted.".into()],
    });
    journal.save(transaction)
}

fn transaction_integrity_hash(transaction: &PackageMutationTransaction) -> Result<String, String> {
    hash_serializable(&(
        &transaction.transaction_id,
        &transaction.machine_id,
        transaction.component_id,
        transaction.operation_id,
        &transaction.source_inspection_id,
        &transaction.source_observation_id,
        &transaction.created_at,
        &transaction.application_version,
        &transaction.handler_version,
        transaction.windows_build,
        &transaction.edition,
    ))
    .map_err(|error| error.to_string())
}

fn validate_transaction(transaction: &PackageMutationTransaction) -> Result<(), String> {
    if transaction.handler_version != PACKAGE_HANDLER_VERSION
        || transaction.component_id != transaction.operation_id.component_id()
        || transaction.intent_hash != transaction_integrity_hash(transaction)?
        || !capture_hash_matches(
            transaction.pre_state.as_ref(),
            transaction.pre_state_hash.as_deref(),
        )
        || !capture_hash_matches(
            transaction.post_state.as_ref(),
            transaction.post_state_hash.as_deref(),
        )
        || !capture_hash_matches(
            transaction.restore_state.as_ref(),
            transaction.restore_state_hash.as_deref(),
        )
    {
        Err("Package transaction integrity validation failed.".into())
    } else {
        Ok(())
    }
}

fn capture_hash_matches(state: Option<&PackageCapturedState>, expected: Option<&str>) -> bool {
    match (state, expected) {
        (None, None) => true,
        (Some(state), Some(expected)) => {
            hash_serializable(state).is_ok_and(|actual| actual == expected)
        }
        _ => false,
    }
}

fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let parse = |value: &str| {
        let mut result = [0_u64; 4];
        for (index, part) in value.split('.').take(4).enumerate() {
            result[index] = part.parse().unwrap_or(0);
        }
        result
    };
    parse(left).cmp(&parse(right))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Barrier,
        atomic::{AtomicBool, AtomicUsize},
    };

    struct FakeBackend {
        inventory: Mutex<Vec<CurrentUserPackage>>,
        reject_remove: AtomicBool,
        reject_restore: AtomicBool,
        ambiguous_after_remove: AtomicBool,
        collateral: Mutex<Option<CurrentUserPackage>>,
        remove_gate: Mutex<Option<(Arc<Barrier>, Arc<Barrier>)>>,
        removes: AtomicUsize,
    }

    impl FakeBackend {
        fn new(packages: Vec<CurrentUserPackage>) -> Self {
            Self {
                inventory: Mutex::new(packages),
                reject_remove: AtomicBool::new(false),
                reject_restore: AtomicBool::new(false),
                ambiguous_after_remove: AtomicBool::new(false),
                collateral: Mutex::new(None),
                remove_gate: Mutex::new(None),
                removes: AtomicUsize::new(0),
            }
        }

        fn block_next_remove(&self, entered: Arc<Barrier>, release: Arc<Barrier>) {
            *self.remove_gate.lock().unwrap() = Some((entered, release));
        }
    }

    impl PackageDeploymentBackend for FakeBackend {
        fn current_user_inventory(&self) -> Result<Vec<CurrentUserPackage>, PackageBackendError> {
            Ok(self.inventory.lock().unwrap().clone())
        }
        fn remove_current_user(
            &self,
            operation: PackageOperationId,
            full: &str,
        ) -> Result<(), PackageBackendError> {
            validate_deployment_identity(operation, full)?;
            self.removes.fetch_add(1, Ordering::SeqCst);
            if let Some((entered, release)) = self.remove_gate.lock().unwrap().take() {
                entered.wait();
                release.wait();
            }
            if self.reject_remove.load(Ordering::SeqCst) {
                return Err(PackageBackendError::new(
                    PackageBackendErrorKind::DeploymentRejected,
                    "synthetic rejection",
                ));
            }
            let mut inventory = self.inventory.lock().unwrap();
            inventory.retain(|item| item.package_full_name != full);
            if let Some(collateral) = self.collateral.lock().unwrap().as_ref() {
                inventory.retain(|item| item.package_full_name != collateral.package_full_name);
            }
            if self.ambiguous_after_remove.load(Ordering::SeqCst) {
                return Err(PackageBackendError::new(
                    PackageBackendErrorKind::DeploymentAmbiguous,
                    "synthetic ambiguous result after removal",
                ));
            }
            Ok(())
        }
        fn register_current_user(
            &self,
            operation: PackageOperationId,
            full: &str,
            _deps: &[String],
        ) -> Result<(), PackageBackendError> {
            validate_deployment_identity(operation, full)?;
            if self.reject_restore.load(Ordering::SeqCst) {
                return Err(PackageBackendError::new(
                    PackageBackendErrorKind::DeploymentRejected,
                    "synthetic restore rejection",
                ));
            }
            self.inventory
                .lock()
                .unwrap()
                .push(package(operation, "1.0.0.0"));
            Ok(())
        }
    }

    #[cfg(not(feature = "mutation-alpha"))]
    struct RegistryBackend {
        state: Mutex<super::super::plan::CapturedRepresentation>,
        write_gate: Option<(Arc<Barrier>, Arc<Barrier>)>,
    }

    #[cfg(not(feature = "mutation-alpha"))]
    impl RegistryBackend {
        fn new(
            state: super::super::plan::CapturedRepresentation,
            write_gate: Option<(Arc<Barrier>, Arc<Barrier>)>,
        ) -> Self {
            Self {
                state: Mutex::new(state),
                write_gate,
            }
        }

        fn read(
            &self,
        ) -> Result<super::super::plan::CapturedRepresentation, super::super::handlers::HandlerError>
        {
            Ok(self.state.lock().unwrap().clone())
        }

        fn write(
            &self,
            state: &super::super::plan::CapturedRepresentation,
        ) -> Result<(), super::super::handlers::HandlerError> {
            if let Some((entered, release)) = &self.write_gate {
                entered.wait();
                release.wait();
            }
            *self.state.lock().unwrap() = state.clone();
            Ok(())
        }
    }

    #[cfg(not(feature = "mutation-alpha"))]
    impl super::super::handlers::MutationBackend for RegistryBackend {
        fn read_widgets(
            &self,
        ) -> Result<super::super::plan::CapturedRepresentation, super::super::handlers::HandlerError>
        {
            self.read()
        }

        fn widgets_externally_managed(&self) -> Result<bool, super::super::handlers::HandlerError> {
            Ok(false)
        }

        fn write_widgets(
            &self,
            state: &super::super::plan::CapturedRepresentation,
        ) -> Result<(), super::super::handlers::HandlerError> {
            self.write(state)
        }

        fn read_task_view(
            &self,
        ) -> Result<super::super::plan::CapturedRepresentation, super::super::handlers::HandlerError>
        {
            self.read()
        }

        fn task_view_externally_managed(
            &self,
        ) -> Result<bool, super::super::handlers::HandlerError> {
            Ok(false)
        }

        fn write_task_view(
            &self,
            state: &super::super::plan::CapturedRepresentation,
        ) -> Result<(), super::super::handlers::HandlerError> {
            self.write(state)
        }

        fn read_show_desktop(
            &self,
        ) -> Result<super::super::plan::CapturedRepresentation, super::super::handlers::HandlerError>
        {
            self.read()
        }

        fn show_desktop_externally_managed(
            &self,
        ) -> Result<bool, super::super::handlers::HandlerError> {
            Ok(false)
        }

        fn write_show_desktop(
            &self,
            state: &super::super::plan::CapturedRepresentation,
        ) -> Result<(), super::super::handlers::HandlerError> {
            self.write(state)
        }

        fn read_cleanup(
            &self,
            _: super::super::handlers::CleanupSetting,
        ) -> Result<super::super::plan::CapturedRepresentation, super::super::handlers::HandlerError>
        {
            self.read()
        }

        fn cleanup_externally_managed(
            &self,
            _: super::super::handlers::CleanupSetting,
        ) -> Result<bool, super::super::handlers::HandlerError> {
            Ok(false)
        }

        fn write_cleanup(
            &self,
            _: super::super::handlers::CleanupSetting,
            state: &super::super::plan::CapturedRepresentation,
        ) -> Result<(), super::super::handlers::HandlerError> {
            self.write(state)
        }
    }

    #[cfg(not(feature = "mutation-alpha"))]
    fn registry_context(enabled: bool) -> super::super::broker::BrokerContext {
        super::super::broker::BrokerContext {
            machine_id: owner_scope(),
            inspection_id: "registry-inspection".into(),
            inspection_timestamp: crate::inspection::timestamp(),
            source_observation_id: "registry-inspection/welcome_experience".into(),
            windows_build: 26_100,
            edition: "Professional".into(),
            architecture: "64-bit".into(),
            authority: "user".into(),
            authority_acceptable: true,
            confidence: "confirmed_representation".into(),
            confidence_sufficient: true,
            applicability: "applicable".into(),
            applicable: true,
            evidence_fingerprint: super::super::plan::hash_text("m4.1-lock-regression"),
            desired_state_revision_id: None,
            detector_current_enabled: Some(enabled),
            detector_status: "successful".into(),
        }
    }

    fn package(operation: PackageOperationId, version: &str) -> CurrentUserPackage {
        let name = operation.package_name();
        CurrentUserPackage {
            name: name.into(),
            package_full_name: format!("{name}_{version}_x64__8wekyb3d8bbwe"),
            package_family_name: format!("{name}_8wekyb3d8bbwe"),
            version: version.into(),
            architecture: "x64".into(),
            publisher: "CN=Microsoft Corporation".into(),
            publisher_id: "8wekyb3d8bbwe".into(),
            resource_id: None,
            install_location_present: true,
            status: "ok".into(),
            framework: false,
            resource_package: false,
            bundle: false,
            optional: false,
            dependencies: Vec::new(),
        }
    }

    fn context(
        operation: PackageOperationId,
        present: bool,
        provisioned: bool,
    ) -> PackageMutationContext {
        let target = package(operation, "1.0.0.0");
        PackageMutationContext {
            machine_id: owner_scope(),
            inspection_id: "inspection-1".into(),
            source_observation_id: "inspection-1/component".into(),
            windows_build: 26100,
            edition: "Professional".into(),
            detector: Some(PackageDetectorEvidence {
                component_id: operation.component_id(),
                detector_status: "successful".into(),
                completeness: PackageCompleteness::Complete,
                package_name: operation.package_name().into(),
                package_full_name: present.then_some(target.package_full_name.clone()),
                package_family_name: present.then_some(target.package_family_name),
                current_user: if present {
                    PackageRegistrationState::Present
                } else {
                    PackageRegistrationState::Absent
                },
                other_users: PackageRegistrationState::Absent,
                provisioning: if provisioned {
                    PackageProvisioningState::Provisioned
                } else {
                    PackageProvisioningState::NotProvisioned
                },
                provisioned_package_full_name: provisioned.then_some(target.package_full_name),
                non_removable: false,
                framework: false,
                resource_package: false,
                bundle: false,
                install_location_present: Some(true),
            }),
        }
    }

    fn owner_scope() -> String {
        crate::owner_scope::from_stable_ids("m4-test-machine", "S-1-5-21-4000")
    }

    fn temp_db(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "deslopper-m4-{label}-{}-{}.sqlite3",
            std::process::id(),
            PACKAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn cleanup(path: &std::path::Path) {
        for candidate in [
            path.to_path_buf(),
            path.with_extension("sqlite3-wal"),
            path.with_extension("sqlite3-shm"),
            path.with_extension("mutation-alpha.lock"),
        ] {
            let _ = std::fs::remove_file(candidate);
        }
    }

    #[test]
    fn every_candidate_is_closed_exact_and_wildcards_are_impossible() {
        for operation in PackageOperationId::ALL {
            assert!(!operation.package_name().contains('*'));
            assert!(!operation.package_name().contains('?'));
            assert!(
                validate_deployment_identity(
                    operation,
                    &package(operation, "1.0.0.0").package_full_name
                )
                .is_ok()
            );
            assert!(
                validate_deployment_identity(operation, "Contoso.Copilot_1.0.0.0_x64__test")
                    .is_err()
            );
        }
        assert!(serde_json::from_value::<OwnerPackageRemoveRequest>(serde_json::json!({"operationId":"remove_phone_link_current_user","sourceInspectionId":"i","packageName":"arbitrary"})).is_err());
    }

    #[test]
    fn present_absent_multiple_versions_and_protected_states_are_classified() {
        for operation in PackageOperationId::ALL {
            let one = package(operation, "1.0.0.0");
            assert_eq!(
                actionability(
                    operation,
                    &context(operation, true, false),
                    std::slice::from_ref(&one)
                )
                .status,
                PackageActionabilityStatus::Ready
            );
            assert_eq!(
                actionability(operation, &context(operation, false, false), &[]).status,
                PackageActionabilityStatus::AlreadyAbsent
            );
            assert_eq!(
                actionability(
                    operation,
                    &context(operation, true, false),
                    &[one.clone(), package(operation, "2.0.0.0")]
                )
                .status,
                PackageActionabilityStatus::RemoveUnavailable
            );
            let mut protected = context(operation, true, false);
            protected.detector.as_mut().unwrap().non_removable = true;
            assert_eq!(
                actionability(operation, &protected, &[one]).status,
                PackageActionabilityStatus::RemoveUnavailable
            );
        }
    }

    #[test]
    fn single_remove_does_not_self_lock_and_reaches_fake_backend_durably() {
        for operation in PackageOperationId::ALL {
            let path = temp_db(operation.key());
            let dependency_operation =
                if operation == PackageOperationId::RemoveSolitaireCurrentUser {
                    PackageOperationId::RemovePhoneLinkCurrentUser
                } else {
                    PackageOperationId::RemoveSolitaireCurrentUser
                };
            let dependency = package(dependency_operation, "2.0.0.0");
            let mut target = package(operation, "1.0.0.0");
            if operation != PackageOperationId::RemoveSolitaireCurrentUser {
                target.dependencies.push(PackageDependencyState {
                    name: dependency.name.clone(),
                    package_full_name: dependency.package_full_name.clone(),
                    package_family_name: dependency.package_family_name.clone(),
                    version: dependency.version.clone(),
                    architecture: dependency.architecture.clone(),
                    framework: false,
                    resource_package: false,
                });
            }
            let backend = Arc::new(FakeBackend::new(vec![target, dependency]));
            let broker = PackageBroker::at(backend.clone(), path.clone());
            let result = broker
                .remove(operation, &context(operation, true, false), || {
                    Ok(context(operation, false, false))
                })
                .unwrap();
            assert_eq!(result.transaction.status, PackageTransactionStatus::Removed);
            assert!(result.transaction.detector_verified);
            assert!(result.transaction.provisioning_unchanged);
            assert!(result.transaction.disappeared_package_full_names.is_empty());
            assert_eq!(backend.removes.load(Ordering::SeqCst), 1);
            assert_eq!(broker.history(&owner_scope()).unwrap().len(), 1);
            cleanup(&path);
        }
    }

    #[test]
    fn rejected_unchanged_and_unexpected_collateral_are_honest() {
        let operation = PackageOperationId::RemovePhoneLinkCurrentUser;
        let path = temp_db("outcomes");
        let backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        backend.reject_remove.store(true, Ordering::SeqCst);
        let broker = PackageBroker::at(backend.clone(), path.clone());
        let rejected = broker
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, true, false))
            })
            .unwrap();
        assert_eq!(
            rejected.transaction.status,
            PackageTransactionStatus::RejectedUnchanged
        );
        cleanup(&path);

        let path = temp_db("collateral");
        let collateral = package(PackageOperationId::RemoveSolitaireCurrentUser, "1.0.0.0");
        let backend = Arc::new(FakeBackend::new(vec![
            package(operation, "1.0.0.0"),
            collateral.clone(),
        ]));
        *backend.collateral.lock().unwrap() = Some(collateral);
        let broker = PackageBroker::at(backend, path.clone());
        let result = broker
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, false, false))
            })
            .unwrap();
        assert_eq!(
            result.transaction.status,
            PackageTransactionStatus::UnexpectedCollateralChange
        );
        cleanup(&path);
    }

    #[test]
    fn ambiguous_result_detector_disagreement_and_provisioning_drift_need_attention() {
        let operation = PackageOperationId::RemovePhoneLinkCurrentUser;
        let path = temp_db("ambiguous");
        let backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        backend.ambiguous_after_remove.store(true, Ordering::SeqCst);
        let broker = PackageBroker::at(backend, path.clone());
        let ambiguous = broker
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, false, false))
            })
            .unwrap();
        assert_eq!(
            ambiguous.transaction.status,
            PackageTransactionStatus::ResultAmbiguous
        );
        cleanup(&path);

        let path = temp_db("detector-disagreed");
        let backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        let broker = PackageBroker::at(backend, path.clone());
        let disagreed = broker
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, true, false))
            })
            .unwrap();
        assert_eq!(
            disagreed.transaction.status,
            PackageTransactionStatus::NeedsAttention
        );
        cleanup(&path);

        let path = temp_db("provisioning-drift");
        let backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        let broker = PackageBroker::at(backend, path.clone());
        let drifted = broker
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, false, true))
            })
            .unwrap();
        assert_eq!(
            drifted.transaction.status,
            PackageTransactionStatus::NeedsAttention
        );
        assert!(!drifted.transaction.provisioning_unchanged);
        cleanup(&path);
    }

    #[test]
    fn restore_capability_and_restore_results_are_state_dependent() {
        let operation = PackageOperationId::RemoveClipchampCurrentUser;
        let target = package(operation, "1.0.0.0");
        assert_eq!(
            classify_restore(
                context(operation, true, true).detector.as_ref().unwrap(),
                &target,
                std::slice::from_ref(&target)
            ),
            PackageRestoreCapability::RestoreAvailable
        );
        assert_eq!(
            classify_restore(
                context(operation, true, false).detector.as_ref().unwrap(),
                &target,
                std::slice::from_ref(&target)
            ),
            PackageRestoreCapability::ReinstallRequired
        );

        let path = temp_db("restore");
        let backend = Arc::new(FakeBackend::new(vec![target]));
        let broker = PackageBroker::at(backend, path.clone());
        let removed = broker
            .remove(operation, &context(operation, true, true), || {
                Ok(context(operation, false, true))
            })
            .unwrap();
        assert_eq!(
            removed.transaction.restore_capability,
            PackageRestoreCapability::RestoreAvailable
        );
        let restored = broker
            .restore(
                &removed.transaction.transaction_id,
                &context(operation, false, true),
                || Ok(context(operation, true, true)),
            )
            .unwrap();
        assert_eq!(
            restored.transaction.status,
            PackageTransactionStatus::Restored
        );
        cleanup(&path);
    }

    #[test]
    fn no_restore_owner_mismatch_and_newer_version_semantics_are_enforced() {
        let operation = PackageOperationId::RemoveConsumerCopilotCurrentUser;
        let path = temp_db("restore-boundary");
        let backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        let broker = PackageBroker::at(backend.clone(), path.clone());
        let removed = broker
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, false, false))
            })
            .unwrap();
        assert!(
            broker
                .restore(
                    &removed.transaction.transaction_id,
                    &context(operation, false, false),
                    || Ok(context(operation, true, false))
                )
                .is_err()
        );
        cleanup(&path);

        let path = temp_db("newer");
        let backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        let broker = PackageBroker::at(backend.clone(), path.clone());
        let removed = broker
            .remove(operation, &context(operation, true, true), || {
                Ok(context(operation, false, true))
            })
            .unwrap();
        backend
            .inventory
            .lock()
            .unwrap()
            .push(package(operation, "2.0.0.0"));
        let newer = broker
            .restore(
                &removed.transaction.transaction_id,
                &context(operation, false, true),
                || Ok(context(operation, true, true)),
            )
            .unwrap();
        assert_eq!(
            newer.transaction.status,
            PackageTransactionStatus::RestoredNewerVersion
        );
        let mut other_owner = context(operation, false, true);
        other_owner.machine_id =
            crate::owner_scope::from_stable_ids("m4-test-machine", "S-1-5-21-4001");
        assert!(
            broker
                .restore(&removed.transaction.transaction_id, &other_owner, || Ok(
                    context(operation, true, true)
                ))
                .is_err()
        );
        cleanup(&path);
    }

    #[test]
    fn restore_failure_and_cross_process_lock_need_attention_without_live_deployment() {
        let operation = PackageOperationId::RemoveSolitaireCurrentUser;
        let path = temp_db("restore-failure");
        let backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        let broker = PackageBroker::at(backend.clone(), path.clone());
        let removed = broker
            .remove(operation, &context(operation, true, true), || {
                Ok(context(operation, false, true))
            })
            .unwrap();
        backend.reject_restore.store(true, Ordering::SeqCst);
        let failed = broker
            .restore(
                &removed.transaction.transaction_id,
                &context(operation, false, true),
                || Ok(context(operation, false, true)),
            )
            .unwrap();
        assert_eq!(
            failed.transaction.status,
            PackageTransactionStatus::RestoreFailed
        );
        let _held = OwnerMutationProcessLock::acquire(broker.journal.lock_path()).unwrap();
        assert!(
            broker
                .remove(operation, &context(operation, false, true), || Ok(context(
                    operation, false, true
                )))
                .is_err()
        );
        cleanup(&path);
    }

    #[test]
    fn separate_concurrent_package_mutation_is_blocked_then_released() {
        let first_operation = PackageOperationId::RemovePhoneLinkCurrentUser;
        let second_operation = PackageOperationId::RemoveSolitaireCurrentUser;
        let path = temp_db("concurrent-package");
        let backend = Arc::new(FakeBackend::new(vec![
            package(first_operation, "1.0.0.0"),
            package(second_operation, "1.0.0.0"),
        ]));
        let entered = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        backend.block_next_remove(entered.clone(), release.clone());
        let first = PackageBroker::at(backend.clone(), path.clone());
        let second = PackageBroker::at(backend.clone(), path.clone());
        let running = std::thread::spawn(move || {
            first.remove(
                first_operation,
                &context(first_operation, true, false),
                || Ok(context(first_operation, false, false)),
            )
        });
        entered.wait();
        let blocked = second
            .remove(
                second_operation,
                &context(second_operation, true, false),
                || Ok(context(second_operation, false, false)),
            )
            .unwrap_err();
        assert_eq!(blocked, "Another Deslopper change is still in progress.");
        assert_eq!(backend.removes.load(Ordering::SeqCst), 1);
        release.wait();
        assert_eq!(
            running.join().unwrap().unwrap().transaction.status,
            PackageTransactionStatus::Removed
        );

        let after_release = second
            .remove(
                second_operation,
                &context(second_operation, true, false),
                || Ok(context(second_operation, false, false)),
            )
            .unwrap();
        assert_eq!(
            after_release.transaction.status,
            PackageTransactionStatus::Removed
        );
        assert_eq!(backend.removes.load(Ordering::SeqCst), 2);
        cleanup(&path);
    }

    #[test]
    fn rejected_failed_restored_and_interrupted_transactions_release_ownership() {
        let first = PackageOperationId::RemovePhoneLinkCurrentUser;
        let second = PackageOperationId::RemoveSolitaireCurrentUser;

        let rejected_path = temp_db("release-rejected");
        let rejected_backend = Arc::new(FakeBackend::new(vec![package(first, "1.0.0.0")]));
        rejected_backend.reject_remove.store(true, Ordering::SeqCst);
        let rejected_broker = PackageBroker::at(rejected_backend.clone(), rejected_path.clone());
        let rejected = rejected_broker
            .remove(first, &context(first, true, false), || {
                Ok(context(first, true, false))
            })
            .unwrap();
        assert_eq!(
            rejected.transaction.status,
            PackageTransactionStatus::RejectedUnchanged
        );
        rejected_backend
            .reject_remove
            .store(false, Ordering::SeqCst);
        assert_eq!(
            rejected_broker
                .remove(first, &context(first, true, false), || {
                    Ok(context(first, false, false))
                })
                .unwrap()
                .transaction
                .status,
            PackageTransactionStatus::Removed
        );
        cleanup(&rejected_path);

        let failed_path = temp_db("release-failed");
        let failed_backend = Arc::new(FakeBackend::new(vec![
            package(first, "1.0.0.0"),
            package(second, "1.0.0.0"),
        ]));
        failed_backend
            .ambiguous_after_remove
            .store(true, Ordering::SeqCst);
        let failed_broker = PackageBroker::at(failed_backend.clone(), failed_path.clone());
        let failed = failed_broker
            .remove(first, &context(first, true, false), || {
                Ok(context(first, false, false))
            })
            .unwrap();
        assert_eq!(
            failed.transaction.status,
            PackageTransactionStatus::ResultAmbiguous
        );
        failed_backend
            .ambiguous_after_remove
            .store(false, Ordering::SeqCst);
        assert_eq!(
            failed_broker
                .remove(second, &context(second, true, false), || {
                    Ok(context(second, false, false))
                })
                .unwrap()
                .transaction
                .status,
            PackageTransactionStatus::Removed
        );
        cleanup(&failed_path);

        let restored_path = temp_db("release-restored");
        let restored_backend = Arc::new(FakeBackend::new(vec![
            package(first, "1.0.0.0"),
            package(second, "1.0.0.0"),
        ]));
        let restored_broker = PackageBroker::at(restored_backend, restored_path.clone());
        let removed = restored_broker
            .remove(first, &context(first, true, true), || {
                Ok(context(first, false, true))
            })
            .unwrap();
        assert_eq!(
            restored_broker
                .restore(
                    &removed.transaction.transaction_id,
                    &context(first, false, true),
                    || Ok(context(first, true, true)),
                )
                .unwrap()
                .transaction
                .status,
            PackageTransactionStatus::Restored
        );
        assert_eq!(
            restored_broker
                .remove(second, &context(second, true, false), || {
                    Ok(context(second, false, false))
                })
                .unwrap()
                .transaction
                .status,
            PackageTransactionStatus::Removed
        );
        cleanup(&restored_path);

        let interrupted_path = temp_db("release-interrupted");
        let interrupted_backend = Arc::new(FakeBackend::new(vec![package(first, "1.0.0.0")]));
        let interrupted_broker =
            PackageBroker::at(interrupted_backend.clone(), interrupted_path.clone());
        let mut interrupted = new_transaction(
            first,
            &context(first, true, false),
            &crate::inspection::timestamp(),
        )
        .unwrap();
        interrupted.status = PackageTransactionStatus::Removing;
        interrupted_broker.journal.save(&interrupted).unwrap();
        drop(interrupted_broker);
        let reopened = PackageBroker::at(interrupted_backend, interrupted_path.clone());
        let recovered = reopened.recover_interrupted(&owner_scope()).unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(
            recovered[0].status,
            PackageTransactionStatus::RecoveryRequired
        );
        assert_eq!(
            reopened
                .remove(first, &context(first, true, false), || {
                    Ok(context(first, false, false))
                })
                .unwrap()
                .transaction
                .status,
            PackageTransactionStatus::Removed
        );
        cleanup(&interrupted_path);
    }

    #[cfg(not(feature = "mutation-alpha"))]
    #[test]
    fn live_registry_mutation_blocks_package_then_completed_owner_releases() {
        let path = temp_db("registry-concurrent");
        let entered = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        let registry_backend = Arc::new(RegistryBackend::new(
            super::super::plan::CapturedRepresentation::Dword(1),
            Some((entered.clone(), release.clone())),
        ));
        let registry = super::super::broker::Broker::with_owner_journal(
            registry_backend,
            super::super::journal::MutationJournal::at(path.clone()),
        );
        let running = std::thread::spawn(move || {
            registry.apply_owner_operation(
                super::super::request::MutationOperationId::WelcomeExperienceEnabled,
                super::super::request::MutationTarget::Disabled,
                &registry_context(true),
                || Ok(registry_context(false)),
            )
        });
        entered.wait();

        let operation = PackageOperationId::RemoveConsumerCopilotCurrentUser;
        let package_backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        let packages = PackageBroker::at(package_backend.clone(), path.clone());
        let blocked = packages
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, false, false))
            })
            .unwrap_err();
        assert_eq!(blocked, "Another Deslopper change is still in progress.");
        assert_eq!(package_backend.removes.load(Ordering::SeqCst), 0);

        release.wait();
        running.join().unwrap().unwrap();
        let after_completion = packages
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, false, false))
            })
            .unwrap();
        assert_eq!(
            after_completion.transaction.status,
            PackageTransactionStatus::Removed
        );
        assert_eq!(package_backend.removes.load(Ordering::SeqCst), 1);
        cleanup(&path);
    }

    #[cfg(not(feature = "mutation-alpha"))]
    #[test]
    fn restored_m3_history_and_persistent_lock_file_do_not_block_package() {
        let path = temp_db("registry-history");
        let registry_backend = Arc::new(RegistryBackend::new(
            super::super::plan::CapturedRepresentation::Dword(1),
            None,
        ));
        let registry = super::super::broker::Broker::with_owner_journal(
            registry_backend,
            super::super::journal::MutationJournal::at(path.clone()),
        );
        let applied = registry
            .apply_owner_operation(
                super::super::request::MutationOperationId::WelcomeExperienceEnabled,
                super::super::request::MutationTarget::Disabled,
                &registry_context(true),
                || Ok(registry_context(false)),
            )
            .unwrap();
        assert_eq!(
            applied.transaction.handler_version,
            super::super::OWNER_HANDLER_VERSION
        );
        registry
            .undo_owner_operation(
                &applied.transaction.transaction_id,
                &registry_context(false),
                || Ok(registry_context(true)),
            )
            .unwrap();
        drop(registry);

        let operation = PackageOperationId::RemoveConsumerCopilotCurrentUser;
        let package_backend = Arc::new(FakeBackend::new(vec![package(operation, "1.0.0.0")]));
        let packages = PackageBroker::at(package_backend.clone(), path.clone());
        let result = packages
            .remove(operation, &context(operation, true, false), || {
                Ok(context(operation, false, false))
            })
            .unwrap();
        assert_eq!(result.transaction.status, PackageTransactionStatus::Removed);
        assert_eq!(package_backend.removes.load(Ordering::SeqCst), 1);
        cleanup(&path);
    }
}
