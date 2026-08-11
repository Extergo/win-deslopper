use std::{
    fs::{File, OpenOptions},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

#[cfg(feature = "mutation-alpha")]
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use super::{
    OWNER_HANDLER_VERSION,
    handlers::{
        HandlerError, MutationBackend as HandlerBackend, OperationHandler,
        TaskbarShowDesktopHandler, TaskbarTaskViewHandler, TaskbarWidgetsHandler, expected,
    },
    is_owner_handler_version,
    journal::MutationJournal,
    plan::{CapturedState, MutationPlan, hash_serializable, hash_text},
    request::{MutationOperationId, MutationTarget},
    rollback::conflicts_with_applied_state,
    transaction::{MutationStep, MutationTransaction, RollbackRecord, TransactionStatus},
};

#[cfg(feature = "mutation-alpha")]
use super::{
    HANDLER_VERSION,
    live_validation::{
        EvidenceExportRequest, LiveEvidenceBundle, LiveValidationGateStatus, ValidationMaturity,
        ValidationScenarioManifest, ValidationTargetType, load_local_manifest,
        local_scoped_approval_allows, local_validation_directory,
    },
    request::{AlphaGateStatus, PlanRequest, RollbackRequest},
    verification::classify_apply,
};

#[cfg(all(test, feature = "mutation-alpha"))]
use super::live_validation::ApprovedOperationScope;

static BROKER_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static WIDGETS_HANDLER: TaskbarWidgetsHandler = TaskbarWidgetsHandler;
static TASK_VIEW_HANDLER: TaskbarTaskViewHandler = TaskbarTaskViewHandler;
static SHOW_DESKTOP_HANDLER: TaskbarShowDesktopHandler = TaskbarShowDesktopHandler;

#[derive(Clone, Debug)]
pub struct BrokerContext {
    pub machine_id: String,
    pub inspection_id: String,
    pub inspection_timestamp: String,
    pub source_observation_id: String,
    pub windows_build: u32,
    pub edition: String,
    pub architecture: String,
    #[cfg(feature = "mutation-alpha")]
    pub domain_joined: Option<bool>,
    #[cfg(feature = "mutation-alpha")]
    pub entra_joined: Option<bool>,
    #[cfg(feature = "mutation-alpha")]
    pub workplace_joined: Option<bool>,
    #[cfg(feature = "mutation-alpha")]
    pub mdm_enrolled: Option<bool>,
    pub authority: String,
    pub authority_acceptable: bool,
    pub confidence: String,
    pub confidence_sufficient: bool,
    pub applicability: String,
    pub applicable: bool,
    pub evidence_fingerprint: String,
    pub desired_state_revision_id: Option<i64>,
    pub detector_current_enabled: Option<bool>,
    pub detector_status: String,
}

#[cfg(all(test, not(feature = "mutation-alpha")))]
mod owner_tests {
    use std::{
        collections::VecDeque,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, AtomicU64, Ordering},
        },
    };

    use super::*;
    use crate::mutation::{
        handlers::{HandlerErrorKind, MutationBackend},
        plan::CapturedRepresentation,
    };

    #[derive(Clone, Copy)]
    enum WriteBehavior {
        Succeed,
        FailBefore,
        DeniedBefore,
        FailAfter,
        FailAmbiguous,
        Ignore,
        Unexpected,
    }

    struct FakeBackend {
        widgets: Mutex<CapturedRepresentation>,
        policy: AtomicBool,
        writes: AtomicU64,
        behaviors: Mutex<VecDeque<WriteBehavior>>,
    }

    impl FakeBackend {
        fn new(initial: CapturedRepresentation) -> Self {
            Self {
                widgets: Mutex::new(initial),
                policy: AtomicBool::new(false),
                writes: AtomicU64::new(0),
                behaviors: Mutex::new(VecDeque::new()),
            }
        }

        fn queue(&self, behavior: WriteBehavior) {
            self.behaviors.lock().unwrap().push_back(behavior);
        }

        fn set(&self, state: CapturedRepresentation) {
            *self.widgets.lock().unwrap() = state;
        }

        fn write(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            self.writes.fetch_add(1, Ordering::SeqCst);
            match self
                .behaviors
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or(WriteBehavior::Succeed)
            {
                WriteBehavior::Succeed => self.set(state.clone()),
                WriteBehavior::FailBefore => {
                    return Err(HandlerError::new(
                        HandlerErrorKind::WriteFailed,
                        "synthetic write failed before change",
                    ));
                }
                WriteBehavior::DeniedBefore => {
                    return Err(HandlerError::new(
                        HandlerErrorKind::PermissionDenied,
                        "synthetic Windows error category: PermissionDenied",
                    ));
                }
                WriteBehavior::FailAfter => {
                    self.set(state.clone());
                    return Err(HandlerError::new(
                        HandlerErrorKind::WriteFailed,
                        "synthetic write failed after change",
                    ));
                }
                WriteBehavior::FailAmbiguous => {
                    self.set(CapturedRepresentation::Dword(2));
                    return Err(HandlerError::new(
                        HandlerErrorKind::WriteFailed,
                        "synthetic write result was ambiguous",
                    ));
                }
                WriteBehavior::Ignore => {}
                WriteBehavior::Unexpected => {
                    self.set(state.clone());
                    self.policy.store(true, Ordering::SeqCst);
                }
            }
            Ok(())
        }
    }

    impl MutationBackend for FakeBackend {
        fn read_widgets(&self) -> Result<CapturedRepresentation, HandlerError> {
            Ok(self.widgets.lock().unwrap().clone())
        }

        fn widgets_externally_managed(&self) -> Result<bool, HandlerError> {
            Ok(self.policy.load(Ordering::SeqCst))
        }

        fn write_widgets(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            self.write(state)
        }

        fn read_task_view(&self) -> Result<CapturedRepresentation, HandlerError> {
            Ok(self.widgets.lock().unwrap().clone())
        }

        fn task_view_externally_managed(&self) -> Result<bool, HandlerError> {
            Ok(false)
        }

        fn write_task_view(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            self.write(state)
        }

        fn read_show_desktop(&self) -> Result<CapturedRepresentation, HandlerError> {
            Ok(CapturedRepresentation::Dword(1))
        }

        fn show_desktop_externally_managed(&self) -> Result<bool, HandlerError> {
            Ok(false)
        }

        fn write_show_desktop(&self, _: &CapturedRepresentation) -> Result<(), HandlerError> {
            panic!("Show Desktop is not an owner operation")
        }
    }

    fn context(enabled: Option<bool>) -> BrokerContext {
        BrokerContext {
            machine_id: crate::owner_scope::from_stable_ids("machine", "S-1-5-21-1000"),
            inspection_id: "inspection-owner".into(),
            inspection_timestamp: crate::inspection::timestamp(),
            source_observation_id: "inspection-owner:taskbar_widgets".into(),
            windows_build: 26_100,
            edition: "Professional".into(),
            architecture: "64-bit".into(),
            authority: "user".into(),
            authority_acceptable: true,
            confidence: "confirmed_representation".into(),
            confidence_sufficient: true,
            applicability: "applicable".into(),
            applicable: true,
            evidence_fingerprint: hash_text("owner-fixture"),
            desired_state_revision_id: None,
            detector_current_enabled: enabled,
            detector_status: if enabled.is_some() {
                "successful".into()
            } else {
                "unknown".into()
            },
        }
    }

    fn temp_database(label: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "deslopper-owner-{label}-{}-{nonce}.db",
            std::process::id()
        ))
    }

    fn broker(
        label: &str,
        initial: CapturedRepresentation,
    ) -> (Broker, Arc<FakeBackend>, std::path::PathBuf) {
        let backend = Arc::new(FakeBackend::new(initial));
        let path = temp_database(label);
        let broker = Broker::with_owner_journal(backend.clone(), MutationJournal::at(path.clone()));
        (broker, backend, path)
    }

    #[test]
    fn owner_registry_exposes_widgets_and_task_view_only() {
        let (broker, backend, path) = broker("closed", CapturedRepresentation::Dword(1));
        let result = broker.apply_owner_operation(
            MutationOperationId::ShowDesktopEnabled,
            MutationTarget::Disabled,
            &context(Some(true)),
            || Ok(context(Some(false))),
        );
        assert_eq!(result.unwrap_err().code, "operation_not_productized");
        assert_eq!(backend.writes.load(Ordering::SeqCst), 0);
        cleanup(&path);
    }

    #[test]
    fn task_view_owner_apply_is_verified_and_undoable() {
        let (broker, backend, path) = broker("task-view", CapturedRepresentation::Dword(0));
        let applied = broker
            .apply_owner_operation(
                MutationOperationId::TaskViewVisibility,
                MutationTarget::Enabled,
                &context(Some(false)),
                || Ok(context(Some(true))),
            )
            .unwrap();
        assert_eq!(applied.outcome, OwnerOperationOutcome::Changed);
        assert_eq!(
            applied.classification,
            OwnerResultClassification::ChangedVerified
        );
        assert_eq!(
            *backend.widgets.lock().unwrap(),
            CapturedRepresentation::Dword(1)
        );
        let undone = broker
            .undo_owner_operation(
                &applied.transaction.transaction_id,
                &context(Some(true)),
                || Ok(context(Some(false))),
            )
            .unwrap();
        assert_eq!(undone.classification, OwnerResultClassification::Restored);
        assert_eq!(
            *backend.widgets.lock().unwrap(),
            CapturedRepresentation::Dword(0)
        );
        cleanup(&path);
    }

    #[test]
    fn widgets_permission_denial_is_machine_scoped_direct_unavailability() {
        let (broker, backend, path) = broker("widgets-denied", CapturedRepresentation::Dword(0));
        backend.queue(WriteBehavior::DeniedBefore);
        let result = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Enabled,
                &context(Some(false)),
                || panic!("unchanged rejected write must not run detector verification"),
            )
            .unwrap();
        assert_eq!(
            result.classification,
            OwnerResultClassification::WriteRejectedUnchanged
        );
        assert!(!result.transaction.rollback.available);
        assert_eq!(
            broker
                .owner_actionability(
                    MutationOperationId::WidgetsVisibility,
                    &context(Some(false))
                )
                .status,
            OwnerActionabilityStatus::DirectChangeUnavailable
        );
        assert_eq!(
            broker
                .owner_actionability(
                    MutationOperationId::TaskViewVisibility,
                    &context(Some(false))
                )
                .status,
            OwnerActionabilityStatus::Ready
        );
        cleanup(&path);
    }

    #[test]
    fn owner_actionability_handles_managed_unknown_and_missing_states() {
        let (broker, backend, path) = broker("actionability", CapturedRepresentation::Missing);
        let missing =
            broker.owner_actionability(MutationOperationId::WidgetsVisibility, &context(None));
        assert_eq!(missing.status, OwnerActionabilityStatus::Ready);
        assert_eq!(missing.available_targets.len(), 2);
        backend.set(CapturedRepresentation::Dword(2));
        assert_eq!(
            broker
                .owner_actionability(MutationOperationId::WidgetsVisibility, &context(None))
                .status,
            OwnerActionabilityStatus::Unknown
        );
        backend.set(CapturedRepresentation::Dword(1));
        backend.policy.store(true, Ordering::SeqCst);
        assert_eq!(
            broker
                .owner_actionability(MutationOperationId::WidgetsVisibility, &context(Some(true)))
                .status,
            OwnerActionabilityStatus::Managed
        );
        cleanup(&path);
    }

    #[test]
    fn owner_mode_refuses_unsupported_build_and_arm64_without_writing() {
        let (broker, backend, path) = broker("unsupported", CapturedRepresentation::Dword(1));
        let mut old_windows = context(Some(true));
        old_windows.windows_build = 19_045;
        assert_eq!(
            broker
                .owner_actionability(MutationOperationId::WidgetsVisibility, &old_windows)
                .status,
            OwnerActionabilityStatus::Unsupported
        );
        assert_eq!(
            broker
                .apply_owner_operation(
                    MutationOperationId::WidgetsVisibility,
                    MutationTarget::Disabled,
                    &old_windows,
                    || Ok(context(Some(false))),
                )
                .unwrap_err()
                .code,
            "unsupported_build"
        );

        let mut arm64 = context(Some(true));
        arm64.architecture = "ARM64".into();
        assert_eq!(
            broker
                .owner_actionability(MutationOperationId::WidgetsVisibility, &arm64)
                .status,
            OwnerActionabilityStatus::Unsupported
        );
        assert_eq!(
            broker
                .apply_owner_operation(
                    MutationOperationId::WidgetsVisibility,
                    MutationTarget::Disabled,
                    &arm64,
                    || Ok(context(Some(false))),
                )
                .unwrap_err()
                .code,
            "unsupported_architecture"
        );
        assert_eq!(backend.writes.load(Ordering::SeqCst), 0);
        cleanup(&path);
    }

    #[test]
    fn owner_apply_refuses_managed_unknown_and_stale_state_without_writing() {
        let (broker, backend, path) = broker("refusals", CapturedRepresentation::Dword(1));
        backend.policy.store(true, Ordering::SeqCst);
        assert!(
            broker
                .apply_owner_operation(
                    MutationOperationId::WidgetsVisibility,
                    MutationTarget::Disabled,
                    &context(Some(true)),
                    || Ok(context(Some(false))),
                )
                .is_err()
        );

        backend.policy.store(false, Ordering::SeqCst);
        backend.set(CapturedRepresentation::Dword(2));
        assert_eq!(
            broker
                .apply_owner_operation(
                    MutationOperationId::WidgetsVisibility,
                    MutationTarget::Disabled,
                    &context(None),
                    || Ok(context(Some(false))),
                )
                .unwrap_err()
                .code,
            "handler_error"
        );

        backend.set(CapturedRepresentation::Dword(0));
        assert_eq!(
            broker
                .apply_owner_operation(
                    MutationOperationId::WidgetsVisibility,
                    MutationTarget::Enabled,
                    &context(Some(true)),
                    || Ok(context(Some(true))),
                )
                .unwrap_err()
                .code,
            "stale_source_state"
        );
        assert_eq!(backend.writes.load(Ordering::SeqCst), 0);
        cleanup(&path);
    }

    #[test]
    fn apply_noop_writes_nothing_and_success_survives_relaunch_then_undo() {
        let (broker, backend, path) = broker("relaunch", CapturedRepresentation::Dword(1));
        let noop = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Enabled,
                &context(Some(true)),
                || Ok(context(Some(true))),
            )
            .unwrap();
        assert_eq!(noop.outcome, OwnerOperationOutcome::AlreadySet);
        assert_eq!(backend.writes.load(Ordering::SeqCst), 0);

        let applied = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || Ok(context(Some(false))),
            )
            .unwrap();
        assert_eq!(applied.outcome, OwnerOperationOutcome::Changed);
        assert_eq!(
            applied.transaction.status,
            TransactionStatus::RollbackAvailable
        );
        drop(broker);

        let reopened =
            Broker::with_owner_journal(backend.clone(), MutationJournal::at(path.clone()));
        let history = reopened
            .owner_history(&context(Some(false)).machine_id)
            .unwrap();
        assert_eq!(history.len(), 2);
        let restored = reopened
            .undo_owner_operation(
                &applied.transaction.transaction_id,
                &context(Some(false)),
                || Ok(context(Some(true))),
            )
            .unwrap();
        assert_eq!(restored.outcome, OwnerOperationOutcome::Restored);
        assert_eq!(restored.transaction.status, TransactionStatus::RolledBack);
        assert_eq!(
            backend.read_widgets().unwrap(),
            CapturedRepresentation::Dword(1)
        );
        cleanup(&path);
    }

    #[test]
    fn exact_undo_restores_original_absence() {
        let (broker, backend, path) = broker("absence", CapturedRepresentation::Missing);
        let applied = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(None),
                || Ok(context(Some(false))),
            )
            .unwrap();
        let restored = broker
            .undo_owner_operation(
                &applied.transaction.transaction_id,
                &context(Some(false)),
                || Ok(context(None)),
            )
            .unwrap();
        assert_eq!(restored.outcome, OwnerOperationOutcome::Restored);
        assert_eq!(
            backend.read_widgets().unwrap(),
            CapturedRepresentation::Missing
        );
        cleanup(&path);
    }

    #[test]
    fn write_failures_capture_actual_state_and_never_offer_fake_undo() {
        let (broker, backend, path) = broker("write-failures", CapturedRepresentation::Dword(1));
        backend.queue(WriteBehavior::FailBefore);
        let before = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || Ok(context(Some(false))),
            )
            .unwrap();
        assert_eq!(before.outcome, OwnerOperationOutcome::CouldNotChange);
        assert_eq!(
            before.classification,
            OwnerResultClassification::WriteRejectedUnchanged
        );
        assert!(before.transaction.post_state.is_some());
        assert!(!before.transaction.rollback.available);

        backend.queue(WriteBehavior::FailAfter);
        backend.queue(WriteBehavior::Succeed);
        let rolled_back = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || Ok(context(Some(false))),
            )
            .unwrap();
        assert_eq!(rolled_back.outcome, OwnerOperationOutcome::Restored);
        assert_eq!(
            rolled_back.classification,
            OwnerResultClassification::VerificationFailedRolledBack
        );
        assert_eq!(
            rolled_back.transaction.verification_result.as_deref(),
            Some("verification_failed_rolled_back")
        );
        assert!(rolled_back.transaction.post_state.is_some());
        assert!(rolled_back.transaction.rollback.complete);
        assert!(!rolled_back.transaction.rollback.available);

        backend.set(CapturedRepresentation::Dword(1));
        backend.queue(WriteBehavior::FailAmbiguous);
        let ambiguous = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || panic!("ambiguous write must not run detector verification"),
            )
            .unwrap();
        assert_eq!(
            ambiguous.classification,
            OwnerResultClassification::WriteResultAmbiguous
        );
        assert_eq!(
            ambiguous.transaction.verification_result.as_deref(),
            Some("write_result_ambiguous")
        );
        assert!(!ambiguous.transaction.rollback.available);

        let changed_backend = Arc::new(FakeBackend::new(CapturedRepresentation::Dword(1)));
        let changed_path = temp_database("verification-changed");
        let changed_broker = Broker::with_owner_journal(
            changed_backend.clone(),
            MutationJournal::at(changed_path.clone()),
        );
        changed_backend.queue(WriteBehavior::Unexpected);
        let changed = changed_broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || panic!("changed verification failure must not run detector verification"),
            )
            .unwrap();
        assert_eq!(
            changed.classification,
            OwnerResultClassification::VerificationFailedChanged
        );
        assert_eq!(
            changed.transaction.verification_result.as_deref(),
            Some("verification_failed_changed")
        );
        cleanup(&path);
        cleanup(&changed_path);
    }

    #[test]
    fn detector_failure_rolls_back_when_safe_and_records_rollback_failure() {
        let (broker, backend, path) = broker("detector-rollback", CapturedRepresentation::Dword(1));
        let restored = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || Ok(context(Some(true))),
            )
            .unwrap();
        assert_eq!(restored.outcome, OwnerOperationOutcome::Restored);
        assert_eq!(
            restored.classification,
            OwnerResultClassification::VerificationFailedRolledBack
        );
        assert_eq!(restored.transaction.status, TransactionStatus::RolledBack);
        assert!(!restored.transaction.rollback.available);

        backend.queue(WriteBehavior::Succeed);
        backend.queue(WriteBehavior::FailBefore);
        let failed_rollback = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || Ok(context(Some(true))),
            )
            .unwrap();
        assert_eq!(
            failed_rollback.outcome,
            OwnerOperationOutcome::NeedsAttention
        );
        assert_eq!(
            failed_rollback.transaction.status,
            TransactionStatus::RollbackVerificationFailed
        );
        assert!(!failed_rollback.transaction.rollback.available);
        cleanup(&path);
    }

    #[test]
    fn ignored_success_is_a_failed_verification_without_undo() {
        let (broker, backend, path) = broker("ignored", CapturedRepresentation::Dword(1));
        backend.queue(WriteBehavior::Ignore);
        let result = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || Ok(context(Some(false))),
            )
            .unwrap();
        assert_eq!(result.outcome, OwnerOperationOutcome::CouldNotChange);
        assert_eq!(
            result.transaction.status,
            TransactionStatus::VerificationFailed
        );
        assert!(!result.transaction.rollback.available);
        cleanup(&path);
    }

    #[test]
    fn undo_refuses_conflict_and_another_user_scope() {
        let (broker, backend, path) = broker("conflict", CapturedRepresentation::Dword(1));
        let applied = broker
            .apply_owner_operation(
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context(Some(true)),
                || Ok(context(Some(false))),
            )
            .unwrap();
        let mut other_user = context(Some(false));
        other_user.machine_id = crate::owner_scope::from_stable_ids("machine", "S-1-5-21-2000");
        assert_eq!(
            broker
                .undo_owner_operation(&applied.transaction.transaction_id, &other_user, || {
                    Ok(other_user.clone())
                })
                .unwrap_err()
                .code,
            "owner_scope_mismatch"
        );

        backend.set(CapturedRepresentation::Dword(1));
        let conflict = broker
            .undo_owner_operation(
                &applied.transaction.transaction_id,
                &context(Some(true)),
                || Ok(context(Some(true))),
            )
            .unwrap();
        assert_eq!(conflict.outcome, OwnerOperationOutcome::NeedsAttention);
        assert!(conflict.transaction.rollback.conflict_detected);
        assert_eq!(backend.writes.load(Ordering::SeqCst), 1);
        cleanup(&path);
    }

    #[test]
    fn exact_prestate_is_durable_before_the_write_boundary() {
        let (broker, _, path) = broker("prestate", CapturedRepresentation::Dword(1));
        broker.inject_fault(Some(FaultPoint::BeforeWrite));
        assert!(
            broker
                .apply_owner_operation(
                    MutationOperationId::WidgetsVisibility,
                    MutationTarget::Disabled,
                    &context(Some(true)),
                    || Ok(context(Some(false))),
                )
                .is_err()
        );
        broker.inject_fault(None);
        let history = broker
            .owner_history(&context(Some(true)).machine_id)
            .unwrap();
        assert_eq!(history.len(), 1);
        assert!(history[0].pre_state.is_some());
        assert!(history[0].pre_state_hash.is_some());
        cleanup(&path);
    }

    #[test]
    fn owner_startup_recovery_never_replays_an_interrupted_write() {
        let (broker, backend, path) = broker("recovery", CapturedRepresentation::Dword(1));
        broker.inject_fault(Some(FaultPoint::ImmediatelyAfterWrite));
        assert!(
            broker
                .apply_owner_operation(
                    MutationOperationId::WidgetsVisibility,
                    MutationTarget::Disabled,
                    &context(Some(true)),
                    || Ok(context(Some(false))),
                )
                .is_err()
        );
        assert_eq!(backend.writes.load(Ordering::SeqCst), 1);
        drop(broker);

        let reopened =
            Broker::with_owner_journal(backend.clone(), MutationJournal::at(path.clone()));
        let recovered = reopened.recover_interrupted(&context(Some(false))).unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].status, TransactionStatus::RecoveryRequired);
        assert_eq!(
            reopened
                .owner_actionability(
                    MutationOperationId::WidgetsVisibility,
                    &context(Some(false)),
                )
                .status,
            OwnerActionabilityStatus::Unknown
        );
        assert_eq!(
            reopened
                .apply_owner_operation(
                    MutationOperationId::WidgetsVisibility,
                    MutationTarget::Enabled,
                    &context(Some(false)),
                    || Ok(context(Some(true))),
                )
                .unwrap_err()
                .code,
            "owner_recovery_required"
        );
        assert_eq!(backend.writes.load(Ordering::SeqCst), 1);
        cleanup(&path);
    }

    fn cleanup(path: &std::path::Path) {
        for candidate in [
            path.to_path_buf(),
            path.with_extension("db-wal"),
            path.with_extension("db-shm"),
            path.with_extension("mutation-alpha.lock"),
        ] {
            let _ = std::fs::remove_file(candidate);
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationDefinition {
    pub operation_id: MutationOperationId,
    pub subject_id: super::request::MutationSubjectId,
    pub title: &'static str,
    pub description: &'static str,
    pub supported_targets: Vec<MutationTarget>,
    pub minimum_build: u32,
    pub supported_editions: &'static [&'static str],
    pub required_authority: &'static str,
    pub required_confidence: &'static str,
    pub required_privilege: &'static str,
    pub side_effects: &'static [&'static str],
    pub restart_requirement: &'static str,
    pub rollback_method: &'static str,
    pub documentation: &'static [&'static str],
    pub maximum_duration_ms: u64,
    pub automatic_remediation_eligible: bool,
}

