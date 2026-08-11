//! Tauri command wiring and local application-state orchestration.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    inspection::{
        CancellationToken, InspectionLifecycle, InspectionPhase, InspectionProgress,
        PROGRESS_EVENT, WindowsPowerShellRunner, complete_lifecycle, run_inspection, timestamp,
    },
    model::{AppState, CatalogueFilter, ComponentId, NavigationDestination},
    package_identity,
    persistence::{self, DesiredState, DesiredStateRevision, Snapshot, Store},
    platform::{self, DetectorStatus, State as PlatformState},
    presentation::AppView,
};

#[cfg(feature = "owner-mode")]
use crate::mutation::{
    Broker, BrokerContext, handlers::WindowsSettingStore, plan::hash_serializable,
};

#[cfg(feature = "owner-mode")]
use crate::mutation::{OwnerApplyRequest, OwnerUndoRequest};

#[cfg(feature = "mutation-alpha")]
use crate::mutation::{
    ApprovalRequest, EvidenceExportRequest, LiveEvidenceBundle, MutationJournal, PlanRequest,
    RollbackRequest, build_live_validation_gate,
};

pub struct ManagedAppState(Mutex<AppState>);

pub struct PlatformSession(Arc<Mutex<Store>>);

struct ActiveInspection {
    id: String,
    token: CancellationToken,
    progress: Arc<Mutex<InspectionProgress>>,
    completed: Arc<AtomicBool>,
}

pub struct InspectionCoordinator(Mutex<Option<ActiveInspection>>);

#[cfg(feature = "owner-mode")]
pub struct MutationSession(Arc<Broker>);

