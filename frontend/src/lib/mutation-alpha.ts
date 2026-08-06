import type {
  BackendClient,
  IssuedMutationPlan,
  LiveEvidenceBundle,
  MutationAlphaStatus,
  MutationCapturedState,
  MutationOperationId,
  MutationOperationOption,
  MutationTarget,
  MutationTransaction,
  VisualVerification
} from './backend';

export const MUTATION_ALPHA_BUILD_MODE = 'internal mutation-alpha compile';

export type MutationPhase =
  | 'idle'
  | 'plan_ready'
  | 'awaiting_approval'
  | 'approved'
  | 'executing'
  | 'verifying'
  | 'applied'
  | 'no_change_needed'
  | 'rolling_back'
  | 'rolled_back'
  | 'failed'
  | 'recovery_required';

export type VisualReport = 'changed_as_expected' | 'did_not_change';

export interface MutationUiError {
  summary: string;
  technical: string;
  code: string;
}

type MutationBackend = Pick<
  BackendClient,
  | 'getMutationAlphaStatus'
  | 'acknowledgeMutationAlphaWarning'
  | 'getMutationOperationOptions'
  | 'generateMutationPlan'
  | 'executeMutation'
  | 'rollbackMutation'
  | 'getMutationHistory'
  | 'cancelMutationPlan'
  | 'exportLiveValidationEvidence'
>;

export function isMutationAlphaBuild(buildMode: string): boolean {
  return buildMode === MUTATION_ALPHA_BUILD_MODE;
}

export function authorizedChoices(
  options: MutationOperationOption[]
): Array<{ operationId: MutationOperationId; target: MutationTarget; title: string }> {
  return options.flatMap((option) =>
    option.definition.supportedTargets.map((target) => ({
      operationId: option.definition.operationId,
      target,
      title: option.definition.title
    }))
  );
}

export function representationText(
  representation: MutationCapturedState['representation'] | undefined
): string {
  if (representation === undefined) return 'Not captured';
  if (representation === 'missing') return 'Value absent';
  return `DWORD ${representation.dword}`;
}

export function effectiveStateText(state: MutationCapturedState | null | undefined): string {
  if (!state?.effectiveStateKnown) return 'Unknown (value absent or unsupported)';
  return state.effectiveEnabled ? 'Enabled' : 'Disabled';
}

export function rollbackText(state: MutationCapturedState | null | undefined): string {
  if (state?.representation === 'missing') {
    return 'Undo will restore this value to absence, not write DWORD 0.';
  }
  return `Undo will restore the exact captured representation: ${representationText(state?.representation)}.`;
}

export function redactIdentifier(value: string): string {
  if (value.length <= 12) return `${value.slice(0, 3)}…${value.slice(-3)}`;
  return `${value.slice(0, 6)}…${value.slice(-6)}`;
}

export function formatEpoch(value: string | number | null): string {
  if (value === null) return 'Not available';
  const parsed = typeof value === 'number' ? value : Number(value);
  if (!Number.isFinite(parsed)) return 'Invalid timestamp';
  return new Date(parsed).toLocaleString();
}

export function validateIssuedPlan(
  issued: IssuedMutationPlan,
  options: MutationOperationOption[],
  now = Date.now()
): string | null {
  const possibleOperations = (issued.plan as MutationPlanWithPossibleOperations).operations;
  if (
    possibleOperations !== undefined &&
    (!Array.isArray(possibleOperations) || possibleOperations.length !== 1)
  ) {
    return 'The plan was rejected because it does not contain exactly one operation.';
  }
  const authorized = options.find(
    (option) =>
      option.eligible &&
      option.definition.operationId === issued.plan.operationId &&
      option.definition.supportedTargets.includes(issued.plan.targetState)
  );
  if (!authorized) return 'The plan contains an operation or target not returned by the backend.';
  const expiresAt = Number(issued.plan.expiresAt);
  if (!Number.isFinite(expiresAt) || expiresAt <= now)
    return 'The plan expired. Generate a fresh plan.';
  if (issued.plan.requiredPrivilege !== 'current_user_unprivileged') {
    return 'The plan unexpectedly requires elevation and cannot continue.';
  }
  const restart = issued.plan.restartRequirement.toLowerCase();
  if (restart.includes('explorer termination') && !restart.includes('no explorer termination')) {
    return 'The plan unexpectedly requires Explorer termination and cannot continue.';
  }
  const executableText = [
    ...issued.plan.expectedSideEffects,
    issued.plan.rollbackMethod,
    issued.confirmationText
  ].join(' ');
  if (
    /powershell|command interpreter|registry tool|arbitrary path|generic registry|shell command|execute script/i.test(
      executableText
    )
  ) {
    return 'The plan contains a generic registry or shell step and cannot continue.';
  }
  if (issued.plan.automaticRemediationEligible) {
    return 'The plan unexpectedly enables automatic repair and cannot continue.';
  }
  if (!issued.approvalPhrase || !isClosedRepresentation(issued.proposedRepresentation)) {
    return 'The plan is missing backend-owned approval or representation details.';
  }
  return null;
}

