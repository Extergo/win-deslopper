import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type ComponentId = 'onedrive' | 'copilot' | 'promotional_content';
export type CatalogueFilter = 'all' | 'cloud' | 'ai' | 'promotions';
export type NavigationDestination =
  'windows_cleanup' | 'inspection_history' | 'drift_history' | 'extensions';

export type InspectionPhase =
  | 'idle'
  | 'preparing'
  | 'gathering_shared_context'
  | 'running_detectors'
  | 'persisting_results'
  | 'calculating_drift'
  | 'completed'
  | 'completed_with_partial_failures'
  | 'cancelling'
  | 'cancelled'
  | 'failed';

export interface InspectionProgress {
  inspectionId: string;
  phase: InspectionPhase;
  completedWork: number;
  totalWork: number;
  currentComponent: string | null;
  detectorResultStatus: 'successful' | 'unknown' | 'failed' | 'cancelled' | 'not_run' | null;
  warningCount: number;
  errorCount: number;
  cancellationAvailable: boolean;
  status: string;
  updatedAt: string;
}

export interface PackageObservation {
  componentId: string;
  packageFamilyName: string | null;
  packageFullName: string | null;
  packageName: string;
  version: string | null;
  architecture: string | null;
  publisherId: string | null;
  currentUser: string;
  otherUsers: string;
  provisioning: string;
  framework: boolean;
  resourcePackage: boolean;
  bundle: boolean;
  nonRemovable: boolean;
  installLocationPresent: boolean | null;
  dependencies: string[];
  sourceQueryCompleteness: string;
  permissionStatus: string;
  observedAt: string;
}

export interface DetectionObservation {
  componentId: string;
  current: Record<string, unknown>;
  authority: string;
  authorityAttribution: {
    authority: string;
    confidence: string;
    exactSourceProven: boolean;
    evidence: string[];
    alternatives: string[];
  };
  applicable: boolean;
  applicability: { status: string; reason: string; representation: string; source: string };
  evidence: Array<{ queryId: string; source: string; detail: string; confidence: number }>;
  detectedAt: string;
  warnings: string[];
  error: string | null;
  policyState: string | null;
  preferenceState: string | null;
  provisioningState: string | null;
  packageIdentities: string[];
  packageCompleteness: string;
  detectorStatus: string;
  packages: PackageObservation[];
  controlPrecedence: {
    documentedDefault: string | null;
    userPreference: string | null;
    localPolicy: string | null;
    domainPolicy: string | null;
    mdmPolicy: string | null;
    effectiveState: string | null;
    conflictingEvidence: boolean;
  };
}

export interface CleanupItemView {
  id: ComponentId;
  iconLabel: string;
  name: string;
  description: string;
  category: string;
  status: string;
  risk: string;
  restart: string;
  compatibility: string;
  proposedActions: string;
  planned: boolean;
}

export interface AppView {
  cleanupItems: CleanupItemView[];
  plannedItems: CleanupItemView[];
  catalogueCount: number;
  plannedCount: number;
  selectedFilter: CatalogueFilter;
  selectedSection: NavigationDestination;
  reviewOpen: boolean;
}

export interface PlatformDashboard {
  snapshot: {
    timestamp: string;
    platform: { productName: string; edition: string; build: number; architecture: string };
    id: string;
    observations: DetectionObservation[];
    lifecycle: Record<string, unknown> | null;
  } | null;
  desiredCount: number;
  driftCount: number;
  managedCount: number;
  unknownCount: number;
  permissionLimitedCount: number;
  failedCount: number;
  desiredStates: DesiredState[];
  historyRetentionDays: number;
  databaseStatus: {
    healthy: boolean;
    message: string;
    originalPath: string;
    recoveryAvailable: boolean;
    migrationStatus: string;
  };
}

export interface DesiredState {
  component_id: string;
  state: Record<string, unknown>;
  scope: string;
  created_at: string;
  modified_at: string;
  persistent: boolean;
  approval_required: boolean;
  selected_build: number;
  selected_edition: string;
  note: string | null;
  validation_status: string;
  revision: number;
}

export interface InspectionHistoryItem {
  inspectionId: string;
  timestamp: string;
  status: string;
  durationMs: number | null;
  windowsBuild: number;
  windowsEdition: string;
  appVersion: string;
  successfulCount: number;
  unknownCount: number;
  failedCount: number;
  cancelledCount: number;
  driftCount: number;
  managementSummary: string;
}

