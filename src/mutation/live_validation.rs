use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    governance::{PolicyTargetType, committed_live_validation_policy},
    plan::{CapturedState, hash_text},
    request::{MutationOperationId, MutationTarget},
    transaction::{MutationTransaction, TransactionStatus},
};
use crate::platform::PlatformInfo;

pub const LIVE_VALIDATION_FLAG: &str = "--enable-live-validation";
pub const SCENARIO_ARGUMENT: &str = "--validation-scenario=";
pub const EXPECTED_MACHINE_ARGUMENT: &str = "--expected-machine-id=";
pub const EXPECTED_CHECKPOINT_ARGUMENT: &str = "--expected-checkpoint-id=";
pub const EXPECTED_SOURCE_COMMIT_ARGUMENT: &str = "--expected-source-commit=";
const MAX_PHYSICAL_APPROVAL_LIFETIME_MS: u64 = 30 * 60 * 1_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationMaturity {
    Unvalidated,
    SyntheticTested,
    LiveTestedSingleBuild,
    LiveTestedMultiBuild,
    LiveTestedMultiEdition,
    PolicyConflictTested,
    RecoveryTested,
    AlphaValidated,
    Rejected,
}

impl ValidationMaturity {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unvalidated => "Internal alpha - unvalidated",
            Self::SyntheticTested => "Internal alpha - not live validated",
            Self::LiveTestedSingleBuild => "Internal alpha - one build live tested",
            Self::LiveTestedMultiBuild => "Internal alpha - multiple builds live tested",
            Self::LiveTestedMultiEdition => "Internal alpha - multiple editions live tested",
            Self::PolicyConflictTested => "Internal alpha - policy conflict tested",
            Self::RecoveryTested => "Internal alpha - recovery tested",
            Self::AlphaValidated => "Internal alpha - live validation complete",
            Self::Rejected => "Rejected - handler unavailable",
        }
    }

    pub const fn allows_internal_alpha(self) -> bool {
        !matches!(self, Self::Rejected)
    }

    #[cfg(test)]
    pub fn advance(self, next: Self) -> Result<Self, &'static str> {
        if next == Self::Rejected || next >= self {
            Ok(next)
        } else {
            Err("Validation maturity cannot move backwards without explicit rejection.")
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatrixScenarioStatus {
    NotProvisioned,
    ProvisionedNotRun,
    Running,
    Passed,
    PassedWithLimitations,
    Failed,
    Blocked,
    HandlerRemoved,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationDimensionResult {
    NotRun,
    Verified,
    PendingRefresh,
    Disagrees,
    IgnoredByShell,
    PolicyOverrode,
    RevertedAfterSignIn,
    Uncertain,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualVerificationOutcome {
    NoChangeNeeded,
    FullyVerified,
    RepresentationAndDetectorVerifiedVisualPending,
    SettingsUiDisagrees,
    StoredValueIgnoredByShell,
    PolicyOverrodeValue,
    RevertedAfterSignIn,
    DetectionUncertain,
    NotCompleted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshRequirement {
    Immediate,
    TaskbarNaturalRefresh,
    SettingsAppReopen,
    DeslopperReopen,
    SignOutSignIn,
    Reboot,
    UnsupportedWithoutExplorerTermination,
    Undetermined,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualVerification {
    pub representation: VerificationDimensionResult,
    pub detector: VerificationDimensionResult,
    pub user_visible_behavior: VerificationDimensionResult,
    pub settings_ui: VerificationDimensionResult,
    pub refresh_requirement: RefreshRequirement,
    pub lifecycle_refresh_completed: bool,
}

impl VisualVerification {
    pub fn outcome(&self) -> VisualVerificationOutcome {
        use VerificationDimensionResult as Dimension;
        if matches!(self.user_visible_behavior, Dimension::IgnoredByShell) {
            return VisualVerificationOutcome::StoredValueIgnoredByShell;
        }
        if matches!(self.settings_ui, Dimension::Disagrees) {
            return VisualVerificationOutcome::SettingsUiDisagrees;
        }
        if matches!(self.user_visible_behavior, Dimension::RevertedAfterSignIn) {
            return VisualVerificationOutcome::RevertedAfterSignIn;
        }
        if self.user_visible_behavior == Dimension::PolicyOverrode
            || self.detector == Dimension::PolicyOverrode
        {
            return VisualVerificationOutcome::PolicyOverrodeValue;
        }
        if self.representation == Dimension::Uncertain
            || self.detector == Dimension::Uncertain
            || self.user_visible_behavior == Dimension::Uncertain
        {
            return VisualVerificationOutcome::DetectionUncertain;
        }
        let lifecycle_ready = !matches!(
            self.refresh_requirement,
            RefreshRequirement::SignOutSignIn | RefreshRequirement::Reboot
        ) || self.lifecycle_refresh_completed;
        if self.representation == Dimension::Verified
            && self.detector == Dimension::Verified
            && self.user_visible_behavior == Dimension::Verified
            && self.settings_ui == Dimension::Verified
            && lifecycle_ready
        {
            VisualVerificationOutcome::FullyVerified
        } else if self.representation == Dimension::Verified && self.detector == Dimension::Verified
        {
            VisualVerificationOutcome::RepresentationAndDetectorVerifiedVisualPending
        } else {
            VisualVerificationOutcome::NotCompleted
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VirtualMachineDetectionResult {
    Detected,
    NotDetected,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentIdentity {
    pub computer_name: String,
    pub machine_id_prefix: String,
    pub edition: String,
    pub build: u32,
    pub update_build_revision: Option<u32>,
    pub virtual_machine_detection: VirtualMachineDetectionResult,
    pub virtual_machine_evidence: String,
    pub current_validation_scenario: Option<String>,
    pub development_host_refused: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveValidationGateStatus {
    pub command_line_opt_in: bool,
    pub governance_policy_loaded: bool,
    pub target_type_policy_allowed: bool,
    pub manifest_loaded: bool,
    pub target_approval_loaded: bool,
    pub denylist_loaded: bool,
    pub scenario_matches: bool,
    pub machine_identity_matches: bool,
    pub checkpoint_matches: bool,
    pub platform_matches: bool,
    pub target_type_matches: bool,
    pub database_belongs_to_guest: bool,
    pub development_host_refused: bool,
    pub approval_id: Option<String>,
    pub approval_source_commit: Option<String>,
    pub approval_source_inspection_id: Option<String>,
    pub approval_evidence_sha256: Option<String>,
    pub approved_operation_scopes: Vec<ApprovedOperationScope>,
    pub maximum_plans: u32,
    pub maximum_executions: u32,
    pub approval_expires_at_epoch_ms: Option<u64>,
    pub approved_target_type: Option<ValidationTargetType>,
    pub approved_target_identity: Option<String>,
    pub approved_target_management: Option<TargetManagementState>,
    pub rollback_environment_available: bool,
    pub local_approval_revalidation_required: bool,
    pub available: bool,
    pub reason: String,
    pub environment: EnvironmentIdentity,
}

impl LiveValidationGateStatus {
    pub fn scope_is_valid(&self) -> bool {
        valid_operation_scopes(&self.approved_operation_scopes)
            && (1..=32).contains(&self.maximum_plans)
            && (1..=32).contains(&self.maximum_executions)
            && self.maximum_executions <= self.maximum_plans
    }

    pub fn allows(&self, operation_id: MutationOperationId, target: MutationTarget) -> bool {
        self.approved_operation_scopes.iter().any(|scope| {
            scope.operation_id == operation_id && scope.allowed_target_states.contains(&target)
        })
    }

    pub fn allowed_targets(&self, operation_id: MutationOperationId) -> Vec<MutationTarget> {
        self.approved_operation_scopes
            .iter()
            .find(|scope| scope.operation_id == operation_id)
            .map_or_else(Vec::new, |scope| scope.allowed_target_states.clone())
    }
}

#[cfg(test)]
impl LiveValidationGateStatus {
    pub fn test_valid() -> Self {
        Self {
            command_line_opt_in: true,
            governance_policy_loaded: true,
            target_type_policy_allowed: true,
            manifest_loaded: true,
            target_approval_loaded: true,
            denylist_loaded: true,
            scenario_matches: true,
            machine_identity_matches: true,
            checkpoint_matches: true,
            platform_matches: true,
            target_type_matches: true,
            database_belongs_to_guest: true,
            development_host_refused: false,
            approval_id: Some("synthetic-test-approval".into()),
            approval_source_commit: Some("a".repeat(40)),
            approval_source_inspection_id: Some("inspection-1".into()),
            approval_evidence_sha256: Some("b".repeat(64)),
            approved_operation_scopes: MutationOperationId::ALL
                .into_iter()
                .map(|operation_id| ApprovedOperationScope {
                    operation_id,
                    allowed_target_states: vec![MutationTarget::Enabled, MutationTarget::Disabled],
                    handler_version: super::HANDLER_VERSION.into(),
                })
                .collect(),
            maximum_plans: 32,
            maximum_executions: 32,
            approval_expires_at_epoch_ms: Some(u64::MAX),
            approved_target_type: Some(ValidationTargetType::VirtualMachine),
            approved_target_identity: Some("b".repeat(64)),
            approved_target_management: Some(TargetManagementState {
                domain_joined: false,
                entra_joined: false,
                workplace_joined: Some(false),
                mdm_enrolled: false,
            }),
            rollback_environment_available: true,
            local_approval_revalidation_required: false,
            available: true,
            reason: "All test identity gates are satisfied.".into(),
            environment: EnvironmentIdentity {
                computer_name: "TEST-VM".into(),
                machine_id_prefix: "machine-".into(),
                edition: "Professional".into(),
                build: 26_100,
                update_build_revision: Some(1),
                virtual_machine_detection: VirtualMachineDetectionResult::Detected,
                virtual_machine_evidence: "Synthetic test VM evidence.".into(),
                current_validation_scenario: Some("broker-test".into()),
                development_host_refused: false,
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValidationScenarioManifest {
    pub schema_version: u32,
    pub scenario_id: String,
    pub expected_machine_id: String,
    pub expected_edition: String,
    pub expected_build: u32,
    pub expected_update_build_revision: Option<u32>,
    pub checkpoint_id: String,
    pub account_class: String,
    pub management_context: String,
    #[serde(default)]
    pub approval_id: Option<String>,
    #[serde(default)]
    pub source_commit: Option<String>,
    #[serde(default)]
    pub source_inspection_id: Option<String>,
    #[serde(default)]
    pub inspection_evidence_sha256: Option<String>,
    #[serde(default)]
    pub approved_operation_scopes: Vec<ApprovedOperationScope>,
    #[serde(default)]
    pub maximum_plans: Option<u32>,
    #[serde(default)]
    pub maximum_executions: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationTargetType {
    VirtualMachine,
    PhysicalLaptop,
}

impl ValidationTargetType {
    const fn policy_type(self) -> PolicyTargetType {
        match self {
            Self::VirtualMachine => PolicyTargetType::VirtualMachine,
            Self::PhysicalLaptop => PolicyTargetType::PhysicalLaptop,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetRecordKind {
    Preparation,
    Approval,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryReadiness {
    NotReady,
    ReadyWithWarnings,
    Ready,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BitLockerRecoveryState {
    NotApplicableUnencrypted,
    RecoveryMaterialConfirmed,
    NotConfirmed,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryMediaState {
    Available,
    BuiltInVerified,
    Missing,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportantDataState {
    Absent,
    PresentBackedUp,
    PresentNotBackedUp,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryRouteState {
    RecoveryPartitionVerified,
    EquivalentRouteVerified,
    Missing,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestartPendingState {
    Clear,
    Pending,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalDisposition {
    ResetBeforeSale,
    ReimageAfterValidation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DevelopmentHostProtectionState {
    PresentDistinct,
    Missing,
    MatchesTarget,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TargetManagementState {
    pub domain_joined: bool,
    pub entra_joined: bool,
    #[serde(default)]
    pub workplace_joined: Option<bool>,
    pub mdm_enrolled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovedOperationScope {
    pub operation_id: MutationOperationId,
    pub allowed_target_states: Vec<MutationTarget>,
    pub handler_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValidationTargetApproval {
    pub schema_version: u32,
    pub record_kind: TargetRecordKind,
    pub scenario_id: String,
    pub target_type: ValidationTargetType,
    pub hashed_machine_identity: String,
    pub windows_edition: String,
    pub windows_build: u32,
    pub windows_ubr: u32,
    pub account_class: String,
    pub management_state: TargetManagementState,
    pub source_checkpoint_commit: String,
    pub handler_versions: BTreeMap<String, String>,
    pub approved_operations: Vec<MutationOperationId>,
    pub approved_target_states: BTreeMap<String, Vec<MutationTarget>>,
    pub evidence_output_location: String,
    pub recovery_readiness: RecoveryReadiness,
    pub important_data_confirmed: bool,
    pub backup_confirmed: bool,
    pub reinstallation_accepted: bool,
    pub winre_verified: bool,
    pub bitlocker_recovery_state: BitLockerRecoveryState,
    pub recovery_media_state: RecoveryMediaState,
    pub development_host_protection_state: DevelopmentHostProtectionState,
    pub explicit_user_approval_timestamp: Option<String>,
    pub approval_status: TargetApprovalStatus,
    pub expires_at_epoch_ms: Option<u64>,
    pub restore_or_reimage_procedure: String,
    pub validation_maturity: String,
    #[serde(default)]
    pub disposable_confirmed: Option<bool>,
    #[serde(default)]
    pub expendable_confirmed: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopedValidationTargetApproval {
    pub schema_version: u32,
    pub record_kind: TargetRecordKind,
    pub approval_id: String,
    pub scenario_id: String,
    pub target_type: ValidationTargetType,
    pub hashed_machine_identity: String,
    pub windows_edition: String,
    pub windows_build: u32,
    pub windows_ubr: u32,
    pub account_class: String,
    pub management_state: TargetManagementState,
    pub source_checkpoint_commit: String,
    pub source_inspection_id: String,
    pub inspection_evidence_sha256: String,
    pub development_host_denylist_identity: String,
    pub approved_operation_scopes: Vec<ApprovedOperationScope>,
    pub maximum_plans: u32,
    pub maximum_executions: u32,
    pub evidence_output_location: String,
    pub recovery_readiness: RecoveryReadiness,
    pub important_data_confirmed: bool,
    pub backup_confirmed: bool,
    pub reinstallation_accepted: bool,
    pub winre_verified: bool,
    pub bitlocker_recovery_state: BitLockerRecoveryState,
    pub recovery_media_state: RecoveryMediaState,
    pub development_host_protection_state: DevelopmentHostProtectionState,
    pub explicit_user_approval_timestamp: Option<String>,
    pub approval_status: TargetApprovalStatus,
    pub expires_at_epoch_ms: Option<u64>,
    pub restore_or_reimage_procedure: String,
    pub validation_maturity: String,
    #[serde(default)]
    pub important_data_state: Option<ImportantDataState>,
    #[serde(default)]
    pub recovery_route_state: Option<RecoveryRouteState>,
    #[serde(default)]
    pub alternate_recovery_device_available: Option<bool>,
    #[serde(default)]
    pub restart_pending_state: Option<RestartPendingState>,
    #[serde(default)]
    pub automatic_repair_disabled: Option<bool>,
    #[serde(default)]
    pub final_plan_approval_required: Option<bool>,
    #[serde(default)]
    pub approval_granted_at_epoch_ms: Option<u64>,
    #[serde(default)]
    pub final_disposition: Option<FinalDisposition>,
    #[serde(default)]
    pub disposable_confirmed: Option<bool>,
    #[serde(default)]
    pub expendable_confirmed: Option<bool>,
}

#[derive(Clone, Debug)]
enum ValidationTargetApprovalDocument {
    Legacy(ValidationTargetApproval),
    Scoped(ScopedValidationTargetApproval),
}

#[derive(Clone, Debug)]
struct NormalizedTargetApproval {
    approval_id: String,
    scenario_id: String,
    target_type: ValidationTargetType,
    hashed_machine_identity: String,
    windows_edition: String,
    windows_build: u32,
    windows_ubr: u32,
    management_state: TargetManagementState,
    source_checkpoint_commit: String,
    source_inspection_id: Option<String>,
    inspection_evidence_sha256: Option<String>,
    development_host_denylist_identity: Option<String>,
    approved_operation_scopes: Vec<ApprovedOperationScope>,
    maximum_plans: u32,
    maximum_executions: u32,
    expires_at_epoch_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DevelopmentHostDenylist {
    schema_version: u32,
    development_host_fingerprints: Vec<String>,
}

pub fn local_validation_directory() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".deslopper")
        .join("local")
}

pub fn load_local_manifest() -> Option<ValidationScenarioManifest> {
    read_json(&local_validation_directory().join("live-validation-scenario.json"))
}

/// Re-read committed policy, the authorizing scoped approval, and the local
/// development-host denylist immediately before an operation is accepted.
/// Test-only gate values may opt out because they have no local documents.
pub fn local_scoped_approval_allows(
    status: &LiveValidationGateStatus,
    operation_id: MutationOperationId,
    target: MutationTarget,
) -> bool {
    if !status.local_approval_revalidation_required {
        return status.allows(operation_id, target);
    }

    scoped_approval_document_allows(
        status,
        operation_id,
        target,
        &local_validation_directory().join("approved-validation-target.json"),
        &local_validation_directory().join("development-host-denylist.json"),
        None,
        current_epoch_ms(),
    )
}

fn scoped_approval_document_allows(
    status: &LiveValidationGateStatus,
    operation_id: MutationOperationId,
    target: MutationTarget,
    approval_path: &std::path::Path,
    denylist_path: &std::path::Path,
    policy_source: Option<&str>,
    now_epoch_ms: u64,
) -> bool {
    let policy = policy_source.map_or_else(
        committed_live_validation_policy,
        super::governance::LiveValidationPolicy::parse,
    );
    let Ok(policy) = policy else {
        return false;
    };
    let document = read_target_approval(approval_path);
    if document.is_none() {
        return false;
    }
    let Some(approval) = document
        .as_ref()
        .and_then(|value| normalize_target_approval(value, now_epoch_ms))
    else {
        return false;
    };
    let denylist = read_json::<DevelopmentHostDenylist>(denylist_path);
    let Some(denylist) = denylist.filter(valid_development_host_denylist) else {
        return false;
    };
    let denylist_identity = approval.development_host_denylist_identity.as_deref();

    let denylist_binding_valid = denylist_identity.is_none_or(|identity| {
        Some(identity)
            == denylist
                .development_host_fingerprints
                .first()
                .map(String::as_str)
    });

    policy.allows(approval.target_type.policy_type())
        && status.governance_policy_loaded
        && status.target_type_policy_allowed
        && status.approved_target_type == Some(approval.target_type)
        && status.approved_target_identity.as_deref()
            == Some(approval.hashed_machine_identity.as_str())
        && status.approved_target_management.as_ref() == Some(&approval.management_state)
        && denylist_binding_valid
        && !denylist
            .development_host_fingerprints
            .contains(&approval.hashed_machine_identity)
        && status.approval_id.as_deref() == Some(approval.approval_id.as_str())
        && status.approval_source_commit.as_deref()
            == Some(approval.source_checkpoint_commit.as_str())
        && status.approval_source_inspection_id.as_deref()
            == approval.source_inspection_id.as_deref()
        && status.approval_evidence_sha256.as_deref()
            == approval.inspection_evidence_sha256.as_deref()
        && status.approved_operation_scopes == approval.approved_operation_scopes
        && status.maximum_plans == approval.maximum_plans
        && status.maximum_executions == approval.maximum_executions
        && approval.approved_operation_scopes.iter().any(|scope| {
            scope.operation_id == operation_id && scope.allowed_target_states.contains(&target)
        })
}

pub fn build_live_validation_gate(
    platform: Option<&PlatformInfo>,
    stored_machine_id: Option<&str>,
    latest_inspection_id: Option<&str>,
    arguments: &[String],
) -> LiveValidationGateStatus {
    let computer_name_result = std::env::var("COMPUTERNAME");
    let computer_name = computer_name_result
        .as_deref()
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unknown")
        .to_owned();
    let scenario_argument = argument_value(arguments, SCENARIO_ARGUMENT);
    let expected_machine_argument = argument_value(arguments, EXPECTED_MACHINE_ARGUMENT);
    let expected_checkpoint_argument = argument_value(arguments, EXPECTED_CHECKPOINT_ARGUMENT);
    let expected_source_commit_argument =
        argument_value(arguments, EXPECTED_SOURCE_COMMIT_ARGUMENT);
    let command_line_opt_in = arguments.iter().any(|value| value == LIVE_VALIDATION_FLAG);
    let directory = local_validation_directory();
    let manifest =
        read_json::<ValidationScenarioManifest>(&directory.join("live-validation-scenario.json"));
    let target_approval_document =
        read_target_approval(&directory.join("approved-validation-target.json"));
    let target_approval = target_approval_document
        .as_ref()
        .and_then(|value| normalize_target_approval(value, current_epoch_ms()));
    let denylist =
        read_json::<DevelopmentHostDenylist>(&directory.join("development-host-denylist.json"));
    let governance_policy = committed_live_validation_policy();

    let edition = platform.map_or_else(|| "unknown".into(), |value| value.edition.clone());
    let build = platform.map_or(0, |value| value.build);
    let update_build_revision = platform.and_then(|value| value.update_build_revision);
    let current_machine_id = platform.map(|value| {
        let mut current = value.clone();
        current.device_name = Some(computer_name.clone());
        crate::persistence::machine_identity(&current)
    });
    let machine_id_prefix = current_machine_id
        .as_deref()
        .unwrap_or("unknown")
        .chars()
        .take(8)
        .collect();
    let (virtual_machine_detection, virtual_machine_evidence) = detect_virtual_machine(platform);
    let fingerprint = development_host_fingerprint(&computer_name, &edition, build);
    let machine_identity_confident = computer_name_result.is_ok()
        && platform.is_some()
        && edition != "unknown"
        && build > 0
        && update_build_revision.is_some();
    let development_host_refused = denylist.as_ref().is_some_and(|value| {
        value.schema_version == 1
            && value
                .development_host_fingerprints
                .iter()
                .any(|candidate| candidate == &fingerprint)
    });
    let manifest_loaded = manifest
        .as_ref()
        .is_some_and(|value| matches!(value.schema_version, 1 | 2));
    let target_approval_loaded = target_approval.is_some();
    let denylist_loaded = denylist
        .as_ref()
        .is_some_and(valid_development_host_denylist);
    let governance_policy_loaded = governance_policy.is_ok();
    let target_type_policy_allowed = target_approval.as_ref().is_some_and(|approval| {
        governance_policy
            .as_ref()
            .is_ok_and(|policy| policy.allows(approval.target_type.policy_type()))
    });
    let scenario_matches = manifest
        .as_ref()
        .is_some_and(|value| Some(value.scenario_id.as_str()) == scenario_argument.as_deref());
    let scenario_machine_identity_matches = manifest.as_ref().is_some_and(|value| {
        Some(value.expected_machine_id.as_str()) == expected_machine_argument.as_deref()
            && value.expected_machine_id == fingerprint
    });
    let machine_identity_matches = machine_identity_confident
        && scenario_machine_identity_matches
        && target_approval
            .as_ref()
            .is_some_and(|value| value.hashed_machine_identity == fingerprint);
    let checkpoint_matches = manifest.as_ref().is_some_and(|value| {
        Some(value.checkpoint_id.as_str()) == expected_checkpoint_argument.as_deref()
    });
    let source_binding_matches = manifest.as_ref().is_some_and(|value| {
        if value.schema_version == 1 {
            target_approval_document.as_ref().is_some_and(|approval| {
                matches!(approval, ValidationTargetApprovalDocument::Legacy(_))
            })
        } else {
            target_approval.as_ref().is_some_and(|approval| {
                value.approval_id.as_deref() == Some(approval.approval_id.as_str())
                    && value.source_commit.as_deref()
                        == Some(approval.source_checkpoint_commit.as_str())
                    && value.source_commit.as_deref() == expected_source_commit_argument.as_deref()
                    && value.source_inspection_id.as_deref()
                        == approval.source_inspection_id.as_deref()
                    && value.source_inspection_id.as_deref() == latest_inspection_id
                    && value.inspection_evidence_sha256.as_deref()
                        == approval.inspection_evidence_sha256.as_deref()
            })
        }
    });
    let approval_scope_matches = manifest.as_ref().is_some_and(|value| {
        target_approval.as_ref().is_some_and(|approval| {
            value.schema_version == 1
                || (value.approved_operation_scopes == approval.approved_operation_scopes
                    && value.maximum_plans == Some(approval.maximum_plans)
                    && value.maximum_executions == Some(approval.maximum_executions))
        })
    });
    let rollback_source_binding_matches = manifest.as_ref().is_some_and(|value| {
        value.schema_version == 1
            || value.source_commit.as_deref() == expected_source_commit_argument.as_deref()
    });
    let denylist_binding_matches = target_approval.as_ref().is_some_and(|approval| {
        approval
            .development_host_denylist_identity
            .as_ref()
            .is_none_or(|identity| {
                denylist.as_ref().is_some_and(|value| {
                    value.development_host_fingerprints.len() == 1
                        && value.development_host_fingerprints[0] == *identity
                })
            })
    });
    let scenario_platform_matches = manifest.as_ref().is_some_and(|value| {
        let management_matches = match value.management_context.as_str() {
            "domain" => platform.and_then(|item| item.domain_joined) == Some(true),
            "mdm" => platform.and_then(|item| item.mdm_enrolled) == Some(true),
            "none" => {
                platform.and_then(|item| item.domain_joined) == Some(false)
                    && platform.and_then(|item| item.entra_joined) == Some(false)
                    && platform.and_then(|item| item.workplace_joined) == Some(false)
                    && platform.and_then(|item| item.mdm_enrolled) == Some(false)
            }
            "local-policy" => true,
            _ => false,
        };
        value.expected_edition == edition
            && value.expected_build == build
            && value
                .expected_update_build_revision
                .is_none_or(|expected| update_build_revision == Some(expected))
            && management_matches
    });
    let target_platform_matches = target_approval.as_ref().is_some_and(|value| {
        value.scenario_id
            == manifest
                .as_ref()
                .map(|scenario| scenario.scenario_id.as_str())
                .unwrap_or_default()
            && value.windows_edition == edition
            && value.windows_build == build
            && update_build_revision == Some(value.windows_ubr)
            && platform.is_some_and(|current| {
                let expected_workplace = value
                    .management_state
                    .workplace_joined
                    .unwrap_or(value.management_state.entra_joined);
                current.domain_joined == Some(value.management_state.domain_joined)
                    && current.entra_joined == Some(value.management_state.entra_joined)
                    && current.workplace_joined == Some(expected_workplace)
                    && current.mdm_enrolled == Some(value.management_state.mdm_enrolled)
            })
    });
    let platform_matches = scenario_platform_matches && target_platform_matches;
    let target_type_matches = target_approval.as_ref().is_some_and(|value| {
        target_type_matches_detection(value.target_type, virtual_machine_detection)
    });
    let database_belongs_to_guest = current_machine_id.as_deref() == stored_machine_id;
    let rollback_environment_available = command_line_opt_in
        && manifest_loaded
        && denylist_loaded
        && scenario_matches
        && scenario_machine_identity_matches
        && checkpoint_matches
        && rollback_source_binding_matches
        && scenario_platform_matches
        && database_belongs_to_guest
        && !development_host_refused;
    let (available, reason) = evaluate_gate(&GateFacts {
        command_line_opt_in,
        governance_policy_loaded,
        target_type_policy_allowed,
        manifest_loaded,
        target_approval_loaded,
        denylist_loaded,
        scenario_matches,
        machine_identity_matches,
        checkpoint_matches,
        source_binding_matches,
        approval_scope_matches,
        denylist_binding_matches,
        platform_matches,
        target_type_matches,
        database_belongs_to_guest,
        development_host_refused,
    });

    let approval_id = target_approval
        .as_ref()
        .map(|value| value.approval_id.clone());
    let approval_source_commit = target_approval
        .as_ref()
        .map(|value| value.source_checkpoint_commit.clone());
    let approval_source_inspection_id = target_approval
        .as_ref()
        .and_then(|value| value.source_inspection_id.clone());
    let approval_evidence_sha256 = target_approval
        .as_ref()
        .and_then(|value| value.inspection_evidence_sha256.clone());
    let approved_operation_scopes = target_approval
        .as_ref()
        .map_or_else(Vec::new, |value| value.approved_operation_scopes.clone());
    let maximum_plans = target_approval
        .as_ref()
        .map_or(0, |value| value.maximum_plans);
    let maximum_executions = target_approval
        .as_ref()
        .map_or(0, |value| value.maximum_executions);
    let approval_expires_at_epoch_ms = target_approval
        .as_ref()
        .map(|value| value.expires_at_epoch_ms);
    let approved_target_type = target_approval.as_ref().map(|value| value.target_type);
    let approved_target_identity = target_approval
        .as_ref()
        .map(|value| value.hashed_machine_identity.clone());
    let approved_target_management = target_approval
        .as_ref()
        .map(|value| value.management_state.clone());

    LiveValidationGateStatus {
        command_line_opt_in,
        governance_policy_loaded,
        target_type_policy_allowed,
        manifest_loaded,
        target_approval_loaded,
        denylist_loaded,
        scenario_matches,
        machine_identity_matches,
        checkpoint_matches,
        platform_matches,
        target_type_matches,
        database_belongs_to_guest,
        development_host_refused,
        approval_id,
        approval_source_commit,
        approval_source_inspection_id,
        approval_evidence_sha256,
        approved_operation_scopes,
        maximum_plans,
        maximum_executions,
        approval_expires_at_epoch_ms,
        approved_target_type,
        approved_target_identity,
        approved_target_management,
        rollback_environment_available,
        local_approval_revalidation_required: target_approval_document.is_some(),
        available,
        reason: reason.into(),
        environment: EnvironmentIdentity {
            computer_name,
            machine_id_prefix,
            edition,
            build,
            update_build_revision,
            virtual_machine_detection,
            virtual_machine_evidence,
            current_validation_scenario: manifest.map(|value| value.scenario_id),
            development_host_refused,
        },
    }
}

struct GateFacts {
    command_line_opt_in: bool,
    governance_policy_loaded: bool,
    target_type_policy_allowed: bool,
    manifest_loaded: bool,
    target_approval_loaded: bool,
    denylist_loaded: bool,
    scenario_matches: bool,
    machine_identity_matches: bool,
    checkpoint_matches: bool,
    source_binding_matches: bool,
    approval_scope_matches: bool,
    denylist_binding_matches: bool,
    platform_matches: bool,
    target_type_matches: bool,
    database_belongs_to_guest: bool,
    development_host_refused: bool,
}

fn evaluate_gate(facts: &GateFacts) -> (bool, &'static str) {
    let available = facts.command_line_opt_in
        && facts.governance_policy_loaded
        && facts.target_type_policy_allowed
        && facts.manifest_loaded
        && facts.target_approval_loaded
        && facts.denylist_loaded
        && facts.scenario_matches
        && facts.machine_identity_matches
        && facts.checkpoint_matches
        && facts.source_binding_matches
        && facts.approval_scope_matches
        && facts.denylist_binding_matches
        && facts.platform_matches
        && facts.target_type_matches
        && facts.database_belongs_to_guest
        && !facts.development_host_refused;
    let reason = if facts.development_host_refused {
        "Live mutation is refused on the recorded development host."
    } else if !facts.command_line_opt_in {
        "The separate --enable-live-validation gate is absent."
    } else if !facts.governance_policy_loaded {
        "The committed live-validation target policy is missing or invalid."
    } else if !facts.target_type_policy_allowed {
        "The approval target type is not explicitly allowed by committed policy."
    } else if !facts.denylist_loaded {
        "The local development-host denylist is missing or invalid."
    } else if !facts.target_approval_loaded {
        "The local validation-target approval is missing, incomplete, expired, or invalid."
    } else if !facts.manifest_loaded {
        "The fixed local live-validation scenario manifest is missing or invalid."
    } else if !facts.scenario_matches {
        "The selected scenario does not match the local manifest."
    } else if !facts.machine_identity_matches {
        "The expected test-machine identity does not match this machine."
    } else if !facts.checkpoint_matches {
        "The expected VM checkpoint identity does not match the scenario manifest."
    } else if !facts.source_binding_matches {
        "The approval source commit, inspection, evidence, or approval identity does not match."
    } else if !facts.approval_scope_matches {
        "The scenario operation scopes or plan/execution limits do not match the approval."
    } else if !facts.denylist_binding_matches {
        "The approval development-host denylist binding does not match."
    } else if !facts.platform_matches {
        "The guest edition, build, or UBR does not match the scenario manifest."
    } else if !facts.target_type_matches {
        "The detected target type does not match the approved validation target."
    } else if !facts.database_belongs_to_guest {
        "The Deslopper database does not belong to the current guest identity."
    } else {
        "All live-validation identity gates are satisfied."
    };
    (available, reason)
}

fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_git_commit_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn canonical_utc_timestamp_epoch_ms(value: &str) -> Option<u64> {
    let bytes = value.as_bytes();
    if bytes.len() != 24
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
        || bytes.get(19) != Some(&b'.')
        || bytes.get(23) != Some(&b'Z')
        || bytes.iter().enumerate().any(|(index, byte)| {
            !matches!(index, 4 | 7 | 10 | 13 | 16 | 19 | 23) && !byte.is_ascii_digit()
        })
    {
        return None;
    }
    let number = |start: usize, end: usize| value.get(start..end)?.parse::<u32>().ok();
    let year = number(0, 4)?;
    let month = number(5, 7)?;
    let day = number(8, 10)?;
    let hour = number(11, 13)?;
    let minute = number(14, 16)?;
    let second = number(17, 19)?;
    let millisecond = number(20, 23)?;
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if year < 1970 || day == 0 || day > days_in_month || hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    // Gregorian civil date to days since 1970-01-01. Keeping this local avoids
    // locale- or timezone-dependent parsing in the approval binding path.
    let adjusted_year = i64::from(year) - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    u64::try_from(days)
        .ok()?
        .checked_mul(86_400_000)?
        .checked_add(u64::from(hour) * 3_600_000)?
        .checked_add(u64::from(minute) * 60_000)?
        .checked_add(u64::from(second) * 1_000)?
        .checked_add(u64::from(millisecond))
}

fn valid_development_host_denylist(value: &DevelopmentHostDenylist) -> bool {
    value.schema_version == 1
        && value.development_host_fingerprints.len() == 1
        && value
            .development_host_fingerprints
            .iter()
            .all(|fingerprint| is_sha256(fingerprint))
}

fn read_target_approval(path: &std::path::Path) -> Option<ValidationTargetApprovalDocument> {
    let raw = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    match value
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
    {
        Some(1) => serde_json::from_value(value)
            .ok()
            .map(ValidationTargetApprovalDocument::Legacy),
        Some(2) => serde_json::from_value(value)
            .ok()
            .map(ValidationTargetApprovalDocument::Scoped),
        Some(3) => serde_json::from_value(value)
            .ok()
            .map(ValidationTargetApprovalDocument::Scoped),
        _ => None,
    }
}

fn normalize_target_approval(
    document: &ValidationTargetApprovalDocument,
    now_epoch_ms: u64,
) -> Option<NormalizedTargetApproval> {
    match document {
        ValidationTargetApprovalDocument::Legacy(value) => {
            if value.target_type != ValidationTargetType::VirtualMachine
                || !target_approval_is_complete(value, now_epoch_ms)
            {
                return None;
            }
            let scopes = value
                .approved_operations
                .iter()
                .map(|operation_id| ApprovedOperationScope {
                    operation_id: *operation_id,
                    allowed_target_states: value
                        .approved_target_states
                        .get(operation_id.key())
                        .cloned()
                        .unwrap_or_default(),
                    handler_version: value
                        .handler_versions
                        .get(operation_id.key())
                        .cloned()
                        .unwrap_or_default(),
                })
                .collect();
            Some(NormalizedTargetApproval {
                approval_id: format!("legacy-v1-{}", value.scenario_id),
                scenario_id: value.scenario_id.clone(),
                target_type: value.target_type,
                hashed_machine_identity: value.hashed_machine_identity.clone(),
                windows_edition: value.windows_edition.clone(),
                windows_build: value.windows_build,
                windows_ubr: value.windows_ubr,
                management_state: value.management_state.clone(),
                source_checkpoint_commit: value.source_checkpoint_commit.clone(),
                source_inspection_id: None,
                inspection_evidence_sha256: None,
                development_host_denylist_identity: None,
                approved_operation_scopes: scopes,
                maximum_plans: 32,
                maximum_executions: 32,
                expires_at_epoch_ms: value.expires_at_epoch_ms?,
            })
        }
        ValidationTargetApprovalDocument::Scoped(value) => {
            if !scoped_target_approval_is_complete(value, now_epoch_ms) {
                return None;
            }
            Some(NormalizedTargetApproval {
                approval_id: value.approval_id.clone(),
                scenario_id: value.scenario_id.clone(),
                target_type: value.target_type,
                hashed_machine_identity: value.hashed_machine_identity.clone(),
                windows_edition: value.windows_edition.clone(),
                windows_build: value.windows_build,
                windows_ubr: value.windows_ubr,
                management_state: value.management_state.clone(),
                source_checkpoint_commit: value.source_checkpoint_commit.clone(),
                source_inspection_id: Some(value.source_inspection_id.clone()),
                inspection_evidence_sha256: Some(value.inspection_evidence_sha256.clone()),
                development_host_denylist_identity: Some(
                    value.development_host_denylist_identity.clone(),
                ),
                approved_operation_scopes: value.approved_operation_scopes.clone(),
                maximum_plans: value.maximum_plans,
                maximum_executions: value.maximum_executions,
                expires_at_epoch_ms: value.expires_at_epoch_ms?,
            })
        }
    }
}

fn valid_operation_scopes(scopes: &[ApprovedOperationScope]) -> bool {
    if scopes.is_empty() {
        return false;
    }
    let mut operations = HashSet::new();
    scopes.iter().all(|scope| {
        let mut states = HashSet::new();
        operations.insert(scope.operation_id)
            && !scope.allowed_target_states.is_empty()
            && scope
                .allowed_target_states
                .iter()
                .all(|state| states.insert(*state))
            && scope.handler_version == super::HANDLER_VERSION
    })
}

fn scoped_target_approval_is_complete(
    value: &ScopedValidationTargetApproval,
    now_epoch_ms: u64,
) -> bool {
    let approval_timestamp = value
        .explicit_user_approval_timestamp
        .as_deref()
        .and_then(canonical_utc_timestamp_epoch_ms);
    let common_complete = matches!(value.schema_version, 2 | 3)
        && value.record_kind == TargetRecordKind::Approval
        && value.approval_status == TargetApprovalStatus::Approved
        && !value.approval_id.trim().is_empty()
        && is_sha256(&value.hashed_machine_identity)
        && is_git_commit_id(&value.source_checkpoint_commit)
        && value.source_inspection_id.starts_with("inspection-")
        && is_sha256(&value.inspection_evidence_sha256)
        && is_sha256(&value.development_host_denylist_identity)
        && value.development_host_denylist_identity != value.hashed_machine_identity
        && !value.account_class.trim().is_empty()
        && valid_operation_scopes(&value.approved_operation_scopes)
        && (1..=32).contains(&value.maximum_plans)
        && (1..=32).contains(&value.maximum_executions)
        && value.maximum_executions <= value.maximum_plans
        && value
            .evidence_output_location
            .replace('\\', "/")
            .starts_with(".deslopper/local/")
        && matches!(
            value.recovery_readiness,
            RecoveryReadiness::Ready | RecoveryReadiness::ReadyWithWarnings
        )
        && value.reinstallation_accepted
        && value.winre_verified
        && matches!(
            value.bitlocker_recovery_state,
            BitLockerRecoveryState::NotApplicableUnencrypted
                | BitLockerRecoveryState::RecoveryMaterialConfirmed
        )
        && matches!(
            value.recovery_media_state,
            RecoveryMediaState::Available | RecoveryMediaState::BuiltInVerified
        )
        && value.development_host_protection_state
            == DevelopmentHostProtectionState::PresentDistinct
        && approval_timestamp.is_some_and(|approved| approved <= now_epoch_ms)
        && value.expires_at_epoch_ms.is_some_and(|expiration| {
            expiration > now_epoch_ms
                && approval_timestamp.is_some_and(|approved| expiration > approved)
        })
        && !value.restore_or_reimage_procedure.trim().is_empty()
        && !value.validation_maturity.trim().is_empty();

    if !common_complete {
        return false;
    }

    match (value.schema_version, value.target_type) {
        (2 | 3, ValidationTargetType::VirtualMachine) => {
            value.disposable_confirmed == Some(true)
                && value.important_data_confirmed
                && value.backup_confirmed
        }
        (3, ValidationTargetType::PhysicalLaptop) => {
            physical_target_approval_is_complete(value, now_epoch_ms)
        }
        _ => false,
    }
}

fn physical_target_approval_is_complete(
    value: &ScopedValidationTargetApproval,
    now_epoch_ms: u64,
) -> bool {
    let approval_start = value.approval_granted_at_epoch_ms;
    let expiration = value.expires_at_epoch_ms;
    value.expendable_confirmed == Some(true)
        && matches!(
            value.account_class.as_str(),
            "local_administrator" | "standard_user"
        )
        && value.recovery_readiness == RecoveryReadiness::Ready
        && value.important_data_confirmed
        && matches!(
            value.important_data_state,
            Some(ImportantDataState::Absent | ImportantDataState::PresentBackedUp)
        )
        && (value.important_data_state == Some(ImportantDataState::Absent)
            || value.backup_confirmed)
        && value.reinstallation_accepted
        && value.winre_verified
        && matches!(
            value.recovery_route_state,
            Some(
                RecoveryRouteState::RecoveryPartitionVerified
                    | RecoveryRouteState::EquivalentRouteVerified
            )
        )
        && value.recovery_media_state == RecoveryMediaState::Available
        && value.alternate_recovery_device_available == Some(true)
        && value.restart_pending_state == Some(RestartPendingState::Clear)
        && matches!(
            value.bitlocker_recovery_state,
            BitLockerRecoveryState::NotApplicableUnencrypted
                | BitLockerRecoveryState::RecoveryMaterialConfirmed
        )
        && !value.management_state.domain_joined
        && !value.management_state.entra_joined
        && value.management_state.workplace_joined == Some(false)
        && !value.management_state.mdm_enrolled
        && value.automatic_repair_disabled == Some(true)
        && value.final_plan_approval_required == Some(true)
        && value.final_disposition.is_some()
        && approval_start.is_some_and(|start| {
            start <= now_epoch_ms
                && value
                    .explicit_user_approval_timestamp
                    .as_deref()
                    .and_then(canonical_utc_timestamp_epoch_ms)
                    == Some(start)
        })
        && expiration.zip(approval_start).is_some_and(|(end, start)| {
            end > now_epoch_ms && end > start && end - start <= MAX_PHYSICAL_APPROVAL_LIFETIME_MS
        })
}

fn target_approval_is_complete(value: &ValidationTargetApproval, now_epoch_ms: u64) -> bool {
    let approval_timestamp = value
        .explicit_user_approval_timestamp
        .as_deref()
        .and_then(canonical_utc_timestamp_epoch_ms);
    let required_operations: HashSet<_> = MutationOperationId::ALL.into_iter().collect();
    let operations: HashSet<_> = value.approved_operations.iter().copied().collect();
    let states_complete = MutationOperationId::ALL.into_iter().all(|operation| {
        value
            .approved_target_states
            .get(operation.key())
            .is_some_and(|states| {
                states.len() == 2
                    && states.contains(&MutationTarget::Enabled)
                    && states.contains(&MutationTarget::Disabled)
            })
    });
    let handlers_complete = MutationOperationId::ALL.into_iter().all(|operation| {
        value
            .handler_versions
            .get(operation.key())
            .is_some_and(|version| !version.trim().is_empty())
    });
    value.schema_version == 1
        && value.target_type == ValidationTargetType::VirtualMachine
        && value.record_kind == TargetRecordKind::Approval
        && value.approval_status == TargetApprovalStatus::Approved
        && is_sha256(&value.hashed_machine_identity)
        && is_git_commit_id(&value.source_checkpoint_commit)
        && !value.account_class.trim().is_empty()
        && operations == required_operations
        && states_complete
        && handlers_complete
        && value
            .evidence_output_location
            .replace('\\', "/")
            .starts_with(".deslopper/local/")
        && matches!(
            value.recovery_readiness,
            RecoveryReadiness::Ready | RecoveryReadiness::ReadyWithWarnings
        )
        && value.important_data_confirmed
        && value.backup_confirmed
        && value.reinstallation_accepted
        && value.winre_verified
        && matches!(
            value.bitlocker_recovery_state,
            BitLockerRecoveryState::NotApplicableUnencrypted
                | BitLockerRecoveryState::RecoveryMaterialConfirmed
        )
        && matches!(
            value.recovery_media_state,
            RecoveryMediaState::Available | RecoveryMediaState::BuiltInVerified
        )
        && value.development_host_protection_state
            == DevelopmentHostProtectionState::PresentDistinct
        && approval_timestamp.is_some_and(|approved| approved <= now_epoch_ms)
        && value.expires_at_epoch_ms.is_some_and(|expiration| {
            expiration > now_epoch_ms
                && approval_timestamp.is_some_and(|approved| expiration > approved)
        })
        && !value.restore_or_reimage_procedure.trim().is_empty()
        && !value.validation_maturity.trim().is_empty()
        && value.disposable_confirmed == Some(true)
}

fn argument_value(arguments: &[String], prefix: &str) -> Option<String> {
    arguments
        .iter()
        .find_map(|value| value.strip_prefix(prefix).map(str::to_owned))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &std::path::Path) -> Option<T> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

pub fn development_host_fingerprint(computer_name: &str, edition: &str, build: u32) -> String {
    hash_text(&format!(
        "{}:{edition}:{build}",
        computer_name.to_ascii_uppercase()
    ))
}

fn detect_virtual_machine(
    platform: Option<&PlatformInfo>,
) -> (VirtualMachineDetectionResult, String) {
    let Some(platform) = platform else {
        return (
            VirtualMachineDetectionResult::Unknown,
            "No current platform inspection is available.".into(),
        );
    };
    let material = format!(
        "{} {}",
        platform.manufacturer.as_deref().unwrap_or_default(),
        platform.model.as_deref().unwrap_or_default()
    )
    .to_ascii_lowercase();
    let known = [
        "virtual machine",
        "vmware",
        "virtualbox",
        "kvm",
        "qemu",
        "xen",
        "parallels",
        "hyper-v",
    ];
    if known.iter().any(|marker| material.contains(marker)) {
        (
            VirtualMachineDetectionResult::Detected,
            "Platform manufacturer/model contains a known virtualization marker.".into(),
        )
    } else if platform.manufacturer.is_some() || platform.model.is_some() {
        (
            VirtualMachineDetectionResult::NotDetected,
            "No known virtualization marker was detected; this is informational only.".into(),
        )
    } else {
        (
            VirtualMachineDetectionResult::Unknown,
            "Manufacturer/model evidence was unavailable; identity gates remain authoritative."
                .into(),
        )
    }
}

fn target_type_matches_detection(
    target_type: ValidationTargetType,
    detection: VirtualMachineDetectionResult,
) -> bool {
    matches!(
        (target_type, detection),
        (
            ValidationTargetType::VirtualMachine,
            VirtualMachineDetectionResult::Detected
        ) | (
            ValidationTargetType::PhysicalLaptop,
            VirtualMachineDetectionResult::NotDetected
        )
    )
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceExportRequest {
    pub transaction_id: String,
    pub visual_verification: VisualVerification,
    pub screenshot_labels: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveEvidenceBundle {
    pub evidence_schema_version: u32,
    pub scenario_id: String,
    pub vm_identity_hash: String,
    pub edition: String,
    pub build: u32,
    pub update_build_revision: Option<u32>,
    pub account_class: String,
    pub management_context: String,
    pub checkpoint_id: String,
    pub operation_id: MutationOperationId,
    pub handler_version: String,
    pub initial_representation: CapturedState,
    pub target: MutationTarget,
    pub post_apply_representation: CapturedState,
    pub rollback_representation: Option<CapturedState>,
    pub initial_detector_state: bool,
    pub post_apply_detector_state: bool,
    pub post_rollback_detector_state: Option<bool>,
    pub visual_verification: VisualVerification,
    pub visual_outcome: VisualVerificationOutcome,
    pub transaction_transitions: Vec<String>,
    pub plan_hash: String,
    pub pre_state_hash: String,
    pub post_state_hash: String,
    pub rollback_state_hash: Option<String>,
    pub verification_status: String,
    pub rollback_status: Option<String>,
    pub screenshots_manifest: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub final_result: MatrixScenarioStatus,
}

impl LiveEvidenceBundle {
    pub fn from_transaction(
        request: &EvidenceExportRequest,
        transaction: &MutationTransaction,
        gate: &LiveValidationGateStatus,
        manifest: &ValidationScenarioManifest,
    ) -> Result<Self, &'static str> {
        if request.transaction_id != transaction.transaction_id {
            return Err("Evidence transaction identity mismatch.");
        }
        validate_labels(&request.screenshot_labels)?;
        validate_redacted_strings(&request.warnings)?;
        let pre = transaction
            .pre_state
            .clone()
            .ok_or("Evidence requires an exact pre-state capture.")?;
        let post = transaction
            .post_state
            .clone()
            .ok_or("Evidence requires a verified post-state capture.")?;
        let no_change_needed = transaction.status == TransactionStatus::NoChangeNeeded
            && transaction.verification_result.as_deref() == Some("already_compliant")
            && !transaction.rollback.available
            && transaction.rollback_state.is_none();
        let visual_outcome = if no_change_needed {
            VisualVerificationOutcome::NoChangeNeeded
        } else {
            request.visual_verification.outcome()
        };
        let rollback_verified = transaction.rollback_state.is_some()
            && transaction.rollback.verification_result.as_deref()
                == Some("exact_pre_state_restored")
            && transaction.status == TransactionStatus::RolledBack;
        let apply_verified = no_change_needed
            || (transaction.verification_result.as_deref() == Some("applied_and_verified")
                && transaction
                    .steps
                    .iter()
                    .any(|step| step.status == TransactionStatus::RollbackAvailable.key()));
        let final_result = if no_change_needed {
            MatrixScenarioStatus::Passed
        } else {
            classify_final_result(
                visual_outcome,
                request.visual_verification.refresh_requirement,
                apply_verified,
                rollback_verified,
            )
        };
        Ok(Self {
            evidence_schema_version: 1,
            scenario_id: manifest.scenario_id.clone(),
            vm_identity_hash: hash_text(&format!(
                "{}:{}",
                manifest.expected_machine_id, manifest.scenario_id
            )),
            edition: gate.environment.edition.clone(),
            build: gate.environment.build,
            update_build_revision: gate.environment.update_build_revision,
            account_class: manifest.account_class.clone(),
            management_context: manifest.management_context.clone(),
            checkpoint_id: hash_text(&manifest.checkpoint_id),
            operation_id: transaction.operation_id,
            handler_version: transaction.handler_version.clone(),
            initial_representation: pre.clone(),
            target: transaction.target_state,
            post_apply_representation: post.clone(),
            rollback_representation: transaction.rollback_state.clone(),
            initial_detector_state: pre.effective_enabled,
            post_apply_detector_state: post.effective_enabled,
            post_rollback_detector_state: transaction
                .rollback_state
                .as_ref()
                .map(|state| state.effective_enabled),
            visual_verification: request.visual_verification.clone(),
            visual_outcome,
            transaction_transitions: transaction
                .steps
                .iter()
                .map(|step| step.status.clone())
                .collect(),
            plan_hash: transaction.plan_hash.clone(),
            pre_state_hash: transaction
                .pre_state_hash
                .clone()
                .ok_or("Evidence requires a pre-state integrity hash.")?,
            post_state_hash: transaction
                .post_state_hash
                .clone()
                .ok_or("Evidence requires a post-state integrity hash.")?,
            rollback_state_hash: transaction.rollback_state_hash.clone(),
            verification_status: transaction
                .verification_result
                .clone()
                .unwrap_or_else(|| "not_verified".into()),
            rollback_status: transaction.rollback.result.clone(),
            screenshots_manifest: request.screenshot_labels.clone(),
            warnings: request.warnings.clone(),
            errors: transaction.error_summary.clone().into_iter().collect(),
            final_result,
        })
    }

    pub fn identity(&self) -> String {
        hash_text(&format!(
            "{}:{}:{}:{}:{}",
            self.scenario_id,
            self.vm_identity_hash,
            self.operation_id.key(),
            self.target.enabled(),
            self.plan_hash
        ))
    }
}

fn classify_final_result(
    visual_outcome: VisualVerificationOutcome,
    refresh_requirement: RefreshRequirement,
    apply_verified: bool,
    rollback_verified: bool,
) -> MatrixScenarioStatus {
    if refresh_requirement == RefreshRequirement::UnsupportedWithoutExplorerTermination
        || visual_outcome == VisualVerificationOutcome::StoredValueIgnoredByShell
    {
        MatrixScenarioStatus::HandlerRemoved
    } else if visual_outcome == VisualVerificationOutcome::FullyVerified
        && apply_verified
        && rollback_verified
    {
        MatrixScenarioStatus::Passed
    } else if visual_outcome
        == VisualVerificationOutcome::RepresentationAndDetectorVerifiedVisualPending
        && apply_verified
        && rollback_verified
    {
        MatrixScenarioStatus::PassedWithLimitations
    } else {
        MatrixScenarioStatus::Failed
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct EvidenceIndex {
    identities: BTreeSet<String>,
    bundles: Vec<LiveEvidenceBundle>,
}

#[cfg(test)]
impl EvidenceIndex {
    pub fn import(&mut self, bundle: LiveEvidenceBundle) -> Result<(), &'static str> {
        validate_bundle_redaction(&bundle)?;
        if !self.identities.insert(bundle.identity()) {
            return Err("Duplicate evidence bundle.");
        }
        self.bundles.push(bundle);
        Ok(())
    }

    pub fn cross_build_outcomes(
        &self,
        operation: MutationOperationId,
    ) -> BTreeSet<(u32, VisualVerificationOutcome)> {
        self.bundles
            .iter()
            .filter(|bundle| bundle.operation_id == operation)
            .map(|bundle| (bundle.build, bundle.visual_outcome))
            .collect()
    }
}

fn validate_labels(values: &[String]) -> Result<(), &'static str> {
    if values.iter().all(|value| {
        !value.is_empty()
            && value.len() <= 80
            && value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
    }) {
        Ok(())
    } else {
        Err("Screenshot manifests accept redacted labels only, never paths.")
    }
}

fn validate_redacted_strings(values: &[String]) -> Result<(), &'static str> {
    let forbidden = ["@", "c:\\users\\", "/users/", "token", "password", "sid="];
    if values.iter().any(|value| {
        let lower = value.to_ascii_lowercase();
        forbidden.iter().any(|marker| lower.contains(marker))
    }) {
        Err("Evidence contains a prohibited identifying or secret-like value.")
    } else {
        Ok(())
    }
}

#[cfg(test)]
fn validate_bundle_redaction(bundle: &LiveEvidenceBundle) -> Result<(), &'static str> {
    validate_labels(&bundle.screenshots_manifest)?;
    validate_redacted_strings(&bundle.warnings)?;
    validate_redacted_strings(&bundle.errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_timestamp(epoch_ms: u64) -> String {
        let days = i64::try_from(epoch_ms / 86_400_000).unwrap();
        let day_ms = epoch_ms % 86_400_000;
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let day_of_era = z - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let mut year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
        let month = shifted_month + if shifted_month < 10 { 3 } else { -9 };
        year += i64::from(month <= 2);
        let hour = day_ms / 3_600_000;
        let minute = day_ms % 3_600_000 / 60_000;
        let second = day_ms % 60_000 / 1_000;
        let millisecond = day_ms % 1_000;
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millisecond:03}Z")
    }

    fn platform(name: &str, edition: &str, build: u32) -> PlatformInfo {
        PlatformInfo {
            product_name: "Windows 11 Pro".into(),
            edition: edition.into(),
            build,
            display_version: "24H2".into(),
            update_build_revision: Some(1000),
            architecture: "64-bit".into(),
            device_name: Some(name.into()),
            manufacturer: Some("Microsoft Corporation".into()),
            model: Some("Virtual Machine".into()),
            user_sid: None,
            owner_scope_id: None,
            elevated: false,
            domain_joined: Some(false),
            entra_joined: Some(false),
            workplace_joined: Some(false),
            mdm_enrolled: Some(false),
            is_windows_11: Some(true),
        }
    }

    fn valid_target_approval(target_type: ValidationTargetType) -> ValidationTargetApproval {
        let handlers = MutationOperationId::ALL
            .into_iter()
            .map(|operation| (operation.key().into(), "alpha-1".into()))
            .collect();
        let states = MutationOperationId::ALL
            .into_iter()
            .map(|operation| {
                (
                    operation.key().into(),
                    vec![MutationTarget::Enabled, MutationTarget::Disabled],
                )
            })
            .collect();
        ValidationTargetApproval {
            schema_version: 1,
            record_kind: TargetRecordKind::Approval,
            scenario_id: "physical-readiness".into(),
            target_type,
            hashed_machine_identity: "b".repeat(64),
            windows_edition: "Professional".into(),
            windows_build: 26_200,
            windows_ubr: 8875,
            account_class: "local_administrator".into(),
            management_state: TargetManagementState {
                domain_joined: false,
                entra_joined: false,
                workplace_joined: Some(false),
                mdm_enrolled: false,
            },
            source_checkpoint_commit: "a".repeat(64),
            handler_versions: handlers,
            approved_operations: MutationOperationId::ALL.to_vec(),
            approved_target_states: states,
            evidence_output_location: ".deslopper/local/evidence".into(),
            recovery_readiness: RecoveryReadiness::ReadyWithWarnings,
            important_data_confirmed: true,
            backup_confirmed: true,
            reinstallation_accepted: true,
            winre_verified: true,
            bitlocker_recovery_state: BitLockerRecoveryState::NotApplicableUnencrypted,
            recovery_media_state: RecoveryMediaState::Available,
            development_host_protection_state: DevelopmentHostProtectionState::PresentDistinct,
            explicit_user_approval_timestamp: Some("2026-08-03T12:00:00.000Z".into()),
            approval_status: TargetApprovalStatus::Approved,
            expires_at_epoch_ms: Some(current_epoch_ms() + 60_000),
            restore_or_reimage_procedure: "Use approved Windows recovery media.".into(),
            validation_maturity: "synthetic_tested".into(),
            disposable_confirmed: (target_type == ValidationTargetType::VirtualMachine)
                .then_some(true),
            expendable_confirmed: (target_type == ValidationTargetType::PhysicalLaptop)
                .then_some(true),
        }
    }

    fn valid_scoped_approval() -> ScopedValidationTargetApproval {
        let approved_at = current_epoch_ms();
        ScopedValidationTargetApproval {
            schema_version: 3,
            record_kind: TargetRecordKind::Approval,
            approval_id: "widgets-enabled-once".into(),
            scenario_id: "physical-readiness".into(),
            target_type: ValidationTargetType::PhysicalLaptop,
            hashed_machine_identity: "b".repeat(64),
            windows_edition: "Professional".into(),
            windows_build: 26_200,
            windows_ubr: 8875,
            account_class: "local_administrator".into(),
            management_state: TargetManagementState {
                domain_joined: false,
                entra_joined: false,
                workplace_joined: Some(false),
                mdm_enrolled: false,
            },
            source_checkpoint_commit: "a".repeat(40),
            source_inspection_id: "inspection-1".into(),
            inspection_evidence_sha256: "c".repeat(64),
            development_host_denylist_identity: "d".repeat(64),
            approved_operation_scopes: vec![ApprovedOperationScope {
                operation_id: MutationOperationId::WidgetsVisibility,
                allowed_target_states: vec![MutationTarget::Enabled],
                handler_version: crate::mutation::HANDLER_VERSION.into(),
            }],
            maximum_plans: 1,
            maximum_executions: 1,
            evidence_output_location: ".deslopper/local/live-evidence".into(),
            recovery_readiness: RecoveryReadiness::Ready,
            important_data_confirmed: true,
            backup_confirmed: true,
            reinstallation_accepted: true,
            winre_verified: true,
            bitlocker_recovery_state: BitLockerRecoveryState::NotApplicableUnencrypted,
            recovery_media_state: RecoveryMediaState::Available,
            development_host_protection_state: DevelopmentHostProtectionState::PresentDistinct,
            explicit_user_approval_timestamp: Some(canonical_timestamp(approved_at)),
            approval_status: TargetApprovalStatus::Approved,
            expires_at_epoch_ms: Some(current_epoch_ms() + 60_000),
            restore_or_reimage_procedure: "Use approved Windows recovery media.".into(),
            validation_maturity: "read_only_validated".into(),
            important_data_state: Some(ImportantDataState::Absent),
            recovery_route_state: Some(RecoveryRouteState::RecoveryPartitionVerified),
            alternate_recovery_device_available: Some(true),
            restart_pending_state: Some(RestartPendingState::Clear),
            automatic_repair_disabled: Some(true),
            final_plan_approval_required: Some(true),
            approval_granted_at_epoch_ms: Some(approved_at),
            final_disposition: Some(FinalDisposition::ResetBeforeSale),
            disposable_confirmed: None,
            expendable_confirmed: Some(true),
        }
    }

    fn valid_scoped_vm_approval() -> ScopedValidationTargetApproval {
        let mut approval = valid_scoped_approval();
        approval.schema_version = 2;
        approval.target_type = ValidationTargetType::VirtualMachine;
        approval.disposable_confirmed = Some(true);
        approval.expendable_confirmed = None;
        approval.important_data_state = None;
        approval.recovery_route_state = None;
        approval.alternate_recovery_device_available = None;
        approval.restart_pending_state = None;
        approval.automatic_repair_disabled = None;
        approval.final_plan_approval_required = None;
        approval.approval_granted_at_epoch_ms = None;
        approval.final_disposition = None;
        approval
    }

    #[test]
    fn scoped_v3_widgets_enabled_only_normalizes_without_legacy_promotion() {
        let now = current_epoch_ms();
        let approval = valid_scoped_approval();
        assert!(scoped_target_approval_is_complete(&approval, now));
        let normalized =
            normalize_target_approval(&ValidationTargetApprovalDocument::Scoped(approval), now)
                .unwrap();
        assert_eq!(normalized.approved_operation_scopes.len(), 1);
        assert_eq!(
            normalized.approved_operation_scopes[0].operation_id,
            MutationOperationId::WidgetsVisibility
        );
        assert_eq!(
            normalized.approved_operation_scopes[0].allowed_target_states,
            vec![MutationTarget::Enabled]
        );
        assert_eq!(normalized.maximum_plans, 1);
        assert_eq!(normalized.maximum_executions, 1);

        let legacy_physical = valid_target_approval(ValidationTargetType::PhysicalLaptop);
        assert!(
            normalize_target_approval(
                &ValidationTargetApprovalDocument::Legacy(legacy_physical),
                now
            )
            .is_none()
        );
        let legacy_vm = valid_target_approval(ValidationTargetType::VirtualMachine);
        assert!(
            normalize_target_approval(&ValidationTargetApprovalDocument::Legacy(legacy_vm), now)
                .is_some()
        );
    }

    #[test]
    fn local_scoped_approval_is_revalidated_against_the_startup_binding() {
        let now = current_epoch_ms();
        let approval = valid_scoped_approval();
        let path = std::env::temp_dir().join(format!(
            "deslopper-scoped-approval-{}-{now}.json",
            std::process::id()
        ));
        let denylist_path = path.with_extension("denylist.json");
        std::fs::write(&path, serde_json::to_vec(&approval).unwrap()).unwrap();
        std::fs::write(
            &denylist_path,
            serde_json::to_vec(&serde_json::json!({
                "schemaVersion": 1,
                "developmentHostFingerprints": [approval.development_host_denylist_identity]
            }))
            .unwrap(),
        )
        .unwrap();
        let policy = r#"
live_validation_policy_schema_version = 2
allow_live_mutation_only_in_explicitly_approved_disposable_target = true
allowed_live_validation_target_types = ["virtual_machine", "physical_laptop"]
physical_target_requires_strict_recovery_readiness = true
"#;

        let mut status = LiveValidationGateStatus::test_valid();
        status.approval_id = Some(approval.approval_id.clone());
        status.approval_source_commit = Some(approval.source_checkpoint_commit.clone());
        status.approval_source_inspection_id = Some(approval.source_inspection_id.clone());
        status.approval_evidence_sha256 = Some(approval.inspection_evidence_sha256.clone());
        status.approved_operation_scopes = approval.approved_operation_scopes.clone();
        status.maximum_plans = approval.maximum_plans;
        status.maximum_executions = approval.maximum_executions;
        status.approved_target_type = Some(approval.target_type);
        status.approved_target_identity = Some(approval.hashed_machine_identity.clone());
        status.local_approval_revalidation_required = true;

        assert!(scoped_approval_document_allows(
            &status,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &path,
            &denylist_path,
            Some(policy),
            now,
        ));
        assert!(!scoped_approval_document_allows(
            &status,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &path,
            &denylist_path,
            Some(policy),
            now,
        ));

        let vm_only_policy = policy.replace(
            "[\"virtual_machine\", \"physical_laptop\"]",
            "[\"virtual_machine\"]",
        );
        assert!(!scoped_approval_document_allows(
            &status,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &path,
            &denylist_path,
            Some(&vm_only_policy),
            now,
        ));
        std::fs::remove_file(&denylist_path).unwrap();
        assert!(!scoped_approval_document_allows(
            &status,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &path,
            &denylist_path,
            Some(policy),
            now,
        ));
        std::fs::write(&denylist_path, b"{\"schemaVersion\":1}").unwrap();
        assert!(!scoped_approval_document_allows(
            &status,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &path,
            &denylist_path,
            Some(policy),
            now,
        ));
        std::fs::write(
            &denylist_path,
            serde_json::to_vec(&serde_json::json!({
                "schemaVersion": 1,
                "developmentHostFingerprints": [approval.development_host_denylist_identity]
            }))
            .unwrap(),
        )
        .unwrap();

        let mut changed = approval;
        changed.approved_operation_scopes[0].allowed_target_states = vec![MutationTarget::Disabled];
        std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(!scoped_approval_document_allows(
            &status,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &path,
            &denylist_path,
            Some(policy),
            now,
        ));
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(denylist_path);
    }

    #[test]
    fn scoped_v3_rejects_empty_duplicate_malformed_and_exhaustive_defaults() {
        let now = current_epoch_ms();
        let mut approval = valid_scoped_approval();
        approval.approved_operation_scopes.clear();
        assert!(!scoped_target_approval_is_complete(&approval, now));

        approval = valid_scoped_approval();
        approval
            .approved_operation_scopes
            .push(approval.approved_operation_scopes[0].clone());
        assert!(!scoped_target_approval_is_complete(&approval, now));

        approval = valid_scoped_approval();
        approval.approved_operation_scopes[0]
            .allowed_target_states
            .push(MutationTarget::Enabled);
        assert!(!scoped_target_approval_is_complete(&approval, now));

        approval = valid_scoped_approval();
        approval.approved_operation_scopes[0]
            .allowed_target_states
            .clear();
        assert!(!scoped_target_approval_is_complete(&approval, now));

        approval = valid_scoped_approval();
        approval.maximum_plans = 0;
        assert!(!scoped_target_approval_is_complete(&approval, now));
        approval.maximum_plans = 1;
        approval.maximum_executions = 2;
        assert!(!scoped_target_approval_is_complete(&approval, now));
    }

    #[test]
    fn scoped_v3_rejects_unknown_operations_states_and_unknown_fields() {
        let value = serde_json::to_value(valid_scoped_approval()).unwrap();
        let mut unknown_operation = value.clone();
        unknown_operation["approvedOperationScopes"][0]["operationId"] =
            serde_json::json!("future_operation");
        assert!(
            serde_json::from_value::<ScopedValidationTargetApproval>(unknown_operation).is_err()
        );

        let mut unknown_state = value.clone();
        unknown_state["approvedOperationScopes"][0]["allowedTargetStates"][0] =
            serde_json::json!("toggle");
        assert!(serde_json::from_value::<ScopedValidationTargetApproval>(unknown_state).is_err());

        let mut missing_target_type = value.clone();
        missing_target_type
            .as_object_mut()
            .unwrap()
            .remove("targetType");
        assert!(
            serde_json::from_value::<ScopedValidationTargetApproval>(missing_target_type).is_err()
        );

        let mut extra = value;
        extra["approvedOperationScopes"][0]["registryPath"] = serde_json::json!("arbitrary");
        assert!(serde_json::from_value::<ScopedValidationTargetApproval>(extra).is_err());
    }

    #[test]
    fn scoped_v3_rejects_wrong_source_identity_evidence_handler_and_expiry() {
        let now = current_epoch_ms();
        let mut approval = valid_scoped_approval();
        approval.source_checkpoint_commit = "a".repeat(39);
        assert!(!scoped_target_approval_is_complete(&approval, now));
        approval = valid_scoped_approval();
        approval.hashed_machine_identity = "invalid".into();
        assert!(!scoped_target_approval_is_complete(&approval, now));
        approval = valid_scoped_approval();
        approval.inspection_evidence_sha256 = "invalid".into();
        assert!(!scoped_target_approval_is_complete(&approval, now));
        approval = valid_scoped_approval();
        approval.approved_operation_scopes[0].handler_version = "wrong".into();
        assert!(!scoped_target_approval_is_complete(&approval, now));
        approval = valid_scoped_approval();
        approval.expires_at_epoch_ms = Some(now);
        assert!(!scoped_target_approval_is_complete(&approval, now));
    }

    #[test]
    fn approval_timestamps_require_canonical_literal_z_and_exact_epoch_binding() {
        let now = current_epoch_ms();
        let canonical = canonical_timestamp(now);
        assert_eq!(canonical_utc_timestamp_epoch_ms(&canonical), Some(now));
        assert!(canonical_utc_timestamp_epoch_ms("2026-08-05T12:00:00.000+00:00").is_none());
        assert!(canonical_utc_timestamp_epoch_ms("2026-08-05T13:00:00.000+01:00").is_none());
        assert!(canonical_utc_timestamp_epoch_ms("2026-08-05T12:00:00.000").is_none());
        assert!(canonical_utc_timestamp_epoch_ms("2026-02-30T12:00:00.000Z").is_none());

        let mut approval = valid_scoped_approval();
        approval.explicit_user_approval_timestamp = Some(canonical_timestamp(now + 1));
        approval.approval_granted_at_epoch_ms = Some(now + 1);
        approval.expires_at_epoch_ms = Some(now + 60_000);
        assert!(!scoped_target_approval_is_complete(&approval, now));

        approval = valid_scoped_approval();
        approval.approval_granted_at_epoch_ms = approval
            .approval_granted_at_epoch_ms
            .map(|epoch| epoch.saturating_sub(1));
        assert!(!scoped_target_approval_is_complete(&approval, now));
    }

    #[test]
    fn strict_physical_target_eligibility_is_complete_and_fail_closed() {
        let now = current_epoch_ms();
        let valid = valid_scoped_approval();
        assert!(scoped_target_approval_is_complete(&valid, now));

        let mut value = valid.clone();
        value.expendable_confirmed = Some(false);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.development_host_denylist_identity = value.hashed_machine_identity.clone();
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.important_data_state = Some(ImportantDataState::PresentNotBackedUp);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.important_data_state = Some(ImportantDataState::PresentBackedUp);
        value.backup_confirmed = false;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.reinstallation_accepted = false;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.winre_verified = false;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.recovery_route_state = Some(RecoveryRouteState::Missing);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.recovery_media_state = RecoveryMediaState::Missing;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.alternate_recovery_device_available = Some(false);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.restart_pending_state = Some(RestartPendingState::Pending);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.bitlocker_recovery_state = BitLockerRecoveryState::Unknown;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.management_state.domain_joined = true;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.management_state.entra_joined = true;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.management_state.workplace_joined = Some(true);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.management_state.workplace_joined = None;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.management_state.mdm_enrolled = true;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.account_class = "domain_administrator".into();
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.automatic_repair_disabled = Some(false);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.final_plan_approval_required = Some(false);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.final_disposition = None;
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.explicit_user_approval_timestamp = Some("not-a-timestamp".into());
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.restore_or_reimage_procedure.clear();
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid.clone();
        value.approval_granted_at_epoch_ms = Some(now - MAX_PHYSICAL_APPROVAL_LIFETIME_MS - 1);
        assert!(!scoped_target_approval_is_complete(&value, now));
        value = valid;
        value.expires_at_epoch_ms = Some(now);
        assert!(!scoped_target_approval_is_complete(&value, now));
    }

    #[test]
    fn scoped_v2_remains_vm_only_and_physical_fields_cannot_substitute() {
        let now = current_epoch_ms();
        assert!(scoped_target_approval_is_complete(
            &valid_scoped_vm_approval(),
            now
        ));
        let mut physical_v2 = valid_scoped_approval();
        physical_v2.schema_version = 2;
        assert!(!scoped_target_approval_is_complete(&physical_v2, now));

        let mut vm_without_checkpoint_recovery = valid_scoped_vm_approval();
        vm_without_checkpoint_recovery
            .restore_or_reimage_procedure
            .clear();
        assert!(!scoped_target_approval_is_complete(
            &vm_without_checkpoint_recovery,
            now
        ));
    }

    #[test]
    fn maturity_is_monotonic_and_rejection_disables_handler() {
        assert_eq!(
            ValidationMaturity::SyntheticTested
                .advance(ValidationMaturity::LiveTestedSingleBuild)
                .unwrap(),
            ValidationMaturity::LiveTestedSingleBuild
        );
        assert!(
            ValidationMaturity::LiveTestedSingleBuild
                .advance(ValidationMaturity::SyntheticTested)
                .is_err()
        );
        assert!(!ValidationMaturity::Rejected.allows_internal_alpha());
    }

    #[test]
    fn visual_state_keeps_sign_out_pending_and_classifies_disagreement() {
        let pending = VisualVerification {
            representation: VerificationDimensionResult::Verified,
            detector: VerificationDimensionResult::Verified,
            user_visible_behavior: VerificationDimensionResult::Verified,
            settings_ui: VerificationDimensionResult::Verified,
            refresh_requirement: RefreshRequirement::SignOutSignIn,
            lifecycle_refresh_completed: false,
        };
        assert_eq!(
            pending.outcome(),
            VisualVerificationOutcome::RepresentationAndDetectorVerifiedVisualPending
        );
        let disagreement = VisualVerification {
            settings_ui: VerificationDimensionResult::Disagrees,
            ..pending
        };
        assert_eq!(
            disagreement.outcome(),
            VisualVerificationOutcome::SettingsUiDisagrees
        );
        let ignored = VisualVerification {
            user_visible_behavior: VerificationDimensionResult::IgnoredByShell,
            settings_ui: VerificationDimensionResult::Verified,
            ..pending.clone()
        };
        assert_eq!(
            ignored.outcome(),
            VisualVerificationOutcome::StoredValueIgnoredByShell
        );
        let reverted = VisualVerification {
            user_visible_behavior: VerificationDimensionResult::RevertedAfterSignIn,
            ..pending.clone()
        };
        assert_eq!(
            reverted.outcome(),
            VisualVerificationOutcome::RevertedAfterSignIn
        );
        let policy = VisualVerification {
            detector: VerificationDimensionResult::PolicyOverrode,
            ..pending
        };
        assert_eq!(
            policy.outcome(),
            VisualVerificationOutcome::PolicyOverrodeValue
        );
    }

    #[test]
    fn virtual_machine_detection_is_informational() {
        let (result, _) = detect_virtual_machine(Some(&platform("GUEST", "Professional", 26100)));
        assert_eq!(result, VirtualMachineDetectionResult::Detected);
        assert!(target_type_matches_detection(
            ValidationTargetType::VirtualMachine,
            VirtualMachineDetectionResult::Detected
        ));
        assert!(!target_type_matches_detection(
            ValidationTargetType::PhysicalLaptop,
            VirtualMachineDetectionResult::Detected
        ));
        assert!(!target_type_matches_detection(
            ValidationTargetType::VirtualMachine,
            VirtualMachineDetectionResult::NotDetected
        ));
    }

    #[test]
    fn live_gate_rejects_missing_opt_in_and_identity_scenario_checkpoint_mismatches() {
        let valid = || GateFacts {
            command_line_opt_in: true,
            governance_policy_loaded: true,
            target_type_policy_allowed: true,
            manifest_loaded: true,
            target_approval_loaded: true,
            denylist_loaded: true,
            scenario_matches: true,
            machine_identity_matches: true,
            checkpoint_matches: true,
            source_binding_matches: true,
            approval_scope_matches: true,
            denylist_binding_matches: true,
            platform_matches: true,
            target_type_matches: true,
            database_belongs_to_guest: true,
            development_host_refused: false,
        };
        assert!(evaluate_gate(&valid()).0);
        let mut missing_flag = valid();
        missing_flag.command_line_opt_in = false;
        assert!(!evaluate_gate(&missing_flag).0);
        let mut missing_policy = valid();
        missing_policy.governance_policy_loaded = false;
        assert!(evaluate_gate(&missing_policy).1.contains("policy"));
        let mut disallowed_target = valid();
        disallowed_target.target_type_policy_allowed = false;
        assert!(evaluate_gate(&disallowed_target).1.contains("target type"));
        let mut wrong_identity = valid();
        wrong_identity.machine_identity_matches = false;
        assert!(
            evaluate_gate(&wrong_identity)
                .1
                .contains("test-machine identity")
        );
        let mut wrong_scenario = valid();
        wrong_scenario.scenario_matches = false;
        assert!(evaluate_gate(&wrong_scenario).1.contains("scenario"));
        let mut wrong_checkpoint = valid();
        wrong_checkpoint.checkpoint_matches = false;
        assert!(evaluate_gate(&wrong_checkpoint).1.contains("checkpoint"));
        let mut wrong_source = valid();
        wrong_source.source_binding_matches = false;
        assert!(evaluate_gate(&wrong_source).1.contains("source"));
        let mut wrong_scope = valid();
        wrong_scope.approval_scope_matches = false;
        assert!(evaluate_gate(&wrong_scope).1.contains("scope"));
        let mut wrong_denylist_binding = valid();
        wrong_denylist_binding.denylist_binding_matches = false;
        assert!(
            evaluate_gate(&wrong_denylist_binding)
                .1
                .contains("denylist")
        );
        let mut copied_database = valid();
        copied_database.database_belongs_to_guest = false;
        assert!(evaluate_gate(&copied_database).1.contains("database"));
        let mut development_host = valid();
        development_host.development_host_refused = true;
        assert!(
            evaluate_gate(&development_host)
                .1
                .contains("development host")
        );
        let mut pending_target = valid();
        pending_target.target_approval_loaded = false;
        assert!(evaluate_gate(&pending_target).1.contains("approval"));
        let mut missing_denylist = valid();
        missing_denylist.denylist_loaded = false;
        assert!(evaluate_gate(&missing_denylist).1.contains("denylist"));
        let mut wrong_platform = valid();
        wrong_platform.platform_matches = false;
        assert!(
            evaluate_gate(&wrong_platform)
                .1
                .contains("edition, build, or UBR")
        );
        let mut wrong_target_type = valid();
        wrong_target_type.target_type_matches = false;
        assert!(evaluate_gate(&wrong_target_type).1.contains("target type"));
    }

    #[test]
    fn legacy_v1_is_vm_only_and_recovery_confirmations_are_fail_closed() {
        let now = current_epoch_ms();
        assert!(!target_approval_is_complete(
            &valid_target_approval(ValidationTargetType::PhysicalLaptop),
            now
        ));
        let mut approval = valid_target_approval(ValidationTargetType::VirtualMachine);
        assert!(target_approval_is_complete(&approval, now));

        approval.backup_confirmed = false;
        assert!(!target_approval_is_complete(&approval, now));
        approval.backup_confirmed = true;
        approval.reinstallation_accepted = false;
        assert!(!target_approval_is_complete(&approval, now));
        approval.reinstallation_accepted = true;
        approval.winre_verified = false;
        assert!(!target_approval_is_complete(&approval, now));
        approval.winre_verified = true;
        approval.bitlocker_recovery_state = BitLockerRecoveryState::Unknown;
        assert!(!target_approval_is_complete(&approval, now));
    }

    #[test]
    fn preparation_missing_host_protection_and_expiration_cannot_approve_mutation() {
        let now = current_epoch_ms();
        let mut approval = valid_target_approval(ValidationTargetType::VirtualMachine);
        approval.record_kind = TargetRecordKind::Preparation;
        approval.approval_status = TargetApprovalStatus::Pending;
        assert!(!target_approval_is_complete(&approval, now));

        approval.record_kind = TargetRecordKind::Approval;
        approval.approval_status = TargetApprovalStatus::Approved;
        approval.development_host_protection_state = DevelopmentHostProtectionState::Missing;
        assert!(!target_approval_is_complete(&approval, now));

        approval.development_host_protection_state =
            DevelopmentHostProtectionState::PresentDistinct;
        approval.expires_at_epoch_ms = Some(now.saturating_sub(1));
        assert!(!target_approval_is_complete(&approval, now));
    }

    #[test]
    fn source_checkpoint_accepts_git_sha1_or_sha256_but_nothing_else() {
        let now = current_epoch_ms();
        let mut approval = valid_target_approval(ValidationTargetType::VirtualMachine);
        approval.source_checkpoint_commit = "a".repeat(40);
        assert!(target_approval_is_complete(&approval, now));
        approval.source_checkpoint_commit = "a".repeat(39);
        assert!(!target_approval_is_complete(&approval, now));
        approval.source_checkpoint_commit = "g".repeat(40);
        assert!(!target_approval_is_complete(&approval, now));
    }

    #[test]
    fn development_host_denylist_must_be_nonempty_valid_and_distinct() {
        let empty = DevelopmentHostDenylist {
            schema_version: 1,
            development_host_fingerprints: Vec::new(),
        };
        assert!(!valid_development_host_denylist(&empty));
        let target = "b".repeat(64);
        let same = DevelopmentHostDenylist {
            schema_version: 1,
            development_host_fingerprints: vec![target.clone()],
        };
        assert!(valid_development_host_denylist(&same));
        assert!(same.development_host_fingerprints.contains(&target));
    }

    #[test]
    fn virtual_machine_target_model_and_legacy_vm_inventory_remain_supported() {
        let approval = valid_target_approval(ValidationTargetType::VirtualMachine);
        assert!(target_approval_is_complete(&approval, current_epoch_ms()));
        let legacy_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("validation/approved-validation-vms.schema.json");
        let legacy: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(legacy_path).unwrap()).unwrap();
        assert_eq!(legacy["properties"]["schemaVersion"]["const"], 2);
    }

    #[test]
    fn evidence_redaction_and_duplicate_import_are_enforced() {
        assert!(validate_labels(&["widgets-disabled.png".into()]).is_ok());
        assert!(validate_labels(&[r"C:\Users\someone\capture.png".into()]).is_err());
        assert!(validate_redacted_strings(&["person@example.test".into()]).is_err());
    }

    #[test]
    fn evidence_cannot_pass_without_real_apply_and_exact_rollback_verification() {
        assert_eq!(
            classify_final_result(
                VisualVerificationOutcome::FullyVerified,
                RefreshRequirement::Immediate,
                false,
                true,
            ),
            MatrixScenarioStatus::Failed
        );
        assert_eq!(
            classify_final_result(
                VisualVerificationOutcome::FullyVerified,
                RefreshRequirement::Immediate,
                true,
                false,
            ),
            MatrixScenarioStatus::Failed
        );
        assert_eq!(
            classify_final_result(
                VisualVerificationOutcome::FullyVerified,
                RefreshRequirement::UnsupportedWithoutExplorerTermination,
                true,
                true,
            ),
            MatrixScenarioStatus::HandlerRemoved
        );
    }

    #[test]
    fn cross_build_comparison_keeps_distinct_build_outcomes() {
        let mut index = EvidenceIndex::default();
        let mut first = test_bundle(26100, "plan-a");
        let mut second = test_bundle(26200, "plan-b");
        second.visual_outcome = VisualVerificationOutcome::StoredValueIgnoredByShell;
        first.screenshots_manifest = vec!["first.png".into()];
        second.screenshots_manifest = vec!["second.png".into()];
        index.import(first.clone()).unwrap();
        assert!(index.import(first).is_err());
        index.import(second).unwrap();
        assert_eq!(
            index
                .cross_build_outcomes(MutationOperationId::WidgetsVisibility)
                .len(),
            2
        );
    }

    #[test]
    fn every_checked_in_live_evidence_bundle_is_unique_redacted_and_closed() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("validation")
            .join("live-evidence");
        let mut index = EvidenceIndex::default();
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(path).unwrap();
            let bundle: LiveEvidenceBundle = serde_json::from_str(&raw).unwrap();
            assert!(MutationOperationId::ALL.contains(&bundle.operation_id));
            index.import(bundle).unwrap();
        }
    }

    #[test]
    fn validation_matrix_uses_explicit_non_aggregated_live_statuses() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("validation")
            .join("matrix.json");
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(value["schemaVersion"], 2);
        let targets = value["targets"].as_array().unwrap();
        assert_eq!(targets.len(), 13);
        let allowed = [
            "not_provisioned",
            "provisioned_not_run",
            "running",
            "passed",
            "passed_with_limitations",
            "failed",
            "blocked",
            "handler_removed",
        ];
        assert!(targets.iter().all(|target| {
            allowed.contains(&target["mutationStatus"].as_str().unwrap_or_default())
        }));
        let preparation = value["preparationTargets"].as_array().unwrap();
        assert_eq!(preparation.len(), 1);
        assert_eq!(preparation[0]["targetType"], "physical_laptop");
        assert_eq!(preparation[0]["approvalStatus"], "not_approved");
        assert_eq!(preparation[0]["mutationStatus"], "mutation_not_attempted");
        assert_eq!(preparation[0]["countsAsCompletedMutationTarget"], false);
    }

    #[test]
    fn preparation_schema_preserves_all_user_confirmation_states() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("validation")
            .join("approved-validation-targets.schema.json");
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let confirmations =
            &value["properties"]["preparationStatus"]["properties"]["userConfirmations"];
        let required = confirmations["required"].as_array().unwrap();
        for field in [
            "importantPersonalData",
            "allImportantDataBackedUp",
            "expendable",
            "reinstallAccepted",
            "recoveryMediaAvailable",
            "alternateComputerAvailable",
            "externalDriveAvailable",
            "dedicatedLocalTestAccountAllowed",
            "futureBitlockerRecoveryKeyVerification",
        ] {
            assert!(required.iter().any(|candidate| candidate == field));
        }
        assert_eq!(
            value["allOf"][0]["then"]["required"][0],
            "preparationStatus"
        );
        assert_eq!(
            value["properties"]["preparationStatus"]["properties"]["mutationStillProhibited"]["const"],
            true
        );
    }

    #[test]
    fn scoped_v3_schema_is_strict_and_v2_is_explicit_vm_compatibility() {
        let validation = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("validation");
        let approval: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(validation.join("approved-validation-targets-v3.schema.json"))
                .unwrap(),
        )
        .unwrap();
        let approval_v2: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(validation.join("approved-validation-targets-v2.schema.json"))
                .unwrap(),
        )
        .unwrap();
        let scenario: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(validation.join("live-validation-scenario-v2.schema.json"))
                .unwrap(),
        )
        .unwrap();
        let draft: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(validation.join("approval-review-draft.schema.json")).unwrap(),
        )
        .unwrap();

        assert_eq!(approval["properties"]["schemaVersion"]["const"], 3);
        assert_eq!(
            approval_v2["properties"]["targetType"]["const"],
            "virtual_machine"
        );
        assert_eq!(scenario["properties"]["schemaVersion"]["const"], 2);
        assert_eq!(
            scenario["properties"]["approvedOperationScopes"]["minItems"],
            1
        );
        assert_eq!(scenario["additionalProperties"], false);
        assert_eq!(draft["properties"]["schemaVersion"]["const"], 3);
        assert_eq!(draft["properties"]["policySchemaVersion"]["const"], 2);
        assert_eq!(
            draft["properties"]["approvedOperationScopes"]["minItems"],
            1
        );
        assert_eq!(draft["additionalProperties"], false);
        assert_eq!(
            approval["properties"]["approvedOperationScopes"]["minItems"],
            1
        );
        assert_eq!(approval["additionalProperties"], false);
        let physical_required = approval["allOf"][1]["then"]["required"].as_array().unwrap();
        for field in [
            "expendableConfirmed",
            "importantDataState",
            "recoveryRouteState",
            "alternateRecoveryDeviceAvailable",
            "restartPendingState",
            "automaticRepairDisabled",
            "finalPlanApprovalRequired",
            "approvalGrantedAtEpochMs",
            "finalDisposition",
        ] {
            assert!(physical_required.iter().any(|candidate| candidate == field));
        }
        assert_eq!(
            approval["$defs"]["operationScope"]["oneOf"][0]["properties"]["handlerVersion"]["const"],
            super::super::HANDLER_VERSION
        );
        assert_eq!(draft["properties"]["mutationAllowed"]["const"], false);
        assert_eq!(draft["properties"]["executed"]["const"], false);
        assert_eq!(draft["properties"]["authorizing"]["const"], false);
        assert_eq!(draft["properties"]["executionAuthorized"]["const"], false);
        assert_eq!(draft["properties"]["maximumPlans"]["const"], 1);
        assert_eq!(draft["properties"]["maximumExecutions"]["const"], 1);
        assert_eq!(
            draft["properties"]["finalDisposition"]["const"],
            "reset_before_sale"
        );
    }

    fn test_bundle(build: u32, plan_hash: &str) -> LiveEvidenceBundle {
        let state = CapturedState {
            representation: super::super::plan::CapturedRepresentation::Dword(1),
            effective_enabled: true,
            effective_state_known: true,
            authority: "user".into(),
            confidence: "confirmed_representation".into(),
            captured_at: "0".into(),
        };
        LiveEvidenceBundle {
            evidence_schema_version: 1,
            scenario_id: "scenario".into(),
            vm_identity_hash: "hash".into(),
            edition: "Professional".into(),
            build,
            update_build_revision: Some(1),
            account_class: "local".into(),
            management_context: "none".into(),
            checkpoint_id: "checkpoint".into(),
            operation_id: MutationOperationId::WidgetsVisibility,
            handler_version: "1".into(),
            initial_representation: state.clone(),
            target: MutationTarget::Enabled,
            post_apply_representation: state.clone(),
            rollback_representation: Some(state),
            initial_detector_state: true,
            post_apply_detector_state: true,
            post_rollback_detector_state: Some(true),
            visual_verification: VisualVerification {
                representation: VerificationDimensionResult::Verified,
                detector: VerificationDimensionResult::Verified,
                user_visible_behavior: VerificationDimensionResult::Verified,
                settings_ui: VerificationDimensionResult::Verified,
                refresh_requirement: RefreshRequirement::Immediate,
                lifecycle_refresh_completed: true,
            },
            visual_outcome: VisualVerificationOutcome::FullyVerified,
            transaction_transitions: vec!["applied".into()],
            plan_hash: plan_hash.into(),
            pre_state_hash: "pre".into(),
            post_state_hash: "post".into(),
            rollback_state_hash: Some("rollback".into()),
            verification_status: "applied_and_verified".into(),
            rollback_status: Some("rolled_back".into()),
            screenshots_manifest: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            final_result: MatrixScenarioStatus::Passed,
        }
    }
}
