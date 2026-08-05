import { describe, expect, it, vi } from 'vitest';

import type {
  BackendClient,
  IssuedMutationPlan,
  MutationAlphaStatus,
  MutationOperationOption,
  MutationTransaction
} from './backend';
import {
  MUTATION_ALPHA_BUILD_MODE,
  MutationAlphaWorkflow,
  authorizedChoices,
  canUndo,
  isMutationAlphaBuild,
  mutationError,
  phaseFromTransaction,
  representationText,
  rollbackText,
  validateIssuedPlan
} from './mutation-alpha';

const now = 1_800_000_000_000;

function status(overrides: Partial<MutationAlphaStatus> = {}): MutationAlphaStatus {
  return {
    compiled: true,
    debugBuild: true,
    commandLineOptIn: true,
    warningAcknowledged: true,
    warningText: 'Acknowledgement is not authorization.',
    liveValidation: {
      commandLineOptIn: true,
      governancePolicyLoaded: true,
      targetTypePolicyAllowed: true,
      manifestLoaded: true,
      targetApprovalLoaded: true,
      denylistLoaded: true,
      scenarioMatches: true,
      machineIdentityMatches: true,
      checkpointMatches: true,
      platformMatches: true,
      targetTypeMatches: true,
      databaseBelongsToGuest: true,
      developmentHostRefused: false,
      approvalId: 'redacted-in-ui',
      approvalSourceCommit: null,
      approvalSourceInspectionId: 'inspection-1',
      approvalEvidenceSha256: null,
      approvedOperationScopes: [
        {
          operationId: 'set_taskbar_widgets_visibility',
          allowedTargetStates: ['enabled'],
          handlerVersion: '1'
        }
      ],
      maximumPlans: 1,
      maximumExecutions: 1,
      approvalExpiresAtEpochMs: now + 60_000,
      approvedTargetType: 'physical_laptop',
      rollbackEnvironmentAvailable: true,
      localApprovalRevalidationRequired: true,
      available: true,
      reason: 'Approved target is eligible.',
      environment: {
        computerName: 'never-render-this',
        machineIdPrefix: 'never-render-this',
        edition: 'Professional',
        build: 26_200,
        updateBuildRevision: 1,
        virtualMachineDetection: 'not_detected',
        virtualMachineEvidence: 'informational',
        currentValidationScenario: 'scenario',
        developmentHostRefused: false
      }
    },
    available: true,
    reason: 'Mutation alpha gates are satisfied.',
    ...overrides
  };
}

function widgetsEnabledOption(): MutationOperationOption {
  return {
    definition: {
      operationId: 'set_taskbar_widgets_visibility',
      subjectId: 'widgets',
      title: 'Widgets button visibility',
      description: 'Show only the current user Widgets button.',
      supportedTargets: ['enabled'],
      minimumBuild: 22_000,
      supportedEditions: ['Professional'],
      requiredAuthority: 'user',
      requiredConfidence: 'confirmed_representation',
      requiredPrivilege: 'current_user_unprivileged',
      sideEffects: ['Taskbar presentation updates.'],
      restartRequirement: 'No Explorer termination; shell refresh may be asynchronous.',
      rollbackMethod: 'Restore the exact captured TaskbarDa DWORD or value absence.',
      documentation: [],
      maximumDurationMs: 5_000,
      automaticRemediationEligible: false
    },
    currentState: {
      representation: 'missing',
      effectiveEnabled: false,
      authority: 'user',
      confidence: 'confirmed_representation',
      capturedAt: String(now)
    },
    validationMaturity: 'synthetic_tested',
    validationLabel: 'Internal alpha - not live validated.',
    eligible: true,
    reason: 'A fresh reviewed plan may be generated.'
  };
}