static INSPECTION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformDashboard {
    pub snapshot: Option<Snapshot>,
    pub desired_count: usize,
    pub drift_count: usize,
    pub managed_count: usize,
    pub unknown_count: usize,
    pub permission_limited_count: usize,
    pub failed_count: usize,
    pub desired_states: Vec<DesiredState>,
    pub history_retention_days: u32,
    pub database_status: persistence::DatabaseStatus,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductInfo {
    pub product_name: &'static str,
    pub version: &'static str,
    pub release_label: &'static str,
    pub build_mode: &'static str,
    pub mutation_availability: &'static str,
    pub database_schema_version: u32,
    pub database_location: &'static str,
    pub supported_windows: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductComponent {
    pub component_id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub purpose: &'static str,
    pub benefit: &'static str,
    pub support: String,
    pub risk: String,
    pub configuration: &'static str,
    pub restart: String,
    pub rollback: &'static str,
    pub privileges: &'static str,
    pub gaming_notes: &'static str,
    pub enterprise_notes: &'static str,
    pub documentation: &'static str,
    pub is_package: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotChange {
    pub component_id: &'static str,
    pub previous_state: PlatformState,
    pub current_state: PlatformState,
    pub previous_status: DetectorStatus,
    pub current_status: DetectorStatus,
    pub explanation: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotComparison {
    pub previous_inspection_id: String,
    pub current_inspection_id: String,
    pub changes: Vec<SnapshotChange>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticsRequest {
    pub include_history_summary: bool,
    pub include_redacted_errors: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionHistoryItem {
    pub inspection_id: String,
    pub timestamp: String,
    pub status: String,
    pub duration_ms: Option<u128>,
    pub windows_build: u32,
    pub windows_edition: String,
    pub app_version: String,
    pub successful_count: usize,
    pub unknown_count: usize,
    pub failed_count: usize,
    pub cancelled_count: usize,
    pub drift_count: usize,
    pub management_summary: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentTimelineEntry {
    pub inspection_id: String,
    pub observation_time: String,
    pub windows_build: u32,
    pub windows_edition: String,
    pub observation: platform::DetectionResult,
    pub desired_state: Option<DesiredState>,
    pub drift: Option<persistence::DriftEvent>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesiredStateOption {
    pub key: String,
    pub label: String,
    pub scope: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesiredStateRequest {
    pub component_id: String,
    pub state_key: String,
    pub scope: String,
    pub persistent_remediation: bool,
    pub always_require_approval: bool,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesiredStateValidation {
    pub status: String,
    pub valid: bool,
    pub warnings: Vec<String>,
    pub reason: String,
}

impl ManagedAppState {
    fn new() -> Self {
        Self(Mutex::new(AppState::new()))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum AppAction {
    SearchChanged { query: String },
    FilterChanged { filter: String },
    TogglePlanned { component_id: String },
    Navigate { destination: String },
    OpenReview,
    CloseReview,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    code: &'static str,
    message: String,
}

impl CommandError {
    fn invalid_action(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_action",
            message: message.into(),
        }
    }

    fn state_unavailable() -> Self {
        Self {
            code: "state_unavailable",
            message: "The local preview state is temporarily unavailable.".to_owned(),
        }
    }

    fn inspection_conflict(message: impl Into<String>) -> Self {
        Self {
            code: "inspection_conflict",
            message: message.into(),
        }
    }

    #[cfg(feature = "owner-mode")]
    fn mutation(error: crate::mutation::BrokerError) -> Self {
        Self {
            code: error.code,
            message: error.message,
        }
    }
}

#[tauri::command]
pub fn get_app_view(state: State<'_, ManagedAppState>) -> Result<AppView, CommandError> {
    let state = state
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    Ok(AppView::from(&*state))
}

#[tauri::command]
pub fn dispatch_app_action(
    action: AppAction,
    state: State<'_, ManagedAppState>,
) -> Result<AppView, CommandError> {
    let mut state = state
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    apply_action(&mut state, action)?;
    Ok(AppView::from(&*state))
}

#[tauri::command]
pub fn get_platform_dashboard(
    session: State<'_, PlatformSession>,
) -> Result<PlatformDashboard, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    Ok(dashboard(&store))
}

#[tauri::command]
pub fn start_inspection(
    app: AppHandle,
    session: State<'_, PlatformSession>,
    coordinator: State<'_, InspectionCoordinator>,
) -> Result<InspectionProgress, CommandError> {
    let mut active = coordinator
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    if active
        .as_ref()
        .is_some_and(|inspection| !inspection.completed.load(Ordering::SeqCst))
    {
        return Err(CommandError::inspection_conflict(
            "A read-only inspection is already running.",
        ));
    }
    let id = format!(
        "inspection-{}-{}",
        timestamp(),
        INSPECTION_SEQUENCE.fetch_add(1, Ordering::SeqCst)
    );
    let token = CancellationToken::default();
    let lifecycle = InspectionLifecycle::new(id.clone(), platform::v1_catalogue().len());
    let initial = lifecycle.progress(None, None);
    let shared_progress = Arc::new(Mutex::new(initial.clone()));
    let completed = Arc::new(AtomicBool::new(false));
    *active = Some(ActiveInspection {
        id: id.clone(),
        token: token.clone(),
        progress: Arc::clone(&shared_progress),
        completed: Arc::clone(&completed),
    });
    drop(active);
    let store = Arc::clone(&session.0);
    std::thread::spawn(move || {
        let progress_for_callback = Arc::clone(&shared_progress);
        let app_for_callback = app.clone();
        let mut outcome =
            run_inspection(&WindowsPowerShellRunner, id.clone(), &token, move |event| {
                if let Ok(mut current) = progress_for_callback.lock() {
                    *current = event.clone();
                }
                let _ = app_for_callback.emit(PROGRESS_EVENT, event);
            });
        let persist_status = format!(
            "Persisting detector observations from {} shared queries",
            outcome.query_execution_counts.len()
        );
        set_lifecycle_phase(
            &mut outcome.lifecycle,
            InspectionPhase::PersistingResults,
            &persist_status,
        );
        emit_lifecycle(&app, &shared_progress, &outcome.lifecycle);
        let mut persistence_failed = false;
        if let Ok(mut current_store) = store.lock() {
            let snapshot = Snapshot {
                id: id.clone(),
                timestamp: outcome.lifecycle.started_at.clone(),
                platform: outcome.platform.clone(),
                observations: outcome.observations.clone(),
                lifecycle: Some(outcome.lifecycle.clone()),
                query_failures: outcome.query_failures.clone(),
                machine_id: persistence::machine_identity(&outcome.platform),
            };
            current_store.snapshots.push(snapshot);
            if persistence::save(&current_store).is_err() {
                persistence_failed = true;
            }
            if !persistence_failed {
                set_lifecycle_phase(
                    &mut outcome.lifecycle,
                    InspectionPhase::CalculatingDrift,
                    "Calculating history-aware drift",
                );
                emit_lifecycle(&app, &shared_progress, &outcome.lifecycle);
                update_drift(&mut current_store);
                refresh_desired_validity(&mut current_store);
                complete_lifecycle(&mut outcome.lifecycle, false);
                if let Some(snapshot) = current_store.snapshots.last_mut() {
                    snapshot.lifecycle = Some(outcome.lifecycle.clone());
                }
                if persistence::apply_history_retention(&mut current_store).is_err() {
                    persistence_failed = true;
                }
            }
        } else {
            persistence_failed = true;
        }
        if persistence_failed {
            complete_lifecycle(&mut outcome.lifecycle, true);
        }
        emit_lifecycle(&app, &shared_progress, &outcome.lifecycle);
        completed.store(true, Ordering::SeqCst);
    });
    Ok(initial)
}

fn set_lifecycle_phase(lifecycle: &mut InspectionLifecycle, phase: InspectionPhase, status: &str) {
    lifecycle.phase = phase;
    lifecycle.status = status.into();
    lifecycle.last_progress_update = timestamp();
}

fn emit_lifecycle(
    app: &AppHandle,
    progress: &Arc<Mutex<InspectionProgress>>,
    lifecycle: &InspectionLifecycle,
) {
    let event = lifecycle.progress(None, None);
    if let Ok(mut current) = progress.lock() {
        *current = event.clone();
    }
    let _ = app.emit(PROGRESS_EVENT, event);
}

#[tauri::command]
pub fn cancel_inspection(
    inspection_id: String,
    app: AppHandle,
    coordinator: State<'_, InspectionCoordinator>,
) -> Result<InspectionProgress, CommandError> {
    let active = coordinator
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let inspection = active
        .as_ref()
        .ok_or_else(|| CommandError::inspection_conflict("No inspection is running."))?;
    if inspection.id != inspection_id {
        return Err(CommandError::inspection_conflict(
            "Only the matching running inspection can be cancelled.",
        ));
    }
    if inspection.completed.load(Ordering::SeqCst) {
        return Err(CommandError::inspection_conflict(
            "The inspection has already completed.",
        ));
    }
    inspection.token.cancel();
    let mut progress = inspection
        .progress
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    progress.phase = InspectionPhase::Cancelling;
    progress.status = "Cancelling the active read-only inspection".into();
    progress.cancellation_available = false;
    progress.updated_at = timestamp();
    let result = progress.clone();
    let _ = app.emit(PROGRESS_EVENT, result.clone());
    Ok(result)
}

#[tauri::command]
pub fn get_running_inspection_state(
    coordinator: State<'_, InspectionCoordinator>,
) -> Result<Option<InspectionProgress>, CommandError> {
    let active = coordinator
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let Some(inspection) = active.as_ref() else {
        return Ok(None);
    };
    let progress = inspection
        .progress
        .lock()
        .map_err(|_| CommandError::state_unavailable())?
        .clone();
    Ok(Some(progress))
}

#[tauri::command]
pub fn get_allowed_desired_state_options(
    component_id: String,
    session: State<'_, PlatformSession>,
) -> Result<Vec<DesiredStateOption>, CommandError> {
    let definition = component_definition(&component_id)?;
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let observation = latest_observation(&store, definition.id)?;
    if matches!(
        observation.current,
        PlatformState::Unknown { .. } | PlatformState::Unsupported { .. }
    ) {
        return Ok(Vec::new());
    }
    let options = if definition.is_package {
        vec![
            DesiredStateOption {
                key: "present_current_user".into(),
                label: "Present for current user".into(),
                scope: "current_user".into(),
                description: "Keep the verified package registered for the current user.".into(),
            },
            DesiredStateOption {
                key: "absent_current_user".into(),
                label: "Absent for current user".into(),
                scope: "current_user".into(),
                description:
                    "Record a preference for a future supported, reviewed current-user operation."
                        .into(),
            },
        ]
    } else {
        vec![
            DesiredStateOption {
                key: "enabled".into(),
                label: "Enabled".into(),
                scope: "effective".into(),
                description: "Prefer the documented experience enabled where applicable.".into(),
            },
            DesiredStateOption {
                key: "disabled".into(),
                label: "Disabled".into(),
                scope: "effective".into(),
                description: "Prefer the documented experience disabled where applicable.".into(),
            },
        ]
    };
    Ok(options)
}

#[tauri::command]
pub fn validate_desired_state(
    request: DesiredStateRequest,
    session: State<'_, PlatformSession>,
) -> Result<DesiredStateValidation, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    validate_desired_request(&request, &store)
}

#[tauri::command]
pub fn save_desired_state(
    request: DesiredStateRequest,
    session: State<'_, PlatformSession>,
) -> Result<PlatformDashboard, CommandError> {
    let mut store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let validation = validate_desired_request(&request, &store)?;
    if !validation.valid {
        return Err(CommandError::invalid_action(validation.reason));
    }
    let definition = component_definition(&request.component_id)?;
    let state = desired_state_from_key(&definition, &request.state_key)?;
    let now = timestamp();
    let build = store
        .snapshots
        .last()
        .map(|snapshot| snapshot.platform.build)
        .unwrap_or(0);
    let edition = store
        .snapshots
        .last()
        .map(|snapshot| snapshot.platform.edition.clone())
        .unwrap_or_else(|| "Unknown".into());
    let existing = store
        .desired
        .iter()
        .find(|desired| desired.component_id == definition.id)
        .cloned();
    let revision = existing
        .as_ref()
        .map(|desired| desired.revision + 1)
        .unwrap_or(1);
    let created_at = existing
        .as_ref()
        .map(|desired| desired.created_at.clone())
        .unwrap_or_else(|| now.clone());
    let desired = DesiredState {
        component_id: definition.id,
        state: state.clone(),
        scope: request.scope.clone(),
        created_at,
        modified_at: now.clone(),
        persistent: request.persistent_remediation,
        approval_required: request.always_require_approval,
        selected_build: build,
        selected_edition: edition,
        note: request
            .note
            .clone()
            .filter(|note| !note.trim().is_empty())
            .map(|note| note.trim().chars().take(500).collect()),
        validation_status: validation.status.clone(),
        revision,
    };
    store
        .desired
        .retain(|value| value.component_id != definition.id);
    store.desired.push(desired.clone());
    store.desired_revisions.push(DesiredStateRevision {
        component_id: definition.id,
        revision,
        state,
        scope: desired.scope.clone(),
        changed_at: now,
        persistent: desired.persistent,
        approval_required: desired.approval_required,
        note: desired.note.clone(),
        validation_status: desired.validation_status.clone(),
    });
    persistence::save(&store).map_err(CommandError::invalid_action)?;
    Ok(dashboard(&store))
}

fn component_definition(component_id: &str) -> Result<platform::ComponentDefinition, CommandError> {
    platform::v1_catalogue()
        .into_iter()
        .find(|definition| definition.id.key() == component_id)
        .ok_or_else(|| CommandError::invalid_action("Unknown v1 component."))
}

fn latest_observation(
    store: &Store,
    component_id: platform::ComponentId,
) -> Result<&platform::DetectionResult, CommandError> {
    store
        .snapshots
        .last()
        .and_then(|snapshot| {
            snapshot
                .observations
                .iter()
                .find(|observation| observation.component_id == component_id)
        })
        .ok_or_else(|| {
            CommandError::invalid_action("Inspect Windows before selecting a desired state.")
        })
}

fn desired_state_from_key(
    definition: &platform::ComponentDefinition,
    state_key: &str,
) -> Result<PlatformState, CommandError> {
    if definition.is_package {
        match state_key {
            "present_current_user" => Ok(PlatformState::Package {
                current_user: platform::PackageRegistrationState::Present,
                all_users: platform::PackageRegistrationState::Unknown,
                provisioned: platform::PackageProvisioningState::Unknown,
                version: None,
            }),
            "absent_current_user" => Ok(PlatformState::Package {
                current_user: platform::PackageRegistrationState::Absent,
                all_users: platform::PackageRegistrationState::Unknown,
                provisioned: platform::PackageProvisioningState::Unknown,
                version: None,
            }),
            _ => Err(CommandError::invalid_action(
                "The requested package desired state is not allowlisted.",
            )),
        }
    } else {
        match state_key {
            "enabled" => Ok(PlatformState::UserPreference { enabled: true }),
            "disabled" => Ok(PlatformState::UserPreference { enabled: false }),
            _ => Err(CommandError::invalid_action(
                "The requested setting desired state is not allowlisted.",
            )),
        }
    }
}

fn validate_desired_request(
    request: &DesiredStateRequest,
    store: &Store,
) -> Result<DesiredStateValidation, CommandError> {
    let definition = component_definition(&request.component_id)?;
    let observation = latest_observation(store, definition.id)?;
    let allowed_scope = if definition.is_package {
        request.scope == "current_user"
    } else {
        request.scope == "effective" || request.scope == "current_user"
    };
    desired_state_from_key(&definition, &request.state_key)?;
    if !allowed_scope {
        return Ok(DesiredStateValidation {
            status: "invalid".into(),
            valid: false,
            warnings: Vec::new(),
            reason: "The requested scope is not valid for this component.".into(),
        });
    }
    if matches!(
        observation.authority,
        platform::Authority::DomainPolicy
            | platform::Authority::Mdm
            | platform::Authority::Firmware
            | platform::Authority::SecurityProduct
    ) {
        return Ok(DesiredStateValidation { status: "externally_managed".into(), valid: false, warnings: vec!["A higher authority controls the effective state.".into()], reason: "The desired state cannot be saved as valid while the component is externally managed.".into() });
    }
    if !observation.applicable {
        return Ok(DesiredStateValidation {
            status: "invalid".into(),
            valid: false,
            warnings: vec![observation.applicability.reason.clone()],
            reason: "The component is unsupported on this edition or build.".into(),
        });
    }
    if matches!(
        observation.detector_status,
        DetectorStatus::Failed | DetectorStatus::Cancelled
    ) {
        return Ok(DesiredStateValidation {
            status: "insufficient_detection_confidence".into(),
            valid: false,
            warnings: observation.warnings.clone(),
            reason: "Detection did not complete successfully.".into(),
        });
    }
    let mut warnings = Vec::new();
    let mut status = "valid";
    if definition.is_package
        && observation.package_completeness != platform::PackageCompleteness::Complete
    {
        status = "requires_review";
        warnings.push(
            "Package scope is incomplete; a future full-absence preview will remain unavailable."
                .into(),
        );
    }
    if definition.is_package
        && request.state_key == "present_current_user"
        && observation.package_identities.is_empty()
    {
        return Ok(DesiredStateValidation {
            status: "missing_rollback_source".into(),
            valid: false,
            warnings: vec![
                "No exact currently observed package identity can anchor a reinstall source."
                    .into(),
            ],
            reason: "A verified identity and rollback source are required.".into(),
        });
    }
    if !request.always_require_approval {
        warnings.push("Desired states are local planning records and do not authorize a Windows change. Owner Mode Apply and Undo are separate explicit actions.".into());
        status = "valid_with_warnings";
    }
    Ok(DesiredStateValidation { status: status.into(), valid: true, warnings, reason: "The requested state is valid for the current observation, subject to the listed warnings.".into() })
}

#[tauri::command]
pub fn clear_desired_state(
    component_id: String,
    session: State<'_, PlatformSession>,
) -> Result<PlatformDashboard, CommandError> {
    let mut store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let definition = component_definition(&component_id)?;
    if let Some(existing) = store
        .desired
        .iter()
        .find(|value| value.component_id == definition.id)
        .cloned()
    {
        store.desired_revisions.push(DesiredStateRevision {
            component_id: definition.id,
            revision: existing.revision + 1,
            state: PlatformState::Unknown {
                error: "Desired state cleared by user".into(),
            },
            scope: existing.scope,
            changed_at: timestamp(),
            persistent: false,
            approval_required: true,
            note: None,
            validation_status: "cleared".into(),
        });
    }
    store.desired.retain(|x| x.component_id != definition.id);
    persistence::save(&store).map_err(CommandError::invalid_action)?;
    Ok(dashboard(&store))
}

#[tauri::command]
pub fn generate_preview_plan(
    component_id: String,
    session: State<'_, PlatformSession>,
) -> Result<serde_json::Value, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let desired = store
        .desired
        .iter()
        .find(|x| x.component_id.key() == component_id)
        .ok_or_else(|| CommandError::invalid_action("No desired state has been selected."))?;
    let definition = platform::v1_catalogue()
        .into_iter()
        .find(|x| x.id.key() == component_id)
        .ok_or_else(|| CommandError::invalid_action("Unknown component."))?;
    let observation = store
        .snapshots
        .last()
        .and_then(|s| {
            s.observations
                .iter()
                .find(|o| o.component_id == definition.id)
        })
        .ok_or_else(|| CommandError::invalid_action("No current observation is available."))?;
    let status = if matches!(observation.current, PlatformState::Unknown { .. })
        || matches!(
            observation.detector_status,
            DetectorStatus::Failed | DetectorStatus::Cancelled
        )
        || (definition.is_package
            && observation.package_completeness != platform::PackageCompleteness::Complete)
    {
        "research_further"
    } else {
        "preview_ready"
    };
    let preview = serde_json::json!({"previewSchemaVersion":1,"componentId":component_id,"sourceInspectionId":store.snapshots.last().map(|snapshot|snapshot.id.clone()),"generatedAt":timestamp(),"currentState":observation.current,"desiredState":desired.state,"authority":observation.authority,"windowsBuild":observation.platform.build,"windowsEdition":observation.platform.edition,"mechanism":definition.configuration,"scope":desired.scope,"elevation":"No authority is requested for this preview","restart":definition.restart,"dataImpact":definition.purpose,"compatibility":definition.enterprise_notes,"knownRisk":definition.risk,"dependencies":definition.dependencies,"automaticReconciliationRecommended":false,"approvalRequired":definition.approval_required,"rollback":definition.rollback,"rollbackComplete":!matches!(definition.id,platform::ComponentId::Onedrive),"status":status,"executorEnabled":false,"createsMutationTransaction":false,"createsApprovalNonce":false,"cannotExecute":"Preview plans never execute Windows changes; use the separate Widgets owner action when it is available","documentation":definition.documentation});
    persistence::save_preview_plan(&component_id, &preview)
        .map_err(CommandError::invalid_action)?;
    Ok(preview)
}

#[tauri::command]
pub fn get_inspection_history(
    session: State<'_, PlatformSession>,
) -> Result<Vec<InspectionHistoryItem>, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    Ok(store
        .snapshots
        .iter()
        .rev()
        .map(|snapshot| {
            let lifecycle = snapshot.lifecycle.as_ref();
            let drift_count = store
                .drift
                .iter()
                .filter(|event| event.current_inspection_id.as_deref() == Some(&snapshot.id))
                .count();
            InspectionHistoryItem {
                inspection_id: snapshot.id.clone(),
                timestamp: snapshot.timestamp.clone(),
                status: lifecycle
                    .map(|value| value.status.clone())
                    .unwrap_or_else(|| "Legacy imported inspection".into()),
                duration_ms: lifecycle
                    .and_then(|value| value.completed_at.as_ref())
                    .and_then(|end| end.parse::<u128>().ok())
                    .zip(snapshot.timestamp.parse::<u128>().ok())
                    .map(|(end, start)| end.saturating_sub(start)),
                windows_build: snapshot.platform.build,
                windows_edition: snapshot.platform.edition.clone(),
                app_version: env!("CARGO_PKG_VERSION").into(),
                successful_count: lifecycle
                    .map(|value| value.successful_detector_count)
                    .unwrap_or_else(|| {
                        snapshot
                            .observations
                            .iter()
                            .filter(|observation| {
                                observation.detector_status == DetectorStatus::Successful
                            })
                            .count()
                    }),
                unknown_count: lifecycle
                    .map(|value| value.unknown_detector_count)
                    .unwrap_or_else(|| {
                        snapshot
                            .observations
                            .iter()
                            .filter(|observation| {
                                observation.detector_status == DetectorStatus::Unknown
                            })
                            .count()
                    }),
                failed_count: lifecycle
                    .map(|value| value.failed_detector_count)
                    .unwrap_or_else(|| {
                        snapshot
                            .observations
                            .iter()
                            .filter(|observation| {
                                observation.detector_status == DetectorStatus::Failed
                            })
                            .count()
                    }),
                cancelled_count: lifecycle
                    .map(|value| value.cancelled_detector_count)
                    .unwrap_or(0),
                drift_count,
                management_summary: format!(
                    "Domain: {}; workplace: {}; MDM: {}",
                    summary_bool(snapshot.platform.domain_joined),
                    summary_bool(snapshot.platform.workplace_joined),
                    summary_bool(snapshot.platform.mdm_enrolled)
                ),
            }
        })
        .collect())
}

fn summary_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    }
}

#[tauri::command]
pub fn get_inspection_detail(
    inspection_id: String,
    session: State<'_, PlatformSession>,
) -> Result<Snapshot, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    store
        .snapshots
        .iter()
        .find(|snapshot| snapshot.id == inspection_id)
        .cloned()
        .ok_or_else(|| CommandError::invalid_action("Unknown inspection ID."))
}

#[tauri::command]
pub fn get_component_observation_timeline(
    component_id: String,
    session: State<'_, PlatformSession>,
) -> Result<Vec<ComponentTimelineEntry>, CommandError> {
    let definition = component_definition(&component_id)?;
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let mut timeline = Vec::new();
    for snapshot in store.snapshots.iter().rev() {
        if let Some(observation) = snapshot
            .observations
            .iter()
            .find(|value| value.component_id == definition.id)
        {
            let desired_state = store
                .desired_revisions
                .iter()
                .filter(|revision| {
                    revision.component_id == definition.id
                        && revision.changed_at <= snapshot.timestamp
                })
                .max_by_key(|revision| revision.revision)
                .map(|revision| DesiredState {
                    component_id: revision.component_id,
                    state: revision.state.clone(),
                    scope: revision.scope.clone(),
                    created_at: revision.changed_at.clone(),
                    modified_at: revision.changed_at.clone(),
                    persistent: revision.persistent,
                    approval_required: revision.approval_required,
                    selected_build: snapshot.platform.build,
                    selected_edition: snapshot.platform.edition.clone(),
                    note: revision.note.clone(),
                    validation_status: revision.validation_status.clone(),
                    revision: revision.revision,
                });
            let drift = store
                .drift
                .iter()
                .find(|event| {
                    event.component_id == definition.id
                        && event.current_inspection_id.as_deref() == Some(&snapshot.id)
                })
                .cloned();
            timeline.push(ComponentTimelineEntry {
                inspection_id: snapshot.id.clone(),
                observation_time: observation.detected_at.clone(),
                windows_build: snapshot.platform.build,
                windows_edition: snapshot.platform.edition.clone(),
                observation: observation.clone(),
                desired_state,
                drift,
            });
        }
    }
    Ok(timeline)
}

#[tauri::command]
pub fn get_drift_history(
    session: State<'_, PlatformSession>,
) -> Result<Vec<persistence::DriftEvent>, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let mut events = store.drift.clone();
    events.sort_by(|left, right| right.last_observed.cmp(&left.last_observed));
    Ok(events)
}

#[tauri::command]
pub fn acknowledge_drift_event(
    component_id: String,
    classification: String,
    session: State<'_, PlatformSession>,
) -> Result<Vec<persistence::DriftEvent>, CommandError> {
    let definition = component_definition(&component_id)?;
    let mut store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let event = store
        .drift
        .iter_mut()
        .find(|event| {
            event.component_id == definition.id
                && event.classification == classification
                && !event.reviewed
        })
        .ok_or_else(|| CommandError::invalid_action("No matching active drift event exists."))?;
    event.reviewed = true;
    event.reviewed_at = Some(timestamp());
    persistence::save(&store).map_err(CommandError::invalid_action)?;
    Ok(store.drift.clone())
}

#[tauri::command]
pub fn get_detailed_package_observations(
    inspection_id: String,
    component_id: Option<String>,
    session: State<'_, PlatformSession>,
) -> Result<Vec<platform::PackageObservation>, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let snapshot = store
        .snapshots
        .iter()
        .find(|snapshot| snapshot.id == inspection_id)
        .ok_or_else(|| CommandError::invalid_action("Unknown inspection ID."))?;
    let component = component_id
        .as_deref()
        .map(component_definition)
        .transpose()?
        .map(|definition| definition.id);
    Ok(snapshot
        .observations
        .iter()
        .filter(|observation| component.is_none_or(|id| id == observation.component_id))
        .flat_map(|observation| observation.packages.clone())
        .collect())
}

#[tauri::command]
pub fn get_migration_status(
    session: State<'_, PlatformSession>,
) -> Result<persistence::DatabaseStatus, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    Ok(store.database_status.clone())
}

#[tauri::command]
pub fn get_vm_validation_metadata() -> serde_json::Value {
    serde_json::json!({"fixtureSchemaVersion":1,"applicabilityRuleVersion":crate::applicability::RULE_VERSION,"identityRuleVersion":crate::package_identity::IDENTITY_RULE_VERSION,"driftClassifications":platform::drift_kind_keys(),"matrix":"validation/matrix.json","captureTool":"tools/capture-vm-fixture.ps1","productionFeature":false})
}

#[tauri::command]
pub fn get_product_info() -> ProductInfo {
    ProductInfo {
        product_name: "Deslopper",
        version: env!("CARGO_PKG_VERSION"),
        release_label: if cfg!(feature = "owner-mode") {
            "Owner Mode M2"
        } else {
            "Read-Only Engineering Build"
        },
        build_mode: if cfg!(feature = "mutation-alpha") {
            "engineering mutation-alpha harness"
        } else if cfg!(feature = "owner-mode") {
            "owner mode"
        } else {
            "explicit read-only"
        },
        mutation_availability: if cfg!(feature = "mutation-alpha") {
            "engineering validation harness"
        } else if cfg!(feature = "owner-mode") {
            "Task View Apply and Undo, with scoped Widgets capability, for the current Windows account"
        } else {
            "unavailable in this build"
        },
        database_schema_version: persistence::SCHEMA_VERSION,
        database_location: "%LOCALAPPDATA%\\Deslopper\\deslopper.db",
        supported_windows: "Windows 11 x64; Windows 10 results are legacy observation only",
    }
}

#[tauri::command]
pub fn get_product_component_catalogue() -> Vec<ProductComponent> {
    platform::v1_catalogue()
        .into_iter()
        .map(|definition| ProductComponent {
            component_id: definition.id.key(),
            name: definition.name,
            category: definition.category,
            purpose: definition.purpose,
            benefit: definition.benefit,
            support: format!("{:?}", definition.support),
            risk: format!("{:?}", definition.risk),
            configuration: definition.configuration,
            restart: format!("{:?}", definition.restart),
            rollback: definition.rollback,
            privileges: definition.privileges,
            gaming_notes: definition.gaming_notes,
            enterprise_notes: definition.enterprise_notes,
            documentation: definition.documentation,
            is_package: definition.is_package,
        })
        .collect()
}

#[tauri::command]
pub fn compare_inspections(
    previous_inspection_id: String,
    current_inspection_id: String,
    session: State<'_, PlatformSession>,
) -> Result<SnapshotComparison, CommandError> {
    if previous_inspection_id == current_inspection_id {
        return Err(CommandError::invalid_action(
            "Choose two different inspections to compare.",
        ));
    }
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let previous = store
        .snapshots
        .iter()
        .find(|snapshot| snapshot.id == previous_inspection_id)
        .ok_or_else(|| CommandError::invalid_action("Unknown previous inspection."))?;
    let current = store
        .snapshots
        .iter()
        .find(|snapshot| snapshot.id == current_inspection_id)
        .ok_or_else(|| CommandError::invalid_action("Unknown current inspection."))?;
    if !previous.machine_id.is_empty()
        && !current.machine_id.is_empty()
        && previous.machine_id != current.machine_id
    {
        return Err(CommandError::invalid_action(
            "Snapshots from different machine identities cannot be compared.",
        ));
    }
    let mut changes = Vec::new();
    for definition in platform::v1_catalogue() {
        let before = previous
            .observations
            .iter()
            .find(|observation| observation.component_id == definition.id);
        let after = current
            .observations
            .iter()
            .find(|observation| observation.component_id == definition.id);
        let (Some(before), Some(after)) = (before, after) else {
            continue;
        };
        if before.current == after.current
            && before.detector_status == after.detector_status
            && before.authority == after.authority
            && before.applicability.status == after.applicability.status
        {
            continue;
        }
        let explanation = if matches!(
            before.detector_status,
            DetectorStatus::Failed | DetectorStatus::Cancelled
        ) || matches!(
            after.detector_status,
            DetectorStatus::Failed | DetectorStatus::Cancelled
        ) {
            "Detector completeness changed; this is not proof that Windows changed.".into()
        } else if before.authority != after.authority {
            "The observed controlling authority changed.".into()
        } else if before.applicability.status != after.applicability.status {
            "Build or edition applicability changed.".into()
        } else {
            "The observed state changed between these inspections.".into()
        };
        changes.push(SnapshotChange {
            component_id: definition.id.key(),
            previous_state: before.current.clone(),
            current_state: after.current.clone(),
            previous_status: before.detector_status,
            current_status: after.detector_status,
            explanation,
        });
    }
    Ok(SnapshotComparison {
        previous_inspection_id,
        current_inspection_id,
        changes,
    })
}

#[tauri::command]
pub fn set_history_retention(
    days: u32,
    session: State<'_, PlatformSession>,
) -> Result<PlatformDashboard, CommandError> {
    if !matches!(days, 0 | 30 | 90 | 180 | 365) {
        return Err(CommandError::invalid_action(
            "History retention must be 30, 90, 180, or 365 days, or kept indefinitely.",
        ));
    }
    let mut store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    store.preferences.history_retention_days = days;
    persistence::apply_history_retention(&mut store).map_err(CommandError::invalid_action)?;
    Ok(dashboard(&store))
}

#[tauri::command]
pub fn clear_local_history(
    confirmed: bool,
    session: State<'_, PlatformSession>,
) -> Result<PlatformDashboard, CommandError> {
    if !confirmed {
        return Err(CommandError::invalid_action(
            "Clearing local history requires explicit confirmation.",
        ));
    }
    let mut store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    persistence::clear_local_history(&mut store).map_err(CommandError::invalid_action)?;
    Ok(dashboard(&store))
}

#[tauri::command]
pub fn generate_diagnostics_export(
    request: DiagnosticsRequest,
    session: State<'_, PlatformSession>,
) -> Result<serde_json::Value, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    Ok(build_diagnostics_export(&store, &request))
}