#[cfg(feature = "mutation-alpha")]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationOption {
    pub definition: OperationDefinition,
    pub current_state: Option<CapturedState>,
    pub validation_maturity: ValidationMaturity,
    pub validation_label: &'static str,
    pub eligible: bool,
    pub reason: String,
}

#[cfg(feature = "mutation-alpha")]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuedPlan {
    pub plan: MutationPlan,
    pub approval_nonce: String,
    pub confirmation_text: String,
    pub approval_phrase: String,
    pub proposed_representation: super::plan::CapturedRepresentation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerActionabilityStatus {
    Ready,
    DirectChangeUnavailable,
    NeedsScan,
    Managed,
    Unsupported,
    Unknown,
    Busy,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerActionability {
    pub status: OwnerActionabilityStatus,
    pub operation_id: MutationOperationId,
    pub current_state: Option<CapturedState>,
    pub available_targets: Vec<MutationTarget>,
    pub reason: String,
    pub scope: &'static str,
    pub undo_supported: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerOperationOutcome {
    Changed,
    AlreadySet,
    CouldNotChange,
    Restored,
    NeedsAttention,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerResultClassification {
    ChangedVerified,
    AlreadySet,
    WriteRejectedUnchanged,
    WriteResultAmbiguous,
    VerificationFailedChanged,
    VerificationFailedRolledBack,
    Restored,
    Conflict,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerOperationResult {
    pub outcome: OwnerOperationOutcome,
    pub classification: OwnerResultClassification,
    pub transaction: MutationTransaction,
    pub current_state: Option<CapturedState>,
    pub message: String,
    pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultPoint {
    BeforePreStateCapture,
    AfterPreStateCapture,
    BeforeWrite,
    ImmediatelyAfterWrite,
    BeforeVerification,
    DuringVerification,
    BeforeTransactionCommit,
    BeforeRollback,
    ImmediatelyAfterRollbackWrite,
    DuringRollbackVerification,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerError {
    pub code: &'static str,
    pub message: String,
}

impl BrokerError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub struct Broker {
    backend: Arc<dyn HandlerBackend>,
    journal: MutationJournal,
    #[cfg(feature = "mutation-alpha")]
    command_line_opt_in: bool,
    #[cfg(feature = "mutation-alpha")]
    live_validation: LiveValidationGateStatus,
    #[cfg(feature = "mutation-alpha")]
    validation_maturities: Mutex<[ValidationMaturity; 3]>,
    #[cfg(feature = "mutation-alpha")]
    warning_acknowledged: AtomicBool,
    execution_lock: Mutex<()>,
    fault_point: Mutex<Option<FaultPoint>>,
}

impl Broker {
    #[cfg(not(feature = "mutation-alpha"))]
    pub fn owner(backend: Arc<dyn HandlerBackend>) -> Self {
        Self {
            backend,
            journal: MutationJournal::default(),
            execution_lock: Mutex::new(()),
            fault_point: Mutex::new(None),
        }
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn new(
        backend: Arc<dyn HandlerBackend>,
        command_line_opt_in: bool,
        live_validation: LiveValidationGateStatus,
    ) -> Self {
        Self {
            backend,
            journal: MutationJournal::default(),
            command_line_opt_in,
            live_validation,
            validation_maturities: Mutex::new([ValidationMaturity::SyntheticTested; 3]),
            warning_acknowledged: AtomicBool::new(false),
            execution_lock: Mutex::new(()),
            fault_point: Mutex::new(None),
        }
    }

    #[cfg(all(test, feature = "mutation-alpha"))]
    pub fn with_journal(
        backend: Arc<dyn HandlerBackend>,
        command_line_opt_in: bool,
        live_validation: LiveValidationGateStatus,
        journal: MutationJournal,
    ) -> Self {
        Self {
            backend,
            journal,
            command_line_opt_in,
            live_validation,
            validation_maturities: Mutex::new([ValidationMaturity::SyntheticTested; 3]),
            warning_acknowledged: AtomicBool::new(false),
            execution_lock: Mutex::new(()),
            fault_point: Mutex::new(None),
        }
    }

    #[cfg(all(test, not(feature = "mutation-alpha")))]
    pub fn with_owner_journal(backend: Arc<dyn HandlerBackend>, journal: MutationJournal) -> Self {
        Self {
            backend,
            journal,
            execution_lock: Mutex::new(()),
            fault_point: Mutex::new(None),
        }
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn gate_status(&self) -> AlphaGateStatus {
        let debug_build = cfg!(debug_assertions);
        let warning_acknowledged = self.warning_acknowledged.load(Ordering::SeqCst);
        let mut live_validation = self.live_validation.clone();
        if !live_validation.scope_is_valid() {
            live_validation.available = false;
            live_validation.reason =
                "The scoped live-validation approval has no valid operation allowance.".into();
        } else if live_validation
            .approval_expires_at_epoch_ms
            .is_none_or(|expiration| expiration <= now_millis() as u64)
        {
            live_validation.available = false;
            live_validation.reason = "The scoped live-validation approval has expired.".into();
        }
        let available = debug_build
            && self.command_line_opt_in
            && warning_acknowledged
            && live_validation.available;
        let reason: String = if !debug_build {
            "Mutation alpha is unavailable outside debug/internal builds.".into()
        } else if !self.command_line_opt_in {
            "Restart with --enable-mutation-alpha in a disposable Windows test environment.".into()
        } else if !warning_acknowledged {
            "Acknowledge the in-application experimental mutation warning.".into()
        } else if !live_validation.available {
            live_validation.reason.clone()
        } else {
            "All internal mutation-alpha gates are present.".into()
        };
        AlphaGateStatus {
            compiled: true,
            debug_build,
            command_line_opt_in: self.command_line_opt_in,
            warning_acknowledged,
            warning_text: "Internal Mutation Alpha can change the current user's Windows taskbar settings. It is experimental, local-only, and permitted only on an explicitly approved disposable validation target. Acknowledgement is not authorization.",
            live_validation,
            available,
            reason,
        }
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn acknowledge_warning(&self, acknowledged: bool) -> AlphaGateStatus {
        self.warning_acknowledged
            .store(acknowledged, Ordering::SeqCst);
        self.gate_status()
    }

    pub fn owner_actionability(
        &self,
        operation_id: MutationOperationId,
        context: &BrokerContext,
    ) -> OwnerActionability {
        let label = owner_operation_label(operation_id);
        let base = |status, reason, current_state, available_targets| OwnerActionability {
            status,
            operation_id,
            current_state,
            available_targets,
            reason,
            scope: owner_operation_scope(operation_id),
            undo_supported: true,
        };
        if self.execution_lock.try_lock().is_err() {
            return base(
                OwnerActionabilityStatus::Busy,
                "Another Windows change is in progress.".into(),
                None,
                Vec::new(),
            );
        }
        if !crate::owner_scope::is_valid(&context.machine_id) {
            return base(
                OwnerActionabilityStatus::NeedsScan,
                "Run a new scan to establish this Windows account's owner scope.".into(),
                None,
                Vec::new(),
            );
        }
        if !is_owner_operation(operation_id) {
            return base(
                OwnerActionabilityStatus::Unsupported,
                "This setting is not available in Owner Mode M2.".into(),
                None,
                Vec::new(),
            );
        }
        if let Err(error) = validate_owner_context(operation_id, context) {
            let status = match error.code {
                "external_authority" => OwnerActionabilityStatus::Managed,
                "unsupported_build" | "unsupported_edition" | "unsupported_architecture" => {
                    OwnerActionabilityStatus::Unsupported
                }
                _ => OwnerActionabilityStatus::Unknown,
            };
            return base(status, error.message, None, Vec::new());
        }
        match self.owner_history(&context.machine_id) {
            Ok(history) if history.iter().any(owner_recovery_blocks_new_apply) => {
                return base(
                    OwnerActionabilityStatus::Unknown,
                    "A previous Owner Mode transaction needs attention before another change can be applied.".into(),
                    None,
                    Vec::new(),
                );
            }
            Err(_) => {
                return base(
                    OwnerActionabilityStatus::Unknown,
                    format!("{label} transaction history could not be verified safely."),
                    None,
                    Vec::new(),
                );
            }
            _ => {}
        }
        if operation_id == MutationOperationId::WidgetsVisibility
            && let Ok(history) = self.owner_history(&context.machine_id)
            && history.iter().any(owner_direct_change_unavailable)
        {
            return base(
                OwnerActionabilityStatus::DirectChangeUnavailable,
                "Windows prevented this setting from being changed for this Windows account. Direct change is unavailable on this machine/account scope.".into(),
                None,
                Vec::new(),
            );
        }
        let state = match handler(operation_id).inspect_pre_state(self.backend.as_ref()) {
            Ok(state) => state,
            Err(error) => {
                let status = if error.kind == super::handlers::HandlerErrorKind::PolicyOverride {
                    OwnerActionabilityStatus::Managed
                } else {
                    OwnerActionabilityStatus::Unknown
                };
                return base(status, error.summary, None, Vec::new());
            }
        };
        if let Err(error) = validate_captured_authority(&state) {
            return base(
                OwnerActionabilityStatus::Managed,
                error.message,
                Some(state),
                Vec::new(),
            );
        }
        let targets = if state.effective_state_known {
            if context.detector_current_enabled != Some(state.effective_enabled) {
                return base(
                    OwnerActionabilityStatus::NeedsScan,
                    format!(
                        "The {label} detector and direct setting no longer agree. Run a new scan."
                    ),
                    Some(state),
                    Vec::new(),
                );
            }
            vec![if state.effective_enabled {
                MutationTarget::Disabled
            } else {
                MutationTarget::Enabled
            }]
        } else if matches!(
            state.representation,
            super::plan::CapturedRepresentation::Missing
        ) {
            vec![MutationTarget::Disabled, MutationTarget::Enabled]
        } else {
            return base(
                OwnerActionabilityStatus::Unknown,
                format!("The {label} setting has an unsupported representation."),
                Some(state),
                Vec::new(),
            );
        };
        base(
            OwnerActionabilityStatus::Ready,
            if state.effective_state_known {
                format!("{label} is ready for a verified owner-mode change.")
            } else {
                format!(
                    "Windows has no explicit {label} preference. Choose Show or Hide; Undo will restore the original absence."
                )
            },
            Some(state),
            targets,
        )
    }

    pub fn apply_owner_operation<F>(
        &self,
        operation_id: MutationOperationId,
        target: MutationTarget,
        context: &BrokerContext,
        detector_verification: F,
    ) -> Result<OwnerOperationResult, BrokerError>
    where
        F: FnOnce() -> Result<BrokerContext, String>,
    {
        if !is_owner_operation(operation_id) {
            return Err(BrokerError::new(
                "operation_not_productized",
                "Only Widgets and Task View are available in Owner Mode M2.",
            ));
        }
        validate_owner_context(operation_id, context)?;
        let label = owner_operation_label(operation_id);
        if self
            .owner_history(&context.machine_id)?
            .iter()
            .any(owner_recovery_blocks_new_apply)
        {
            return Err(BrokerError::new(
                "owner_recovery_required",
                "A previous Owner Mode transaction needs attention before another change can be applied.",
            ));
        }
        let _in_process = self.execution_lock.try_lock().map_err(|_| {
            BrokerError::new(
                "operation_in_progress",
                "Another Windows change is in progress.",
            )
        })?;
        let _cross_process = MutationProcessLock::acquire(self.journal.lock_path())
            .map_err(|message| BrokerError::new("cross_process_lock_unavailable", message))?;
        let definition = operation_definition(operation_id);
        handler(operation_id)
            .validate_target(target)
            .map_err(handler_error)?;
        let pre_state = handler(operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .map_err(handler_error)?;
        validate_captured_authority(&pre_state)?;
        if !pre_state.effective_state_known
            && !matches!(
                pre_state.representation,
                super::plan::CapturedRepresentation::Missing
            )
        {
            return Err(BrokerError::new(
                "unsupported_representation",
                format!(
                    "The {label} setting has an unsupported representation and was not changed."
                ),
            ));
        }
        if pre_state.effective_state_known
            && context.detector_current_enabled != Some(pre_state.effective_enabled)
        {
            return Err(BrokerError::new(
                "stale_source_state",
                format!(
                    "The {label} detector and direct setting changed before Apply. Run a new scan."
                ),
            ));
        }

        let plan = owner_plan(context, &definition, target, pre_state.clone())?;
        self.journal.save_plan(&plan).map_err(journal_error)?;
        let mut transaction = new_transaction(&plan);
        transaction.approved_at = Some(crate::inspection::timestamp());
        self.journal
            .save_transaction(&transaction)
            .map_err(journal_error)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::Validating,
            "owner_preflight",
        )?;
        self.fault(FaultPoint::BeforePreStateCapture)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::CapturingPreState,
            "capture_pre_state",
        )?;
        transaction.pre_state_hash =
            Some(hash_serializable(&pre_state).map_err(serialization_error)?);
        transaction.pre_state = Some(pre_state.clone());
        transaction.started_at = Some(crate::inspection::timestamp());
        self.journal
            .save_transaction(&transaction)
            .map_err(journal_error)?;
        self.fault(FaultPoint::AfterPreStateCapture)?;
        self.journal
            .consume_plan(
                &plan.plan_id,
                transaction.started_at.as_deref().unwrap_or_default(),
            )
            .map_err(journal_error)?;

        let rechecked = handler(operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .map_err(handler_error)?;
        validate_captured_authority(&rechecked)?;
        if !same_effective_state(&pre_state, &rechecked) {
            transaction.error_category = Some("changed_source_state".into());
            transaction.error_summary =
                Some("The Widgets setting changed after pre-state capture.".into());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::FailedBeforeMutation,
                "prewrite_state_changed",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::CouldNotChange,
                transaction,
                Some(rechecked),
                format!("{label} changed before Deslopper could apply the request."),
                OwnerResultClassification::Conflict,
                None,
            ));
        }

        let write_required = pre_state.representation != expected(target)
            || !pre_state.effective_state_known
            || pre_state.effective_enabled != target.enabled();
        if !write_required {
            transaction.post_state_hash = transaction.pre_state_hash.clone();
            transaction.post_state = Some(pre_state.clone());
            transaction.verification_result = Some("already_compliant".into());
            transaction.completed_at = Some(crate::inspection::timestamp());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::NoChangeNeeded,
                "already_compliant",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::AlreadySet,
                transaction,
                Some(pre_state),
                format!("{label} was already set that way. No registry write was performed."),
                OwnerResultClassification::AlreadySet,
                None,
            ));
        }

        transaction.rollback.available = true;
        self.journal
            .save_transaction(&transaction)
            .map_err(journal_error)?;
        self.fault(FaultPoint::BeforeWrite)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::Applying,
            "apply_owner_operation",
        )?;
        let write_result = handler(operation_id).apply(self.backend.as_ref(), target);
        self.fault(FaultPoint::ImmediatelyAfterWrite)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::Verifying,
            "capture_actual_post_attempt_state",
        )?;
        let actual = match handler(operation_id).inspect_pre_state(self.backend.as_ref()) {
            Ok(actual) => {
                set_post_state(&mut transaction, &actual)?;
                self.journal
                    .save_transaction(&transaction)
                    .map_err(journal_error)?;
                actual
            }
            Err(error) => {
                transaction.rollback.available = false;
                transaction.error_category = Some(format!("{:?}", error.kind));
                transaction.error_summary = Some(error.summary);
                transaction.verification_result = Some("write_result_ambiguous".into());
                transaction.recovery_requirement = Some(
                    "The post-attempt registry representation could not be read. Do not retry blindly."
                        .into(),
                );
                transition(
                    &self.journal,
                    &mut transaction,
                    TransactionStatus::RecoveryRequired,
                    "post_attempt_state_unreadable",
                )?;
                return Ok(owner_result(
                    OwnerOperationOutcome::NeedsAttention,
                    transaction,
                    None,
                    "Deslopper could not determine the setting after the write attempt.",
                    OwnerResultClassification::WriteResultAmbiguous,
                    None,
                ));
            }
        };

        if let Err(error) = write_result {
            transaction.error_category = Some(format!("{:?}", error.kind));
            transaction.error_summary = Some(error.summary.clone());
            if same_effective_state(&actual, &pre_state) {
                transaction.rollback.available = false;
                transaction.verification_result = Some("write_rejected_unchanged".into());
                transaction.completed_at = Some(crate::inspection::timestamp());
                transition(
                    &self.journal,
                    &mut transaction,
                    TransactionStatus::FailedAfterMutation,
                    "write_failed_without_state_change",
                )?;
                return Ok(owner_result(
                    OwnerOperationOutcome::CouldNotChange,
                    transaction,
                    Some(actual),
                    "Windows prevented this setting from being changed.",
                    OwnerResultClassification::WriteRejectedUnchanged,
                    None,
                ));
            }
            if is_safe_attempted_state(&actual, target) {
                return self.auto_restore_owner(
                    transaction,
                    "write_reported_failure",
                    "Windows reported a write error after the requested value appeared.",
                );
            }
            transaction.rollback.available = false;
            transaction.verification_result = Some("write_result_ambiguous".into());
            transaction.recovery_requirement = Some(
                "The actual setting differs from both the original and requested values.".into(),
            );
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RecoveryRequired,
                "ambiguous_write_failure",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::NeedsAttention,
                transaction,
                Some(actual),
                "The write failed and the current setting is ambiguous; Deslopper did not overwrite it.",
                OwnerResultClassification::WriteResultAmbiguous,
                None,
            ));
        }

        if !is_safe_attempted_state(&actual, target) {
            transaction.rollback.available = false;
            transaction.error_category = Some("verification_mismatch".into());
            if same_effective_state(&actual, &pre_state) {
                transaction.verification_result = Some("requested_state_not_applied".into());
                transaction.error_summary =
                    Some("The write returned success but the original setting remained.".into());
                transaction.completed_at = Some(crate::inspection::timestamp());
                transition(
                    &self.journal,
                    &mut transaction,
                    TransactionStatus::VerificationFailed,
                    "direct_verification_original_state_present",
                )?;
                return Ok(owner_result(
                    OwnerOperationOutcome::CouldNotChange,
                    transaction,
                    Some(actual),
                    format!(
                        "Windows did not apply the requested {label} setting; the original state remains."
                    ),
                    OwnerResultClassification::WriteRejectedUnchanged,
                    None,
                ));
            }
            transaction.error_summary = Some(
                "The post-write state differed from both the original and requested setting."
                    .into(),
            );
            transaction.verification_result = Some("verification_failed_changed".into());
            transaction.recovery_requirement = Some(
                "The current value could not be safely attributed to this attempt; no blind rollback was performed."
                    .into(),
            );
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RecoveryRequired,
                "direct_verification_ambiguous_state",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::NeedsAttention,
                transaction,
                Some(actual),
                format!(
                    "The {label} setting is in an unexpected state and was not overwritten again."
                ),
                OwnerResultClassification::VerificationFailedChanged,
                None,
            ));
        }

        self.fault(FaultPoint::BeforeVerification)?;
        self.fault(FaultPoint::DuringVerification)?;
        match handler(operation_id).verify(self.backend.as_ref(), target) {
            Ok(verified) => {
                set_post_state(&mut transaction, &verified)?;
                transaction.verification_result = Some("direct_representation_verified".into());
                self.journal
                    .save_transaction(&transaction)
                    .map_err(journal_error)?;
            }
            Err(error) => {
                transaction.error_category = Some(format!("{:?}", error.kind));
                transaction.error_summary = Some(error.summary);
                return self.auto_restore_owner(
                    transaction,
                    "direct_verification_failed",
                    "The direct registry verification failed.",
                );
            }
        }

        let detector_context = match detector_verification() {
            Ok(context) => context,
            Err(message) => {
                transaction.error_category = Some("detector_verification_failed".into());
                transaction.error_summary = Some(message);
                return self.auto_restore_owner(
                    transaction,
                    "detector_verification_failed",
                    &format!("The matching {label} detector could not complete."),
                );
            }
        };
        let detector_matches = detector_context.machine_id == context.machine_id
            && detector_context.windows_build == context.windows_build
            && detector_context.edition == context.edition
            && detector_context.authority_acceptable
            && detector_context.applicable
            && detector_context.detector_status == "successful"
            && detector_context.detector_current_enabled == Some(target.enabled());
        if !detector_matches {
            transaction.error_category = Some("detector_verification_failed".into());
            transaction.error_summary = Some(format!(
                "The {label} detector did not confirm the requested state."
            ));
            return self.auto_restore_owner(
                transaction,
                "detector_verification_failed",
                &format!("The matching {label} detector did not confirm the change."),
            );
        }
        let final_direct = handler(operation_id)
            .verify(self.backend.as_ref(), target)
            .map_err(handler_error)?;
        set_post_state(&mut transaction, &final_direct)?;
        transaction.verification_result = Some("direct_and_detector_verified".into());
        transaction.completed_at = Some(crate::inspection::timestamp());
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::RollbackAvailable,
            "owner_verification_complete",
        )?;
        self.fault(FaultPoint::BeforeTransactionCommit)?;
        Ok(owner_result(
            OwnerOperationOutcome::Changed,
            transaction,
            Some(final_direct),
            format!("{label} changed and the setting was verified."),
            OwnerResultClassification::ChangedVerified,
            Some("Explorer may refresh the visible taskbar asynchronously; Deslopper did not restart Explorer.".into()),
        ))
    }

    pub fn undo_owner_operation<F>(
        &self,
        transaction_id: &str,
        context: &BrokerContext,
        detector_verification: F,
    ) -> Result<OwnerOperationResult, BrokerError>
    where
        F: FnOnce() -> Result<BrokerContext, String>,
    {
        let _in_process = self.execution_lock.try_lock().map_err(|_| {
            BrokerError::new(
                "operation_in_progress",
                "Another Windows change is in progress.",
            )
        })?;
        let _cross_process = MutationProcessLock::acquire(self.journal.lock_path())
            .map_err(|message| BrokerError::new("cross_process_lock_unavailable", message))?;
        let mut transaction = self
            .journal
            .load_transaction(transaction_id)
            .map_err(journal_error)?
            .ok_or_else(|| BrokerError::new("invalid_transaction", "The change was not found."))?;
        self.validate_transaction_integrity(&transaction)?;
        let plan = self
            .journal
            .load_plan(&transaction.plan_id)
            .map_err(journal_error)?
            .ok_or_else(|| {
                BrokerError::new("tampered_transaction", "The owner intent is missing.")
            })?;
        if !is_owner_handler_version(&plan.handler_version)
            || !is_owner_operation(transaction.operation_id)
        {
            return Err(BrokerError::new(
                "not_owner_transaction",
                "This record was not created by the Owner Mode taskbar flow.",
            ));
        }
        validate_owner_context(transaction.operation_id, context)?;
        let label = owner_operation_label(transaction.operation_id);
        if transaction.status != TransactionStatus::RollbackAvailable
            || !transaction.rollback.available
        {
            return Err(BrokerError::new(
                "rollback_unavailable",
                "This change does not have a safe available Undo.",
            ));
        }
        if transaction.machine_id != context.machine_id
            || !crate::owner_scope::is_valid(&transaction.machine_id)
        {
            return Err(BrokerError::new(
                "owner_scope_mismatch",
                "Undo belongs to a different Windows machine or account.",
            ));
        }
        let pre_state = transaction.pre_state.clone().ok_or_else(|| {
            BrokerError::new(
                "rollback_unavailable",
                "The exact original state is missing.",
            )
        })?;
        let applied_state = transaction.post_state.clone().ok_or_else(|| {
            BrokerError::new(
                "rollback_unavailable",
                "The verified applied state is missing.",
            )
        })?;
        let current = handler(transaction.operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .map_err(handler_error)?;
        validate_captured_authority(&current)?;
        if conflicts_with_applied_state(&current, &applied_state) {
            transaction.rollback.conflict_detected = true;
            transaction.rollback.available = false;
            transaction.recovery_requirement = Some(
                "The setting changed after Deslopper applied it; normal Undo will not overwrite the newer value."
                    .into(),
            );
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RecoveryRequired,
                "owner_undo_conflict",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::NeedsAttention,
                transaction,
                Some(current.clone()),
                format!("{label} changed after Deslopper applied it, so Undo was not performed."),
                OwnerResultClassification::Conflict,
                Some(format!(
                    "Original: {:?}; applied: {:?}; current: {:?}",
                    pre_state.representation, applied_state.representation, current.representation
                )),
            ));
        }
        transaction.rollback.attempted_at = Some(crate::inspection::timestamp());
        self.fault(FaultPoint::BeforeRollback)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::RollingBack,
            "owner_undo",
        )?;
        let write_result =
            handler(transaction.operation_id).rollback(self.backend.as_ref(), &pre_state);
        self.fault(FaultPoint::ImmediatelyAfterRollbackWrite)?;
        let restored = handler(transaction.operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .map_err(handler_error)?;
        transaction.rollback_state_hash =
            Some(hash_serializable(&restored).map_err(serialization_error)?);
        transaction.rollback_state = Some(restored.clone());
        if let Err(error) = write_result {
            transaction.error_category = Some(format!("{:?}", error.kind));
            transaction.error_summary = Some(error.summary);
        }
        self.fault(FaultPoint::DuringRollbackVerification)?;
        if handler(transaction.operation_id)
            .verify_rollback(self.backend.as_ref(), &pre_state)
            .is_err()
        {
            transaction.rollback.available = false;
            transaction.rollback.result = Some("rollback_verification_failed".into());
            transaction.rollback.verification_result = Some("exact_pre_state_not_proven".into());
            transaction.recovery_requirement =
                Some("The exact original state was not restored.".into());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RollbackVerificationFailed,
                "owner_undo_verification_failed",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::NeedsAttention,
                transaction,
                Some(restored),
                format!("Undo could not prove that the original {label} setting was restored."),
                OwnerResultClassification::VerificationFailedChanged,
                None,
            ));
        }
        let detector_context = match detector_verification() {
            Ok(context) => context,
            Err(message) => {
                transaction.rollback.available = false;
                transaction.rollback.result = Some("restored_detector_unavailable".into());
                transaction.rollback.verification_result = Some("detector_unavailable".into());
                transaction.error_category = Some("detector_verification_failed".into());
                transaction.error_summary = Some(message);
                transaction.recovery_requirement = Some(
                    "The registry was restored but detector verification did not complete.".into(),
                );
                transition(
                    &self.journal,
                    &mut transaction,
                    TransactionStatus::RollbackVerificationFailed,
                    "owner_undo_detector_unavailable",
                )?;
                return Ok(owner_result(
                    OwnerOperationOutcome::NeedsAttention,
                    transaction,
                    Some(restored),
                    "The original registry representation was restored, but detector verification did not complete.",
                    OwnerResultClassification::VerificationFailedChanged,
                    None,
                ));
            }
        };
        let expected_detector = if pre_state.effective_state_known {
            Some(pre_state.effective_enabled)
        } else {
            None
        };
        let detector_matches = detector_context.machine_id == context.machine_id
            && detector_context.authority_acceptable
            && detector_context.applicable
            && (expected_detector.is_none()
                || detector_context.detector_current_enabled == expected_detector);
        if !detector_matches {
            transaction.rollback.available = false;
            transaction.rollback.result = Some("restored_detector_disagreed".into());
            transaction.rollback.verification_result = Some("detector_disagreed".into());
            transaction.recovery_requirement = Some(format!(
                "The registry was restored but the {label} detector disagreed."
            ));
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RollbackVerificationFailed,
                "owner_undo_detector_failed",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::NeedsAttention,
                transaction,
                Some(restored),
                "The original registry representation was restored, but detector verification needs attention.",
                OwnerResultClassification::VerificationFailedChanged,
                None,
            ));
        }
        transaction.rollback.available = false;
        transaction.rollback.complete = true;
        transaction.rollback.result = Some("rolled_back".into());
        transaction.rollback.verification_result = Some("exact_pre_state_restored".into());
        transaction.completed_at = Some(crate::inspection::timestamp());
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::RolledBack,
            "owner_undo_verified",
        )?;
        Ok(owner_result(
            OwnerOperationOutcome::Restored,
            transaction,
            Some(restored),
            format!("The exact original {label} setting was restored."),
            OwnerResultClassification::Restored,
            Some("Explorer may refresh the visible taskbar asynchronously; Deslopper did not restart Explorer.".into()),
        ))
    }

    fn auto_restore_owner(
        &self,
        mut transaction: MutationTransaction,
        failure_category: &str,
        failure_summary: &str,
    ) -> Result<OwnerOperationResult, BrokerError> {
        transaction.error_category = Some(failure_category.into());
        transaction.error_summary = Some(failure_summary.into());
        let pre_state = transaction.pre_state.clone().ok_or_else(|| {
            BrokerError::new(
                "rollback_unavailable",
                "The exact original state is missing.",
            )
        })?;
        let target = transaction.target_state;
        let current = match handler(transaction.operation_id)
            .inspect_pre_state(self.backend.as_ref())
        {
            Ok(current) => current,
            Err(error) => {
                transaction.rollback.available = false;
                transaction.recovery_requirement = Some(
                    "The current state could not be attributed safely after failed verification."
                        .into(),
                );
                transaction.error_summary = Some(error.summary);
                transition(
                    &self.journal,
                    &mut transaction,
                    TransactionStatus::RecoveryRequired,
                    "automatic_rollback_not_safe",
                )?;
                return Ok(owner_result(
                    OwnerOperationOutcome::NeedsAttention,
                    transaction,
                    None,
                    "Verification failed and Deslopper could not safely determine whether to restore.",
                    OwnerResultClassification::WriteResultAmbiguous,
                    None,
                ));
            }
        };
        set_post_state(&mut transaction, &current)?;
        if !is_safe_attempted_state(&current, target) {
            transaction.rollback.available = false;
            transaction.recovery_requirement = Some(
                "The current state was not the exact attempted state; no blind rollback was performed."
                    .into(),
            );
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RecoveryRequired,
                "automatic_rollback_not_attributable",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::NeedsAttention,
                transaction,
                Some(current),
                "Verification failed and the current value could not be safely attributed to this attempt.",
                OwnerResultClassification::WriteResultAmbiguous,
                None,
            ));
        }
        transaction.rollback.attempted_at = Some(crate::inspection::timestamp());
        self.fault(FaultPoint::BeforeRollback)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::RollingBack,
            "automatic_exact_rollback",
        )?;
        let write_result =
            handler(transaction.operation_id).rollback(self.backend.as_ref(), &pre_state);
        self.fault(FaultPoint::ImmediatelyAfterRollbackWrite)?;
        let restored = match handler(transaction.operation_id)
            .inspect_pre_state(self.backend.as_ref())
        {
            Ok(restored) => restored,
            Err(error) => {
                transaction.rollback.available = false;
                transaction.rollback.result = Some("rollback_state_unreadable".into());
                transaction.error_summary = Some(error.summary);
                transaction.recovery_requirement =
                    Some("Automatic rollback was attempted but could not be verified.".into());
                transition(
                    &self.journal,
                    &mut transaction,
                    TransactionStatus::RollbackVerificationFailed,
                    "automatic_rollback_unreadable",
                )?;
                return Ok(owner_result(
                    OwnerOperationOutcome::NeedsAttention,
                    transaction,
                    None,
                    "Automatic rollback was attempted but the restored state could not be read.",
                    OwnerResultClassification::VerificationFailedChanged,
                    None,
                ));
            }
        };
        transaction.rollback_state_hash =
            Some(hash_serializable(&restored).map_err(serialization_error)?);
        transaction.rollback_state = Some(restored.clone());
        self.fault(FaultPoint::DuringRollbackVerification)?;
        let exact = handler(transaction.operation_id)
            .verify_rollback(self.backend.as_ref(), &pre_state)
            .is_ok();
        if exact {
            transaction.rollback.available = false;
            transaction.rollback.complete = true;
            transaction.rollback.result = Some(
                if write_result.is_ok() {
                    "automatic_rollback_succeeded"
                } else {
                    "automatic_rollback_verified_after_write_error"
                }
                .into(),
            );
            transaction.rollback.verification_result = Some("exact_pre_state_restored".into());
            transaction.verification_result = Some("verification_failed_rolled_back".into());
            transaction.completed_at = Some(crate::inspection::timestamp());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RolledBack,
                "automatic_rollback_verified",
            )?;
            return Ok(owner_result(
                OwnerOperationOutcome::Restored,
                transaction,
                Some(restored),
                "The change could not be verified, so Deslopper restored the exact original setting.",
                OwnerResultClassification::VerificationFailedRolledBack,
                None,
            ));
        }
        transaction.rollback.available = false;
        transaction.rollback.complete = false;
        transaction.rollback.result = Some("automatic_rollback_failed".into());
        transaction.rollback.verification_result = Some("exact_pre_state_not_proven".into());
        transaction.recovery_requirement =
            Some("Automatic rollback did not restore the exact original setting.".into());
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::RollbackVerificationFailed,
            "automatic_rollback_verification_failed",
        )?;
        Ok(owner_result(
            OwnerOperationOutcome::NeedsAttention,
            transaction,
            Some(restored),
            "The change failed verification and automatic rollback also needs attention.",
            OwnerResultClassification::VerificationFailedChanged,
            None,
        ))
    }

    pub fn owner_history(
        &self,
        owner_scope: &str,
    ) -> Result<Vec<MutationTransaction>, BrokerError> {
        if !crate::owner_scope::is_valid(owner_scope) {
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        for transaction in self.journal.history().map_err(journal_error)? {
            if transaction.machine_id != owner_scope
                || !is_owner_operation(transaction.operation_id)
            {
                continue;
            }
            let Some(plan) = self
                .journal
                .load_plan(&transaction.plan_id)
                .map_err(journal_error)?
            else {
                continue;
            };
            if is_owner_handler_version(&plan.handler_version) {
                self.validate_transaction_integrity(&transaction)?;
                result.push(transaction);
            }
        }
        Ok(result)
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn operation_options(&self, context: &BrokerContext) -> Vec<OperationOption> {
        if !self.live_validation.available
            || !self.live_validation.scope_is_valid()
            || self
                .live_validation
                .approval_expires_at_epoch_ms
                .is_none_or(|expiration| expiration <= now_millis() as u64)
        {
            return Vec::new();
        }
        self.live_validation
            .approved_operation_scopes
            .iter()
            .map(|scope| scope.operation_id)
            .map(|operation_id| {
                let mut definition = operation_definition(operation_id);
                definition.supported_targets = self.live_validation.allowed_targets(operation_id);
                let maturity = self.maturity(operation_id);
                let eligibility = validate_context(context, &definition);
                let state = handler(operation_id).inspect_pre_state(self.backend.as_ref());
                let state_authority = state
                    .as_ref()
                    .map_err(|error| error.summary.clone())
                    .and_then(|state| {
                        validate_captured_authority(state).map_err(|error| error.message)
                    });
                let eligible = maturity.allows_internal_alpha()
                    && eligibility.is_ok()
                    && state.is_ok()
                    && state_authority.is_ok()
                    && self.live_validation.available;
                let reason = if !maturity.allows_internal_alpha() {
                    "Live validation rejected this handler.".into()
                } else if !self.live_validation.available {
                    self.live_validation.reason.clone()
                } else {
                    eligibility
                        .err()
                        .map(|error| error.message)
                        .or_else(|| state.as_ref().err().map(|error| error.summary.clone()))
                        .or_else(|| state_authority.err())
                        .unwrap_or_else(|| "A fresh reviewed plan may be generated.".into())
                };
                OperationOption {
                    definition,
                    current_state: state.ok(),
                    validation_maturity: maturity,
                    validation_label: maturity.label(),
                    eligible,
                    reason,
                }
            })
            .collect()
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn generate_plan(
        &self,
        request: &PlanRequest,
        context: &BrokerContext,
    ) -> Result<IssuedPlan, BrokerError> {
        self.require_gate()?;
        self.require_authorized(request.operation_id, request.target)?;
        self.require_target_eligibility(context)?;
        let approval_class = self.approval_class()?;
        let _cross_process = MutationProcessLock::acquire(self.journal.lock_path())
            .map_err(|message| BrokerError::new("cross_process_lock_unavailable", message))?;
        let usage = self
            .journal
            .approval_usage(&approval_class)
            .map_err(journal_error)?;
        if usage.plans >= self.live_validation.maximum_plans {
            return Err(BrokerError::new(
                "approval_plan_limit_exhausted",
                "The scoped approval has no remaining plan allowance.",
            ));
        }
        if request.source_inspection_id != context.inspection_id {
            return Err(BrokerError::new(
                "stale_inspection",
                "The selected inspection is no longer the latest inspected machine state.",
            ));
        }
        let definition = operation_definition(request.operation_id);
        if !self.maturity(request.operation_id).allows_internal_alpha() {
            return Err(BrokerError::new(
                "handler_rejected",
                "Live validation rejected this operation handler.",
            ));
        }
        if definition.subject_id != request.operation_id.subject() {
            return Err(BrokerError::new(
                "component_operation_mismatch",
                "The closed operation-to-component mapping did not match.",
            ));
        }
        validate_context(context, &definition)?;
        handler(request.operation_id)
            .validate_target(request.target)
            .map_err(handler_error)?;
        let current_state = handler(request.operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .map_err(handler_error)?;
        validate_captured_authority(&current_state)?;
        let now = now_millis();
        let sequence = BROKER_SEQUENCE.fetch_add(1, Ordering::SeqCst);
        let plan_id = format!("mutation-plan-{now}-{sequence}");
        let nonce = hash_text(&format!(
            "{plan_id}:{}:{}:{}",
            std::process::id(),
            context.evidence_fingerprint,
            sequence
        ));
        let mut plan = MutationPlan {
            plan_id: plan_id.clone(),
            machine_id: context.machine_id.clone(),
            source_inspection_id: context.inspection_id.clone(),
            source_observation_id: context.source_observation_id.clone(),
            subject_id: definition.subject_id,
            operation_id: definition.operation_id,
            current_state,
            target_state: request.target,
            authority: context.authority.clone(),
            applicability: context.applicability.clone(),
            windows_build: context.windows_build,
            edition: context.edition.clone(),
            evidence_fingerprint: context.evidence_fingerprint.clone(),
            generated_at: now.to_string(),
            expires_at: (now + 300_000).to_string(),
            approval_class,
            rollback_method: definition.rollback_method.into(),
            documentation: definition
                .documentation
                .iter()
                .map(|source| (*source).into())
                .collect(),
            required_privilege: definition.required_privilege.into(),
            expected_side_effects: definition
                .side_effects
                .iter()
                .map(|effect| (*effect).into())
                .collect(),
            restart_requirement: definition.restart_requirement.into(),
            handler_version: HANDLER_VERSION.into(),
            desired_state_revision_id: context.desired_state_revision_id,
            automatic_remediation_eligible: false,
            approval_nonce_hash: hash_text(&nonce),
            consumed_at: None,
            plan_hash: String::new(),
        };
        plan.refresh_hash()
            .map_err(|error| BrokerError::new("plan_serialization_failed", error.to_string()))?;
        self.journal.save_plan(&plan).map_err(journal_error)?;
        let mut transaction = new_transaction(&plan);
        self.journal
            .save_transaction(&transaction)
            .map_err(journal_error)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::AwaitingApproval,
            "await_approval",
        )?;
        Ok(IssuedPlan {
            confirmation_text: confirmation_text(request.operation_id, request.target),
            approval_phrase: approval_phrase(request.operation_id),
            proposed_representation: super::plan::CapturedRepresentation::Dword(u32::from(
                request.target.enabled(),
            )),
            plan,
            approval_nonce: nonce,
        })
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn execute(
        &self,
        request: &super::request::ApprovalRequest,
        context: &BrokerContext,
    ) -> Result<MutationTransaction, BrokerError> {
        self.require_gate()?;
        self.require_target_eligibility(context)?;
        if !request.acknowledged {
            return Err(BrokerError::new(
                "approval_required",
                "The exact reviewed operation must be explicitly approved.",
            ));
        }
        let _in_process = self.execution_lock.try_lock().map_err(|_| {
            BrokerError::new(
                "operation_in_progress",
                "Another mutation transaction is active.",
            )
        })?;
        let _cross_process = MutationProcessLock::acquire(self.journal.lock_path())
            .map_err(|message| BrokerError::new("cross_process_lock_unavailable", message))?;
        let plan = self
            .journal
            .load_plan(&request.plan_id)
            .map_err(journal_error)?
            .ok_or_else(|| BrokerError::new("invalid_plan_id", "The plan does not exist."))?;
        if !plan.hash_is_valid() {
            return Err(BrokerError::new(
                "tampered_plan",
                "The persisted plan hash is invalid.",
            ));
        }
        if plan.consumed_at.is_some() {
            return Err(BrokerError::new(
                "consumed_plan",
                "The plan has already been consumed and cannot be replayed.",
            ));
        }
        if plan.is_expired(now_millis()) {
            return Err(BrokerError::new(
                "expired_plan",
                "The reviewed plan expired; generate and approve a fresh plan.",
            ));
        }
        if hash_text(&request.approval_nonce) != plan.approval_nonce_hash {
            return Err(BrokerError::new(
                "invalid_approval_nonce",
                "The broker-issued one-time approval nonce is invalid.",
            ));
        }

        self.require_authorized(plan.operation_id, plan.target_state)?;
        let approval_class = self.approval_class()?;
        if plan.approval_class != approval_class {
            return Err(BrokerError::new(
                "approval_binding_mismatch",
                "The plan is not bound to the active scoped approval.",
            ));
        }
        let usage = self
            .journal
            .approval_usage(&approval_class)
            .map_err(journal_error)?;
        if usage.executions >= self.live_validation.maximum_executions {
            return Err(BrokerError::new(
                "approval_execution_limit_exhausted",
                "The scoped approval has no remaining execution allowance.",
            ));
        }

        let mut transaction = self
            .journal
            .load_transaction(&format!("transaction-{}", plan.plan_id))
            .map_err(journal_error)?
            .ok_or_else(|| {
                BrokerError::new("transaction_missing", "The audit transaction is missing.")
            })?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::Validating,
            "validate_plan",
        )?;
        validate_plan_context(&plan, context)?;
        validate_context(context, &operation_definition(plan.operation_id))?;
        let approved_at = crate::inspection::timestamp();
        transaction.approved_at = Some(approved_at);
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::Approved,
            "approval",
        )?;
        self.fault(FaultPoint::BeforePreStateCapture)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::CapturingPreState,
            "capture_pre_state",
        )?;
        let pre_state = handler(plan.operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .map_err(handler_error)?;
        if let Err(error) = validate_captured_authority(&pre_state) {
            transaction.error_category = Some("changed_authority".into());
            transaction.error_summary = Some(error.message.clone());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::FailedBeforeMutation,
                "failed_before_mutation",
            )?;
            return Err(error);
        }
        if pre_state.representation != plan.current_state.representation
            || pre_state.effective_enabled != plan.current_state.effective_enabled
            || pre_state.authority != plan.current_state.authority
            || pre_state.confidence != plan.current_state.confidence
        {
            transaction.error_category = Some("changed_source_state".into());
            transaction.error_summary =
                Some("The setting changed after review; no mutation was attempted.".into());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::FailedBeforeMutation,
                "failed_before_mutation",
            )?;
            return Err(BrokerError::new(
                "changed_source_state",
                "The actual setting no longer matches the reviewed source state.",
            ));
        }
        transaction.pre_state_hash =
            Some(hash_serializable(&pre_state).map_err(serialization_error)?);
        transaction.pre_state = Some(pre_state.clone());
        self.journal
            .save_transaction(&transaction)
            .map_err(journal_error)?;
        self.fault(FaultPoint::AfterPreStateCapture)?;
        let consumed_at = crate::inspection::timestamp();
        self.journal
            .consume_plan(&plan.plan_id, &consumed_at)
            .map_err(journal_error)?;
        transaction.started_at = Some(consumed_at);
        self.journal
            .save_transaction(&transaction)
            .map_err(journal_error)?;

        let write_required = pre_state.representation != expected(plan.target_state)
            || !pre_state.effective_state_known
            || pre_state.effective_enabled != plan.target_state.enabled();
        if !write_required {
            transaction.post_state_hash = transaction.pre_state_hash.clone();
            transaction.post_state = Some(pre_state);
            transaction.verification_result = Some("already_compliant".into());
            transaction.rollback.available = false;
            transaction.rollback.complete = false;
            transaction.completed_at = Some(crate::inspection::timestamp());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::NoChangeNeeded,
                "already_compliant",
            )?;
            return Ok(transaction);
        }
        transaction.rollback.available = true;
        transaction.rollback.complete = false;
        self.journal
            .save_transaction(&transaction)
            .map_err(journal_error)?;

        self.require_gate()?;
        self.require_authorized(plan.operation_id, plan.target_state)?;
        self.require_target_eligibility(context)?;
        self.fault(FaultPoint::BeforeWrite)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::Applying,
            "apply",
        )?;
        if let Err(error) =
            handler(plan.operation_id).apply(self.backend.as_ref(), plan.target_state)
        {
            fail_after_mutation(&self.journal, &mut transaction, &error)?;
            return Err(handler_error(error));
        }
        self.fault(FaultPoint::ImmediatelyAfterWrite)?;
        self.fault(FaultPoint::BeforeVerification)?;
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::Verifying,
            "verify",
        )?;
        self.fault(FaultPoint::DuringVerification)?;
        let post_state =
            match handler(plan.operation_id).verify(self.backend.as_ref(), plan.target_state) {
                Ok(state) => state,
                Err(error) => {
                    transaction.verification_result = Some("verification_mismatch".into());
                    transaction.error_category = Some(format!("{:?}", error.kind));
                    transaction.error_summary = Some(error.summary.clone());
                    transaction.recovery_requirement =
                        Some("Review or exact rollback required.".into());
                    transition(
                        &self.journal,
                        &mut transaction,
                        TransactionStatus::VerificationFailed,
                        "verification_failed",
                    )?;
                    return Err(handler_error(error));
                }
            };
        transaction.post_state_hash =
            Some(hash_serializable(&post_state).map_err(serialization_error)?);
        transaction.post_state = Some(post_state.clone());
        transaction.verification_result = Some(
            classify_apply(&pre_state, &post_state, plan.target_state)
                .map_err(handler_error)?
                .into(),
        );
        transaction.completed_at = Some(crate::inspection::timestamp());
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::Applied,
            "handler_verified",
        )?;
        self.fault(FaultPoint::BeforeTransactionCommit)?;
        Ok(transaction)
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn rollback(
        &self,
        request: &RollbackRequest,
        context: &BrokerContext,
    ) -> Result<MutationTransaction, BrokerError> {
        self.require_rollback_gate()?;
        if !request.acknowledged {
            return Err(BrokerError::new(
                "rollback_approval_required",
                "Exact rollback requires explicit approval.",
            ));
        }
        let _in_process = self.execution_lock.try_lock().map_err(|_| {
            BrokerError::new(
                "operation_in_progress",
                "Another mutation transaction is active.",
            )
        })?;
        let _cross_process = MutationProcessLock::acquire(self.journal.lock_path())
            .map_err(|message| BrokerError::new("cross_process_lock_unavailable", message))?;
        let mut transaction = self
            .journal
            .load_transaction(&request.transaction_id)
            .map_err(journal_error)?
            .ok_or_else(|| {
                BrokerError::new("invalid_transaction", "The transaction does not exist.")
            })?;
        self.validate_transaction_integrity(&transaction)?;
        if !matches!(
            transaction.status,
            TransactionStatus::RollbackAvailable
                | TransactionStatus::VerificationFailed
                | TransactionStatus::RecoveryRequired
        ) {
            return Err(BrokerError::new(
                "rollback_unavailable",
                "The transaction has not reached a state that permits reviewed rollback.",
            ));
        }
        if transaction.machine_id != context.machine_id
            || transaction.windows_build != context.windows_build
            || transaction.edition != context.edition
        {
            return Err(BrokerError::new(
                "rollback_context_mismatch",
                "Machine identity, build, or edition changed after the transaction.",
            ));
        }
        if !context.authority_acceptable {
            return Err(BrokerError::new(
                "external_authority",
                "Rollback will not fight a newly effective external policy.",
            ));
        }
        let pre_state = transaction.pre_state.clone().ok_or_else(|| {
            BrokerError::new("rollback_unavailable", "No exact pre-state was captured.")
        })?;
        let applied_state = transaction.post_state.clone().ok_or_else(|| {
            BrokerError::new("rollback_unavailable", "No verified applied state exists.")
        })?;
        let current = handler(transaction.operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .map_err(handler_error)?;
        validate_captured_authority(&current)?;
        let conflict = conflicts_with_applied_state(&current, &applied_state);
        transaction.rollback.conflict_detected = conflict;
        if conflict && !request.allow_conflict {
            self.journal
                .save_transaction(&transaction)
                .map_err(journal_error)?;
            return Err(BrokerError::new(
                "rollback_conflict",
                "The setting changed after Deslopper applied it; explicit conflict approval is required.",
            ));
        }
        self.fault(FaultPoint::BeforeRollback)?;
        transaction.rollback.attempted_at = Some(crate::inspection::timestamp());
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::RollingBack,
            "rollback",
        )?;
        if let Err(error) =
            handler(transaction.operation_id).rollback(self.backend.as_ref(), &pre_state)
        {
            transaction.rollback.result = Some("write_failed".into());
            transaction.error_category = Some(format!("{:?}", error.kind));
            transaction.error_summary = Some(error.summary.clone());
            transaction.recovery_requirement = Some("Manual recovery may be required.".into());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RollbackVerificationFailed,
                "rollback_write_failed",
            )?;
            return Err(handler_error(error));
        }
        self.fault(FaultPoint::ImmediatelyAfterRollbackWrite)?;
        self.fault(FaultPoint::DuringRollbackVerification)?;
        let restored = match handler(transaction.operation_id)
            .verify_rollback(self.backend.as_ref(), &pre_state)
        {
            Ok(restored) => restored,
            Err(error) => {
                transaction.rollback.result = Some("write_succeeded_verification_failed".into());
                transaction.rollback.verification_result =
                    Some("exact_pre_state_not_proven".into());
                transaction.error_category = Some(format!("{:?}", error.kind));
                transaction.error_summary = Some(error.summary.clone());
                transaction.recovery_requirement = Some("Manual recovery may be required.".into());
                transition(
                    &self.journal,
                    &mut transaction,
                    TransactionStatus::RollbackVerificationFailed,
                    "rollback_verification_failed",
                )?;
                return Err(handler_error(error));
            }
        };
        transaction.rollback_state_hash =
            Some(hash_serializable(&restored).map_err(serialization_error)?);
        transaction.rollback_state = Some(restored);
        transaction.rollback.result = Some("rolled_back".into());
        transaction.rollback.verification_result = Some("exact_pre_state_restored".into());
        transaction.completed_at = Some(crate::inspection::timestamp());
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::RolledBack,
            "rollback_verified",
        )?;
        Ok(transaction)
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn history(&self) -> Result<Vec<MutationTransaction>, BrokerError> {
        let transactions = self.journal.history().map_err(journal_error)?;
        for transaction in &transactions {
            self.validate_transaction_integrity(transaction)?;
        }
        Ok(transactions)
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn export_live_validation_evidence(
        &self,
        request: &EvidenceExportRequest,
    ) -> Result<LiveEvidenceBundle, BrokerError> {
        self.require_gate()?;
        let transaction = self
            .journal
            .load_transaction(&request.transaction_id)
            .map_err(journal_error)?
            .ok_or_else(|| {
                BrokerError::new("invalid_transaction", "The transaction does not exist.")
            })?;
        self.validate_transaction_integrity(&transaction)?;
        let manifest: ValidationScenarioManifest = load_local_manifest().ok_or_else(|| {
            BrokerError::new(
                "validation_manifest_missing",
                "The fixed local validation manifest is unavailable.",
            )
        })?;
        let bundle = LiveEvidenceBundle::from_transaction(
            request,
            &transaction,
            &self.live_validation,
            &manifest,
        )
        .map_err(|message| BrokerError::new("invalid_evidence", message))?;
        let output_directory = local_validation_directory().join("live-evidence");
        std::fs::create_dir_all(&output_directory).map_err(|error| {
            BrokerError::new(
                "evidence_write_failed",
                format!("Could not create the fixed evidence directory: {error}"),
            )
        })?;
        let output_path = output_directory.join(format!("{}.json", bundle.identity()));
        let raw = serde_json::to_vec_pretty(&bundle).map_err(|error| {
            BrokerError::new("evidence_serialization_failed", error.to_string())
        })?;
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&output_path)
            .map_err(|_| {
                BrokerError::new(
                    "duplicate_evidence",
                    "This evidence identity already exists and was not overwritten.",
                )
            })?;
        use std::io::Write;
        file.write_all(&raw)
            .map_err(|error| BrokerError::new("evidence_write_failed", error.to_string()))?;
        file.sync_all()
            .map_err(|error| BrokerError::new("evidence_write_failed", error.to_string()))?;
        Ok(bundle)
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn cancel_before_mutation(
        &self,
        plan_id: &str,
    ) -> Result<MutationTransaction, BrokerError> {
        self.require_gate()?;
        let plan = self
            .journal
            .load_plan(plan_id)
            .map_err(journal_error)?
            .ok_or_else(|| BrokerError::new("invalid_plan_id", "The plan does not exist."))?;
        if plan.consumed_at.is_some() {
            return Err(BrokerError::new(
                "consumed_plan",
                "A consumed plan cannot be cancelled as a pre-mutation request.",
            ));
        }
        let now = crate::inspection::timestamp();
        self.journal
            .consume_plan(plan_id, &now)
            .map_err(journal_error)?;
        let mut transaction = self
            .journal
            .load_transaction(&format!("transaction-{plan_id}"))
            .map_err(journal_error)?
            .ok_or_else(|| {
                BrokerError::new("transaction_missing", "The audit transaction is missing.")
            })?;
        transaction.completed_at = Some(now);
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::CancelledBeforeMutation,
            "cancelled_before_mutation",
        )?;
        Ok(transaction)
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn complete_effective_verification(
        &self,
        mut transaction: MutationTransaction,
        context: &BrokerContext,
    ) -> Result<MutationTransaction, BrokerError> {
        self.validate_transaction_integrity(&transaction)?;
        if transaction.status == TransactionStatus::NoChangeNeeded {
            return Ok(transaction);
        }
        let handler_context_valid = handler(transaction.operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .is_ok_and(|current| {
                validate_captured_authority(&current).is_ok()
                    && current.effective_enabled == transaction.target_state.enabled()
                    && transaction.post_state.as_ref().is_some_and(|post_state| {
                        post_state.representation == current.representation
                            && post_state.effective_enabled == current.effective_enabled
                    })
            });
        let context_valid = transaction.machine_id == context.machine_id
            && transaction.windows_build == context.windows_build
            && transaction.edition == context.edition
            && context.authority_acceptable
            && context.confidence_sufficient
            && context.applicable
            && handler_context_valid;
        if !context_valid {
            transaction.verification_result = Some("effective_context_changed".into());
            transaction.error_category = Some("post_apply_context_changed".into());
            transaction.error_summary = Some(
                "The post-apply inspection found changed authority, applicability, or platform context."
                    .into(),
            );
            transaction.recovery_requirement =
                Some("Review the fresh inspection before exact rollback.".into());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::VerificationFailed,
                "effective_verification_failed",
            )?;
            return Ok(transaction);
        }
        transition(
            &self.journal,
            &mut transaction,
            TransactionStatus::RollbackAvailable,
            "effective_reinspection_verified",
        )?;
        Ok(transaction)
    }

    #[cfg(feature = "mutation-alpha")]
    pub fn complete_rollback_effective_verification(
        &self,
        mut transaction: MutationTransaction,
        context: &BrokerContext,
    ) -> Result<MutationTransaction, BrokerError> {
        self.validate_transaction_integrity(&transaction)?;
        let handler_context_valid = handler(transaction.operation_id)
            .inspect_pre_state(self.backend.as_ref())
            .is_ok_and(|current| {
                validate_captured_authority(&current).is_ok()
                    && transaction.pre_state.as_ref().is_some_and(|pre_state| {
                        pre_state.representation == current.representation
                            && pre_state.effective_enabled == current.effective_enabled
                    })
            });
        let context_valid = transaction.machine_id == context.machine_id
            && transaction.windows_build == context.windows_build
            && transaction.edition == context.edition
            && context.authority_acceptable
            && context.confidence_sufficient
            && context.applicable
            && handler_context_valid;
        if !context_valid {
            transaction.rollback.verification_result =
                Some("effective_context_changed_after_rollback".into());
            transaction.error_category = Some("post_rollback_context_changed".into());
            transaction.error_summary = Some(
                "The post-rollback inspection found changed authority, applicability, or platform context."
                    .into(),
            );
            transaction.recovery_requirement =
                Some("Review the fresh inspection before taking further action.".into());
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RollbackVerificationFailed,
                "effective_rollback_verification_failed",
            )?;
        }
        Ok(transaction)
    }

    pub fn recover_interrupted(
        &self,
        context: &BrokerContext,
    ) -> Result<Vec<MutationTransaction>, BrokerError> {
        let mut recovered = Vec::new();
        for mut transaction in self.journal.interrupted().map_err(journal_error)? {
            self.validate_transaction_integrity(&transaction)?;
            if transaction.machine_id != context.machine_id {
                transaction.recovery_requirement = Some("different_machine_identity".into());
            } else {
                let current = handler(transaction.operation_id)
                    .inspect_pre_state(self.backend.as_ref())
                    .map_err(handler_error)?;
                transaction.recovery_requirement = Some(
                    if transaction
                        .post_state
                        .as_ref()
                        .is_some_and(|state| state.representation == current.representation)
                    {
                        "target_state_present_verify_before_action"
                    } else if transaction
                        .pre_state
                        .as_ref()
                        .is_some_and(|state| state.representation == current.representation)
                    {
                        "pre_state_present_no_repeat_apply"
                    } else {
                        "unexpected_or_uncertain_state_manual_review"
                    }
                    .into(),
                );
            }
            transition(
                &self.journal,
                &mut transaction,
                TransactionStatus::RecoveryRequired,
                "startup_recovery_required",
            )?;
            recovered.push(transaction);
        }
        Ok(recovered)
    }

    fn validate_transaction_integrity(
        &self,
        transaction: &MutationTransaction,
    ) -> Result<(), BrokerError> {
        let plan = self
            .journal
            .load_plan(&transaction.plan_id)
            .map_err(journal_error)?
            .ok_or_else(|| {
                BrokerError::new(
                    "tampered_transaction",
                    "The transaction's reviewed plan is missing.",
                )
            })?;
        let binding_matches = plan.hash_is_valid()
            && transaction.transaction_id == format!("transaction-{}", plan.plan_id)
            && transaction.plan_hash == plan.plan_hash
            && transaction.machine_id == plan.machine_id
            && transaction.subject_id == plan.subject_id
            && transaction.operation_id == plan.operation_id
            && transaction.source_inspection_id == plan.source_inspection_id
            && transaction.source_observation_id == plan.source_observation_id
            && transaction.windows_build == plan.windows_build
            && transaction.edition == plan.edition
            && transaction.target_state == plan.target_state
            && operation_definition(transaction.operation_id).subject_id == transaction.subject_id;
        let capture_matches = capture_hash_matches(
            transaction.pre_state.as_ref(),
            transaction.pre_state_hash.as_deref(),
        ) && capture_hash_matches(
            transaction.post_state.as_ref(),
            transaction.post_state_hash.as_deref(),
        ) && capture_hash_matches(
            transaction.rollback_state.as_ref(),
            transaction.rollback_state_hash.as_deref(),
        );
        if !binding_matches || !capture_matches {
            return Err(BrokerError::new(
                "tampered_transaction",
                "The transaction binding or captured-state integrity hash is invalid.",
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn inject_fault(&self, point: Option<FaultPoint>) {
        if let Ok(mut current) = self.fault_point.lock() {
            *current = point;
        }
    }

    #[cfg(feature = "mutation-alpha")]
    fn maturity(&self, operation_id: MutationOperationId) -> ValidationMaturity {
        self.validation_maturities
            .lock()
            .map(|values| values[operation_index(operation_id)])
            .unwrap_or(ValidationMaturity::Rejected)
    }

    #[cfg(test)]
    #[cfg(feature = "mutation-alpha")]
    pub fn set_validation_maturity(
        &self,
        operation_id: MutationOperationId,
        maturity: ValidationMaturity,
    ) {
        if let Ok(mut values) = self.validation_maturities.lock() {
            values[operation_index(operation_id)] = maturity;
        }
    }

    #[cfg(feature = "mutation-alpha")]
    fn require_gate(&self) -> Result<(), BrokerError> {
        let gate = self.gate_status();
        if gate.available {
            Ok(())
        } else {
            Err(BrokerError::new("mutation_alpha_disabled", gate.reason))
        }
    }

    #[cfg(feature = "mutation-alpha")]
    fn require_rollback_gate(&self) -> Result<(), BrokerError> {
        if cfg!(debug_assertions)
            && self.command_line_opt_in
            && self.live_validation.rollback_environment_available
        {
            Ok(())
        } else {
            Err(BrokerError::new(
                "rollback_environment_disabled",
                "The transaction-bound rollback environment is unavailable.",
            ))
        }
    }

    #[cfg(feature = "mutation-alpha")]
    fn approval_class(&self) -> Result<String, BrokerError> {
        self.live_validation
            .approval_id
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("scoped:{value}"))
            .ok_or_else(|| {
                BrokerError::new(
                    "approval_scope_missing",
                    "No validated scoped approval identity is available.",
                )
            })
    }

    #[cfg(feature = "mutation-alpha")]
    fn require_authorized(
        &self,
        operation_id: MutationOperationId,
        target: MutationTarget,
    ) -> Result<(), BrokerError> {
        if self.live_validation.allows(operation_id, target)
            && local_scoped_approval_allows(&self.live_validation, operation_id, target)
        {
            Ok(())
        } else {
            Err(BrokerError::new(
                "operation_target_not_approved",
                "The requested operation and target are not explicitly authorized by the scoped approval.",
            ))
        }
    }

    #[cfg(feature = "mutation-alpha")]
    fn require_target_eligibility(&self, context: &BrokerContext) -> Result<(), BrokerError> {
        let Some(target_type) = self.live_validation.approved_target_type else {
            return Err(BrokerError::new(
                "target_type_not_approved",
                "The scoped approval has no bound validation target type.",
            ));
        };
        let Some(expected) = self.live_validation.approved_target_management.as_ref() else {
            return Err(BrokerError::new(
                "target_management_not_approved",
                "The scoped approval has no bound target management state.",
            ));
        };
        let expected_workplace = expected.workplace_joined.unwrap_or(expected.entra_joined);
        let matches = context.domain_joined == Some(expected.domain_joined)
            && context.entra_joined == Some(expected.entra_joined)
            && context.workplace_joined == Some(expected_workplace)
            && context.mdm_enrolled == Some(expected.mdm_enrolled);
        let strict_physical = target_type != ValidationTargetType::PhysicalLaptop
            || (context.domain_joined == Some(false)
                && context.entra_joined == Some(false)
                && context.workplace_joined == Some(false)
                && context.mdm_enrolled == Some(false));
        if matches && strict_physical {
            Ok(())
        } else {
            Err(BrokerError::new(
                "target_management_changed",
                "Fresh target management evidence does not match the approved target eligibility.",
            ))
        }
    }

    fn fault(&self, point: FaultPoint) -> Result<(), BrokerError> {
        let active = self
            .fault_point
            .lock()
            .map_err(|_| BrokerError::new("fault_state_unavailable", "Fault state lock failed."))?;
        if active.as_ref() == Some(&point) {
            Err(BrokerError::new(
                "injected_fault",
                format!("Development-only fault injected at {point:?}."),
            ))
        } else {
            Ok(())
        }
    }
}

fn validate_owner_context(
    operation_id: MutationOperationId,
    context: &BrokerContext,
) -> Result<(), BrokerError> {
    if !crate::owner_scope::is_valid(&context.machine_id) {
        return Err(BrokerError::new(
            "owner_scope_unavailable",
            "Run a new scan to establish a stable Windows machine and account scope.",
        ));
    }
    if context.windows_build < 22_000 || !context.applicable {
        return Err(BrokerError::new(
            "unsupported_build",
            "Owner Mode M2 supports Windows 11 only.",
        ));
    }
    let architecture = context.architecture.trim().to_ascii_lowercase();
    let is_x64 = !architecture.contains("arm")
        && (architecture == "64-bit"
            || architecture.contains("x64")
            || architecture.contains("amd64"));
    if !is_x64 {
        return Err(BrokerError::new(
            "unsupported_architecture",
            "Owner Mode M2 supports the x64 product build only.",
        ));
    }
    if !context.authority_acceptable {
        return Err(BrokerError::new(
            "external_authority",
            format!(
                "Windows policy manages or blocks this {} setting.",
                owner_operation_label(operation_id)
            ),
        ));
    }
    if !context.confidence_sufficient {
        return Err(BrokerError::new(
            "insufficient_confidence",
            format!(
                "The {} setting could not be determined confidently.",
                owner_operation_label(operation_id)
            ),
        ));
    }
    validate_context(context, &operation_definition(operation_id))
}

fn owner_plan(
    context: &BrokerContext,
    definition: &OperationDefinition,
    target: MutationTarget,
    current_state: CapturedState,
) -> Result<MutationPlan, BrokerError> {
    let now = now_millis();
    let sequence = BROKER_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    let plan_id = format!("owner-intent-{now}-{sequence}");
    let mut plan = MutationPlan {
        plan_id: plan_id.clone(),
        machine_id: context.machine_id.clone(),
        source_inspection_id: context.inspection_id.clone(),
        source_observation_id: context.source_observation_id.clone(),
        subject_id: definition.subject_id,
        operation_id: definition.operation_id,
        current_state,
        target_state: target,
        authority: context.authority.clone(),
        applicability: context.applicability.clone(),
        windows_build: context.windows_build,
        edition: context.edition.clone(),
        evidence_fingerprint: context.evidence_fingerprint.clone(),
        generated_at: now.to_string(),
        expires_at: (now + 300_000).to_string(),
        // Retained only for backward-compatible serialization of the shared
        // plan schema. Owner Mode does not interpret this as authorization.
        approval_class: "not_applicable".into(),
        rollback_method: definition.rollback_method.into(),
        documentation: definition
            .documentation
            .iter()
            .map(|source| (*source).into())
            .collect(),
        required_privilege: definition.required_privilege.into(),
        expected_side_effects: definition
            .side_effects
            .iter()
            .map(|effect| (*effect).into())
            .collect(),
        restart_requirement: definition.restart_requirement.into(),
        handler_version: OWNER_HANDLER_VERSION.into(),
        desired_state_revision_id: context.desired_state_revision_id,
        automatic_remediation_eligible: false,
        approval_nonce_hash: hash_text(&format!("{plan_id}:internal-owner-intent")),
        consumed_at: None,
        plan_hash: String::new(),
    };
    plan.refresh_hash()
        .map_err(|error| BrokerError::new("plan_serialization_failed", error.to_string()))?;
    Ok(plan)
}

fn same_effective_state(left: &CapturedState, right: &CapturedState) -> bool {
    left.representation == right.representation
        && left.effective_enabled == right.effective_enabled
        && left.effective_state_known == right.effective_state_known
        && left.authority == right.authority
}

fn owner_recovery_blocks_new_apply(transaction: &MutationTransaction) -> bool {
    matches!(
        transaction.status,
        TransactionStatus::RecoveryRequired | TransactionStatus::RollbackVerificationFailed
    )
}

fn owner_direct_change_unavailable(transaction: &MutationTransaction) -> bool {
    transaction.operation_id == MutationOperationId::WidgetsVisibility
        && transaction.status == TransactionStatus::FailedAfterMutation
        && !transaction.rollback.available
        && !transaction.rollback.complete
        && matches!(
            transaction.verification_result.as_deref(),
            Some("write_rejected_unchanged" | "write_failed_original_state_present")
        )
        && transaction
            .error_category
            .as_deref()
            .is_some_and(|category| category == "PermissionDenied" || category == "WriteFailed")
        && transaction
            .error_summary
            .as_deref()
            .is_some_and(|summary| summary.contains("PermissionDenied"))
        && transaction
            .pre_state
            .as_ref()
            .zip(transaction.post_state.as_ref())
            .is_some_and(|(pre, post)| same_effective_state(pre, post))
}

fn is_owner_operation(operation_id: MutationOperationId) -> bool {
    matches!(
        operation_id,
        MutationOperationId::WidgetsVisibility | MutationOperationId::TaskViewVisibility
    )
}

fn owner_operation_label(operation_id: MutationOperationId) -> &'static str {
    match operation_id {
        MutationOperationId::WidgetsVisibility => "Widgets",
        MutationOperationId::TaskViewVisibility => "Task View",
        MutationOperationId::ShowDesktopEnabled => "Show desktop",
    }
}

fn owner_operation_scope(operation_id: MutationOperationId) -> &'static str {
    match operation_id {
        MutationOperationId::WidgetsVisibility => {
            "Changes the Widgets button for this Windows account."
        }
        MutationOperationId::TaskViewVisibility => {
            "Changes the Task View button for this Windows account."
        }
        MutationOperationId::ShowDesktopEnabled => "Unavailable in Owner Mode.",
    }
}