function issuedPlan(): IssuedMutationPlan {
  return {
    plan: {
      planId: 'mutation-plan-1234567890',
      sourceInspectionId: 'inspection-1',
      subjectId: 'widgets',
      operationId: 'set_taskbar_widgets_visibility',
      targetState: 'enabled',
      currentState: {
        representation: 'missing',
        effectiveEnabled: false,
        authority: 'user',
        confidence: 'confirmed_representation',
        capturedAt: String(now)
      },
      authority: 'user',
      applicability: 'applicable',
      generatedAt: String(now),
      expiresAt: String(now + 60_000),
      requiredPrivilege: 'current_user_unprivileged',
      expectedSideEffects: ['Taskbar presentation updates.'],
      restartRequirement: 'No Explorer termination; shell refresh may be asynchronous.',
      rollbackMethod: 'Restore the exact captured TaskbarDa DWORD or value absence.',
      handlerVersion: '1',
      automaticRemediationEligible: false,
      documentation: [],
      planHash: 'plan-hash'
    },
    approvalNonce: 'nonce-1234567890',
    confirmationText: 'Show Widgets and retain the exact previous representation.',
    approvalPhrase: 'APPROVE WIDGETS TEST',
    proposedRepresentation: { dword: 1 }
  };
}

function transaction(overrides: Partial<MutationTransaction> = {}): MutationTransaction {
  return {
    transactionId: 'transaction-1',
    planId: 'mutation-plan-1234567890',
    subjectId: 'widgets',
    operationId: 'set_taskbar_widgets_visibility',
    createdAt: String(now),
    approvedAt: String(now),
    startedAt: String(now),
    completedAt: String(now + 1),
    status: 'rollback_available',
    requiredPrivilege: 'current_user_unprivileged',
    handlerVersion: '1',
    applicationVersion: '0.1.0',
    windowsBuild: 26_200,
    edition: 'Professional',
    targetState: 'enabled',
    preState: widgetsEnabledOption().currentState,
    postState: {
      representation: { dword: 1 },
      effectiveEnabled: true,
      authority: 'user',
      confidence: 'confirmed_representation',
      capturedAt: String(now + 1)
    },
    rollbackState: null,
    verificationResult: 'applied_and_verified',
    errorCategory: null,
    errorSummary: null,
    recoveryRequirement: null,
    steps: [
      {
        sequence: 1,
        stepType: 'validate_plan',
        startedAt: String(now),
        completedAt: String(now),
        status: 'validating',
        redactedEvidence: [],
        errorCategory: null,
        errorSummary: null
      },
      {
        sequence: 2,
        stepType: 'effective_reinspection_verified',
        startedAt: String(now),
        completedAt: String(now + 1),
        status: 'rollback_available',
        redactedEvidence: [],
        errorCategory: null,
        errorSummary: null
      }
    ],
    rollback: {
      available: true,
      complete: false,
      attemptedAt: null,
      result: null,
      verificationResult: null,
      conflictDetected: false
    },
    ...overrides
  };
}

function fakeBackend() {
  const value = {
    getMutationAlphaStatus: vi.fn().mockResolvedValue(status()),
    acknowledgeMutationAlphaWarning: vi.fn().mockResolvedValue(status()),
    getMutationOperationOptions: vi.fn().mockResolvedValue([widgetsEnabledOption()]),
    generateMutationPlan: vi.fn().mockResolvedValue(issuedPlan()),
    executeMutation: vi.fn().mockResolvedValue(transaction()),
    rollbackMutation: vi.fn().mockResolvedValue(
      transaction({
        status: 'rolled_back',
        rollbackState: widgetsEnabledOption().currentState,
        rollback: {
          available: false,
          complete: true,
          attemptedAt: String(now + 2),
          result: 'rolled_back',
          verificationResult: 'exact_pre_state_restored',
          conflictDetected: false
        }
      })
    ),
    getMutationHistory: vi.fn().mockResolvedValue([]),
    cancelMutationPlan: vi
      .fn()
      .mockResolvedValue(transaction({ status: 'cancelled_before_mutation' })),
    exportLiveValidationEvidence: vi.fn().mockResolvedValue({
      scenarioId: 'scenario',
      operationId: 'set_taskbar_widgets_visibility',
      build: 26_200,
      visualOutcome: 'fully_verified',
      finalResult: 'passed'
    })
  };
  return value as typeof value & BackendClient;
}

