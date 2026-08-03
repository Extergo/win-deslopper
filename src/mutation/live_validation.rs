use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    plan::{CapturedState, hash_text},
    request::{MutationOperationId, MutationTarget},
    transaction::{MutationTransaction, TransactionStatus},
};
use crate::platform::PlatformInfo;

pub const LIVE_VALIDATION_FLAG: &str = "--enable-live-validation";
pub const SCENARIO_ARGUMENT: &str = "--validation-scenario=";
pub const EXPECTED_MACHINE_ARGUMENT: &str = "--expected-machine-id=";
pub const EXPECTED_CHECKPOINT_ARGUMENT: &str = "--expected-checkpoint-id=";

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
    pub available: bool,
    pub reason: String,
    pub environment: EnvironmentIdentity,
}

#[cfg(test)]
impl LiveValidationGateStatus {
    pub fn test_valid() -> Self {
        Self {
            command_line_opt_in: true,
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
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationTargetType {
    VirtualMachine,
    PhysicalLaptop,
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
    pub mdm_enrolled: bool,
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

pub fn build_live_validation_gate(
    platform: Option<&PlatformInfo>,
    stored_machine_id: Option<&str>,
    arguments: &[String],
) -> LiveValidationGateStatus {
    let computer_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".into());
    let scenario_argument = argument_value(arguments, SCENARIO_ARGUMENT);
    let expected_machine_argument = argument_value(arguments, EXPECTED_MACHINE_ARGUMENT);
    let expected_checkpoint_argument = argument_value(arguments, EXPECTED_CHECKPOINT_ARGUMENT);
    let command_line_opt_in = arguments.iter().any(|value| value == LIVE_VALIDATION_FLAG);
    let directory = local_validation_directory();
    let manifest =
        read_json::<ValidationScenarioManifest>(&directory.join("live-validation-scenario.json"));
    let target_approval =
        read_json::<ValidationTargetApproval>(&directory.join("approved-validation-target.json"));
    let denylist =
        read_json::<DevelopmentHostDenylist>(&directory.join("development-host-denylist.json"));

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
    let development_host_refused = denylist.as_ref().is_some_and(|value| {
        value.schema_version == 1
            && value
                .development_host_fingerprints
                .iter()
                .any(|candidate| candidate == &fingerprint)
    });
    let manifest_loaded = manifest
        .as_ref()
        .is_some_and(|value| value.schema_version == 1);
    let target_approval_loaded = target_approval
        .as_ref()
        .is_some_and(|value| target_approval_is_complete(value, current_epoch_ms()));
    let denylist_loaded = denylist
        .as_ref()
        .is_some_and(valid_development_host_denylist);
    let scenario_matches = manifest
        .as_ref()
        .is_some_and(|value| Some(value.scenario_id.as_str()) == scenario_argument.as_deref());
    let machine_identity_matches = manifest.as_ref().is_some_and(|value| {
        Some(value.expected_machine_id.as_str()) == expected_machine_argument.as_deref()
            && value.expected_machine_id == fingerprint
    }) && target_approval
        .as_ref()
        .is_some_and(|value| value.hashed_machine_identity == fingerprint);
    let checkpoint_matches = manifest.as_ref().is_some_and(|value| {
        Some(value.checkpoint_id.as_str()) == expected_checkpoint_argument.as_deref()
    });
    let scenario_platform_matches = manifest.as_ref().is_some_and(|value| {
        let management_matches = match value.management_context.as_str() {
            "domain" => platform.and_then(|item| item.domain_joined) == Some(true),
            "mdm" => platform.and_then(|item| item.mdm_enrolled) == Some(true),
            "none" => {
                platform.and_then(|item| item.domain_joined) != Some(true)
                    && platform.and_then(|item| item.mdm_enrolled) != Some(true)
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
                current.domain_joined == Some(value.management_state.domain_joined)
                    && current.workplace_joined == Some(value.management_state.entra_joined)
                    && current.mdm_enrolled == Some(value.management_state.mdm_enrolled)
            })
    });
    let platform_matches = scenario_platform_matches && target_platform_matches;
    let target_type_matches = target_approval.as_ref().is_some_and(|value| {
        matches!(
            (value.target_type, virtual_machine_detection),
            (
                ValidationTargetType::VirtualMachine,
                VirtualMachineDetectionResult::Detected
            ) | (
                ValidationTargetType::PhysicalLaptop,
                VirtualMachineDetectionResult::NotDetected
            )
        )
    });
    let database_belongs_to_guest = current_machine_id.as_deref() == stored_machine_id;
    let (available, reason) = evaluate_gate(&GateFacts {
        command_line_opt_in,
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
    });

    LiveValidationGateStatus {
        command_line_opt_in,
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
    manifest_loaded: bool,
    target_approval_loaded: bool,
    denylist_loaded: bool,
    scenario_matches: bool,
    machine_identity_matches: bool,
    checkpoint_matches: bool,
    platform_matches: bool,
    target_type_matches: bool,
    database_belongs_to_guest: bool,
    development_host_refused: bool,
}

fn evaluate_gate(facts: &GateFacts) -> (bool, &'static str) {
    let available = facts.command_line_opt_in
        && facts.manifest_loaded
        && facts.target_approval_loaded
        && facts.denylist_loaded
        && facts.scenario_matches
        && facts.machine_identity_matches
        && facts.checkpoint_matches
        && facts.platform_matches
        && facts.target_type_matches
        && facts.database_belongs_to_guest
        && !facts.development_host_refused;
    let reason = if facts.development_host_refused {
        "Live mutation is refused on the recorded development host."
    } else if !facts.command_line_opt_in {
        "The separate --enable-live-validation gate is absent."
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

fn valid_development_host_denylist(value: &DevelopmentHostDenylist) -> bool {
    value.schema_version == 1
        && !value.development_host_fingerprints.is_empty()
        && value
            .development_host_fingerprints
            .iter()
            .all(|fingerprint| is_sha256(fingerprint))
}

fn target_approval_is_complete(value: &ValidationTargetApproval, now_epoch_ms: u64) -> bool {
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
    let target_disposition_confirmed = match value.target_type {
        ValidationTargetType::VirtualMachine => value.disposable_confirmed == Some(true),
        ValidationTargetType::PhysicalLaptop => value.expendable_confirmed == Some(true),
    };

    value.schema_version == 1
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
        && value
            .explicit_user_approval_timestamp
            .as_ref()
            .is_some_and(|timestamp| !timestamp.trim().is_empty())
        && value
            .expires_at_epoch_ms
            .is_some_and(|expiration| expiration > now_epoch_ms)
        && !value.restore_or_reimage_procedure.trim().is_empty()
        && !value.validation_maturity.trim().is_empty()
        && target_disposition_confirmed
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
        let visual_outcome = request.visual_verification.outcome();
        let rollback_verified = transaction.rollback_state.is_some()
            && transaction.rollback.verification_result.as_deref()
                == Some("exact_pre_state_restored")
            && transaction.status == TransactionStatus::RolledBack;
        let apply_verified = transaction.verification_result.as_deref()
            == Some("applied_and_verified")
            && transaction
                .steps
                .iter()
                .any(|step| step.status == TransactionStatus::RollbackAvailable.key());
        let final_result = classify_final_result(
            visual_outcome,
            request.visual_verification.refresh_requirement,
            apply_verified,
            rollback_verified,
        );
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
            elevated: false,
            domain_joined: Some(false),
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
            explicit_user_approval_timestamp: Some("2026-08-03T12:00:00Z".into()),
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
    }

    #[test]
    fn live_gate_rejects_missing_opt_in_and_identity_scenario_checkpoint_mismatches() {
        let valid = || GateFacts {
            command_line_opt_in: true,
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
        };
        assert!(evaluate_gate(&valid()).0);
        let mut missing_flag = valid();
        missing_flag.command_line_opt_in = false;
        assert!(!evaluate_gate(&missing_flag).0);
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
        let mut wrong_target_type = valid();
        wrong_target_type.target_type_matches = false;
        assert!(evaluate_gate(&wrong_target_type).1.contains("target type"));
    }

    #[test]
    fn physical_target_recovery_confirmations_are_all_fail_closed() {
        let now = current_epoch_ms();
        let mut approval = valid_target_approval(ValidationTargetType::PhysicalLaptop);
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
        let mut approval = valid_target_approval(ValidationTargetType::PhysicalLaptop);
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
        let mut approval = valid_target_approval(ValidationTargetType::PhysicalLaptop);
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

    fn test_bundle(build: u32, plan_hash: &str) -> LiveEvidenceBundle {
        let state = CapturedState {
            representation: super::super::plan::CapturedRepresentation::Dword(1),
            effective_enabled: true,
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