fn is_safe_attempted_state(state: &CapturedState, target: MutationTarget) -> bool {
    state.authority == "user"
        && state.effective_state_known
        && state.representation == expected(target)
        && state.effective_enabled == target.enabled()
}

fn set_post_state(
    transaction: &mut MutationTransaction,
    state: &CapturedState,
) -> Result<(), BrokerError> {
    transaction.post_state_hash = Some(hash_serializable(state).map_err(serialization_error)?);
    transaction.post_state = Some(state.clone());
    Ok(())
}

fn owner_result(
    outcome: OwnerOperationOutcome,
    transaction: MutationTransaction,
    current_state: Option<CapturedState>,
    message: impl Into<String>,
    classification: OwnerResultClassification,
    note: Option<String>,
) -> OwnerOperationResult {
    OwnerOperationResult {
        outcome,
        classification,
        transaction,
        current_state,
        message: message.into(),
        note,
    }
}

fn operation_definition(operation_id: MutationOperationId) -> OperationDefinition {
    const ALL_EDITIONS: &[&str] = &[
        "Core",
        "Home",
        "Professional",
        "Pro",
        "Enterprise",
        "Education",
    ];
    const SETTINGS_REFERENCE: &str =
        "https://learn.microsoft.com/windows/apps/develop/settings/settings-windows-11";
    const TASKBAR_SUPPORT: &str = "https://support.microsoft.com/windows/experience/personalization/customize-the-taskbar-in-windows";
    match operation_id {
        MutationOperationId::WidgetsVisibility => OperationDefinition {
            operation_id,
            subject_id: operation_id.subject(),
            title: "Widgets button visibility",
            description: "Show or hide only the current user's Widgets taskbar button.",
            supported_targets: vec![MutationTarget::Enabled, MutationTarget::Disabled],
            minimum_build: 22_000,
            supported_editions: ALL_EDITIONS,
            required_authority: "user",
            required_confidence: "confirmed_representation",
            required_privilege: "current_user_unprivileged",
            side_effects: &["Taskbar presentation updates without uninstalling Widgets."],
            restart_requirement: "No Explorer termination; shell refresh may be asynchronous.",
            rollback_method: "Restore the exact captured TaskbarDa DWORD or value absence.",
            documentation: &[SETTINGS_REFERENCE, TASKBAR_SUPPORT],
            maximum_duration_ms: 5_000,
            automatic_remediation_eligible: false,
        },
        MutationOperationId::TaskViewVisibility => OperationDefinition {
            operation_id,
            subject_id: operation_id.subject(),
            title: "Task View button visibility",
            description: "Show or hide only the current user's Task View taskbar button.",
            supported_targets: vec![MutationTarget::Enabled, MutationTarget::Disabled],
            minimum_build: 22_000,
            supported_editions: ALL_EDITIONS,
            required_authority: "user",
            required_confidence: "confirmed_representation",
            required_privilege: "current_user_unprivileged",
            side_effects: &["Taskbar presentation changes; virtual desktops remain available."],
            restart_requirement: "No Explorer termination; shell refresh may be asynchronous.",
            rollback_method: "Restore the exact captured ShowTaskViewButton DWORD or value absence.",
            documentation: &[SETTINGS_REFERENCE, TASKBAR_SUPPORT],
            maximum_duration_ms: 5_000,
            automatic_remediation_eligible: false,
        },
        MutationOperationId::ShowDesktopEnabled => OperationDefinition {
            operation_id,
            subject_id: operation_id.subject(),
            title: "Show Desktop corner",
            description: "Enable or disable the current user's far-corner Show Desktop target.",
            supported_targets: vec![MutationTarget::Enabled, MutationTarget::Disabled],
            minimum_build: 22_000,
            supported_editions: ALL_EDITIONS,
            required_authority: "user",
            required_confidence: "confirmed_representation",
            required_privilege: "current_user_unprivileged",
            side_effects: &["Only the far-corner taskbar gesture changes."],
            restart_requirement: "No Explorer termination; shell refresh may be asynchronous.",
            rollback_method: "Restore the exact captured TaskbarSd DWORD or value absence.",
            documentation: &[SETTINGS_REFERENCE, TASKBAR_SUPPORT],
            maximum_duration_ms: 5_000,
            automatic_remediation_eligible: false,
        },
    }
}