describe('Mutation Alpha feature boundary and option scope', () => {
  it('instantiates only for the exact internal build metadata', async () => {
    const backend = fakeBackend();
    const workflow = new MutationAlphaWorkflow(backend, () => now);
    await workflow.initialise('normal read-only');
    expect(isMutationAlphaBuild('normal read-only')).toBe(false);
    expect(isMutationAlphaBuild(MUTATION_ALPHA_BUILD_MODE)).toBe(true);
    expect(backend.getMutationAlphaStatus).not.toHaveBeenCalled();
    expect(backend.getMutationOperationOptions).not.toHaveBeenCalled();
  });

  it('shows a denied development host as blocked without requesting options', async () => {
    const backend = fakeBackend();
    backend.getMutationAlphaStatus.mockResolvedValue(
      status({
        available: false,
        reason: 'The development host is permanently denied.',
        liveValidation: {
          ...status().liveValidation,
          developmentHostRefused: true,
          available: false,
          reason: 'Development host denied.'
        }
      })
    );
    const workflow = new MutationAlphaWorkflow(backend, () => now);
    await workflow.initialise(MUTATION_ALPHA_BUILD_MODE);
    expect(workflow.status?.liveValidation.developmentHostRefused).toBe(true);
    expect(workflow.options).toEqual([]);
    expect(backend.getMutationOperationOptions).not.toHaveBeenCalled();
  });

  it('keeps missing approval blocked and renders only the returned Widgets Enabled scope', async () => {
    const noApprovalBackend = fakeBackend();
    noApprovalBackend.getMutationAlphaStatus.mockResolvedValue(
      status({
        available: false,
        reason: 'A separate approved validation-target record is required.',
        liveValidation: {
          ...status().liveValidation,
          targetApprovalLoaded: false,
          available: false,
          reason: 'Approval missing.'
        }
      })
    );
    const blocked = new MutationAlphaWorkflow(noApprovalBackend, () => now);
    await blocked.initialise(MUTATION_ALPHA_BUILD_MODE);
    expect(blocked.options).toEqual([]);

    const choices = authorizedChoices([widgetsEnabledOption()]);
    expect(choices).toEqual([
      {
        operationId: 'set_taskbar_widgets_visibility',
        target: 'enabled',
        title: 'Widgets button visibility'
      }
    ]);
    expect(JSON.stringify(choices)).not.toMatch(/disabled|task.view|show.desktop/i);
  });
});

describe('Mutation Alpha plan review and approval', () => {
  it('accepts one authorized operation and shows the exact absence rollback', () => {
    expect(validateIssuedPlan(issuedPlan(), [widgetsEnabledOption()], now)).toBeNull();
    expect(representationText(issuedPlan().proposedRepresentation)).toBe('DWORD 1');
    expect(rollbackText(issuedPlan().plan.currentState)).toBe(
      'Undo will restore this value to absence, not write DWORD 0.'
    );
  });

  it('rejects multi-operation, expired, elevated, and Explorer-termination plans', () => {
    const multi = issuedPlan() as IssuedMutationPlan & { plan: { operations: unknown[] } };
    multi.plan.operations = [{}, {}];
    expect(validateIssuedPlan(multi, [widgetsEnabledOption()], now)).toMatch(/exactly one/i);

    const expired = issuedPlan();
    expired.plan.expiresAt = String(now - 1);
    expect(validateIssuedPlan(expired, [widgetsEnabledOption()], now)).toMatch(/expired/i);

    const elevated = issuedPlan();
    elevated.plan.requiredPrivilege = 'administrator';
    expect(validateIssuedPlan(elevated, [widgetsEnabledOption()], now)).toMatch(/elevation/i);

    const explorer = issuedPlan();
    explorer.plan.restartRequirement = 'Explorer termination is required.';
    expect(validateIssuedPlan(explorer, [widgetsEnabledOption()], now)).toMatch(/Explorer/i);
  });

  it('requires an exact backend phrase and cannot execute before it matches', async () => {
    const backend = fakeBackend();
    const workflow = new MutationAlphaWorkflow(backend, () => now);
    await workflow.initialise(MUTATION_ALPHA_BUILD_MODE);
    await workflow.createPlan('set_taskbar_widgets_visibility', 'enabled', 'inspection-1');
    workflow.approvalInput = 'approve widgets test';
    await workflow.execute();
    expect(workflow.approvalMatches()).toBe(false);
    expect(backend.executeMutation).not.toHaveBeenCalled();
    workflow.approvalInput = 'APPROVE WIDGETS TEST';
    expect(workflow.approvalMatches()).toBe(true);
    await workflow.execute();
    expect(backend.executeMutation).toHaveBeenCalledOnce();
  });
});