export function phaseFromTransaction(transaction: MutationTransaction): MutationPhase {
  switch (transaction.status) {
    case 'approved':
      return 'approved';
    case 'capturing_pre_state':
    case 'applying':
      return 'executing';
    case 'verifying':
    case 'applied':
      return 'verifying';
    case 'rollback_available':
      return 'applied';
    case 'no_change_needed':
      return 'no_change_needed';
    case 'rolling_back':
      return 'rolling_back';
    case 'rolled_back':
      return 'rolled_back';
    case 'recovery_required':
      return 'recovery_required';
    case 'cancelled_before_mutation':
      return 'idle';
    case 'verification_failed':
    case 'rollback_verification_failed':
    case 'failed_before_mutation':
    case 'failed_after_mutation':
      return 'failed';
    default:
      return 'awaiting_approval';
  }
}

export function canUndo(transaction: MutationTransaction | null): boolean {
  return Boolean(
    transaction &&
    transaction.rollback.available &&
    !transaction.rollback.complete &&
    ['rollback_available', 'verification_failed', 'recovery_required'].includes(transaction.status)
  );
}

export function mutationError(error: unknown): MutationUiError {
  const code = readString(error, 'code') ?? 'backend_unavailable';
  const message = readString(error, 'message') ?? (typeof error === 'string' ? error : '');
  const summary = errorSummary(code);
  return {
    code,
    summary,
    technical: message || 'The Mutation Alpha backend did not return a typed error.'
  };
}

export class MutationAlphaWorkflow {
  status: MutationAlphaStatus | null = null;
  options: MutationOperationOption[] = [];
  history: MutationTransaction[] = [];
  issuedPlan: IssuedMutationPlan | null = null;
  transaction: MutationTransaction | null = null;
  phase: MutationPhase = 'idle';
  approvalInput = '';
  visualReport: VisualReport | null = null;
  evidence: LiveEvidenceBundle | null = null;
  error: MutationUiError | null = null;
  busy = false;
  planConsumed = false;

  constructor(
    private readonly backend: MutationBackend,
    private readonly now: () => number = Date.now
  ) {}

  async initialise(buildMode: string): Promise<void> {
    if (!isMutationAlphaBuild(buildMode)) return;
    this.busy = true;
    this.error = null;
    try {
      [this.status, this.history] = await Promise.all([
        this.backend.getMutationAlphaStatus(),
        this.backend.getMutationHistory()
      ]);
      const latest = this.history[0];
      if (latest) {
        this.transaction = latest;
        this.phase = phaseFromTransaction(latest);
        this.planConsumed = latest.status !== 'awaiting_approval';
      }
      if (this.canRequestOptions()) this.options = await this.backend.getMutationOperationOptions();
    } catch (error) {
      this.error = mutationError(error);
    } finally {
      this.busy = false;
    }
  }

  async acknowledgeWarning(): Promise<void> {
    this.busy = true;
    this.error = null;
    try {
      this.status = await this.backend.acknowledgeMutationAlphaWarning(true);
      this.options = this.canRequestOptions()
        ? await this.backend.getMutationOperationOptions()
        : [];
    } catch (error) {
      this.error = mutationError(error);
    } finally {
      this.busy = false;
    }
  }

