import { describe, expect, it, vi } from 'vitest';

import {
  createBackendClient,
  describeCommandError,
  type AppView,
  type BackendInvoker,
  type BackendListener,
  type InspectionProgress
} from './backend';

const emptyView: AppView = {
  cleanupItems: [],
  plannedItems: [],
  catalogueCount: 3,
  plannedCount: 0,
  selectedFilter: 'all',
  selectedSection: 'windows_cleanup',
  reviewOpen: false
};

describe('backend client', () => {
  it('loads the Rust-owned view through the narrow read command', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue(emptyView);
    const client = createBackendClient(invokeCommand);

    await expect(client.getAppView()).resolves.toEqual(emptyView);
    expect(invokeCommand).toHaveBeenCalledWith('get_app_view');
  });

  it('sends typed actions through the single mutation command', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({
      ...emptyView,
      plannedCount: 1
    });
    const client = createBackendClient(invokeCommand);

    await client.dispatch({ type: 'toggle_planned', componentId: 'copilot' });

    expect(invokeCommand).toHaveBeenCalledWith('dispatch_app_action', {
      action: { type: 'toggle_planned', componentId: 'copilot' }
    });
  });

  it('starts and cancels only through typed lifecycle commands', async () => {
    const progress: InspectionProgress = {
      inspectionId: 'inspection-1',
      phase: 'running_detectors',
      completedWork: 3,
      totalWork: 20,
      currentComponent: 'phone_link',
      detectorResultStatus: null,
      warningCount: 0,
      errorCount: 0,
      cancellationAvailable: true,
      status: 'Running detectors',
      updatedAt: '1'
    };
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue(progress);
    const client = createBackendClient(invokeCommand);

    await client.startInspection();
    await client.cancelInspection('inspection-1');

    expect(invokeCommand).toHaveBeenNthCalledWith(1, 'start_inspection');
    expect(invokeCommand).toHaveBeenNthCalledWith(2, 'cancel_inspection', {
      inspectionId: 'inspection-1'
    });
  });

  it('subscribes to typed progress instead of polling the dashboard', async () => {
    const progress = { inspectionId: 'inspection-2', phase: 'preparing' } as InspectionProgress;
    const stop = vi.fn();
    const listenForEvent = vi.fn<BackendListener>().mockImplementation(async (_event, handler) => {
      handler(progress);
      return stop;
    });
    const handler = vi.fn();
    const client = createBackendClient(vi.fn<BackendInvoker>(), listenForEvent);

    await expect(client.subscribeInspectionProgress(handler)).resolves.toBe(stop);
    expect(listenForEvent).toHaveBeenCalledWith(
      'deslopper://inspection-progress',
      expect.any(Function)
    );
    expect(handler).toHaveBeenCalledWith(progress);
  });

  it('loads inspection, drift, package, and component histories through dedicated commands', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue([]);
    const client = createBackendClient(invokeCommand);
    await client.getInspectionHistory();
    await client.getDriftHistory();
    await client.getComponentTimeline('consumer_copilot');
    await client.getPackageObservations('inspection-1', 'consumer_copilot');
    expect(invokeCommand.mock.calls.map(([command]) => command)).toEqual([
      'get_inspection_history',
      'get_drift_history',
      'get_component_observation_timeline',
      'get_detailed_package_observations'
    ]);
  });

  it('sends only a finite desired-state request to Rust for validation and save', async () => {
    const request = {
      componentId: 'consumer_copilot',
      stateKey: 'absent_current_user',
      scope: 'current_user',
      persistentRemediation: false,
      alwaysRequireApproval: true,
      note: 'Keep reviewed'
    };
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({ valid: true });
    const client = createBackendClient(invokeCommand);
    await client.validateDesiredState(request);
    await client.saveDesiredState(request);
    expect(invokeCommand).toHaveBeenNthCalledWith(1, 'validate_desired_state', { request });
    expect(invokeCommand).toHaveBeenNthCalledWith(2, 'save_desired_state', { request });
  });

  it('generates a separate non-executable preview and product diagnostics contract', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({ executorEnabled: false });
    const client = createBackendClient(invokeCommand);
    await client.generatePreviewPlan('consumer_copilot');
    await client.generateDiagnosticsExport({
      includeHistorySummary: true,
      includeRedactedErrors: true
    });
    expect(invokeCommand).toHaveBeenNthCalledWith(1, 'generate_preview_plan', {
      componentId: 'consumer_copilot'
    });
    expect(invokeCommand).toHaveBeenNthCalledWith(2, 'generate_diagnostics_export', {
      request: { includeHistorySummary: true, includeRedactedErrors: true }
    });
    expect(JSON.stringify(invokeCommand.mock.calls)).not.toMatch(/upload|network|http/i);
  });

  it('uses closed local-data commands for retention, review, comparison, and clearing', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue([]);
    const client = createBackendClient(invokeCommand);
    await client.setHistoryRetention(180);
    await client.acknowledgeDriftEvent('onedrive', 'Preference');
    await client.compareInspections('inspection-1', 'inspection-2');
    await client.clearLocalHistory(true);
    expect(invokeCommand.mock.calls).toEqual([
      ['set_history_retention', { days: 180 }],
      ['acknowledge_drift_event', { componentId: 'onedrive', classification: 'Preference' }],
      [
        'compare_inspections',
        { previousInspectionId: 'inspection-1', currentInspectionId: 'inspection-2' }
      ],
      ['clear_local_history', { confirmed: true }]
    ]);
  });

  it('keeps permission-limited package evidence as a named state', () => {
    const row = {
      currentUser: 'present',
      otherUsers: 'permission_limited',
      provisioning: 'query_failed',
      sourceQueryCompleteness: 'permission_limited'
    };
    expect(row.otherUsers).not.toBe('absent');
    expect(row.sourceQueryCompleteness).toBe('permission_limited');
  });

  it('generates mutation plans from closed IDs and targets only', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({});
    const client = createBackendClient(invokeCommand);
    await client.generateMutationPlan('set_taskbar_widgets_visibility', 'disabled', 'inspection-1');
    expect(invokeCommand).toHaveBeenCalledWith('generate_mutation_plan', {
      request: {
        operationId: 'set_taskbar_widgets_visibility',
        target: 'disabled',
        sourceInspectionId: 'inspection-1'
      }
    });
    expect(JSON.stringify(invokeCommand.mock.calls[0])).not.toContain('registry');
  });

  it('exposes the normal Widgets owner workflow without arbitrary mutation parameters', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({});
    const client = createBackendClient(invokeCommand);

    await client.getWidgetsActionability();
    await client.applyWidgets('disabled', 'inspection-1');
    await client.undoWidgets('transaction-1');
    await client.getOwnerChangeHistory();

    expect(invokeCommand.mock.calls).toEqual([
      ['get_widgets_actionability'],
      [
        'apply_owner_operation',
        {
          request: {
            operationId: 'set_taskbar_widgets_visibility',
            target: 'disabled',
            sourceInspectionId: 'inspection-1'
          }
        }
      ],
      ['undo_owner_operation', { request: { transactionId: 'transaction-1' } }],
      ['get_owner_change_history']
    ]);
    expect(JSON.stringify(invokeCommand.mock.calls)).not.toMatch(
      /registry|taskbarDa|currentVersion|explorer\\advanced|task_view|show_desktop/i
    );
  });

  it('approval sends only plan identity, broker nonce, and acknowledgement', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({});
    const client = createBackendClient(invokeCommand);
    await client.executeMutation('plan-1', 'nonce-1', true);
    expect(invokeCommand).toHaveBeenCalledWith('approve_and_execute_mutation', {
      request: { planId: 'plan-1', approvalNonce: 'nonce-1', acknowledged: true }
    });
  });

  it('rollback cannot submit replacement state or operation parameters', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({});
    const client = createBackendClient(invokeCommand);
    await client.rollbackMutation('transaction-1', true, false);
    expect(invokeCommand).toHaveBeenCalledWith('rollback_mutation', {
      request: { transactionId: 'transaction-1', acknowledged: true, allowConflict: false }
    });
  });

  it('cancels an unconsumed plan by identity without mutation parameters', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({});
    const client = createBackendClient(invokeCommand);
    await client.cancelMutationPlan('plan-1');
    expect(invokeCommand).toHaveBeenCalledWith('cancel_mutation_plan', { planId: 'plan-1' });
  });

  it('exports only typed manual verification and redacted screenshot labels', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({});
    const client = createBackendClient(invokeCommand);
    const visualVerification = {
      representation: 'verified' as const,
      detector: 'verified' as const,
      userVisibleBehavior: 'verified' as const,
      settingsUi: 'verified' as const,
      refreshRequirement: 'immediate' as const,
      lifecycleRefreshCompleted: true
    };
    await client.exportLiveValidationEvidence(
      'transaction-1',
      visualVerification,
      ['widgets-disabled.png'],
      ['Natural taskbar refresh observed.']
    );
    expect(invokeCommand).toHaveBeenCalledWith('export_live_validation_evidence', {
      request: {
        transactionId: 'transaction-1',
        visualVerification,
        screenshotLabels: ['widgets-disabled.png'],
        warnings: ['Natural taskbar refresh observed.']
      }
    });
    expect(JSON.stringify(invokeCommand.mock.calls[0])).not.toContain('C:\\Users');
  });
});

describe('describeCommandError', () => {
  it('uses a typed backend message when one is available', () => {
    expect(
      describeCommandError({
        code: 'state_unavailable',
        message: 'Preview state unavailable.'
      })
    ).toBe('Preview state unavailable.');
  });

  it('does not leak opaque values into the interface', () => {
    expect(describeCommandError({ detail: new Error('internal') })).toBe(
      'The local preview could not be updated. Please try again.'
    );
  });
});