describe('Mutation Alpha execution, visual confirmation, undo, and errors', () => {
  it('maps durable execution states and requires a separate visual report', async () => {
    expect(phaseFromTransaction(transaction({ status: 'applying' }))).toBe('executing');
    expect(phaseFromTransaction(transaction({ status: 'verifying' }))).toBe('verifying');
    expect(phaseFromTransaction(transaction())).toBe('applied');
    expect(phaseFromTransaction(transaction({ status: 'recovery_required' }))).toBe(
      'recovery_required'
    );

    const backend = fakeBackend();
    const workflow = new MutationAlphaWorkflow(backend, () => now);
    workflow.transaction = transaction();
    workflow.phase = 'applied';
    workflow.recordVisualReport('did_not_change');
    expect(workflow.visualReport).toBe('did_not_change');
    expect(backend.executeMutation).not.toHaveBeenCalled();
    expect(backend.exportLiveValidationEvidence).not.toHaveBeenCalled();
  });

  it('uses only transaction-bound rollback and prevents terminal reuse', async () => {
    const backend = fakeBackend();
    const workflow = new MutationAlphaWorkflow(backend, () => now);
    workflow.transaction = transaction();
    workflow.phase = 'applied';
    expect(canUndo(workflow.transaction)).toBe(true);
    await workflow.undo();
    expect(backend.rollbackMutation).toHaveBeenCalledWith('transaction-1', true, false);
    expect(backend.generateMutationPlan).not.toHaveBeenCalled();
    expect(workflow.transaction?.status).toBe('rolled_back');
    expect(canUndo(workflow.transaction)).toBe(false);
    await workflow.undo();
    expect(backend.rollbackMutation).toHaveBeenCalledOnce();
  });

  it('persists visual confirmation only through evidence export after rollback', async () => {
    const backend = fakeBackend();
    const workflow = new MutationAlphaWorkflow(backend, () => now);
    workflow.transaction = transaction();
    workflow.phase = 'applied';
    workflow.recordVisualReport('changed_as_expected');
    await workflow.exportVisualEvidence();
    expect(backend.exportLiveValidationEvidence).not.toHaveBeenCalled();

    workflow.transaction = transaction({
      status: 'rolled_back',
      rollbackState: widgetsEnabledOption().currentState,
      rollback: {
        available: false,
        complete: true,
        attemptedAt: String(now + 2),
        result: 'rolled_back',
        verificationResult: 'exact_pre_state_restored',
        conflictDetected: false
      }
    });
    workflow.phase = 'rolled_back';
    await workflow.exportVisualEvidence();
    expect(backend.exportLiveValidationEvidence).toHaveBeenCalledOnce();
    expect(backend.executeMutation).not.toHaveBeenCalled();
  });

  it('uses plain-language failures for denylist, expiry, changed state, rollback, and recovery', () => {
    expect(mutationError({ code: 'development_host_denied', message: 'denied' }).summary).toMatch(
      /blocked/i
    );
    expect(mutationError({ code: 'expired_approval', message: 'expired' }).summary).toMatch(
      /expired/i
    );
    expect(mutationError({ code: 'changed_source_state', message: 'changed' }).summary).toMatch(
      /changed/i
    );
    expect(mutationError({ code: 'rollback_failed', message: 'failed' }).summary).toMatch(/Undo/i);
    expect(mutationError({ code: 'recovery_required', message: 'review' }).summary).toMatch(
      /recovery/i
    );
  });
});