  async createPlan(
    operationId: MutationOperationId,
    target: MutationTarget,
    sourceInspectionId: string | null
  ): Promise<void> {
    this.error = null;
    const option = this.options.find(
      (candidate) =>
        candidate.eligible &&
        candidate.definition.operationId === operationId &&
        candidate.definition.supportedTargets.includes(target)
    );
    if (!option) {
      this.error = localError(
        'operation_target_not_approved',
        'That operation is not in the backend-authorized option set.'
      );
      return;
    }
    if (!sourceInspectionId) {
      this.error = localError(
        'stale_inspection',
        'Run a fresh read-only inspection before creating a plan.'
      );
      return;
    }
    this.busy = true;
    try {
      const issued = await this.backend.generateMutationPlan(
        operationId,
        target,
        sourceInspectionId
      );
      const rejection = validateIssuedPlan(issued, this.options, this.now());
      if (rejection) {
        this.error = localError('plan_rejected', rejection);
        return;
      }
      this.issuedPlan = issued;
      this.transaction = null;
      this.approvalInput = '';
      this.planConsumed = false;
      this.phase = 'plan_ready';
      this.phase = 'awaiting_approval';
      await this.refreshHistory();
    } catch (error) {
      this.error = mutationError(error);
      this.phase = 'failed';
    } finally {
      this.busy = false;
    }
  }

  approvalMatches(): boolean {
    return Boolean(
      this.issuedPlan && !this.planConsumed && this.approvalInput === this.issuedPlan.approvalPhrase
    );
  }

  async execute(): Promise<void> {
    if (!this.issuedPlan || !this.approvalMatches()) {
      this.error = localError(
        'approval_required',
        'Type the exact backend-provided approval phrase before execution.'
      );
      return;
    }
    const rejection = validateIssuedPlan(this.issuedPlan, this.options, this.now());
    if (rejection) {
      this.error = localError('plan_rejected', rejection);
      return;
    }
    this.error = null;
    this.busy = true;
    this.phase = 'approved';
    const issued = this.issuedPlan;
    this.phase = 'executing';
    try {
      this.transaction = await this.backend.executeMutation(
        issued.plan.planId,
        issued.approvalNonce,
        true
      );
      this.planConsumed = true;
      this.approvalInput = '';
      this.phase = phaseFromTransaction(this.transaction);
      await this.refreshHistory();
    } catch (error) {
      this.error = mutationError(error);
      await this.refreshHistorySafely(issued.plan.planId);
      if (!this.transaction) this.phase = 'failed';
    } finally {
      this.busy = false;
    }
  }

  async cancelPlan(): Promise<void> {
    if (!this.issuedPlan || this.planConsumed) return;
    this.busy = true;
    this.error = null;
    try {
      this.transaction = await this.backend.cancelMutationPlan(this.issuedPlan.plan.planId);
      this.planConsumed = true;
      this.phase = phaseFromTransaction(this.transaction);
      await this.refreshHistory();
    } catch (error) {
      this.error = mutationError(error);
    } finally {
      this.busy = false;
    }
  }

  async refreshCurrentTransaction(): Promise<void> {
    if (!this.issuedPlan) return;
    await this.refreshHistorySafely(this.issuedPlan.plan.planId);
  }

  recordVisualReport(report: VisualReport): void {
    if (this.phase !== 'applied') return;
    this.visualReport = report;
  }

  async undo(): Promise<void> {
    if (!canUndo(this.transaction)) return;
    const transactionId = this.transaction?.transactionId;
    if (!transactionId) return;
    this.busy = true;
    this.error = null;
    this.phase = 'rolling_back';
    try {
      this.transaction = await this.backend.rollbackMutation(transactionId, true, false);
      this.phase = phaseFromTransaction(this.transaction);
      await this.refreshHistory();
    } catch (error) {
      this.error = mutationError(error);
      await this.refreshHistorySafely(this.issuedPlan?.plan.planId ?? '');
      if (this.phase === 'rolling_back') this.phase = 'failed';
    } finally {
      this.busy = false;
    }
  }