export interface DriftEvent {
  component_id: string;
  previous: Record<string, unknown>;
  current: Record<string, unknown>;
  desired: Record<string, unknown> | null;
  classification: string;
  cause: string;
  confidence: string;
  first_detected: string;
  last_observed: string;
  resolved: boolean;
  reviewed: boolean;
  reviewed_at: string | null;
  returned_to_desired: boolean;
  occurrence_count: number;
  supporting_facts: string[];
  alternative_causes: string[];
  inference_rule_version: number;
  previous_inspection_id: string | null;
  current_inspection_id: string | null;
}

export interface ProductInfo {
  productName: string;
  version: string;
  releaseLabel: string;
  buildMode: string;
  mutationAvailability: string;
  databaseSchemaVersion: number;
  databaseLocation: string;
  supportedWindows: string;
}

export interface ProductComponent {
  componentId: string;
  name: string;
  category: string;
  purpose: string;
  benefit: string;
  support: string;
  risk: string;
  configuration: string;
  restart: string;
  rollback: string;
  privileges: string;
  gamingNotes: string;
  enterpriseNotes: string;
  documentation: string;
  isPackage: boolean;
}

export interface SnapshotChange {
  componentId: string;
  previousState: Record<string, unknown>;
  currentState: Record<string, unknown>;
  previousStatus: string;
  currentStatus: string;
  explanation: string;
}

export interface SnapshotComparison {
  previousInspectionId: string;
  currentInspectionId: string;
  changes: SnapshotChange[];
}

export interface ComponentTimelineEntry {
  inspectionId: string;
  observationTime: string;
  windowsBuild: number;
  windowsEdition: string;
  observation: DetectionObservation;
  desiredState: DesiredState | null;
  drift: DriftEvent | null;
}

export interface DesiredStateRequest {
  componentId: string;
  stateKey: string;
  scope: string;
  persistentRemediation: boolean;
  alwaysRequireApproval: boolean;
  note: string | null;
}

export interface DesiredStateValidation {
  status: string;
  valid: boolean;
  warnings: string[];
  reason: string;
}

export type MutationOperationId =
  | 'set_taskbar_widgets_visibility'
  | 'set_taskbar_task_view_visibility'
  | 'set_taskbar_show_desktop_enabled';
export type MutationTarget = 'enabled' | 'disabled';

export interface ApprovedMutationOperationScope {
  operationId: MutationOperationId;
  allowedTargetStates: MutationTarget[];
  handlerVersion: string;
}

export interface MutationAlphaStatus {
  compiled: boolean;
  debugBuild: boolean;
  commandLineOptIn: boolean;
  warningAcknowledged: boolean;
  warningText: string;
  liveValidation: {
    commandLineOptIn: boolean;
    governancePolicyLoaded: boolean;
    targetTypePolicyAllowed: boolean;
    manifestLoaded: boolean;
    targetApprovalLoaded: boolean;
    denylistLoaded: boolean;
    scenarioMatches: boolean;
    machineIdentityMatches: boolean;
    checkpointMatches: boolean;
    platformMatches: boolean;
    targetTypeMatches: boolean;
    databaseBelongsToGuest: boolean;
    developmentHostRefused: boolean;
    approvalId: string | null;
    approvalSourceCommit: string | null;
    approvalSourceInspectionId: string | null;
    approvalEvidenceSha256: string | null;
    approvedOperationScopes: ApprovedMutationOperationScope[];
    maximumPlans: number;
    maximumExecutions: number;
    approvalExpiresAtEpochMs: number | null;
    approvedTargetType: 'virtual_machine' | 'physical_laptop' | null;
    rollbackEnvironmentAvailable: boolean;
    localApprovalRevalidationRequired: boolean;
    available: boolean;
    reason: string;
    environment: {
      computerName: string;
      machineIdPrefix: string;
      edition: string;
      build: number;
      updateBuildRevision: number | null;
      virtualMachineDetection: 'detected' | 'not_detected' | 'unknown';
      virtualMachineEvidence: string;
      currentValidationScenario: string | null;
      developmentHostRefused: boolean;
    };
  };
  available: boolean;
  reason: string;
}

export interface MutationCapturedState {
  representation: 'missing' | { dword: number };
  effectiveEnabled: boolean;
  effectiveStateKnown: boolean;
  authority: string;
  confidence: string;
  capturedAt: string;
}

