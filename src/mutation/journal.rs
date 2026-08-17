use rusqlite::{OptionalExtension, params};

use super::{
    plan::{CapturedState, MutationPlan, hash_serializable},
    transaction::MutationTransaction,
};

#[derive(Clone, Default)]
pub struct MutationJournal {
    database_path: Option<std::path::PathBuf>,
}

#[cfg(feature = "mutation-alpha")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ApprovalUsage {
    pub plans: u32,
    pub executions: u32,
}

impl MutationJournal {
    #[cfg(test)]
    pub fn at(path: std::path::PathBuf) -> Self {
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

    pub fn lock_path(&self) -> std::path::PathBuf {
        self.database_path
            .clone()
            .unwrap_or_else(crate::persistence::path)
            .with_extension("mutation-alpha.lock")
    }

    pub fn save_plan(&self, plan: &MutationPlan) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO mutation_plans(id,machine_id,source_inspection_id,source_observation_id,component_id,operation_id,generated_at,expires_at,plan_hash,approval_nonce_hash,consumed_at,plan_json)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                plan.plan_id,
                plan.machine_id,
                plan.source_inspection_id,
                plan.source_observation_id,
                plan.subject_id.key(),
                plan.operation_id.key(),
                plan.generated_at,
                plan.expires_at,
                plan.plan_hash,
                plan.approval_nonce_hash,
                plan.consumed_at,
                serde_json::to_string(plan).map_err(|error| error.to_string())?,
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn load_plan(&self, plan_id: &str) -> Result<Option<MutationPlan>, String> {
        let conn = self.connection()?;
        let row: Option<(String, Option<String>)> = conn
            .query_row(
                "SELECT plan_json,consumed_at FROM mutation_plans WHERE id=?1",
                [plan_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        row.map(|(raw, consumed_at)| {
            let mut plan: MutationPlan =
                serde_json::from_str(&raw).map_err(|error| error.to_string())?;
            plan.consumed_at = consumed_at;
            Ok(plan)
        })
        .transpose()
    }

    pub fn consume_plan(&self, plan_id: &str, consumed_at: &str) -> Result<(), String> {
        let conn = self.connection()?;
        let changed = conn
            .execute(
                "UPDATE mutation_plans SET consumed_at=?2 WHERE id=?1 AND consumed_at IS NULL",
                params![plan_id, consumed_at],
            )
            .map_err(|error| error.to_string())?;
        if changed == 1 {
            Ok(())
        } else {
            Err("The plan is missing or has already been consumed.".into())
        }
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn approval_usage(&self, approval_class: &str) -> Result<ApprovalUsage, String> {
        let conn = self.connection()?;
        let mut plan_statement = conn
            .prepare("SELECT plan_json FROM mutation_plans")
            .map_err(|error| error.to_string())?;
        let plans = plan_statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        let mut matching_plan_ids = std::collections::HashSet::new();
        for raw in plans {
            let plan: MutationPlan = serde_json::from_str(&raw.map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
            if plan.approval_class == approval_class {
                matching_plan_ids.insert(plan.plan_id);
            }
        }

        let mut executions = 0_u32;
        let mut transaction_statement = conn
            .prepare("SELECT transaction_json FROM mutation_transactions")
            .map_err(|error| error.to_string())?;
        let transactions = transaction_statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        for raw in transactions {
            let transaction: MutationTransaction =
                serde_json::from_str(&raw.map_err(|error| error.to_string())?)
                    .map_err(|error| error.to_string())?;
            if matching_plan_ids.contains(&transaction.plan_id) && transaction.approved_at.is_some()
            {
                executions = executions.saturating_add(1);
            }
        }
        Ok(ApprovalUsage {
            plans: matching_plan_ids.len().try_into().unwrap_or(u32::MAX),
            executions,
        })
    }

    pub fn save_transaction(&self, transaction: &MutationTransaction) -> Result<(), String> {
        let mut conn = self.connection()?;
        let tx = conn.transaction().map_err(|error| error.to_string())?;
        let raw = serde_json::to_string(transaction).map_err(|error| error.to_string())?;
        tx.execute(
            "INSERT INTO mutation_transactions(id,plan_id,machine_id,component_id,operation_id,source_inspection_id,source_observation_id,desired_state_revision_id,created_at,approved_at,started_at,completed_at,status,required_privilege,handler_version,application_version,windows_build,windows_edition,plan_hash,pre_state_hash,post_state_hash,rollback_state_hash,verification_result,error_category,error_summary,recovery_requirement,transaction_json)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27)
             ON CONFLICT(id) DO UPDATE SET approved_at=excluded.approved_at,started_at=excluded.started_at,completed_at=excluded.completed_at,status=excluded.status,pre_state_hash=excluded.pre_state_hash,post_state_hash=excluded.post_state_hash,rollback_state_hash=excluded.rollback_state_hash,verification_result=excluded.verification_result,error_category=excluded.error_category,error_summary=excluded.error_summary,recovery_requirement=excluded.recovery_requirement,transaction_json=excluded.transaction_json",
            params![
                transaction.transaction_id,
                transaction.plan_id,
                transaction.machine_id,
                transaction.subject_id.key(),
                transaction.operation_id.key(),
                transaction.source_inspection_id,
                transaction.source_observation_id,
                transaction.desired_state_revision_id,
                transaction.created_at,
                transaction.approved_at,
                transaction.started_at,
                transaction.completed_at,
                transaction.status.key(),
                transaction.required_privilege,
                transaction.handler_version,
                transaction.application_version,
                transaction.windows_build,
                transaction.edition,
                transaction.plan_hash,
                transaction.pre_state_hash,
                transaction.post_state_hash,
                transaction.rollback_state_hash,
                transaction.verification_result,
                transaction.error_category,
                transaction.error_summary,
                transaction.recovery_requirement,
                raw,
            ],
        )
        .map_err(|error| error.to_string())?;

        tx.execute(
            "DELETE FROM mutation_steps WHERE transaction_id=?1",
            [&transaction.transaction_id],
        )
        .map_err(|error| error.to_string())?;
        for step in &transaction.steps {
            tx.execute(
                "INSERT INTO mutation_steps(transaction_id,sequence_number,step_type,started_at,completed_at,status,redacted_evidence_json,error_category,error_summary) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![transaction.transaction_id,step.sequence,step.step_type,step.started_at,step.completed_at,step.status,serde_json::to_string(&step.redacted_evidence).map_err(|error| error.to_string())?,step.error_category,step.error_summary],
            ).map_err(|error| error.to_string())?;
        }
        save_capture(
            &tx,
            transaction,
            "pre_state",
            transaction.pre_state.as_ref(),
        )?;
        save_capture(
            &tx,
            transaction,
            "post_state",
            transaction.post_state.as_ref(),
        )?;
        save_capture(
            &tx,
            transaction,
            "rollback_state",
            transaction.rollback_state.as_ref(),
        )?;
        tx.execute(
            "INSERT INTO mutation_rollbacks(transaction_id,rollback_plan_json,rollback_available,rollback_complete,attempted_at,result,verification_result,conflict_detected)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(transaction_id) DO UPDATE SET rollback_plan_json=excluded.rollback_plan_json,rollback_available=excluded.rollback_available,rollback_complete=excluded.rollback_complete,attempted_at=excluded.attempted_at,result=excluded.result,verification_result=excluded.verification_result,conflict_detected=excluded.conflict_detected",
            params![transaction.transaction_id,serde_json::to_string(&transaction.pre_state).map_err(|error| error.to_string())?,transaction.rollback.available as i64,transaction.rollback.complete as i64,transaction.rollback.attempted_at,transaction.rollback.result,transaction.rollback.verification_result,transaction.rollback.conflict_detected as i64],
        ).map_err(|error| error.to_string())?;
        tx.commit().map_err(|error| error.to_string())
    }

    pub fn load_transaction(
        &self,
        transaction_id: &str,
    ) -> Result<Option<MutationTransaction>, String> {
        let conn = self.connection()?;
        let raw: Option<String> = conn
            .query_row(
                "SELECT transaction_json FROM mutation_transactions WHERE id=?1",
                [transaction_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        raw.map(|value| serde_json::from_str(&value).map_err(|error| error.to_string()))
            .transpose()
    }

    pub fn history(&self) -> Result<Vec<MutationTransaction>, String> {
        let conn = self.connection()?;
        let mut statement = conn
            .prepare("SELECT transaction_json FROM mutation_transactions ORDER BY created_at DESC")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        rows.map(|row| {
            let raw = row.map_err(|error| error.to_string())?;
            serde_json::from_str(&raw).map_err(|error| error.to_string())
        })
        .collect()
    }

    pub fn interrupted(&self) -> Result<Vec<MutationTransaction>, String> {
        Ok(self
            .history()?
            .into_iter()
            .filter(|transaction| transaction.status.is_transitional())
            .collect())
    }
}

fn save_capture(
    tx: &rusqlite::Transaction<'_>,
    transaction: &MutationTransaction,
    capture_type: &str,
    state: Option<&CapturedState>,
) -> Result<(), String> {
    if let Some(state) = state {
        let hash = hash_serializable(state).map_err(|error| error.to_string())?;
        tx.execute(
            "INSERT INTO mutation_state_captures(transaction_id,capture_type,state_json,evidence_summary_json,captured_at,integrity_hash)
             VALUES(?1,?2,?3,?4,?5,?6)
             ON CONFLICT(transaction_id,capture_type) DO UPDATE SET state_json=excluded.state_json,evidence_summary_json=excluded.evidence_summary_json,captured_at=excluded.captured_at,integrity_hash=excluded.integrity_hash",
            params![transaction.transaction_id,capture_type,serde_json::to_string(state).map_err(|error| error.to_string())?,serde_json::to_string(&["Fixed current-user taskbar representation inspected by the owning handler."]).map_err(|error| error.to_string())?,state.captured_at,hash],
        ).map_err(|error| error.to_string())?;
    }
    Ok(())
}
