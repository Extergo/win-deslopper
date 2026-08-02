use serde::{Deserialize, Serialize};

use super::{
    plan::CapturedState,
    request::{MutationOperationId, MutationSubjectId, MutationTarget},
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Created,
    Validating,
    AwaitingApproval,
    Approved,
    CapturingPreState,
    Applying,
    Verifying,
    Applied,
    VerificationFailed,
    RollbackAvailable,
    RollingBack,
    RolledBack,
    RollbackVerificationFailed,
    FailedBeforeMutation,
    FailedAfterMutation,
    RecoveryRequired,
    CancelledBeforeMutation,
}

impl TransactionStatus {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validating => "validating",
            Self::AwaitingApproval => "awaiting_approval",
            Self::Approved => "approved",
            Self::CapturingPreState => "capturing_pre_state",
            Self::Applying => "applying",
            Self::Verifying => "verifying",
            Self::Applied => "applied",
            Self::VerificationFailed => "verification_failed",
            Self::RollbackAvailable => "rollback_available",
            Self::RollingBack => "rolling_back",
            Self::RolledBack => "rolled_back",
            Self::RollbackVerificationFailed => "rollback_verification_failed",
            Self::FailedBeforeMutation => "failed_before_mutation",
            Self::FailedAfterMutation => "failed_after_mutation",
            Self::RecoveryRequired => "recovery_required",
            Self::CancelledBeforeMutation => "cancelled_before_mutation",
        }
    }

    pub const fn is_transitional(self) -> bool {
        matches!(
            self,
            Self::Validating
                | Self::Approved
                | Self::CapturingPreState
                | Self::Applying
                | Self::Verifying
                | Self::Applied
                | Self::RollingBack
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationStep {
    pub sequence: u32,
    pub step_type: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub redacted_evidence: Vec<String>,
    pub error_category: Option<String>,
    pub error_summary: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackRecord {
    pub available: bool,
    pub complete: bool,
    pub attempted_at: Option<String>,
    pub result: Option<String>,
    pub verification_result: Option<String>,
    pub conflict_detected: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationTransaction {
    pub transaction_id: String,
    pub plan_id: String,
    pub machine_id: String,
    pub subject_id: MutationSubjectId,
    pub operation_id: MutationOperationId,
    pub source_inspection_id: String,
    pub source_observation_id: String,
    pub desired_state_revision_id: Option<i64>,
    pub created_at: String,
    pub approved_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub status: TransactionStatus,
    pub required_privilege: String,
    pub handler_version: String,
    pub application_version: String,
    pub windows_build: u32,
    pub edition: String,
    pub plan_hash: String,
    pub pre_state_hash: Option<String>,
    pub post_state_hash: Option<String>,
    pub rollback_state_hash: Option<String>,
    pub target_state: MutationTarget,
    pub pre_state: Option<CapturedState>,
    pub post_state: Option<CapturedState>,
    pub rollback_state: Option<CapturedState>,
    pub verification_result: Option<String>,
    pub error_category: Option<String>,
    pub error_summary: Option<String>,
    pub recovery_requirement: Option<String>,
    pub steps: Vec<MutationStep>,
    pub rollback: RollbackRecord,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_fixtures_replay_state_machine_without_executing_windows_writes() {
        let directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("validation/mutation-fixtures");
        let mut checked = 0;
        for entry in std::fs::read_dir(directory).expect("mutation fixture directory must exist") {
            let path = entry
                .expect("mutation fixture entry must be readable")
                .path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(path).expect("mutation fixture must be readable");
            let value: serde_json::Value =
                serde_json::from_str(&raw).expect("mutation fixture must be valid JSON");
            assert_eq!(value["fixtureSchemaVersion"], 1);
            assert_eq!(value["liveMutationPerformed"], false);
            let operation = value["operationId"]
                .as_str()
                .expect("operation ID must be a string");
            assert!(
                MutationOperationId::ALL
                    .iter()
                    .any(|candidate| candidate.key() == operation),
                "fixture operation must be closed"
            );
            let transitions = value["expectedTransitions"]
                .as_array()
                .expect("transitions must be an array");
            assert!(
                transitions
                    .iter()
                    .all(|transition| known_status(transition.as_str().unwrap_or_default()))
            );
            assert_eq!(value["preState"], value["rollbackState"]);
            checked += 1;
        }
        assert!(checked > 0);
    }

    fn known_status(value: &str) -> bool {
        [
            TransactionStatus::Created,
            TransactionStatus::Validating,
            TransactionStatus::AwaitingApproval,
            TransactionStatus::Approved,
            TransactionStatus::CapturingPreState,
            TransactionStatus::Applying,
            TransactionStatus::Verifying,
            TransactionStatus::Applied,
            TransactionStatus::VerificationFailed,
            TransactionStatus::RollbackAvailable,
            TransactionStatus::RollingBack,
            TransactionStatus::RolledBack,
            TransactionStatus::RollbackVerificationFailed,
            TransactionStatus::FailedBeforeMutation,
            TransactionStatus::FailedAfterMutation,
            TransactionStatus::RecoveryRequired,
            TransactionStatus::CancelledBeforeMutation,
        ]
        .iter()
        .any(|status| status.key() == value)
    }
}