export interface MutationOperationOption {
  definition: {
    operationId: MutationOperationId;
    subjectId: string;
    title: string;
    description: string;
    supportedTargets: MutationTarget[];
    minimumBuild: number;
    supportedEditions: string[];
    requiredAuthority: string;
    requiredConfidence: string;
    requiredPrivilege: string;
    sideEffects: string[];
    restartRequirement: string;
    rollbackMethod: string;
    documentation: string[];
    maximumDurationMs: number;
    automaticRemediationEligible: boolean;
  };
  currentState: MutationCapturedState | null;
  validationMaturity:
    | 'unvalidated'
    | 'synthetic_tested'
    | 'live_tested_single_build'
    | 'live_tested_multi_build'
    | 'live_tested_multi_edition'
    | 'policy_conflict_tested'
    | 'recovery_tested'
    | 'alpha_validated'
    | 'rejected';
  validationLabel: string;
  eligible: boolean;
  reason: string;
}

export type VerificationDimensionResult =
  | 'not_run'
  | 'verified'
  | 'pending_refresh'
  | 'disagrees'
  | 'ignored_by_shell'
  | 'policy_overrode'
  | 'reverted_after_sign_in'
  | 'uncertain';

export interface VisualVerification {
  representation: VerificationDimensionResult;
  detector: VerificationDimensionResult;
  userVisibleBehavior: VerificationDimensionResult;
  settingsUi: VerificationDimensionResult;
  refreshRequirement:
    | 'immediate'
    | 'taskbar_natural_refresh'
    | 'settings_app_reopen'
    | 'deslopper_reopen'
    | 'sign_out_sign_in'
    | 'reboot'
    | 'unsupported_without_explorer_termination'
    | 'undetermined';
  lifecycleRefreshCompleted: boolean;
}

export interface LiveEvidenceBundle {
  scenarioId: string;
  operationId: MutationOperationId;
  build: number;
  visualOutcome: string;
  finalResult: string;
}

export interface MutationPlan {
  planId: string;
  sourceInspectionId: string;
  subjectId: string;
  operationId: MutationOperationId;
  targetState: MutationTarget;
  currentState: MutationCapturedState;
  authority: string;
  applicability: string;
  generatedAt: string;
  expiresAt: string;
  requiredPrivilege: string;
  expectedSideEffects: string[];
  restartRequirement: string;
  rollbackMethod: string;
  handlerVersion: string;
  automaticRemediationEligible: boolean;
  documentation: string[];
  planHash: string;
}

export interface IssuedMutationPlan {
  plan: MutationPlan;
  approvalNonce: string;
  confirmationText: string;
  approvalPhrase: string;
  proposedRepresentation: MutationCapturedState['representation'];
}

export interface MutationTransaction {
  transactionId: string;
  planId: string;
  subjectId: string;
  operationId: MutationOperationId;
  createdAt: string;
  approvedAt: string | null;
  startedAt: string | null;
  completedAt: string | null;
  status: string;
  requiredPrivilege: string;
  handlerVersion: string;
  applicationVersion: string;
  windowsBuild: number;
  edition: string;
  targetState: MutationTarget;
  preState: MutationCapturedState | null;
  postState: MutationCapturedState | null;
  rollbackState: MutationCapturedState | null;
  verificationResult: string | null;
  errorCategory: string | null;
  errorSummary: string | null;
  recoveryRequirement: string | null;
  steps: Array<{
    sequence: number;
    stepType: string;
    startedAt: string;
    completedAt: string | null;
    status: string;
    redactedEvidence: string[];
    errorCategory: string | null;
    errorSummary: string | null;
  }>;
  rollback: {
    available: boolean;
    complete: boolean;
    attemptedAt: string | null;
    result: string | null;
    verificationResult: string | null;
    conflictDetected: boolean;
  };
}

export type AppAction =
  | { type: 'search_changed'; query: string }
  | { type: 'filter_changed'; filter: CatalogueFilter }
  | { type: 'toggle_planned'; componentId: ComponentId }
  | { type: 'navigate'; destination: NavigationDestination }
  | { type: 'open_review' }
  | { type: 'close_review' };

export interface CommandError {
  code: string;
  message: string;
}

export type BackendInvoker = (command: string, args?: Record<string, unknown>) => Promise<unknown>;
export type BackendListener = (
  event: string,
  handler: (payload: unknown) => void
) => Promise<UnlistenFn>;