#[cfg(feature = "mutation-alpha")]
const fn operation_index(operation_id: MutationOperationId) -> usize {
    match operation_id {
        MutationOperationId::WidgetsVisibility => 0,
        MutationOperationId::TaskViewVisibility => 1,
        MutationOperationId::ShowDesktopEnabled => 2,
    }
}

fn handler(operation_id: MutationOperationId) -> &'static dyn OperationHandler {
    let selected: &'static dyn OperationHandler = match operation_id {
        MutationOperationId::WidgetsVisibility => &WIDGETS_HANDLER,
        MutationOperationId::TaskViewVisibility => &TASK_VIEW_HANDLER,
        MutationOperationId::ShowDesktopEnabled => &SHOW_DESKTOP_HANDLER,
    };
    debug_assert_eq!(selected.operation_id(), operation_id);
    selected
}

fn validate_context(
    context: &BrokerContext,
    definition: &OperationDefinition,
) -> Result<(), BrokerError> {
    if context.windows_build < definition.minimum_build {
        return Err(BrokerError::new(
            "unsupported_build",
            "The inspected Windows build is unsupported for this operation.",
        ));
    }
    if !definition
        .supported_editions
        .iter()
        .any(|edition| context.edition.contains(edition))
    {
        return Err(BrokerError::new(
            "unsupported_edition",
            "The inspected Windows edition is unsupported for this operation.",
        ));
    }
    if !context.applicable {
        return Err(BrokerError::new(
            "unsupported_applicability",
            "The operation is not applicable to the inspected state.",
        ));
    }
    if !context.authority_acceptable {
        return Err(BrokerError::new(
            "external_authority",
            "An external policy authority controls or may override this setting.",
        ));
    }
    if !context.confidence_sufficient {
        return Err(BrokerError::new(
            "insufficient_confidence",
            "Detection confidence is insufficient for mutation.",
        ));
    }
    if context.confidence != definition.required_confidence {
        return Err(BrokerError::new(
            "insufficient_confidence",
            "The inspected representation does not meet the operation's required confidence.",
        ));
    }
    let inspected = context.inspection_timestamp.parse::<u128>().unwrap_or(0);
    if now_millis().saturating_sub(inspected) > 600_000 {
        return Err(BrokerError::new(
            "stale_inspection",
            "The source inspection is older than ten minutes.",
        ));
    }
    Ok(())
}