fn build_diagnostics_export(store: &Store, request: &DiagnosticsRequest) -> serde_json::Value {
    let latest = store.snapshots.last();
    let detectors: Vec<serde_json::Value> = latest
        .map(|snapshot| {
            snapshot
                .observations
                .iter()
                .map(|observation| {
                    let warnings: Vec<String> = observation
                        .warnings
                        .iter()
                        .map(|value| crate::privacy::redact_diagnostic(value))
                        .collect();
                    let error = request
                        .include_redacted_errors
                        .then(|| {
                            observation
                                .error
                                .as_deref()
                                .map(crate::privacy::redact_diagnostic)
                        })
                        .flatten();
                    let evidence: Vec<serde_json::Value> = observation
                        .evidence
                        .iter()
                        .map(|item| {
                            serde_json::json!({
                                "queryId": item.query_id,
                                "source": crate::privacy::redact_diagnostic(&item.source),
                                "detail": crate::privacy::redact_diagnostic(&item.detail),
                                "confidence": item.confidence,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "componentId": observation.component_id.key(),
                        "detectorStatus": observation.detector_status,
                        "authority": observation.authority,
                        "authorityConfidence": observation.authority_attribution.confidence,
                        "applicability": observation.applicability.status,
                        "packageCompleteness": observation.package_completeness,
                        "warnings": warnings,
                        "error": error,
                        "redactedEvidence": evidence,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let history_summary = request.include_history_summary.then(|| {
        serde_json::json!({
            "inspectionCount": store.snapshots.len(),
            "activeDriftCount": store.drift.iter().filter(|event| !event.resolved).count(),
            "desiredStateCount": store.desired.len(),
        })
    });
    let lifecycle = latest.and_then(|snapshot| snapshot.lifecycle.as_ref());
    serde_json::json!({
        "diagnosticsSchemaVersion": 1,
        "exportedAtEpochMs": timestamp(),
        "product": {
            "name": "Deslopper",
            "version": env!("CARGO_PKG_VERSION"),
            "release": if cfg!(feature = "owner-mode") { "Owner Mode M1" } else { "Read-Only Engineering Build" },
            "buildMode": if cfg!(feature = "mutation-alpha") { "engineering_mutation_alpha_harness" } else if cfg!(feature = "owner-mode") { "owner_mode" } else { "explicit_read_only" },
            "mutationAvailability": if cfg!(feature = "mutation-alpha") { "engineering_validation_harness" } else if cfg!(feature = "owner-mode") { "widgets_apply_and_undo" } else { "unavailable_in_this_build" },
        },
        "windows": latest.map(|snapshot| serde_json::json!({
            "productName": snapshot.platform.product_name,
            "edition": snapshot.platform.edition,
            "build": snapshot.platform.build,
            "updateBuildRevision": snapshot.platform.update_build_revision,
            "displayVersion": snapshot.platform.display_version,
            "architecture": snapshot.platform.architecture,
        })),
        "inspectionSummary": latest.map(|snapshot| serde_json::json!({
            "inspectionId": snapshot.id,
            "phase": lifecycle.map(|value| format!("{:?}", value.phase)),
            "successful": lifecycle.map(|value| value.successful_detector_count),
            "unknown": lifecycle.map(|value| value.unknown_detector_count),
            "failed": lifecycle.map(|value| value.failed_detector_count),
            "cancelled": lifecycle.map(|value| value.cancelled_detector_count),
            "warningCount": lifecycle.map(|value| value.warning_count),
            "errorCount": lifecycle.map(|value| value.error_count),
        })),
        "detectors": detectors,
        "historySummary": history_summary,
        "schemas": {
            "database": persistence::SCHEMA_VERSION,
            "diagnostics": 1,
            "applicabilityRules": crate::applicability::RULE_VERSION,
            "packageIdentityRules": crate::package_identity::IDENTITY_RULE_VERSION,
        },
        "capabilities": {
            "tauriWindow": ["core:event:allow-listen", "core:event:allow-unlisten"],
            "filesystem": false,
            "network": false,
            "shell": false,
            "mutation": false,
        },
        "privacy": {
            "machineIdentityIncluded": false,
            "hostnameIncluded": false,
            "developmentHostDenylistIncluded": false,
            "approvalManifestIncluded": false,
            "rawProfilePathsIncluded": false,
            "automaticUpload": false,
        }
    })
}

fn dashboard(store: &Store) -> PlatformDashboard {
    let latest = store.snapshots.last().cloned();
    PlatformDashboard {
        snapshot: latest.clone(),
        desired_count: store.desired.len(),
        drift_count: store.drift.iter().filter(|x| !x.resolved).count(),
        managed_count: latest
            .as_ref()
            .map(|s| {
                s.observations
                    .iter()
                    .filter(|x| {
                        matches!(
                            x.authority,
                            platform::Authority::DomainPolicy
                                | platform::Authority::Mdm
                                | platform::Authority::LocalPolicy
                        )
                    })
                    .count()
            })
            .unwrap_or(0),
        unknown_count: latest
            .as_ref()
            .map(|s| {
                s.observations
                    .iter()
                    .filter(|x| matches!(x.current, PlatformState::Unknown { .. }))
                    .count()
            })
            .unwrap_or(0),
        permission_limited_count: latest
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .observations
                    .iter()
                    .filter(|observation| {
                        observation.package_completeness
                            == platform::PackageCompleteness::PermissionLimited
                            || observation.packages.iter().any(|package| {
                                package.current_user
                                    == platform::PackageRegistrationState::PermissionLimited
                                    || package.other_users
                                        == platform::PackageRegistrationState::PermissionLimited
                                    || package.provisioning
                                        == platform::PackageProvisioningState::PermissionLimited
                            })
                    })
                    .count()
            })
            .unwrap_or(0),
        failed_count: latest
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .observations
                    .iter()
                    .filter(|observation| observation.detector_status == DetectorStatus::Failed)
                    .count()
            })
            .unwrap_or(0),
        desired_states: store.desired.clone(),
        history_retention_days: store.preferences.history_retention_days,
        database_status: store.database_status.clone(),
    }
}
fn update_drift(store: &mut Store) {
    if store.snapshots.len() < 2 {
        return;
    }
    let previous = store.snapshots[store.snapshots.len() - 2].clone();
    let Some(current) = store.snapshots.last().cloned() else {
        return;
    };
    if !previous.machine_id.is_empty()
        && !current.machine_id.is_empty()
        && previous.machine_id != current.machine_id
    {
        return;
    }
    for observation in &current.observations {
        if matches!(
            observation.detector_status,
            DetectorStatus::Failed | DetectorStatus::Cancelled
        ) {
            continue;
        }
        if let Some(old) = previous
            .observations
            .iter()
            .find(|x| x.component_id == observation.component_id)
        {
            if matches!(
                old.detector_status,
                DetectorStatus::Failed | DetectorStatus::Cancelled
            ) {
                continue;
            }
            let desired = store
                .desired
                .iter()
                .find(|x| x.component_id == observation.component_id)
                .map(|x| x.state.clone());
            let is_package = platform::v1_catalogue()
                .into_iter()
                .find(|definition| definition.id == observation.component_id)
                .is_some_and(|definition| definition.is_package);
            let classification = if is_package
                && (old.package_completeness != platform::PackageCompleteness::Complete
                    || observation.package_completeness != platform::PackageCompleteness::Complete)
            {
                (old.package_completeness != observation.package_completeness)
                    .then(|| "DetectionUncertainty".to_owned())
            } else if package_identity::identity_migrated(
                observation.component_id,
                &old.package_identities,
                &observation.package_identities,
            ) {
                Some("PackageIdentityMigration".to_owned())
            } else if old.applicability.status != observation.applicability.status {
                Some("Availability".to_owned())
            } else if old.authority_attribution != observation.authority_attribution {
                Some("ManagementAuthority".to_owned())
            } else if old.control_precedence.user_preference
                != observation.control_precedence.user_preference
            {
                Some("Preference".to_owned())
            } else {
                platform::classify_drift(
                    &old.current,
                    &observation.current,
                    desired.as_ref().unwrap_or(&observation.current),
                )
                .map(|kind| format!("{kind:?}"))
            };
            let Some(classification) = classification else {
                continue;
            };
            if classification == "NormalServicing" {
                continue;
            }
            let inference = infer_cause(old, observation, &previous, &current, &classification);
            if let Some(existing) = store.drift.iter_mut().find(|event| {
                event.component_id == observation.component_id
                    && event.classification == classification
                    && !event.resolved
            }) {
                existing.current = observation.current.clone();
                existing.last_observed = observation.detected_at.clone();
                existing.occurrence_count += 1;
                existing.cause = inference.primary;
                existing.confidence = inference.confidence;
                existing.supporting_facts = inference.supporting_facts;
                existing.alternative_causes = inference.alternatives;
                existing.inference_rule_version = inference.rule_version;
                existing.current_inspection_id = Some(current.id.clone());
                existing.reviewed = false;
                existing.reviewed_at = None;
                continue;
            }
            store.drift.push(persistence::DriftEvent {
                component_id: observation.component_id,
                previous: old.current.clone(),
                current: observation.current.clone(),
                desired,
                classification,
                cause: inference.primary,
                confidence: inference.confidence,
                first_detected: observation.detected_at.clone(),
                last_observed: observation.detected_at.clone(),
                resolved: false,
                reviewed: false,
                reviewed_at: None,
                returned_to_desired: false,
                occurrence_count: 1,
                supporting_facts: inference.supporting_facts,
                alternative_causes: inference.alternatives,
                inference_rule_version: inference.rule_version,
                previous_inspection_id: Some(previous.id.clone()),
                current_inspection_id: Some(current.id.clone()),
            });
        }
    }
    for event in &mut store.drift {
        if event.resolved {
            continue;
        }
        let Some(desired) = event.desired.as_ref() else {
            continue;
        };
        let returned = current
            .observations
            .iter()
            .find(|observation| observation.component_id == event.component_id)
            .is_some_and(|observation| {
                !matches!(
                    observation.detector_status,
                    DetectorStatus::Failed | DetectorStatus::Cancelled | DetectorStatus::Unknown
                ) && &observation.current == desired
            });
        if returned {
            event.resolved = true;
            event.returned_to_desired = true;
            event.last_observed = current.timestamp.clone();
            event.current_inspection_id = Some(current.id.clone());
        }
    }
}

const CAUSE_INFERENCE_RULE_VERSION: u32 = 2;

struct CauseInference {
    primary: String,
    confidence: String,
    supporting_facts: Vec<String>,
    alternatives: Vec<String>,
    rule_version: u32,
}

fn infer_cause(
    previous: &platform::DetectionResult,
    current: &platform::DetectionResult,
    previous_snapshot: &Snapshot,
    current_snapshot: &Snapshot,
    classification: &str,
) -> CauseInference {
    if current_snapshot.platform.build != previous_snapshot.platform.build {
        return CauseInference {
            primary: "Windows feature update changed the component representation".into(),
            confidence: "Strong".into(),
            supporting_facts: vec![format!(
                "Build changed from {} to {}",
                previous_snapshot.platform.build, current_snapshot.platform.build
            )],
            alternatives: vec!["Windows servicing".into(), "Package servicing".into()],
            rule_version: CAUSE_INFERENCE_RULE_VERSION,
        };
    }
    if current_snapshot.platform.update_build_revision
        != previous_snapshot.platform.update_build_revision
    {
        return CauseInference {
            primary: "Windows servicing changed the update build revision".into(),
            confidence: "Moderate".into(),
            supporting_facts: vec![format!(
                "UBR changed from {:?} to {:?}",
                previous_snapshot.platform.update_build_revision,
                current_snapshot.platform.update_build_revision
            )],
            alternatives: vec![
                "User preference change".into(),
                "Management policy refresh".into(),
            ],
            rule_version: CAUSE_INFERENCE_RULE_VERSION,
        };
    }
    if classification == "NormalServicing" {
        return CauseInference {
            primary: "Microsoft Store or inbox package servicing".into(),
            confidence: "Strong".into(),
            supporting_facts: vec![
                "Package presence and provisioning were stable while the exact version changed"
                    .into(),
            ],
            alternatives: vec!["Application self-update".into()],
            rule_version: CAUSE_INFERENCE_RULE_VERSION,
        };
    }
    if classification == "PackageIdentityMigration" {
        return CauseInference {
            primary: "Known package identity migration".into(),
            confidence: "Confirmed".into(),
            supporting_facts: vec![format!(
                "Known identities changed from {:?} to {:?}",
                previous.package_identities, current.package_identities
            )],
            alternatives: vec![
                "Feature update".into(),
                "Store-delivered application replacement".into(),
            ],
            rule_version: CAUSE_INFERENCE_RULE_VERSION,
        };
    }
    if classification == "PackageProvisioning" {
        return CauseInference {
            primary: "Windows, Store, or OEM package provisioning".into(),
            confidence: "Moderate".into(),
            supporting_facts: vec![
                "Provisioning state changed while package scopes remained independently observed"
                    .into(),
            ],
            alternatives: vec!["Feature update".into(), "Image servicing".into()],
            rule_version: CAUSE_INFERENCE_RULE_VERSION,
        };
    }
    if classification == "ManagementAuthority" {
        return CauseInference {
            primary: match current.authority {
                platform::Authority::DomainPolicy => "Domain policy change",
                platform::Authority::Mdm => "MDM policy change",
                _ => "Management authority changed",
            }
            .into(),
            confidence: if current.authority_attribution.exact_source_proven {
                "Confirmed".into()
            } else {
                "Moderate".into()
            },
            supporting_facts: current.authority_attribution.evidence.clone(),
            alternatives: current
                .authority_attribution
                .alternatives
                .iter()
                .map(|authority| format!("{authority:?}"))
                .collect(),
            rule_version: CAUSE_INFERENCE_RULE_VERSION,
        };
    }
    if matches!(current.current, PlatformState::Unknown { .. })
        && !matches!(previous.current, PlatformState::Unknown { .. })
    {
        return CauseInference {
            primary: "Detection source became unavailable".into(),
            confidence: "Confirmed".into(),
            supporting_facts: current.warnings.clone(),
            alternatives: vec![
                "Permission changed".into(),
                "Read-only query timed out".into(),
            ],
            rule_version: CAUSE_INFERENCE_RULE_VERSION,
        };
    }
    CauseInference {
        primary: "User action or management policy refresh".into(),
        confidence: "Weak".into(),
        supporting_facts: vec![format!("Observed classification: {classification}")],
        alternatives: vec![
            "Application reinstall".into(),
            "Windows servicing".into(),
            "Store package update".into(),
        ],
        rule_version: CAUSE_INFERENCE_RULE_VERSION,
    }
}

fn refresh_desired_validity(store: &mut Store) {
    let Some(snapshot) = store.snapshots.last() else {
        return;
    };
    for desired in &mut store.desired {
        let Some(observation) = snapshot
            .observations
            .iter()
            .find(|observation| observation.component_id == desired.component_id)
        else {
            desired.validation_status = "stale".into();
            continue;
        };
        desired.validation_status = if !observation.applicable {
            "invalid"
        } else if matches!(
            observation.detector_status,
            DetectorStatus::Failed | DetectorStatus::Cancelled | DetectorStatus::Unknown
        ) || desired.selected_build != snapshot.platform.build
            || desired.selected_edition != snapshot.platform.edition
        {
            "requires_review"
        } else if matches!(
            observation.authority,
            platform::Authority::DomainPolicy | platform::Authority::Mdm
        ) {
            "stale"
        } else {
            "valid"
        }
        .into();
    }
}

fn apply_action(state: &mut AppState, action: AppAction) -> Result<(), CommandError> {
    match action {
        AppAction::SearchChanged { query } => state.set_query(query),
        AppAction::FilterChanged { filter } => {
            let filter = CatalogueFilter::from_key(&filter).ok_or_else(|| {
                CommandError::invalid_action(format!("Unknown catalogue filter: {filter}"))
            })?;
            state.set_filter(filter);
        }
        AppAction::TogglePlanned { component_id } => {
            let component_id = ComponentId::from_key(&component_id).ok_or_else(|| {
                CommandError::invalid_action(format!("Unknown component: {component_id}"))
            })?;
            state.toggle_planned(component_id);
        }
        AppAction::Navigate { destination } => {
            let destination = NavigationDestination::from_key(&destination).ok_or_else(|| {
                CommandError::invalid_action(format!(
                    "Unknown navigation destination: {destination}"
                ))
            })?;
            state.set_destination(destination);
        }
        AppAction::OpenReview => state.open_review(),
        AppAction::CloseReview => state.close_review(),
    }

    Ok(())
}

#[cfg(not(feature = "owner-mode"))]
pub fn run() -> tauri::Result<()> {
    let app = tauri::Builder::default()
        .manage(ManagedAppState::new())
        .manage(PlatformSession(Arc::new(Mutex::new(persistence::load()))))
        .manage(InspectionCoordinator(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            get_app_view,
            dispatch_app_action,
            get_platform_dashboard,
            start_inspection,
            cancel_inspection,
            get_running_inspection_state,
            get_inspection_history,
            get_inspection_detail,
            get_component_observation_timeline,
            get_drift_history,
            acknowledge_drift_event,
            get_detailed_package_observations,
            get_allowed_desired_state_options,
            validate_desired_state,
            save_desired_state,
            clear_desired_state,
            generate_preview_plan,
            get_migration_status,
            get_vm_validation_metadata,
            get_product_info,
            get_product_component_catalogue,
            compare_inspections,
            set_history_retention,
            clear_local_history,
            generate_diagnostics_export
        ])
        .build(tauri::generate_context!())?;
    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
            let coordinator = handle.state::<InspectionCoordinator>();
            if let Ok(active) = coordinator.0.lock()
                && let Some(inspection) = active.as_ref()
                && !inspection.completed.load(Ordering::SeqCst)
            {
                inspection.token.cancel();
            }
        }
    });
    Ok(())
}

#[cfg(feature = "owner-mode")]
#[tauri::command]
pub fn get_widgets_actionability(
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<crate::mutation::broker::OwnerActionability, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let context = mutation_context(
        &store,
        Some(crate::mutation::MutationOperationId::WidgetsVisibility),
    )?;
    Ok(mutation.0.owner_actionability(
        crate::mutation::MutationOperationId::WidgetsVisibility,
        &context,
    ))
}

#[cfg(feature = "owner-mode")]
#[tauri::command]
pub fn get_task_view_actionability(
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<crate::mutation::broker::OwnerActionability, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let operation_id = crate::mutation::MutationOperationId::TaskViewVisibility;
    let context = mutation_context(&store, Some(operation_id))?;
    Ok(mutation.0.owner_actionability(operation_id, &context))
}

#[cfg(feature = "owner-mode")]
#[tauri::command]
pub fn apply_owner_operation(
    request: OwnerApplyRequest,
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<crate::mutation::broker::OwnerOperationResult, CommandError> {
    {
        let store = session
            .0
            .lock()
            .map_err(|_| CommandError::state_unavailable())?;
        if store.snapshots.last().map(|snapshot| snapshot.id.as_str())
            != Some(request.source_inspection_id.as_str())
        {
            return Err(CommandError::invalid_action(
                "A newer scan exists. Review the latest taskbar state before applying.",
            ));
        }
    }
    let (fresh_snapshot, fresh_context) = owner_reinspection(request.operation_id)?;
    persist_owner_snapshot(&session, fresh_snapshot)?;
    let mut verification_snapshot = None;
    let result = mutation
        .0
        .apply_owner_operation(request.operation_id, request.target, &fresh_context, || {
            let (snapshot, context) =
                owner_reinspection(request.operation_id).map_err(|error| error.message)?;
            verification_snapshot = Some(snapshot);
            Ok(context)
        })
        .map_err(CommandError::mutation)?;
    if result.outcome == crate::mutation::broker::OwnerOperationOutcome::Restored {
        let (restored_snapshot, _) = owner_reinspection(request.operation_id)?;
        persist_owner_snapshot(&session, restored_snapshot)?;
    } else if let Some(snapshot) = verification_snapshot {
        persist_owner_snapshot(&session, snapshot)?;
    }
    Ok(result)
}

#[cfg(feature = "owner-mode")]
#[tauri::command]
pub fn undo_owner_operation(
    request: OwnerUndoRequest,
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<crate::mutation::broker::OwnerOperationResult, CommandError> {
    let operation_id = {
        let store = session
            .0
            .lock()
            .map_err(|_| CommandError::state_unavailable())?;
        let scope = store
            .snapshots
            .last()
            .map(|snapshot| snapshot.machine_id.as_str())
            .ok_or_else(|| CommandError::invalid_action("Run a fresh inspection before Undo."))?;
        mutation
            .0
            .owner_history(scope)
            .map_err(CommandError::mutation)?
            .into_iter()
            .find(|transaction| transaction.transaction_id == request.transaction_id)
            .map(|transaction| transaction.operation_id)
            .ok_or_else(|| CommandError::invalid_action("The Owner Mode change was not found."))?
    };
    let (fresh_snapshot, fresh_context) = owner_reinspection(operation_id)?;
    persist_owner_snapshot(&session, fresh_snapshot)?;
    let mut verification_snapshot = None;
    let result = mutation
        .0
        .undo_owner_operation(&request.transaction_id, &fresh_context, || {
            let (snapshot, context) =
                owner_reinspection(operation_id).map_err(|error| error.message)?;
            verification_snapshot = Some(snapshot);
            Ok(context)
        })
        .map_err(CommandError::mutation)?;
    if let Some(snapshot) = verification_snapshot {
        persist_owner_snapshot(&session, snapshot)?;
    }
    Ok(result)
}

#[cfg(feature = "owner-mode")]
#[tauri::command]
pub fn get_owner_change_history(
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<Vec<crate::mutation::MutationTransaction>, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let Some(scope) = store
        .snapshots
        .last()
        .map(|snapshot| snapshot.machine_id.as_str())
    else {
        return Ok(Vec::new());
    };
    mutation
        .0
        .owner_history(scope)
        .map_err(CommandError::mutation)
}

#[cfg(feature = "owner-mode")]
fn owner_reinspection(
    operation_id: crate::mutation::MutationOperationId,
) -> Result<(Snapshot, BrokerContext), CommandError> {
    let id = format!("owner-reinspection-{}", timestamp());
    let token = CancellationToken::default();
    let mut outcome = run_inspection(&WindowsPowerShellRunner, id.clone(), &token, |_| {});
    complete_lifecycle(&mut outcome.lifecycle, false);
    let machine_id = persistence::machine_identity(&outcome.platform);
    let snapshot = Snapshot {
        id,
        timestamp: outcome.lifecycle.started_at.clone(),
        platform: outcome.platform,
        observations: outcome.observations,
        lifecycle: Some(outcome.lifecycle),
        query_failures: outcome.query_failures,
        machine_id,
    };
    let store = Store {
        schema_version: persistence::SCHEMA_VERSION,
        snapshots: vec![snapshot.clone()],
        ..Default::default()
    };
    let context = mutation_context(&store, Some(operation_id))?;
    Ok((snapshot, context))
}

#[cfg(feature = "owner-mode")]
fn persist_owner_snapshot(
    session: &PlatformSession,
    snapshot: Snapshot,
) -> Result<(), CommandError> {
    let mut store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    store.snapshots.push(snapshot);
    update_drift(&mut store);
    refresh_desired_validity(&mut store);
    persistence::apply_history_retention(&mut store).map_err(CommandError::invalid_action)
}

#[cfg(all(feature = "owner-mode", not(feature = "mutation-alpha")))]
pub fn run() -> tauri::Result<()> {
    let store = persistence::load();
    let broker = Arc::new(Broker::owner(Arc::new(WindowsSettingStore)));
    if let Ok(context) = mutation_context(
        &store,
        Some(crate::mutation::MutationOperationId::WidgetsVisibility),
    ) && crate::owner_scope::is_valid(&context.machine_id)
    {
        let _ = broker.recover_interrupted(&context);
    }
    let app = tauri::Builder::default()
        .manage(ManagedAppState::new())
        .manage(PlatformSession(Arc::new(Mutex::new(store))))
        .manage(InspectionCoordinator(Mutex::new(None)))
        .manage(MutationSession(broker))
        .invoke_handler(tauri::generate_handler![
            get_app_view,
            dispatch_app_action,
            get_platform_dashboard,
            start_inspection,
            cancel_inspection,
            get_running_inspection_state,
            get_inspection_history,
            get_inspection_detail,
            get_component_observation_timeline,
            get_drift_history,
            acknowledge_drift_event,
            get_detailed_package_observations,
            get_allowed_desired_state_options,
            validate_desired_state,
            save_desired_state,
            clear_desired_state,
            generate_preview_plan,
            get_migration_status,
            get_vm_validation_metadata,
            get_product_info,
            get_product_component_catalogue,
            compare_inspections,
            set_history_retention,
            clear_local_history,
            generate_diagnostics_export,
            get_widgets_actionability,
            get_task_view_actionability,
            apply_owner_operation,
            undo_owner_operation,
            get_owner_change_history
        ])
        .build(tauri::generate_context!())?;
    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
            let coordinator = handle.state::<InspectionCoordinator>();
            if let Ok(active) = coordinator.0.lock()
                && let Some(inspection) = active.as_ref()
                && !inspection.completed.load(Ordering::SeqCst)
            {
                inspection.token.cancel();
            }
        }
    });
    Ok(())
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn get_mutation_alpha_status(
    mutation: State<'_, MutationSession>,
) -> crate::mutation::AlphaGateStatus {
    mutation.0.gate_status()
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn acknowledge_mutation_alpha_warning(
    acknowledged: bool,
    mutation: State<'_, MutationSession>,
) -> crate::mutation::AlphaGateStatus {
    mutation.0.acknowledge_warning(acknowledged)
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn get_mutation_operation_options(
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<Vec<crate::mutation::broker::OperationOption>, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let context = mutation_context(&store, None)?;
    Ok(mutation.0.operation_options(&context))
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn generate_mutation_plan(
    request: PlanRequest,
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<crate::mutation::broker::IssuedPlan, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let context = mutation_context(&store, Some(request.operation_id))?;
    mutation
        .0
        .generate_plan(&request, &context)
        .map_err(CommandError::mutation)
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn approve_and_execute_mutation(
    request: ApprovalRequest,
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<crate::mutation::MutationTransaction, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let plan = MutationJournal::default()
        .load_plan(&request.plan_id)
        .map_err(CommandError::invalid_action)?
        .ok_or_else(|| CommandError::invalid_action("Unknown mutation plan."))?;
    if store.snapshots.last().map(|snapshot| snapshot.id.as_str())
        != Some(plan.source_inspection_id.as_str())
    {
        return Err(CommandError::invalid_action(
            "A newer inspection exists; generate and approve a fresh plan.",
        ));
    }
    drop(store);
    let context = live_mutation_context(plan.operation_id)?;
    let transaction = mutation
        .0
        .execute(&request, &context)
        .map_err(CommandError::mutation)?;
    let post_context = live_mutation_context(plan.operation_id)?;
    mutation
        .0
        .complete_effective_verification(transaction, &post_context)
        .map_err(CommandError::mutation)
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn rollback_mutation(
    request: RollbackRequest,
    session: State<'_, PlatformSession>,
    mutation: State<'_, MutationSession>,
) -> Result<crate::mutation::MutationTransaction, CommandError> {
    let store = session
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    let transaction = MutationJournal::default()
        .load_transaction(&request.transaction_id)
        .map_err(CommandError::invalid_action)?
        .ok_or_else(|| CommandError::invalid_action("Unknown mutation transaction."))?;
    drop(store);
    let context = live_mutation_context(transaction.operation_id)?;
    let transaction = mutation
        .0
        .rollback(&request, &context)
        .map_err(CommandError::mutation)?;
    let post_context = live_mutation_context(transaction.operation_id)?;
    mutation
        .0
        .complete_rollback_effective_verification(transaction, &post_context)
        .map_err(CommandError::mutation)
}

#[cfg(feature = "mutation-alpha")]
fn live_mutation_context(
    operation_id: crate::mutation::MutationOperationId,
) -> Result<BrokerContext, CommandError> {
    let id = format!("mutation-reinspection-{}", timestamp());
    let token = CancellationToken::default();
    let mut outcome = run_inspection(&WindowsPowerShellRunner, id.clone(), &token, |_| {});
    complete_lifecycle(&mut outcome.lifecycle, false);
    let machine_id = persistence::machine_identity(&outcome.platform);
    let snapshot = Snapshot {
        id,
        timestamp: outcome.lifecycle.started_at.clone(),
        platform: outcome.platform,
        observations: outcome.observations,
        lifecycle: Some(outcome.lifecycle),
        query_failures: outcome.query_failures,
        machine_id,
    };
    let store = Store {
        schema_version: persistence::SCHEMA_VERSION,
        snapshots: vec![snapshot],
        ..Default::default()
    };
    mutation_context(&store, Some(operation_id))
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn get_mutation_history(
    mutation: State<'_, MutationSession>,
) -> Result<Vec<crate::mutation::MutationTransaction>, CommandError> {
    mutation.0.history().map_err(CommandError::mutation)
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn cancel_mutation_plan(
    plan_id: String,
    mutation: State<'_, MutationSession>,
) -> Result<crate::mutation::MutationTransaction, CommandError> {
    mutation
        .0
        .cancel_before_mutation(&plan_id)
        .map_err(CommandError::mutation)
}

#[cfg(feature = "mutation-alpha")]
#[tauri::command]
pub fn export_live_validation_evidence(
    request: EvidenceExportRequest,
    mutation: State<'_, MutationSession>,
) -> Result<LiveEvidenceBundle, CommandError> {
    mutation
        .0
        .export_live_validation_evidence(&request)
        .map_err(CommandError::mutation)
}

#[cfg(feature = "owner-mode")]
fn mutation_context(
    store: &Store,
    operation_id: Option<crate::mutation::MutationOperationId>,
) -> Result<BrokerContext, CommandError> {
    let snapshot = store
        .snapshots
        .last()
        .ok_or_else(|| CommandError::invalid_action("Run a fresh inspection before mutation."))?;
    let lifecycle = snapshot.lifecycle.as_ref().ok_or_else(|| {
        CommandError::invalid_action("The source inspection has no lifecycle record.")
    })?;
    if !matches!(
        lifecycle.phase,
        InspectionPhase::Completed | InspectionPhase::CompletedWithPartialFailures
    ) {
        return Err(CommandError::invalid_action(
            "Only a completed inspection can anchor a mutation plan.",
        ));
    }
    let subject = operation_id
        .map(|operation| operation.subject().key())
        .unwrap_or("mutation_alpha");
    let widgets_policy = snapshot
        .observations
        .iter()
        .find(|observation| observation.component_id == platform::ComponentId::WidgetsPlatform);
    let detector_component = match operation_id {
        Some(crate::mutation::MutationOperationId::TaskViewVisibility) => {
            platform::ComponentId::TaskbarTaskView
        }
        _ => platform::ComponentId::TaskbarWidgets,
    };
    let selected_detector = snapshot
        .observations
        .iter()
        .find(|observation| observation.component_id == detector_component);
    let detector_current_enabled =
        selected_detector.and_then(|observation| match &observation.current {
            PlatformState::UserPreference { enabled } | PlatformState::Policy { enabled, .. } => {
                Some(*enabled)
            }
            _ => None,
        });
    let detector_status = selected_detector
        .map(|observation| match observation.detector_status {
            DetectorStatus::Successful => "successful",
            DetectorStatus::Unknown => "unknown",
            DetectorStatus::Failed => "failed",
            DetectorStatus::Cancelled => "cancelled",
            DetectorStatus::NotRun => "not_run",
        })
        .unwrap_or("not_run")
        .to_owned();
    let selected_policy_observation = match operation_id {
        Some(crate::mutation::MutationOperationId::WidgetsVisibility) => widgets_policy,
        Some(crate::mutation::MutationOperationId::TaskViewVisibility) => selected_detector,
        _ => None,
    };
    let externally_managed = selected_policy_observation.is_some_and(|observation| {
        observation.policy_state.is_some()
            || matches!(
                observation.authority,
                platform::Authority::DomainPolicy
                    | platform::Authority::Mdm
                    | platform::Authority::LocalPolicy
            )
    });
    let evidence = serde_json::json!({
        "machineId": snapshot.machine_id,
        "build": snapshot.platform.build,
        "edition": snapshot.platform.edition,
        "domainJoined": snapshot.platform.domain_joined,
        "entraJoined": snapshot.platform.entra_joined,
        "workplaceJoined": snapshot.platform.workplace_joined,
        "mdmEnrolled": snapshot.platform.mdm_enrolled,
        "subject": subject,
        "ownerControl": selected_policy_observation.map(|observation| serde_json::json!({
            "policyState": observation.policy_state,
            "authority": observation.authority,
            "applicability": observation.applicability.status,
            "detectorStatus": observation.detector_status,
            "precedence": observation.control_precedence,
        })),
    });
    Ok(BrokerContext {
        machine_id: snapshot.machine_id.clone(),
        inspection_id: snapshot.id.clone(),
        inspection_timestamp: snapshot.timestamp.clone(),
        source_observation_id: format!("{}:{subject}", snapshot.id),
        windows_build: snapshot.platform.build,
        edition: snapshot.platform.edition.clone(),
        architecture: snapshot.platform.architecture.clone(),
        #[cfg(feature = "mutation-alpha")]
        domain_joined: snapshot.platform.domain_joined,
        #[cfg(feature = "mutation-alpha")]
        entra_joined: snapshot.platform.entra_joined,
        #[cfg(feature = "mutation-alpha")]
        workplace_joined: snapshot.platform.workplace_joined,
        #[cfg(feature = "mutation-alpha")]
        mdm_enrolled: snapshot.platform.mdm_enrolled,
        authority: if externally_managed {
            "external_policy".into()
        } else {
            "user".into()
        },
        authority_acceptable: !externally_managed,
        confidence: "confirmed_representation".into(),
        confidence_sufficient: true,
        applicability: if snapshot.platform.is_windows_11 == Some(true)
            && snapshot.platform.build >= 22_000
        {
            "applicable".into()
        } else {
            "unsupported_build".into()
        },
        applicable: snapshot.platform.is_windows_11 == Some(true)
            && snapshot.platform.build >= 22_000,
        evidence_fingerprint: hash_serializable(&evidence)
            .map_err(|error| CommandError::invalid_action(error.to_string()))?,
        desired_state_revision_id: None,
        detector_current_enabled,
        detector_status,
    })
}

#[cfg(feature = "mutation-alpha")]
pub fn run() -> tauri::Result<()> {
    let store = persistence::load();
    let arguments: Vec<String> = std::env::args().collect();
    let command_line_opt_in = arguments
        .iter()
        .any(|argument| argument == "--enable-mutation-alpha");
    let latest = store.snapshots.last();
    let live_validation = build_live_validation_gate(
        latest.map(|snapshot| &snapshot.platform),
        latest.map(|snapshot| snapshot.machine_id.as_str()),
        latest.map(|snapshot| snapshot.id.as_str()),
        &arguments,
    );
    let broker = Arc::new(Broker::new(
        Arc::new(WindowsSettingStore),
        command_line_opt_in,
        live_validation,
    ));
    if let Ok(context) = mutation_context(&store, None) {
        let _ = broker.recover_interrupted(&context);
    }
    let app = tauri::Builder::default()
        .manage(ManagedAppState::new())
        .manage(PlatformSession(Arc::new(Mutex::new(store))))
        .manage(InspectionCoordinator(Mutex::new(None)))
        .manage(MutationSession(broker))
        .invoke_handler(tauri::generate_handler![
            get_app_view,
            dispatch_app_action,
            get_platform_dashboard,
            start_inspection,
            cancel_inspection,
            get_running_inspection_state,
            get_inspection_history,
            get_inspection_detail,
            get_component_observation_timeline,
            get_drift_history,
            acknowledge_drift_event,
            get_detailed_package_observations,
            get_allowed_desired_state_options,
            validate_desired_state,
            save_desired_state,
            clear_desired_state,
            generate_preview_plan,
            get_migration_status,
            get_vm_validation_metadata,
            get_product_info,
            get_product_component_catalogue,
            compare_inspections,
            set_history_retention,
            clear_local_history,
            generate_diagnostics_export,
            get_widgets_actionability,
            get_task_view_actionability,
            apply_owner_operation,
            undo_owner_operation,
            get_owner_change_history,
            get_mutation_alpha_status,
            acknowledge_mutation_alpha_warning,
            get_mutation_operation_options,
            generate_mutation_plan,
            approve_and_execute_mutation,
            rollback_mutation,
            get_mutation_history,
            cancel_mutation_plan,
            export_live_validation_evidence
        ])
        .build(tauri::generate_context!())?;
    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
            let coordinator = handle.state::<InspectionCoordinator>();
            if let Ok(active) = coordinator.0.lock()
                && let Some(inspection) = active.as_ref()
                && !inspection.completed.load(Ordering::SeqCst)
            {
                inspection.token.cancel();
            }
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_update_rust_owned_preview_state() {
        let mut state = AppState::new();

        apply_action(
            &mut state,
            AppAction::SearchChanged {
                query: "copilot".to_owned(),
            },
        )
        .expect("valid search action should be accepted");
        assert_eq!(state.visible_components().len(), 1);

        apply_action(
            &mut state,
            AppAction::TogglePlanned {
                component_id: "copilot".to_owned(),
            },
        )
        .expect("known component should be accepted");
        assert_eq!(state.planned_count(), 1);

        apply_action(&mut state, AppAction::OpenReview).expect("review action should be accepted");
        assert!(state.is_review_open());
    }

    #[test]
    fn invalid_transport_values_are_rejected() {
        let mut state = AppState::new();
        let error = apply_action(
            &mut state,
            AppAction::FilterChanged {
                filter: "everything".to_owned(),
            },
        )
        .expect_err("unknown filters must be rejected");

        assert_eq!(error.code, "invalid_action");
        assert_eq!(state.filter(), CatalogueFilter::All);
    }

    #[test]
    fn product_catalogue_exposes_all_twenty_one_read_only_components() {
        let catalogue = get_product_component_catalogue();
        assert_eq!(catalogue.len(), 21);
        assert!(catalogue.iter().all(|component| !component.name.is_empty()));
    }

    #[test]
    fn owner_command_registration_exposes_only_product_taskbar_operations() {
        let source = include_str!("app.rs");
        let owner = source
            .split("#[cfg(all(feature = \"owner-mode\", not(feature = \"mutation-alpha\")))]\npub fn run()")
            .nth(1)
            .and_then(|value| value.split("#[cfg(feature = \"mutation-alpha\")]").next())
            .expect("owner run block should remain visible to the safety test");
        for required in [
            "get_widgets_actionability",
            "get_task_view_actionability",
            "apply_owner_operation",
            "undo_owner_operation",
            "get_owner_change_history",
        ] {
            assert!(owner.contains(required), "owner block omitted {required}");
        }
        for prohibited in [
            "get_mutation_alpha_status",
            "generate_mutation_plan",
            "approve_and_execute_mutation",
            "rollback_mutation",
            "export_live_validation_evidence",
            "set_taskbar_show_desktop_enabled",
        ] {
            assert!(
                !owner.contains(prohibited),
                "owner block exposed {prohibited}"
            );
        }
    }

    #[test]
    fn diagnostics_contract_omits_machine_identity_and_private_validation_data() {
        let export = build_diagnostics_export(
            &Store::default(),
            &DiagnosticsRequest {
                include_history_summary: true,
                include_redacted_errors: true,
            },
        );
        let serialized = export.to_string().to_ascii_lowercase();
        for prohibited in [
            "machine_id",
            "computername",
            "device_name",
            "development-host-denylist",
            "validation_identity_hash",
            "github",
        ] {
            assert!(
                !serialized.contains(prohibited),
                "diagnostics included {prohibited}"
            );
        }
        assert_eq!(export["capabilities"]["mutation"], false);
        assert_eq!(export["privacy"]["automaticUpload"], false);
        assert_eq!(export["privacy"]["machineIdentityIncluded"], false);
        assert_eq!(export["privacy"]["hostnameIncluded"], false);
        assert_eq!(export["privacy"]["approvalManifestIncluded"], false);
    }
}