export interface BackendClient {
  getAppView(): Promise<AppView>;
  dispatch(action: AppAction): Promise<AppView>;
  getPlatformDashboard(): Promise<PlatformDashboard>;
  startInspection(): Promise<InspectionProgress>;
  cancelInspection(inspectionId: string): Promise<InspectionProgress>;
  getRunningInspectionState(): Promise<InspectionProgress | null>;
  subscribeInspectionProgress(handler: (progress: InspectionProgress) => void): Promise<UnlistenFn>;
  getInspectionHistory(): Promise<InspectionHistoryItem[]>;
  getInspectionDetail(inspectionId: string): Promise<PlatformDashboard['snapshot']>;
  getComponentTimeline(componentId: string): Promise<ComponentTimelineEntry[]>;
  getDriftHistory(): Promise<DriftEvent[]>;
  getPackageObservations(inspectionId: string, componentId?: string): Promise<PackageObservation[]>;
  getAllowedDesiredStateOptions(
    componentId: string
  ): Promise<Array<{ key: string; label: string; scope: string; description: string }>>;
  validateDesiredState(request: DesiredStateRequest): Promise<DesiredStateValidation>;
  saveDesiredState(request: DesiredStateRequest): Promise<PlatformDashboard>;
  clearDesiredState(componentId: string): Promise<PlatformDashboard>;
  generatePreviewPlan(componentId: string): Promise<Record<string, unknown>>;
  getProductInfo(): Promise<ProductInfo>;
  getProductComponentCatalogue(): Promise<ProductComponent[]>;
  compareInspections(
    previousInspectionId: string,
    currentInspectionId: string
  ): Promise<SnapshotComparison>;
  acknowledgeDriftEvent(componentId: string, classification: string): Promise<DriftEvent[]>;
  setHistoryRetention(days: number): Promise<PlatformDashboard>;
  clearLocalHistory(confirmed: boolean): Promise<PlatformDashboard>;
  generateDiagnosticsExport(request: {
    includeHistorySummary: boolean;
    includeRedactedErrors: boolean;
  }): Promise<Record<string, unknown>>;
  getMutationAlphaStatus(): Promise<MutationAlphaStatus>;
  acknowledgeMutationAlphaWarning(acknowledged: boolean): Promise<MutationAlphaStatus>;
  getMutationOperationOptions(): Promise<MutationOperationOption[]>;
  generateMutationPlan(
    operationId: MutationOperationId,
    target: MutationTarget,
    sourceInspectionId: string
  ): Promise<IssuedMutationPlan>;
  executeMutation(
    planId: string,
    approvalNonce: string,
    acknowledged: boolean
  ): Promise<MutationTransaction>;
  rollbackMutation(
    transactionId: string,
    acknowledged: boolean,
    allowConflict: boolean
  ): Promise<MutationTransaction>;
  getMutationHistory(): Promise<MutationTransaction[]>;
  cancelMutationPlan(planId: string): Promise<MutationTransaction>;
  exportLiveValidationEvidence(
    transactionId: string,
    visualVerification: VisualVerification,
    screenshotLabels: string[],
    warnings: string[]
  ): Promise<LiveEvidenceBundle>;
}

const tauriInvoker: BackendInvoker = (command, args) => invoke<unknown>(command, args);
const tauriListener: BackendListener = async (event, handler) =>
  await listen<unknown>(event, (message) => handler(message.payload));

