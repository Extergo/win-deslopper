use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::request::{MutationOperationId, MutationSubjectId, MutationTarget};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapturedRepresentation {
    Missing,
    Dword(u32),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedState {
    pub representation: CapturedRepresentation,
    pub effective_enabled: bool,
    #[serde(default = "default_effective_state_known")]
    pub effective_state_known: bool,
    pub authority: String,
    pub confidence: String,
    pub captured_at: String,
}

const fn default_effective_state_known() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationPlan {
    pub plan_id: String,
    pub machine_id: String,
    pub source_inspection_id: String,
    pub source_observation_id: String,
    pub subject_id: MutationSubjectId,
    pub operation_id: MutationOperationId,
    pub current_state: CapturedState,
    pub target_state: MutationTarget,
    pub authority: String,
    pub applicability: String,
    pub windows_build: u32,
    pub edition: String,
    pub evidence_fingerprint: String,
    pub generated_at: String,
    pub expires_at: String,
    pub approval_class: String,
    pub rollback_method: String,
    pub documentation: Vec<String>,
    pub required_privilege: String,
    pub expected_side_effects: Vec<String>,
    pub restart_requirement: String,
    pub handler_version: String,
    pub desired_state_revision_id: Option<i64>,
    pub automatic_remediation_eligible: bool,
    pub approval_nonce_hash: String,
    pub consumed_at: Option<String>,
    pub plan_hash: String,
}

impl MutationPlan {
    pub fn refresh_hash(&mut self) -> Result<(), serde_json::Error> {
        self.plan_hash.clear();
        self.plan_hash = hash_serializable(self)?;
        Ok(())
    }

    pub fn hash_is_valid(&self) -> bool {
        let mut unsigned = self.clone();
        let expected = unsigned.plan_hash.clone();
        unsigned.plan_hash.clear();
        // Consumption is journal state, not part of the immutable reviewed intent.
        unsigned.consumed_at = None;
        hash_serializable(&unsigned).is_ok_and(|actual| actual == expected)
    }

    pub fn is_expired(&self, now_millis: u128) -> bool {
        self.expires_at
            .parse::<u128>()
            .map_or(true, |expiry| expiry < now_millis)
    }
}

pub fn hash_serializable<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub fn hash_text(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