fn validate_captured_authority(state: &CapturedState) -> Result<(), BrokerError> {
    if state.authority == "user" {
        Ok(())
    } else {
        Err(BrokerError::new(
            "external_authority",
            "A fixed Windows policy representation controls or blocks this taskbar setting.",
        ))
    }
}

#[cfg(feature = "mutation-alpha")]
fn validate_plan_context(plan: &MutationPlan, context: &BrokerContext) -> Result<(), BrokerError> {
    if plan.machine_id != context.machine_id {
        return Err(BrokerError::new(
            "wrong_machine",
            "The plan belongs to a different machine identity.",
        ));
    }
    if plan.windows_build != context.windows_build {
        return Err(BrokerError::new(
            "changed_windows_build",
            "Windows build changed after plan generation.",
        ));
    }
    if plan.edition != context.edition {
        return Err(BrokerError::new(
            "changed_windows_edition",
            "Windows edition changed after plan generation.",
        ));
    }
    if plan.authority != context.authority || plan.applicability != context.applicability {
        return Err(BrokerError::new(
            "changed_authority",
            "Authority or applicability changed after review.",
        ));
    }
    if plan.evidence_fingerprint != context.evidence_fingerprint {
        return Err(BrokerError::new(
            "changed_evidence",
            "Inspection evidence changed after review.",
        ));
    }
    Ok(())
}