export function createBackendClient(
  invokeCommand: BackendInvoker = tauriInvoker,
  listenForEvent: BackendListener = tauriListener
): BackendClient {
  return {
    getAppView: async () => (await invokeCommand('get_app_view')) as AppView,
    dispatch: async (action) => (await invokeCommand('dispatch_app_action', { action })) as AppView,
    getPlatformDashboard: async () =>
      (await invokeCommand('get_platform_dashboard')) as PlatformDashboard,
    startInspection: async () => (await invokeCommand('start_inspection')) as InspectionProgress,
    cancelInspection: async (inspectionId) =>
      (await invokeCommand('cancel_inspection', { inspectionId })) as InspectionProgress,
    getRunningInspectionState: async () =>
      (await invokeCommand('get_running_inspection_state')) as InspectionProgress | null,
    subscribeInspectionProgress: async (handler) =>
      await listenForEvent('deslopper://inspection-progress', (payload) =>
        handler(payload as InspectionProgress)
      ),
    getInspectionHistory: async () =>
      (await invokeCommand('get_inspection_history')) as InspectionHistoryItem[],
    getInspectionDetail: async (inspectionId) =>
      (await invokeCommand('get_inspection_detail', {
        inspectionId
      })) as PlatformDashboard['snapshot'],
    getComponentTimeline: async (componentId) =>
      (await invokeCommand('get_component_observation_timeline', {
        componentId
      })) as ComponentTimelineEntry[],
    getDriftHistory: async () => (await invokeCommand('get_drift_history')) as DriftEvent[],
    getPackageObservations: async (inspectionId, componentId) =>
      (await invokeCommand('get_detailed_package_observations', {
        inspectionId,
        componentId
      })) as PackageObservation[],
    getAllowedDesiredStateOptions: async (componentId) =>
      (await invokeCommand('get_allowed_desired_state_options', { componentId })) as Array<{
        key: string;
        label: string;
        scope: string;
        description: string;
      }>,
    validateDesiredState: async (request) =>
      (await invokeCommand('validate_desired_state', { request })) as DesiredStateValidation,
    saveDesiredState: async (request) =>
      (await invokeCommand('save_desired_state', { request })) as PlatformDashboard,
    clearDesiredState: async (componentId) =>
      (await invokeCommand('clear_desired_state', { componentId })) as PlatformDashboard,
    generatePreviewPlan: async (componentId) =>
      (await invokeCommand('generate_preview_plan', { componentId })) as Record<string, unknown>,
    getProductInfo: async () => (await invokeCommand('get_product_info')) as ProductInfo,
    getProductComponentCatalogue: async () =>
      (await invokeCommand('get_product_component_catalogue')) as ProductComponent[],
    compareInspections: async (previousInspectionId, currentInspectionId) =>
      (await invokeCommand('compare_inspections', {
        previousInspectionId,
        currentInspectionId
      })) as SnapshotComparison,
    acknowledgeDriftEvent: async (componentId, classification) =>
      (await invokeCommand('acknowledge_drift_event', {
        componentId,
        classification
      })) as DriftEvent[],
    setHistoryRetention: async (days) =>
      (await invokeCommand('set_history_retention', { days })) as PlatformDashboard,
    clearLocalHistory: async (confirmed) =>
      (await invokeCommand('clear_local_history', { confirmed })) as PlatformDashboard,
    generateDiagnosticsExport: async (request) =>
      (await invokeCommand('generate_diagnostics_export', { request })) as Record<string, unknown>,
    getMutationAlphaStatus: async () =>
      (await invokeCommand('get_mutation_alpha_status')) as MutationAlphaStatus,
    acknowledgeMutationAlphaWarning: async (acknowledged) =>
      (await invokeCommand('acknowledge_mutation_alpha_warning', {
        acknowledged
      })) as MutationAlphaStatus,
    getMutationOperationOptions: async () =>
      (await invokeCommand('get_mutation_operation_options')) as MutationOperationOption[],
    generateMutationPlan: async (operationId, target, sourceInspectionId) =>
      (await invokeCommand('generate_mutation_plan', {
        request: { operationId, target, sourceInspectionId }
      })) as IssuedMutationPlan,
    executeMutation: async (planId, approvalNonce, acknowledged) =>
      (await invokeCommand('approve_and_execute_mutation', {
        request: { planId, approvalNonce, acknowledged }
      })) as MutationTransaction,
    rollbackMutation: async (transactionId, acknowledged, allowConflict) =>
      (await invokeCommand('rollback_mutation', {
        request: { transactionId, acknowledged, allowConflict }
      })) as MutationTransaction,
    getMutationHistory: async () =>
      (await invokeCommand('get_mutation_history')) as MutationTransaction[],
    cancelMutationPlan: async (planId) =>
      (await invokeCommand('cancel_mutation_plan', { planId })) as MutationTransaction,
    exportLiveValidationEvidence: async (
      transactionId,
      visualVerification,
      screenshotLabels,
      warnings
    ) =>
      (await invokeCommand('export_live_validation_evidence', {
        request: { transactionId, visualVerification, screenshotLabels, warnings }
      })) as LiveEvidenceBundle
  };
}

export function describeCommandError(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }

  if (typeof error === 'string' && error.trim().length > 0) {
    return error;
  }

  return 'The local preview could not be updated. Please try again.';
}