  async exportVisualEvidence(): Promise<void> {
    const noChangeNeeded = this.transaction?.status === 'no_change_needed';
    if (
      !this.transaction ||
      (!noChangeNeeded && this.transaction.status !== 'rolled_back') ||
      (!noChangeNeeded && !this.visualReport) ||
      this.evidence
    ) {
      return;
    }
    this.busy = true;
    this.error = null;
    try {
      this.evidence = await this.backend.exportLiveValidationEvidence(
        this.transaction.transactionId,
        visualVerification(this.transaction, this.visualReport),
        [],
        [
          noChangeNeeded
            ? 'No write was needed because the exact target representation was already present.'
            : this.visualReport === 'changed_as_expected'
              ? 'User reported that the taskbar changed as expected.'
              : 'User reported that the taskbar did not change as expected.'
        ]
      );
    } catch (error) {
      this.error = mutationError(error);
    } finally {
      this.busy = false;
    }
  }

  private canRequestOptions(): boolean {
    return Boolean(
      this.status?.available &&
      !this.status.liveValidation.developmentHostRefused &&
      this.status.liveValidation.targetApprovalLoaded
    );
  }

  private async refreshHistory(): Promise<void> {
    this.history = await this.backend.getMutationHistory();
    const planId = this.issuedPlan?.plan.planId;
    if (!planId) return;
    const current = this.history.find((candidate) => candidate.planId === planId);
    if (current) {
      this.transaction = current;
      this.phase = phaseFromTransaction(current);
    }
  }

  private async refreshHistorySafely(planId: string): Promise<void> {
    try {
      this.history = await this.backend.getMutationHistory();
      const current = this.history.find((candidate) => candidate.planId === planId);
      if (current) {
        this.transaction = current;
        this.planConsumed = current.status !== 'awaiting_approval';
        this.phase = phaseFromTransaction(current);
      }
    } catch {
      // Preserve the original mutation error; polling is informational only.
    }
  }
}

interface MutationPlanWithPossibleOperations {
  operations?: unknown[];
}

function isClosedRepresentation(representation: MutationCapturedState['representation']): boolean {
  return (
    representation === 'missing' ||
    (typeof representation === 'object' &&
      representation !== null &&
      Number.isInteger(representation.dword) &&
      (representation.dword === 0 || representation.dword === 1))
  );
}

function visualVerification(
  transaction: MutationTransaction,
  report: VisualReport | null
): VisualVerification {
  if (transaction.status === 'no_change_needed') {
    return {
      representation: 'verified',
      detector: 'not_run',
      userVisibleBehavior: 'not_run',
      settingsUi: 'not_run',
      refreshRequirement: 'undetermined',
      lifecycleRefreshCompleted: false
    };
  }
  const backendVerified = transaction.verificationResult === 'applied_and_verified';
  return {
    representation: backendVerified ? 'verified' : 'uncertain',
    detector: backendVerified ? 'verified' : 'uncertain',
    userVisibleBehavior: report === 'changed_as_expected' ? 'verified' : 'ignored_by_shell',
    settingsUi: 'not_run',
    refreshRequirement: 'undetermined',
    lifecycleRefreshCompleted: false
  };
}

function errorSummary(code: string): string {
  if (code.includes('development_host') || code.includes('denylist')) {
    return 'This machine is blocked by development-host protection.';
  }
  if (code.includes('expired')) return 'The approval or plan expired.';
  if (code.includes('limit_exhausted')) return 'The approval allowance is exhausted.';
  if (code.includes('stale_inspection')) return 'The source inspection is stale.';
  if (code.includes('changed') || code.includes('context_mismatch')) {
    return 'The inspected state changed, so the broker stopped.';
  }
  if (code.includes('policy')) return 'Windows policy blocks this operation.';
  if (code.includes('rollback')) return 'Undo did not complete safely.';
  if (code.includes('verification')) return 'The change could not be verified.';
  if (code.includes('recovery')) return 'Manual recovery review is required.';
  if (code.includes('approval') || code.includes('manifest')) {
    return 'The target is not currently authorized.';
  }
  if (code.includes('plan')) return 'The broker rejected the plan.';
  if (code.includes('mutation_alpha')) return 'Mutation Alpha is unavailable.';
  return 'The local Mutation Alpha backend could not complete that action.';
}

function localError(code: string, technical: string): MutationUiError {
  return { code, summary: errorSummary(code), technical };
}

function readString(value: unknown, key: string): string | null {
  if (typeof value !== 'object' || value === null || !(key in value)) return null;
  const candidate = (value as Record<string, unknown>)[key];
  return typeof candidate === 'string' ? candidate : null;
}