fn capture_hash_matches(state: Option<&CapturedState>, expected: Option<&str>) -> bool {
    match (state, expected) {
        (None, None) => true,
        (Some(state), Some(expected)) => {
            hash_serializable(state).is_ok_and(|actual| actual == expected)
        }
        _ => false,
    }
}

fn new_transaction(plan: &MutationPlan) -> MutationTransaction {
    let created_at = plan.generated_at.clone();
    MutationTransaction {
        transaction_id: format!("transaction-{}", plan.plan_id),
        plan_id: plan.plan_id.clone(),
        machine_id: plan.machine_id.clone(),
        subject_id: plan.subject_id,
        operation_id: plan.operation_id,
        source_inspection_id: plan.source_inspection_id.clone(),
        source_observation_id: plan.source_observation_id.clone(),
        desired_state_revision_id: plan.desired_state_revision_id,
        created_at: created_at.clone(),
        approved_at: None,
        started_at: None,
        completed_at: None,
        status: TransactionStatus::Created,
        required_privilege: plan.required_privilege.clone(),
        handler_version: plan.handler_version.clone(),
        application_version: env!("CARGO_PKG_VERSION").into(),
        windows_build: plan.windows_build,
        edition: plan.edition.clone(),
        plan_hash: plan.plan_hash.clone(),
        pre_state_hash: None,
        post_state_hash: None,
        rollback_state_hash: None,
        target_state: plan.target_state,
        pre_state: None,
        post_state: None,
        rollback_state: None,
        verification_result: None,
        error_category: None,
        error_summary: None,
        recovery_requirement: None,
        steps: vec![MutationStep {
            sequence: 1,
            step_type: "created".into(),
            started_at: created_at.clone(),
            completed_at: Some(created_at),
            status: TransactionStatus::Created.key().into(),
            redacted_evidence: vec!["Closed operation transaction created.".into()],
            error_category: None,
            error_summary: None,
        }],
        rollback: RollbackRecord {
            available: false,
            complete: false,
            attempted_at: None,
            result: None,
            verification_result: None,
            conflict_detected: false,
        },
    }
}

fn transition(
    journal: &MutationJournal,
    transaction: &mut MutationTransaction,
    status: TransactionStatus,
    step_type: &str,
) -> Result<(), BrokerError> {
    let now = crate::inspection::timestamp();
    transaction.status = status;
    let sequence = transaction.steps.len() as u32 + 1;
    transaction.steps.push(MutationStep {
        sequence,
        step_type: step_type.into(),
        started_at: now.clone(),
        completed_at: Some(now),
        status: status.key().into(),
        redacted_evidence: vec!["Closed operation transition persisted.".into()],
        error_category: None,
        error_summary: None,
    });
    journal.save_transaction(transaction).map_err(journal_error)
}

#[cfg(feature = "mutation-alpha")]
fn fail_after_mutation(
    journal: &MutationJournal,
    transaction: &mut MutationTransaction,
    error: &HandlerError,
) -> Result<(), BrokerError> {
    transaction.error_category = Some(format!("{:?}", error.kind));
    transaction.error_summary = Some(error.summary.clone());
    transaction.recovery_requirement =
        Some("Inspect current state before rollback or retry.".into());
    transition(
        journal,
        transaction,
        TransactionStatus::FailedAfterMutation,
        "failed_after_mutation",
    )
}

#[cfg(feature = "mutation-alpha")]
fn confirmation_text(operation: MutationOperationId, target: MutationTarget) -> String {
    let action = if target.enabled() { "Show" } else { "Hide" };
    match operation {
        MutationOperationId::WidgetsVisibility => format!(
            "{action} the Widgets button for the current Windows user. This will not uninstall Widgets or Windows Web Experience Pack. Deslopper will verify the taskbar setting and retain the exact previous representation for rollback."
        ),
        MutationOperationId::TaskViewVisibility => format!(
            "{action} the Task View button for the current Windows user. This will not remove virtual desktops. Deslopper will verify the taskbar setting and retain the exact previous representation for rollback."
        ),
        MutationOperationId::ShowDesktopEnabled => format!(
            "{} the far-corner Show Desktop target for the current Windows user. Deslopper will verify the taskbar setting and retain the exact previous representation for rollback.",
            if target.enabled() {
                "Enable"
            } else {
                "Disable"
            }
        ),
    }
}

#[cfg(feature = "mutation-alpha")]
fn approval_phrase(operation: MutationOperationId) -> String {
    match operation {
        MutationOperationId::WidgetsVisibility => "APPROVE WIDGETS TEST",
        MutationOperationId::TaskViewVisibility => "APPROVE TASK VIEW TEST",
        MutationOperationId::ShowDesktopEnabled => "APPROVE SHOW DESKTOP TEST",
    }
    .into()
}

fn now_millis() -> u128 {
    crate::inspection::timestamp().parse().unwrap_or(0)
}

fn handler_error(error: HandlerError) -> BrokerError {
    BrokerError::new("handler_error", error.summary)
}

fn journal_error(message: String) -> BrokerError {
    BrokerError::new("journal_error", message)
}

fn serialization_error(error: serde_json::Error) -> BrokerError {
    BrokerError::new("serialization_error", error.to_string())
}

struct MutationProcessLock {
    _file: File,
    #[cfg(not(windows))]
    path: std::path::PathBuf,
}

impl MutationProcessLock {
    fn acquire(path: std::path::PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            let file = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .share_mode(0)
                .open(&path)
                .map_err(|_| "Another Deslopper process owns the mutation lock.".to_owned())?;
            Ok(Self { _file: file })
        }
        #[cfg(not(windows))]
        {
            let file = OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|_| "Another Deslopper process owns the mutation lock.".to_owned())?;
            Ok(Self { _file: file, path })
        }
    }
}

#[cfg(not(windows))]
impl Drop for MutationProcessLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(all(test, feature = "mutation-alpha"))]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::mutation::{
        handlers::{HandlerError, HandlerErrorKind, MutationBackend},
        journal::MutationJournal,
        live_validation::{
            MatrixScenarioStatus, RefreshRequirement, VerificationDimensionResult,
            VisualVerification,
        },
        plan::CapturedRepresentation,
        request::{ApprovalRequest, PlanRequest, RollbackRequest},
    };

    #[derive(Default)]
    struct FakeBackend {
        widgets: Mutex<Option<CapturedRepresentation>>,
        task_view: Mutex<Option<CapturedRepresentation>>,
        show_desktop: Mutex<Option<CapturedRepresentation>>,
        widgets_policy: AtomicBool,
        task_view_policy: AtomicBool,
        show_desktop_policy: AtomicBool,
        fail_write: AtomicBool,
        ignore_write: AtomicBool,
        write_count: AtomicU64,
    }

    impl FakeBackend {
        fn ready() -> Self {
            Self {
                widgets: Mutex::new(Some(CapturedRepresentation::Dword(1))),
                task_view: Mutex::new(Some(CapturedRepresentation::Dword(1))),
                show_desktop: Mutex::new(Some(CapturedRepresentation::Dword(1))),
                ..Default::default()
            }
        }

        fn read(
            value: &Mutex<Option<CapturedRepresentation>>,
        ) -> Result<CapturedRepresentation, HandlerError> {
            value
                .lock()
                .map_err(|_| HandlerError::new(HandlerErrorKind::ReadFailed, "test lock failed"))?
                .clone()
                .ok_or_else(|| {
                    HandlerError::new(
                        HandlerErrorKind::MissingRepresentation,
                        "test representation missing",
                    )
                })
        }

        fn write(
            &self,
            value: &Mutex<Option<CapturedRepresentation>>,
            state: &CapturedRepresentation,
        ) -> Result<(), HandlerError> {
            self.write_count.fetch_add(1, Ordering::SeqCst);
            if self.fail_write.load(Ordering::SeqCst) {
                return Err(HandlerError::new(
                    HandlerErrorKind::WriteFailed,
                    "injected test write failure",
                ));
            }
            if !self.ignore_write.load(Ordering::SeqCst) {
                *value.lock().map_err(|_| {
                    HandlerError::new(HandlerErrorKind::WriteFailed, "test lock failed")
                })? = Some(state.clone());
            }
            Ok(())
        }
    }

    impl MutationBackend for FakeBackend {
        fn read_widgets(&self) -> Result<CapturedRepresentation, HandlerError> {
            Self::read(&self.widgets)
        }
        fn widgets_externally_managed(&self) -> Result<bool, HandlerError> {
            Ok(self.widgets_policy.load(Ordering::SeqCst))
        }
        fn write_widgets(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            self.write(&self.widgets, state)
        }
        fn read_task_view(&self) -> Result<CapturedRepresentation, HandlerError> {
            Self::read(&self.task_view)
        }
        fn task_view_externally_managed(&self) -> Result<bool, HandlerError> {
            Ok(self.task_view_policy.load(Ordering::SeqCst))
        }
        fn write_task_view(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            self.write(&self.task_view, state)
        }
        fn read_show_desktop(&self) -> Result<CapturedRepresentation, HandlerError> {
            Self::read(&self.show_desktop)
        }
        fn show_desktop_externally_managed(&self) -> Result<bool, HandlerError> {
            Ok(self.show_desktop_policy.load(Ordering::SeqCst))
        }
        fn write_show_desktop(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            self.write(&self.show_desktop, state)
        }
    }

    fn temp_database(label: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "deslopper-mutation-{label}-{}-{nonce}.db",
            std::process::id()
        ))
    }

    fn context() -> BrokerContext {
        BrokerContext {
            machine_id: crate::owner_scope::from_stable_ids("machine", "S-1-5-21-1000"),
            inspection_id: "inspection-1".into(),
            inspection_timestamp: crate::inspection::timestamp(),
            source_observation_id: "inspection-1:taskbar_widgets".into(),
            windows_build: 26_100,
            edition: "Professional".into(),
            architecture: "64-bit".into(),
            #[cfg(feature = "mutation-alpha")]
            domain_joined: Some(false),
            #[cfg(feature = "mutation-alpha")]
            entra_joined: Some(false),
            #[cfg(feature = "mutation-alpha")]
            workplace_joined: Some(false),
            #[cfg(feature = "mutation-alpha")]
            mdm_enrolled: Some(false),
            authority: "user".into(),
            authority_acceptable: true,
            confidence: "confirmed_representation".into(),
            confidence_sufficient: true,
            applicability: "applicable".into(),
            applicable: true,
            evidence_fingerprint: hash_text("fixture evidence"),
            desired_state_revision_id: None,
            detector_current_enabled: Some(true),
            detector_status: "successful".into(),
        }
    }

    fn ready_broker(label: &str) -> (Broker, Arc<FakeBackend>, std::path::PathBuf) {
        let backend = Arc::new(FakeBackend::ready());
        let path = temp_database(label);
        let broker = Broker::with_journal(
            backend.clone(),
            true,
            LiveValidationGateStatus::test_valid(),
            MutationJournal::at(path.clone()),
        );
        broker.acknowledge_warning(true);
        (broker, backend, path)
    }

    fn scoped_broker(
        label: &str,
        maximum_plans: u32,
        maximum_executions: u32,
    ) -> (Broker, Arc<FakeBackend>, std::path::PathBuf) {
        let backend = Arc::new(FakeBackend::ready());
        *backend.widgets.lock().unwrap() = Some(CapturedRepresentation::Missing);
        let path = temp_database(label);
        let mut gate = LiveValidationGateStatus::test_valid();
        gate.approval_id = Some(format!("{label}-approval"));
        gate.approved_operation_scopes = vec![ApprovedOperationScope {
            operation_id: MutationOperationId::WidgetsVisibility,
            allowed_target_states: vec![MutationTarget::Enabled],
            handler_version: HANDLER_VERSION.into(),
        }];
        gate.maximum_plans = maximum_plans;
        gate.maximum_executions = maximum_executions;
        let broker = Broker::with_journal(
            backend.clone(),
            true,
            gate,
            MutationJournal::at(path.clone()),
        );
        broker.acknowledge_warning(true);
        (broker, backend, path)
    }

    fn issue(
        broker: &Broker,
        operation_id: MutationOperationId,
        target: MutationTarget,
        context: &BrokerContext,
    ) -> IssuedPlan {
        broker
            .generate_plan(
                &PlanRequest {
                    operation_id,
                    target,
                    source_inspection_id: context.inspection_id.clone(),
                },
                context,
            )
            .unwrap()
    }

    fn execute_verified(
        broker: &Broker,
        issued: IssuedPlan,
        context: &BrokerContext,
    ) -> MutationTransaction {
        let transaction = broker
            .execute(
                &ApprovalRequest {
                    plan_id: issued.plan.plan_id,
                    approval_nonce: issued.approval_nonce,
                    acknowledged: true,
                },
                context,
            )
            .unwrap();
        if transaction.status == TransactionStatus::NoChangeNeeded {
            return transaction;
        }
        assert_eq!(transaction.status, TransactionStatus::Applied);
        broker
            .complete_effective_verification(transaction, context)
            .unwrap()
    }

    #[test]
    fn registry_contains_exactly_three_closed_operations() {
        assert_eq!(MutationOperationId::ALL.len(), 3);
        let (broker, _, path) = ready_broker("registry");
        assert_eq!(broker.operation_options(&context()).len(), 3);
        cleanup(&path);
    }

    #[test]
    fn issued_plan_serializes_backend_owned_review_facts() {
        let (broker, _, path) = scoped_broker("issued-review-facts", 1, 1);
        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &context(),
        );
        let serialized = serde_json::to_value(&issued).unwrap();
        assert_eq!(serialized["approvalPhrase"], "APPROVE WIDGETS TEST");
        assert_eq!(serialized["proposedRepresentation"]["dword"], 1);
        assert!(
            serialized["confirmationText"]
                .as_str()
                .is_some_and(|value| value.contains("exact previous representation"))
        );
        assert!(
            broker
                .gate_status()
                .warning_text
                .contains("not authorization")
        );
        cleanup(&path);
    }

    #[test]
    fn scoped_approval_exposes_widgets_enabled_only_and_rejects_every_other_direction() {
        let (mut broker, backend, path) = scoped_broker("scoped-options", 1, 1);
        let context = context();
        let options = broker.operation_options(&context);
        assert_eq!(options.len(), 1);
        assert_eq!(
            options[0].definition.operation_id,
            MutationOperationId::WidgetsVisibility
        );
        assert_eq!(
            options[0].definition.supported_targets,
            vec![MutationTarget::Enabled]
        );

        for (operation_id, target) in [
            (
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
            ),
            (
                MutationOperationId::TaskViewVisibility,
                MutationTarget::Enabled,
            ),
            (
                MutationOperationId::TaskViewVisibility,
                MutationTarget::Disabled,
            ),
            (
                MutationOperationId::ShowDesktopEnabled,
                MutationTarget::Enabled,
            ),
            (
                MutationOperationId::ShowDesktopEnabled,
                MutationTarget::Disabled,
            ),
        ] {
            let error = broker
                .generate_plan(
                    &PlanRequest {
                        operation_id,
                        target,
                        source_inspection_id: context.inspection_id.clone(),
                    },
                    &context,
                )
                .unwrap_err();
            assert_eq!(error.code, "operation_target_not_approved");
        }

        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &context,
        );
        let second = broker
            .generate_plan(
                &PlanRequest {
                    operation_id: MutationOperationId::WidgetsVisibility,
                    target: MutationTarget::Enabled,
                    source_inspection_id: context.inspection_id.clone(),
                },
                &context,
            )
            .unwrap_err();
        assert_eq!(second.code, "approval_plan_limit_exhausted");

        let request = ApprovalRequest {
            plan_id: issued.plan.plan_id.clone(),
            approval_nonce: issued.approval_nonce.clone(),
            acknowledged: true,
        };
        let transaction = broker.execute(&request, &context).unwrap();
        let transaction = broker
            .complete_effective_verification(transaction, &context)
            .unwrap();
        assert_eq!(transaction.status, TransactionStatus::RollbackAvailable);
        let replay = broker.execute(&request, &context).unwrap_err();
        assert_eq!(replay.code, "consumed_plan");

        broker.live_validation.approval_expires_at_epoch_ms = Some(0);
        assert!(!broker.gate_status().available);
        let rolled_back = broker
            .rollback(
                &RollbackRequest {
                    transaction_id: transaction.transaction_id,
                    acknowledged: true,
                    allow_conflict: false,
                },
                &context,
            )
            .unwrap();
        assert_eq!(rolled_back.status, TransactionStatus::RolledBack);
        assert_eq!(
            FakeBackend::read(&backend.widgets).unwrap(),
            CapturedRepresentation::Missing
        );
        cleanup(&path);
    }

    #[test]
    fn physical_target_management_is_revalidated_for_plan_execution_and_prewrite() {
        let (mut broker, _, path) = scoped_broker("physical-management", 1, 1);
        broker.live_validation.approved_target_type = Some(ValidationTargetType::PhysicalLaptop);
        let clean = context();
        assert!(broker.require_target_eligibility(&clean).is_ok());

        for managed in ["domain", "entra", "workplace", "mdm"] {
            let mut changed = clean.clone();
            match managed {
                "domain" => changed.domain_joined = Some(true),
                "entra" => changed.entra_joined = Some(true),
                "workplace" => changed.workplace_joined = Some(true),
                "mdm" => changed.mdm_enrolled = Some(true),
                _ => unreachable!(),
            }
            assert_eq!(
                broker
                    .require_target_eligibility(&changed)
                    .unwrap_err()
                    .code,
                "target_management_changed"
            );
        }
        let mut unknown = clean;
        unknown.entra_joined = None;
        assert!(broker.require_target_eligibility(&unknown).is_err());
        cleanup(&path);
    }

    #[test]
    fn empty_approval_scope_exposes_nothing_and_fails_the_gate() {
        let (mut broker, _, path) = scoped_broker("scoped-empty", 1, 1);
        broker.live_validation.approved_operation_scopes.clear();
        assert!(broker.operation_options(&context()).is_empty());
        assert!(!broker.gate_status().available);
        assert_eq!(
            broker
                .generate_plan(
                    &PlanRequest {
                        operation_id: MutationOperationId::WidgetsVisibility,
                        target: MutationTarget::Enabled,
                        source_inspection_id: context().inspection_id,
                    },
                    &context(),
                )
                .unwrap_err()
                .code,
            "mutation_alpha_disabled"
        );
        cleanup(&path);
    }

    #[test]
    fn scoped_execution_allowance_is_consumed_once_even_with_two_plans() {
        let (broker, _, path) = scoped_broker("scoped-executions", 2, 1);
        let context = context();
        let first = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &context,
        );
        let second = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &context,
        );
        let transaction = broker
            .execute(
                &ApprovalRequest {
                    plan_id: first.plan.plan_id,
                    approval_nonce: first.approval_nonce,
                    acknowledged: true,
                },
                &context,
            )
            .unwrap();
        assert_eq!(transaction.status, TransactionStatus::Applied);
        let error = broker
            .execute(
                &ApprovalRequest {
                    plan_id: second.plan.plan_id,
                    approval_nonce: second.approval_nonce,
                    acknowledged: true,
                },
                &context,
            )
            .unwrap_err();
        assert_eq!(error.code, "approval_execution_limit_exhausted");
        cleanup(&path);
    }

    #[test]
    fn every_handler_accepts_only_its_two_closed_targets() {
        for operation in MutationOperationId::ALL {
            for target in [MutationTarget::Enabled, MutationTarget::Disabled] {
                let label = format!("targets-{}-{:?}", operation.key(), target);
                let (broker, _, path) = ready_broker(&label);
                let context = context();
                let issued = issue(&broker, operation, target, &context);
                let transaction = execute_verified(&broker, issued, &context);
                assert_eq!(
                    transaction.status,
                    if target == MutationTarget::Enabled {
                        TransactionStatus::NoChangeNeeded
                    } else {
                        TransactionStatus::RollbackAvailable
                    }
                );
                cleanup(&path);
            }
        }
    }

    #[test]
    fn unknown_operation_and_unknown_request_parameter_are_rejected() {
        assert!(serde_json::from_str::<PlanRequest>(r#"{"operationId":"unknown","target":"disabled","sourceInspectionId":"inspection-1"}"#).is_err());
        assert!(serde_json::from_str::<PlanRequest>(r#"{"operationId":"set_taskbar_widgets_visibility","target":"disabled","sourceInspectionId":"inspection-1","registryPath":"evil"}"#).is_err());
        assert!(serde_json::from_str::<PlanRequest>(r#"{"operationId":"set_taskbar_widgets_visibility","target":"disabled","sourceInspectionId":"inspection-1","componentId":"taskbar_show_desktop"}"#).is_err());
        for operation in MutationOperationId::ALL {
            assert_eq!(
                operation_definition(operation).subject_id,
                operation.subject()
            );
        }
    }

    #[test]
    fn all_runtime_gates_are_required() {
        let backend = Arc::new(FakeBackend::ready());
        let path = temp_database("gates");
        let broker = Broker::with_journal(
            backend,
            false,
            LiveValidationGateStatus::test_valid(),
            MutationJournal::at(path.clone()),
        );
        assert!(!broker.gate_status().available);
        broker.acknowledge_warning(true);
        assert!(!broker.gate_status().available);
        cleanup(&path);
    }

    #[test]
    fn live_validation_gate_and_development_host_refusal_are_enforced() {
        let backend = Arc::new(FakeBackend::ready());
        let path = temp_database("live-gate");
        let mut refused = LiveValidationGateStatus::test_valid();
        refused.available = false;
        refused.development_host_refused = true;
        refused.environment.development_host_refused = true;
        refused.reason = "Live mutation is refused on the recorded development host.".into();
        let broker =
            Broker::with_journal(backend, true, refused, MutationJournal::at(path.clone()));
        broker.acknowledge_warning(true);
        assert!(!broker.gate_status().available);
        assert!(broker.gate_status().reason.contains("development host"));
        cleanup(&path);

        let (broker, _, path) = ready_broker("live-gate-valid");
        assert!(broker.gate_status().available);
        cleanup(&path);
    }

    #[test]
    fn rejected_handler_is_removed_from_planning_eligibility() {
        let (broker, _, path) = ready_broker("rejected-handler");
        broker.set_validation_maturity(
            MutationOperationId::WidgetsVisibility,
            ValidationMaturity::Rejected,
        );
        let context = context();
        let option = broker
            .operation_options(&context)
            .into_iter()
            .find(|option| option.definition.operation_id == MutationOperationId::WidgetsVisibility)
            .unwrap();
        assert!(!option.eligible);
        assert_eq!(option.validation_maturity, ValidationMaturity::Rejected);
        assert_eq!(
            broker
                .generate_plan(
                    &PlanRequest {
                        operation_id: MutationOperationId::WidgetsVisibility,
                        target: MutationTarget::Disabled,
                        source_inspection_id: context.inspection_id.clone(),
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "handler_rejected"
        );
        cleanup(&path);
    }

    #[test]
    fn successful_apply_verify_and_exact_rollback_are_durable() {
        let (broker, backend, path) = ready_broker("success");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &context,
        );
        let transaction = execute_verified(&broker, issued, &context);
        assert_eq!(transaction.status, TransactionStatus::RollbackAvailable);
        assert_eq!(
            FakeBackend::read(&backend.widgets).unwrap(),
            CapturedRepresentation::Dword(0)
        );
        let rolled_back = broker
            .rollback(
                &RollbackRequest {
                    transaction_id: transaction.transaction_id,
                    acknowledged: true,
                    allow_conflict: false,
                },
                &context,
            )
            .unwrap();
        assert_eq!(rolled_back.status, TransactionStatus::RolledBack);
        assert_eq!(
            FakeBackend::read(&backend.widgets).unwrap(),
            CapturedRepresentation::Dword(1)
        );
        assert_eq!(broker.history().unwrap().len(), 1);
        let connection = crate::persistence::open_at(&path).unwrap();
        for table in [
            "mutation_steps",
            "mutation_state_captures",
            "mutation_rollbacks",
        ] {
            let count: i64 = connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert!(count > 0, "{table} must contain durable audit data");
        }
        let consumed: i64 = connection
            .query_row(
                "SELECT count(*) FROM mutation_plans WHERE consumed_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(consumed, 1);
        drop(connection);
        cleanup(&path);
    }

    #[test]
    fn already_compliant_is_a_durable_terminal_noop_without_write_or_rollback() {
        let (broker, backend, path) = ready_broker("already-compliant");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &context,
        );
        let request = ApprovalRequest {
            plan_id: issued.plan.plan_id.clone(),
            approval_nonce: issued.approval_nonce.clone(),
            acknowledged: true,
        };
        let transaction = broker.execute(&request, &context).unwrap();
        assert_eq!(transaction.status, TransactionStatus::NoChangeNeeded);
        assert_eq!(
            transaction.verification_result.as_deref(),
            Some("already_compliant")
        );
        assert!(!transaction.rollback.available);
        assert!(!transaction.rollback.complete);
        assert!(transaction.rollback.result.is_none());
        assert_eq!(transaction.pre_state, transaction.post_state);
        assert_eq!(backend.write_count.load(Ordering::SeqCst), 0);
        let terminal = broker
            .complete_effective_verification(transaction.clone(), &context)
            .unwrap();
        assert_eq!(terminal.status, TransactionStatus::NoChangeNeeded);
        let rollback = broker
            .rollback(
                &RollbackRequest {
                    transaction_id: terminal.transaction_id.clone(),
                    acknowledged: true,
                    allow_conflict: false,
                },
                &context,
            )
            .unwrap_err();
        assert_eq!(rollback.code, "rollback_unavailable");
        let replay = broker.execute(&request, &context).unwrap_err();
        assert_eq!(replay.code, "consumed_plan");
        let history = broker.history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, TransactionStatus::NoChangeNeeded);
        assert_eq!(backend.write_count.load(Ordering::SeqCst), 0);
        let evidence = LiveEvidenceBundle::from_transaction(
            &EvidenceExportRequest {
                transaction_id: terminal.transaction_id,
                visual_verification: VisualVerification {
                    representation: VerificationDimensionResult::Verified,
                    detector: VerificationDimensionResult::NotRun,
                    user_visible_behavior: VerificationDimensionResult::NotRun,
                    settings_ui: VerificationDimensionResult::NotRun,
                    refresh_requirement: RefreshRequirement::Undetermined,
                    lifecycle_refresh_completed: false,
                },
                screenshot_labels: Vec::new(),
                warnings: vec![
                    "Exact target representation already present; no write performed.".into(),
                ],
            },
            &history[0],
            &LiveValidationGateStatus::test_valid(),
            &ValidationScenarioManifest {
                schema_version: 2,
                scenario_id: "noop-evidence".into(),
                expected_machine_id: "machine-1".into(),
                expected_edition: "Professional".into(),
                expected_build: 26_100,
                expected_update_build_revision: Some(1),
                checkpoint_id: "checkpoint".into(),
                account_class: "local".into(),
                management_context: "none".into(),
                approval_id: None,
                source_commit: None,
                source_inspection_id: None,
                inspection_evidence_sha256: None,
                approved_operation_scopes: Vec::new(),
                maximum_plans: None,
                maximum_executions: None,
            },
        )
        .unwrap();
        assert_eq!(evidence.final_result, MatrixScenarioStatus::Passed);
        assert!(evidence.rollback_representation.is_none());
        assert!(evidence.rollback_status.is_none());
        cleanup(&path);
    }

    #[test]
    fn widgets_absent_zero_one_and_unexpected_representations_are_not_conflated() {
        let (broker, backend, path) = ready_broker("widgets-representations");
        let context = context();

        *backend.widgets.lock().unwrap() = Some(CapturedRepresentation::Missing);
        let missing = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &context,
        );
        assert_eq!(
            missing.plan.current_state.representation,
            CapturedRepresentation::Missing
        );
        assert!(!missing.plan.current_state.effective_state_known);

        *backend.widgets.lock().unwrap() = Some(CapturedRepresentation::Dword(0));
        let zero = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &context,
        );
        assert!(zero.plan.current_state.effective_state_known);
        assert!(!zero.plan.current_state.effective_enabled);

        *backend.widgets.lock().unwrap() = Some(CapturedRepresentation::Dword(1));
        let one = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Enabled,
            &context,
        );
        assert!(one.plan.current_state.effective_state_known);
        assert!(one.plan.current_state.effective_enabled);

        *backend.widgets.lock().unwrap() = Some(CapturedRepresentation::Dword(2));
        let unexpected = broker
            .generate_plan(
                &PlanRequest {
                    operation_id: MutationOperationId::WidgetsVisibility,
                    target: MutationTarget::Enabled,
                    source_inspection_id: context.inspection_id.clone(),
                },
                &context,
            )
            .unwrap_err();
        assert_eq!(unexpected.code, "handler_error");
        cleanup(&path);
    }

    #[test]
    fn missing_value_representation_is_restored_as_missing() {
        let (broker, backend, path) = ready_broker("missing-rollback");
        *backend.widgets.lock().unwrap() = Some(CapturedRepresentation::Missing);
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &context,
        );
        let transaction = execute_verified(&broker, issued, &context);
        let rolled_back = broker
            .rollback(
                &RollbackRequest {
                    transaction_id: transaction.transaction_id,
                    acknowledged: true,
                    allow_conflict: false,
                },
                &context,
            )
            .unwrap();
        assert_eq!(rolled_back.status, TransactionStatus::RolledBack);
        assert_eq!(
            FakeBackend::read(&backend.widgets).unwrap(),
            CapturedRepresentation::Missing
        );
        cleanup(&path);
    }

    #[test]
    fn approval_nonce_freshness_and_acknowledgement_are_enforced() {
        let (broker, _, path) = ready_broker("approval-validation");
        let context = context();
        let mut stale_request = PlanRequest {
            operation_id: MutationOperationId::WidgetsVisibility,
            target: MutationTarget::Disabled,
            source_inspection_id: "older-inspection".into(),
        };
        assert_eq!(
            broker
                .generate_plan(&stale_request, &context)
                .unwrap_err()
                .code,
            "stale_inspection"
        );
        stale_request.source_inspection_id = context.inspection_id.clone();
        let issued = broker.generate_plan(&stale_request, &context).unwrap();
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id.clone(),
                        approval_nonce: issued.approval_nonce.clone(),
                        acknowledged: false,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "approval_required"
        );
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: "wrong-nonce".into(),
                        acknowledged: true,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "invalid_approval_nonce"
        );
        cleanup(&path);
    }

    #[test]
    fn every_material_plan_binding_is_revalidated() {
        let (broker, _, path) = ready_broker("plan-bindings");
        let base = context();
        for (suffix, changed, expected) in [
            (
                "build",
                {
                    let mut value = base.clone();
                    value.windows_build += 1;
                    value
                },
                "changed_windows_build",
            ),
            (
                "edition",
                {
                    let mut value = base.clone();
                    value.edition = "Home".into();
                    value
                },
                "changed_windows_edition",
            ),
            (
                "authority",
                {
                    let mut value = base.clone();
                    value.authority = "local_policy".into();
                    value
                },
                "changed_authority",
            ),
            (
                "evidence",
                {
                    let mut value = base.clone();
                    value.evidence_fingerprint = hash_text("changed evidence");
                    value
                },
                "changed_evidence",
            ),
        ] {
            let issued = issue(
                &broker,
                MutationOperationId::ShowDesktopEnabled,
                MutationTarget::Disabled,
                &base,
            );
            assert_eq!(
                broker
                    .execute(
                        &ApprovalRequest {
                            plan_id: issued.plan.plan_id,
                            approval_nonce: issued.approval_nonce,
                            acknowledged: true,
                        },
                        &changed,
                    )
                    .unwrap_err()
                    .code,
                expected,
                "binding case {suffix}"
            );
        }
        cleanup(&path);
    }

    #[test]
    fn full_detector_context_is_required_before_final_verified_state() {
        let (broker, _, path) = ready_broker("effective-verification");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::TaskViewVisibility,
            MutationTarget::Disabled,
            &context,
        );
        let transaction = broker
            .execute(
                &ApprovalRequest {
                    plan_id: issued.plan.plan_id,
                    approval_nonce: issued.approval_nonce,
                    acknowledged: true,
                },
                &context,
            )
            .unwrap();
        assert_eq!(transaction.status, TransactionStatus::Applied);
        let mut policy_changed = context.clone();
        policy_changed.authority_acceptable = false;
        let transaction = broker
            .complete_effective_verification(transaction, &policy_changed)
            .unwrap();
        assert_eq!(transaction.status, TransactionStatus::VerificationFailed);
        cleanup(&path);
    }

    #[test]
    fn stale_machine_build_authority_and_source_state_are_rejected() {
        let (broker, backend, path) = ready_broker("stale");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::TaskViewVisibility,
            MutationTarget::Disabled,
            &context,
        );
        let mut wrong_machine = context.clone();
        wrong_machine.machine_id = "machine-2".into();
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id.clone(),
                        approval_nonce: issued.approval_nonce.clone(),
                        acknowledged: true,
                    },
                    &wrong_machine,
                )
                .unwrap_err()
                .code,
            "wrong_machine"
        );
        *backend.task_view.lock().unwrap() = Some(CapturedRepresentation::Dword(0));
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: issued.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "changed_source_state"
        );
        cleanup(&path);
    }

    #[test]
    fn write_failure_and_verification_mismatch_never_report_success() {
        let (broker, backend, path) = ready_broker("failures");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::ShowDesktopEnabled,
            MutationTarget::Disabled,
            &context,
        );
        backend.fail_write.store(true, Ordering::SeqCst);
        assert!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: issued.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .is_err()
        );
        let history = broker.history().unwrap();
        assert_eq!(history[0].status, TransactionStatus::FailedAfterMutation);
        cleanup(&path);
    }

    #[test]
    fn verification_mismatch_is_recorded_when_a_setter_does_not_take_effect() {
        let (broker, backend, path) = ready_broker("verification-mismatch");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &context,
        );
        backend.ignore_write.store(true, Ordering::SeqCst);
        assert!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: issued.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .is_err()
        );
        assert_eq!(
            broker.history().unwrap()[0].status,
            TransactionStatus::VerificationFailed
        );
        cleanup(&path);
    }

    #[test]
    fn cancellation_before_mutation_consumes_the_plan_and_blocks_replay() {
        let (broker, _, path) = ready_broker("cancel");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::TaskViewVisibility,
            MutationTarget::Disabled,
            &context,
        );
        let transaction = broker.cancel_before_mutation(&issued.plan.plan_id).unwrap();
        assert_eq!(
            transaction.status,
            TransactionStatus::CancelledBeforeMutation
        );
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: issued.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "consumed_plan"
        );
        cleanup(&path);
    }

    #[test]
    fn unsupported_external_and_low_confidence_contexts_are_rejected() {
        let (broker, _, path) = ready_broker("context-rejections");
        let mut unsupported = context();
        unsupported.windows_build = 19_000;
        assert_eq!(
            broker.operation_options(&unsupported)[0].reason,
            "The inspected Windows build is unsupported for this operation."
        );
        let mut external = context();
        external.authority_acceptable = false;
        assert!(
            broker
                .generate_plan(
                    &PlanRequest {
                        operation_id: MutationOperationId::WidgetsVisibility,
                        target: MutationTarget::Disabled,
                        source_inspection_id: external.inspection_id.clone(),
                    },
                    &external,
                )
                .is_err()
        );
        let mut weak = context();
        weak.confidence_sufficient = false;
        assert!(
            broker
                .generate_plan(
                    &PlanRequest {
                        operation_id: MutationOperationId::WidgetsVisibility,
                        target: MutationTarget::Disabled,
                        source_inspection_id: weak.inspection_id.clone(),
                    },
                    &weak,
                )
                .is_err()
        );
        let mut unsupported_edition = context();
        unsupported_edition.edition = "Server".into();
        assert_eq!(
            broker
                .generate_plan(
                    &PlanRequest {
                        operation_id: MutationOperationId::WidgetsVisibility,
                        target: MutationTarget::Disabled,
                        source_inspection_id: unsupported_edition.inspection_id.clone(),
                    },
                    &unsupported_edition,
                )
                .unwrap_err()
                .code,
            "unsupported_edition"
        );
        let mut inapplicable = context();
        inapplicable.applicable = false;
        assert_eq!(
            broker
                .generate_plan(
                    &PlanRequest {
                        operation_id: MutationOperationId::WidgetsVisibility,
                        target: MutationTarget::Disabled,
                        source_inspection_id: inapplicable.inspection_id.clone(),
                    },
                    &inapplicable,
                )
                .unwrap_err()
                .code,
            "unsupported_applicability"
        );
        cleanup(&path);
    }

    #[test]
    fn fixed_policy_representations_block_every_handler_and_policy_races() {
        for (operation, label) in [
            (MutationOperationId::WidgetsVisibility, "widgets"),
            (MutationOperationId::TaskViewVisibility, "task-view"),
            (MutationOperationId::ShowDesktopEnabled, "show-desktop"),
        ] {
            let (broker, backend, path) = ready_broker(&format!("policy-{label}"));
            match operation {
                MutationOperationId::WidgetsVisibility => {
                    backend.widgets_policy.store(true, Ordering::SeqCst)
                }
                MutationOperationId::TaskViewVisibility => {
                    backend.task_view_policy.store(true, Ordering::SeqCst)
                }
                MutationOperationId::ShowDesktopEnabled => {
                    backend.show_desktop_policy.store(true, Ordering::SeqCst)
                }
            }
            let context = context();
            let option = broker
                .operation_options(&context)
                .into_iter()
                .find(|option| option.definition.operation_id == operation)
                .unwrap();
            assert!(!option.eligible);
            assert_eq!(
                broker
                    .generate_plan(
                        &PlanRequest {
                            operation_id: operation,
                            target: MutationTarget::Disabled,
                            source_inspection_id: context.inspection_id.clone(),
                        },
                        &context,
                    )
                    .unwrap_err()
                    .code,
                "external_authority"
            );
            cleanup(&path);
        }

        let (broker, backend, path) = ready_broker("policy-race");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::TaskViewVisibility,
            MutationTarget::Disabled,
            &context,
        );
        backend.task_view_policy.store(true, Ordering::SeqCst);
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: issued.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "external_authority"
        );
        assert_eq!(
            broker.history().unwrap()[0].status,
            TransactionStatus::FailedBeforeMutation
        );
        cleanup(&path);

        let (broker, backend, path) = ready_broker("policy-final-verification-race");
        let final_context = context.clone();
        let issued = issue(
            &broker,
            MutationOperationId::TaskViewVisibility,
            MutationTarget::Disabled,
            &final_context,
        );
        let transaction = broker
            .execute(
                &ApprovalRequest {
                    plan_id: issued.plan.plan_id,
                    approval_nonce: issued.approval_nonce,
                    acknowledged: true,
                },
                &final_context,
            )
            .unwrap();
        backend.task_view_policy.store(true, Ordering::SeqCst);
        let transaction = broker
            .complete_effective_verification(transaction, &final_context)
            .unwrap();
        assert_eq!(transaction.status, TransactionStatus::VerificationFailed);

        backend.task_view_policy.store(false, Ordering::SeqCst);
        let transaction = broker
            .rollback(
                &RollbackRequest {
                    transaction_id: transaction.transaction_id,
                    acknowledged: true,
                    allow_conflict: false,
                },
                &final_context,
            )
            .unwrap();
        backend.task_view_policy.store(true, Ordering::SeqCst);
        let transaction = broker
            .complete_rollback_effective_verification(transaction, &final_context)
            .unwrap();
        assert_eq!(
            transaction.status,
            TransactionStatus::RollbackVerificationFailed
        );
        cleanup(&path);
    }

    #[test]
    fn tampered_and_expired_plan_integrity_is_rejected() {
        let (broker, _, path) = ready_broker("tampered");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::ShowDesktopEnabled,
            MutationTarget::Disabled,
            &context,
        );
        let conn = crate::persistence::open_at(&path).unwrap();
        conn.execute(
            "UPDATE mutation_plans SET plan_json=replace(plan_json,?2,?3) WHERE id=?1",
            rusqlite::params![
                &issued.plan.plan_id,
                &context.machine_id,
                "owner-scope-v1:sha256:tampered"
            ],
        )
        .unwrap();
        drop(conn);
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: issued.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "tampered_plan"
        );
        let mut expired = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &context,
        );
        expired.plan.expires_at = "0".into();
        expired.plan.refresh_hash().unwrap();
        let conn = crate::persistence::open_at(&path).unwrap();
        conn.execute(
            "UPDATE mutation_plans SET expires_at='0',plan_hash=?2,plan_json=?3 WHERE id=?1",
            rusqlite::params![
                expired.plan.plan_id,
                expired.plan.plan_hash,
                serde_json::to_string(&expired.plan).unwrap()
            ],
        )
        .unwrap();
        drop(conn);
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: expired.plan.plan_id,
                        approval_nonce: expired.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "expired_plan"
        );
        cleanup(&path);
    }

    #[test]
    fn tampered_transaction_capture_is_rejected_before_history_or_rollback() {
        let (broker, _, path) = ready_broker("tampered-transaction");
        let context = context();
        let transaction = execute_verified(
            &broker,
            issue(
                &broker,
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context,
            ),
            &context,
        );
        let connection = crate::persistence::open_at(&path).unwrap();
        let raw: String = connection
            .query_row(
                "SELECT transaction_json FROM mutation_transactions WHERE id=?1",
                [&transaction.transaction_id],
                |row| row.get(0),
            )
            .unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        value["preState"]["representation"]["dword"] = serde_json::json!(0);
        connection
            .execute(
                "UPDATE mutation_transactions SET transaction_json=?2 WHERE id=?1",
                rusqlite::params![transaction.transaction_id, value.to_string()],
            )
            .unwrap();
        drop(connection);
        assert_eq!(broker.history().unwrap_err().code, "tampered_transaction");
        assert_eq!(
            broker
                .rollback(
                    &RollbackRequest {
                        transaction_id: transaction.transaction_id,
                        acknowledged: true,
                        allow_conflict: false,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "tampered_transaction"
        );
        cleanup(&path);
    }

    #[test]
    fn rollback_conflict_requires_a_separate_override() {
        let (broker, backend, path) = ready_broker("rollback-conflict");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &context,
        );
        let transaction = execute_verified(&broker, issued, &context);
        *backend.widgets.lock().unwrap() = Some(CapturedRepresentation::Dword(1));
        assert_eq!(
            broker
                .rollback(
                    &RollbackRequest {
                        transaction_id: transaction.transaction_id,
                        acknowledged: true,
                        allow_conflict: false,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "rollback_conflict"
        );
        cleanup(&path);
    }

    #[test]
    fn rollback_write_and_verification_failures_are_durable() {
        let (broker, backend, path) = ready_broker("rollback-write-failure");
        let first_context = context();
        let transaction = execute_verified(
            &broker,
            issue(
                &broker,
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &first_context,
            ),
            &first_context,
        );
        backend.fail_write.store(true, Ordering::SeqCst);
        assert!(
            broker
                .rollback(
                    &RollbackRequest {
                        transaction_id: transaction.transaction_id,
                        acknowledged: true,
                        allow_conflict: false,
                    },
                    &first_context,
                )
                .is_err()
        );
        assert_eq!(
            broker.history().unwrap()[0].status,
            TransactionStatus::RollbackVerificationFailed
        );
        cleanup(&path);

        let (broker, backend, path) = ready_broker("rollback-verify-failure");
        let context = context();
        let transaction = execute_verified(
            &broker,
            issue(
                &broker,
                MutationOperationId::ShowDesktopEnabled,
                MutationTarget::Disabled,
                &context,
            ),
            &context,
        );
        backend.ignore_write.store(true, Ordering::SeqCst);
        assert!(
            broker
                .rollback(
                    &RollbackRequest {
                        transaction_id: transaction.transaction_id,
                        acknowledged: true,
                        allow_conflict: false,
                    },
                    &context,
                )
                .is_err()
        );
        assert_eq!(
            broker.history().unwrap()[0].status,
            TransactionStatus::RollbackVerificationFailed
        );
        cleanup(&path);
    }

    #[test]
    fn concurrent_execution_is_rejected_by_the_broker_lock() {
        let (broker, _, path) = ready_broker("concurrency");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &context,
        );
        let guard = broker.execution_lock.lock().unwrap();
        assert_eq!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: issued.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .unwrap_err()
                .code,
            "operation_in_progress"
        );
        drop(guard);
        cleanup(&path);
    }

    #[test]
    fn all_apply_fault_points_leave_a_recoverable_audit_state() {
        for (index, fault) in [
            FaultPoint::BeforePreStateCapture,
            FaultPoint::AfterPreStateCapture,
            FaultPoint::BeforeWrite,
            FaultPoint::ImmediatelyAfterWrite,
            FaultPoint::BeforeVerification,
            FaultPoint::DuringVerification,
            FaultPoint::BeforeTransactionCommit,
        ]
        .into_iter()
        .enumerate()
        {
            let (broker, _, path) = ready_broker(&format!("apply-fault-{index}"));
            let context = context();
            let issued = issue(
                &broker,
                MutationOperationId::WidgetsVisibility,
                MutationTarget::Disabled,
                &context,
            );
            broker.inject_fault(Some(fault));
            assert!(
                broker
                    .execute(
                        &ApprovalRequest {
                            plan_id: issued.plan.plan_id,
                            approval_nonce: issued.approval_nonce,
                            acknowledged: true,
                        },
                        &context,
                    )
                    .is_err()
            );
            broker.inject_fault(None);
            let recovered = broker.recover_interrupted(&context).unwrap();
            assert_eq!(recovered.len(), 1);
            assert_eq!(recovered[0].status, TransactionStatus::RecoveryRequired);
            cleanup(&path);
        }
    }

    #[test]
    fn all_rollback_fault_points_preserve_safe_recovery_semantics() {
        for (index, fault) in [
            FaultPoint::BeforeRollback,
            FaultPoint::ImmediatelyAfterRollbackWrite,
            FaultPoint::DuringRollbackVerification,
        ]
        .into_iter()
        .enumerate()
        {
            let (broker, _, path) = ready_broker(&format!("rollback-fault-{index}"));
            let context = context();
            let transaction = execute_verified(
                &broker,
                issue(
                    &broker,
                    MutationOperationId::TaskViewVisibility,
                    MutationTarget::Disabled,
                    &context,
                ),
                &context,
            );
            broker.inject_fault(Some(fault));
            assert!(
                broker
                    .rollback(
                        &RollbackRequest {
                            transaction_id: transaction.transaction_id,
                            acknowledged: true,
                            allow_conflict: false,
                        },
                        &context,
                    )
                    .is_err()
            );
            broker.inject_fault(None);
            let history = broker.history().unwrap();
            if fault == FaultPoint::BeforeRollback {
                assert_eq!(history[0].status, TransactionStatus::RollbackAvailable);
            } else {
                let recovered = broker.recover_interrupted(&context).unwrap();
                assert_eq!(recovered.len(), 1);
                assert_eq!(recovered[0].status, TransactionStatus::RecoveryRequired);
            }
            cleanup(&path);
        }
    }

    #[test]
    fn injected_crash_after_write_recovers_without_duplicate_apply() {
        let (broker, _, path) = ready_broker("recovery");
        let context = context();
        let issued = issue(
            &broker,
            MutationOperationId::WidgetsVisibility,
            MutationTarget::Disabled,
            &context,
        );
        broker.inject_fault(Some(FaultPoint::ImmediatelyAfterWrite));
        assert!(
            broker
                .execute(
                    &ApprovalRequest {
                        plan_id: issued.plan.plan_id,
                        approval_nonce: issued.approval_nonce,
                        acknowledged: true,
                    },
                    &context,
                )
                .is_err()
        );
        broker.inject_fault(None);
        let recovered = broker.recover_interrupted(&context).unwrap();
        assert_eq!(recovered[0].status, TransactionStatus::RecoveryRequired);
        assert_eq!(
            recovered[0].recovery_requirement.as_deref(),
            Some("unexpected_or_uncertain_state_manual_review")
        );
        cleanup(&path);
    }

    fn cleanup(path: &std::path::Path) {
        for candidate in [
            path.to_path_buf(),
            path.with_extension("db-wal"),
            path.with_extension("db-shm"),
            path.with_extension("mutation-alpha.lock"),
        ] {
            let _ = std::fs::remove_file(candidate);
        }
    }
}
