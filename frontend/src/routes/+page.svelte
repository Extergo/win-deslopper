<script lang="ts">
  import { onMount, tick } from 'svelte';

  import {
    createBackendClient,
    describeCommandError,
    type AppAction,
    type AppView,
    type CatalogueFilter,
    type ComponentTimelineEntry,
    type DesiredStateRequest,
    type DesiredStateValidation,
    type DriftEvent,
    type InspectionHistoryItem,
    type InspectionProgress,
    type IssuedMutationPlan,
    type MutationAlphaStatus,
    type MutationOperationId,
    type MutationOperationOption,
    type MutationTarget,
    type MutationTransaction,
    type NavigationDestination,
    type PlatformDashboard,
    type VisualVerification
  } from '$lib/backend';
  import {
    installInspectionProgressListener,
    isTerminalInspectionPhase
  } from '$lib/inspection-progress';
  import '$lib/theme.css';

  const backend = createBackendClient();
  const filters: Array<{ value: CatalogueFilter; label: string }> = [
    { value: 'all', label: 'All' },
    { value: 'cloud', label: 'Cloud' },
    { value: 'ai', label: 'AI' },
    { value: 'promotions', label: 'Promotions' }
  ];
  const destinations: Array<{
    value: NavigationDestination;
    label: string;
    icon: string;
    description: string;
  }> = [
    {
      value: 'windows_cleanup',
      label: 'Windows Cleanup',
      icon: 'W',
      description: 'Preview optional components'
    },
    {
      value: 'inspection_history',
      label: 'Inspection history',
      icon: 'H',
      description: 'Saved read-only runs'
    },
    {
      value: 'drift_history',
      label: 'Drift history',
      icon: 'D',
      description: 'Changes over time'
    },
    {
      value: 'extensions',
      label: 'Extensions',
      icon: '+',
      description: 'Coming in a future milestone'
    }
  ];

  let view: AppView | null = null;
  let platform: PlatformDashboard | null = null;
  let loading = true;
  let pendingRequests = 0;
  let latestRequest = 0;
  let searchQuery = '';
  let errorMessage = '';
  let dialogElement: HTMLElement | undefined;
  let inspectionProgress: InspectionProgress | null = null;
  let inspectionHistory: InspectionHistoryItem[] = [];
  let driftHistory: DriftEvent[] = [];
  let selectedInspection: PlatformDashboard['snapshot'] = null;
  let selectedTimeline: ComponentTimelineEntry[] = [];
  let timelineComponent = '';
  let driftStatusFilter = 'active';
  let driftTypeFilter = 'all';
  let servicingVisible = false;
  let elapsedSeconds = 0;
  let desiredComponent = '';
  let desiredOptions: Array<{ key: string; label: string; scope: string; description: string }> =
    [];
  let desiredStateKey = '';
  let desiredPersistent = false;
  let desiredApproval = true;
  let desiredNote = '';
  let desiredValidation: DesiredStateValidation | null = null;
  let mutationAlpha: MutationAlphaStatus | null = null;
  let mutationOptions: MutationOperationOption[] = [];
  let mutationHistory: MutationTransaction[] = [];
  let mutationTargets: Partial<Record<MutationOperationId, MutationTarget>> = {};
  let issuedMutationPlan: IssuedMutationPlan | null = null;
  let mutationDialog: HTMLDialogElement | undefined;
  let mutationProgress = '';
  let validationEvidence: VisualVerification = {
    representation: 'not_run',
    detector: 'not_run',
    userVisibleBehavior: 'not_run',
    settingsUi: 'not_run',
    refreshRequirement: 'undetermined',
    lifecycleRefreshCompleted: false
  };
  let validationScreenshotLabels = '';
  let validationWarnings = '';

  $: inspectionRunning = inspectionProgress !== null && !isTerminal(inspectionProgress.phase);
  $: busy = pendingRequests > 0 || inspectionRunning;

  onMount(() => {
    void load();
    const unlisten = installInspectionProgressListener(backend, {
      onProgress: handleInspectionProgress,
      onError: (message) => {
        errorMessage = message;
      }
    });
    void backend.getRunningInspectionState().then((progress) => {
      inspectionProgress = progress;
    });
    const timer = window.setInterval(() => {
      if (inspectionProgress && !isTerminal(inspectionProgress.phase)) {
        const started = Number(inspectionProgress.updatedAt);
        elapsedSeconds = Number.isFinite(started)
          ? Math.max(elapsedSeconds + 1, Math.floor((Date.now() - started) / 1000))
          : elapsedSeconds + 1;
      }
    }, 1000);
    return () => {
      unlisten();
      window.clearInterval(timer);
    };
  });

  function isTerminal(phase: InspectionProgress['phase']): boolean {
    return isTerminalInspectionPhase(phase);
  }

  function handleInspectionProgress(progress: InspectionProgress): void {
    inspectionProgress = progress;
    if (isTerminal(progress.phase)) {
      void refreshHistoricalData();
    }
  }

  async function load(): Promise<void> {
    loading = true;
    errorMessage = '';

    try {
      view = await backend.getAppView();
      platform = await backend.getPlatformDashboard();
      await refreshHistoricalData();
      await loadMutationAlpha();
    } catch (error) {
      errorMessage = describeCommandError(error);
    } finally {
      loading = false;
    }
  }

  async function loadMutationAlpha(): Promise<void> {
    try {
      mutationAlpha = await backend.getMutationAlphaStatus();
      if (mutationAlpha.warningAcknowledged) await refreshMutationAlpha();
    } catch {
      mutationAlpha = null;
      mutationOptions = [];
      mutationHistory = [];
    }
  }

  async function acknowledgeMutationAlpha(): Promise<void> {
    mutationAlpha = await backend.acknowledgeMutationAlphaWarning(true);
    await refreshMutationAlpha();
  }

  async function refreshMutationAlpha(): Promise<void> {
    if (!mutationAlpha?.warningAcknowledged || !platform?.snapshot) return;
    [mutationOptions, mutationHistory] = await Promise.all([
      backend.getMutationOperationOptions(),
      backend.getMutationHistory()
    ]);
    for (const option of mutationOptions) {
      mutationTargets[option.definition.operationId] = option.currentState?.effectiveEnabled
        ? 'disabled'
        : 'enabled';
    }
  }

  async function reviewMutation(option: MutationOperationOption): Promise<void> {
    if (!platform?.snapshot) return;
    mutationProgress = 'Generating a machine-bound plan…';
    try {
      issuedMutationPlan = await backend.generateMutationPlan(
        option.definition.operationId,
        mutationTargets[option.definition.operationId] ?? 'disabled',
        platform.snapshot.id
      );
      await tick();
      mutationDialog?.showModal();
      mutationDialog?.focus();
    } catch (error) {
      errorMessage = describeCommandError(error);
    } finally {
      mutationProgress = '';
    }
  }

  async function executeReviewedMutation(): Promise<void> {
    if (!issuedMutationPlan) return;
    mutationProgress = 'Reinspecting, applying, and verifying…';
    try {
      await backend.executeMutation(
        issuedMutationPlan.plan.planId,
        issuedMutationPlan.approvalNonce,
        true
      );
      mutationDialog?.close();
      issuedMutationPlan = null;
      await refreshMutationAlpha();
    } catch (error) {
      errorMessage = describeCommandError(error);
      await refreshMutationAlpha();
    } finally {
      mutationProgress = '';
    }
  }

  async function rollbackMutation(transaction: MutationTransaction): Promise<void> {
    mutationProgress = 'Checking rollback conflict and restoring exact pre-state…';
    try {
      await backend.rollbackMutation(transaction.transactionId, true, false);
      await refreshMutationAlpha();
    } catch (error) {
      errorMessage = describeCommandError(error);
    } finally {
      mutationProgress = '';
    }
  }

  async function cancelMutationReview(): Promise<void> {
    if (!issuedMutationPlan) return;
    try {
      await backend.cancelMutationPlan(issuedMutationPlan.plan.planId);
      mutationDialog?.close();
      issuedMutationPlan = null;
      await refreshMutationAlpha();
    } catch (error) {
      errorMessage = describeCommandError(error);
    }
  }

  async function exportValidationEvidence(transaction: MutationTransaction): Promise<void> {
    mutationProgress = 'Validating and exporting a redacted live-evidence bundleâ€¦';
    try {
      const bundle = await backend.exportLiveValidationEvidence(
        transaction.transactionId,
        validationEvidence,
        validationScreenshotLabels
          .split(',')
          .map((value) => value.trim())
          .filter(Boolean),
        validationWarnings
          .split('\n')
          .map((value) => value.trim())
          .filter(Boolean)
      );
      mutationProgress = `Evidence recorded: ${bundle.finalResult.replaceAll('_', ' ')} (${bundle.visualOutcome.replaceAll('_', ' ')})`;
    } catch (error) {
      errorMessage = describeCommandError(error);
      mutationProgress = '';
    }
  }

  async function inspectNow(): Promise<void> {
    pendingRequests += 1;
    errorMessage = '';
    try {
      inspectionProgress = await backend.startInspection();
      elapsedSeconds = 0;
    } catch (error) {
      errorMessage = describeCommandError(error);
    } finally {
      pendingRequests -= 1;
    }
  }

  async function cancelInspection(): Promise<void> {
    if (!inspectionProgress) return;
    try {
      inspectionProgress = await backend.cancelInspection(inspectionProgress.inspectionId);
    } catch (error) {
      errorMessage = describeCommandError(error);
    }
  }

  async function refreshHistoricalData(): Promise<void> {
    const [dashboard, inspections, drift] = await Promise.all([
      backend.getPlatformDashboard(),
      backend.getInspectionHistory(),
      backend.getDriftHistory()
    ]);
    platform = dashboard;
    inspectionHistory = inspections;
    driftHistory = drift;
  }

  async function openInspection(inspectionId: string): Promise<void> {
    selectedInspection = await backend.getInspectionDetail(inspectionId);
  }

  async function openTimeline(componentId: string): Promise<void> {
    timelineComponent = componentId;
    selectedTimeline = await backend.getComponentTimeline(componentId);
  }

  function desiredRequest(): DesiredStateRequest {
    const option = desiredOptions.find((value) => value.key === desiredStateKey);
    return {
      componentId: desiredComponent,
      stateKey: desiredStateKey,
      scope: option?.scope ?? 'effective',
      persistentRemediation: desiredPersistent,
      alwaysRequireApproval: desiredApproval,
      note: desiredNote.trim() || null
    };
  }

  async function editDesiredState(componentId: string): Promise<void> {
    desiredComponent = componentId;
    desiredOptions = await backend.getAllowedDesiredStateOptions(componentId);
    desiredStateKey = desiredOptions[0]?.key ?? '';
    desiredPersistent = false;
    desiredApproval = true;
    desiredNote = '';
    desiredValidation = desiredStateKey
      ? await backend.validateDesiredState(desiredRequest())
      : {
          status: 'invalid',
          valid: false,
          warnings: [],
          reason: 'No valid desired states are available.'
        };
  }

  async function validateDesiredEditor(): Promise<void> {
    if (desiredStateKey) desiredValidation = await backend.validateDesiredState(desiredRequest());
  }

  async function saveDesiredEditor(): Promise<void> {
    desiredValidation = await backend.validateDesiredState(desiredRequest());
    if (!desiredValidation.valid) return;
    platform = await backend.saveDesiredState(desiredRequest());
    desiredComponent = '';
  }

  async function clearDesiredEditor(): Promise<void> {
    platform = await backend.clearDesiredState(desiredComponent);
    desiredComponent = '';
  }

  async function update(action: AppAction): Promise<void> {
    const request = ++latestRequest;
    pendingRequests += 1;
    errorMessage = '';

    try {
      const nextView = await backend.dispatch(action);
      if (request === latestRequest) {
        view = nextView;
      }

      if (action.type === 'open_review' && nextView.reviewOpen) {
        await tick();
        dialogElement?.focus();
      }
    } catch (error) {
      if (request === latestRequest) {
        errorMessage = describeCommandError(error);
      }
    } finally {
      pendingRequests -= 1;
    }
  }

  function searchChanged(event: Event): void {
    searchQuery = (event.currentTarget as HTMLInputElement).value;
    void update({ type: 'search_changed', query: searchQuery });
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape' && view?.reviewOpen) {
      event.preventDefault();
      void update({ type: 'close_review' });
    }
  }

  function stateLabel(state: Record<string, unknown>): string {
    const kind = Object.keys(state)[0] ?? 'Unknown';
    const value = state[kind];
    if (typeof value === 'object' && value !== null && 'enabled' in value) {
      return `${kind}: ${(value as { enabled: boolean }).enabled ? 'Enabled' : 'Disabled'}`;
    }
    if (kind === 'Package' && typeof value === 'object' && value !== null) {
      const packageState = value as { current_user?: boolean; provisioned?: boolean };
      return `${packageState.current_user ? 'Installed' : 'Not registered'} · ${packageState.provisioned ? 'Provisioned' : 'Not provisioned'}`;
    }
    return kind.replaceAll('_', ' ');
  }

  function displayTimestamp(timestamp: string): string {
    const numeric = Number(timestamp);
    return Number.isFinite(numeric) ? new Date(numeric).toLocaleString() : timestamp;
  }

  $: filteredDrift = driftHistory.filter((event) => {
    if (driftStatusFilter === 'active' && event.resolved) return false;
    if (driftStatusFilter === 'resolved' && !event.resolved) return false;
    if (!servicingVisible && event.classification === 'NormalServicing') return false;
    return driftTypeFilter === 'all' || event.classification === driftTypeFilter;
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<svelte:head>
  <title>Deslopper</title>
  <meta
    name="description"
    content="A non-destructive preview for reviewing optional Windows experiences."
  />
</svelte:head>

{#if loading}
  <main class="boot-state" aria-live="polite">
    <div class="brand-mark" aria-hidden="true">D</div>
    <div>
      <strong>Preparing your local preview</strong>
      <span>Loading the Rust-owned catalogue…</span>
    </div>
  </main>
{:else if view}
  <div class="app-shell" aria-busy={busy}>
    <aside class="navigation" aria-label="Primary navigation">
      <div class="brand">
        <div class="brand-mark" aria-hidden="true">D</div>
        <div class="brand-copy">
          <strong>Deslopper</strong>
          <span>UI preview</span>
        </div>
      </div>

      <p class="nav-eyebrow">Workspace</p>
      <nav>
        {#each destinations as destination (destination.value)}
          <button
            class="nav-item"
            class:active={view.selectedSection === destination.value}
            aria-current={view.selectedSection === destination.value ? 'page' : undefined}
            onclick={() => void update({ type: 'navigate', destination: destination.value })}
          >
            <span class="nav-icon" aria-hidden="true">{destination.icon}</span>
            <span>
              <strong>{destination.label}</strong>
              <small>{destination.description}</small>
            </span>
          </button>
        {/each}
      </nav>

      <div class="navigation-spacer"></div>

      <section class="preview-note" aria-label="Preview safety status">
        <div class="preview-note-heading">
          <span class="status-dot" aria-hidden="true"></span>
          <strong>{mutationAlpha ? 'Internal alpha' : 'Preview mode'}</strong>
        </div>
        <p>
          {mutationAlpha
            ? 'Three current-user operations require all alpha gates and explicit approval.'
            : 'No administrator access or Windows changes.'}
        </p>
      </section>
    </aside>

    <main class="workspace">
      {#if errorMessage}
        <div class="error-banner" role="alert">
          <span aria-hidden="true">!</span>
          <p>{errorMessage}</p>
          <button onclick={() => void load()}>Retry</button>
        </div>
      {/if}

      {#if view.selectedSection === 'windows_cleanup'}
        <section class="cleanup-page" aria-labelledby="cleanup-title">
          <header class="page-header">
            <div>
              <p class="eyebrow">Local catalogue</p>
              <h1 id="cleanup-title">Windows Cleanup</h1>
              <p>
                Review optional Windows components and promotional features.
                {mutationAlpha
                  ? 'Mutation remains limited to the gated experimental panel below.'
                  : 'This build never changes your system.'}
              </p>
            </div>
            <span class="preview-badge">
              <span aria-hidden="true"></span>
              {mutationAlpha ? 'Mutation alpha' : 'Preview only'}
            </span>
          </header>

          <section class="inspection-overview" aria-label="Windows inspection overview">
            <div>
              <span class="eyebrow">Read-only inspection</span>
              <strong>{platform?.snapshot?.platform.productName ?? 'Not inspected yet'}</strong>
              <small>
                {#if platform?.snapshot}
                  {platform.snapshot.platform.edition} · build {platform.snapshot.platform.build} ·
                  {platform.snapshot.platform.architecture}
                {:else}
                  No snapshot has been saved.
                {/if}
              </small>
            </div>
            <div class="inspection-stats">
              <span><strong>{platform?.unknownCount ?? 0}</strong> unknown</span>
              <span><strong>{platform?.driftCount ?? 0}</strong> drifted</span>
              <span><strong>{platform?.desiredCount ?? 0}</strong> desired</span>
            </div>
            <div class="inspection-actions">
              <button class="action-button" disabled={busy} onclick={() => void inspectNow()}>
                {platform?.snapshot ? 'Inspect again' : 'Inspect Windows'}
              </button>
              {#if inspectionRunning && inspectionProgress?.cancellationAvailable}
                <button class="action-button secondary" onclick={() => void cancelInspection()}
                  >Cancel</button
                >
              {/if}
            </div>
            {#if inspectionProgress}
              <div class="inspection-progress" aria-live="polite">
                <progress
                  max={inspectionProgress.totalWork || 1}
                  value={inspectionProgress.completedWork}
                ></progress>
                <div>
                  <strong>{inspectionProgress.status}</strong>
                  <span>
                    {inspectionProgress.completedWork}/{inspectionProgress.totalWork} detectors ·
                    {inspectionProgress.warningCount} warnings · {inspectionProgress.errorCount} errors
                    ·
                    {elapsedSeconds}s elapsed
                  </span>
                  {#if inspectionProgress.currentComponent}
                    <small
                      >Current component: {inspectionProgress.currentComponent.replaceAll(
                        '_',
                        ' '
                      )}</small
                    >
                  {/if}
                </div>
              </div>
              {#if inspectionProgress.phase === 'completed_with_partial_failures'}
                <p class="partial-message">
                  Inspection completed, but some evidence is incomplete. No unavailable result was
                  treated as absence.
                </p>
              {:else if inspectionProgress.phase === 'cancelled'}
                <p class="partial-message">
                  Inspection was cancelled. Completed detector evidence was preserved; queued
                  detectors were not treated as successful.
                </p>
              {/if}
            {/if}
          </section>

          {#if mutationAlpha}
            <section class="mutation-alpha-panel" aria-labelledby="mutation-alpha-title">
              <div class="catalogue-heading">
                <div>
                  <p class="eyebrow">Internal build · current user only</p>
                  <h2 id="mutation-alpha-title">Experimental mutation alpha</h2>
                  <p>
                    No elevation, bulk apply, automatic repair, package removal, services, tasks, or
                    Explorer termination.
                  </p>
                </div>
                <span class="preview-badge mutation">Alpha</span>
              </div>

              <div
                class:environment-refused={mutationAlpha.liveValidation.developmentHostRefused}
                class="environment-banner"
                role="status"
              >
                <strong>Validation environment identity</strong>
                <span>Computer: {mutationAlpha.liveValidation.environment.computerName}</span>
                <span>Machine: {mutationAlpha.liveValidation.environment.machineIdPrefix}â€¦</span>
                <span>
                  {mutationAlpha.liveValidation.environment.edition} Â· build
                  {mutationAlpha.liveValidation.environment.build}.{mutationAlpha.liveValidation
                    .environment.updateBuildRevision ?? '?'}
                </span>
                <span>
                  VM detection: {mutationAlpha.liveValidation.environment.virtualMachineDetection.replaceAll(
                    '_',
                    ' '
                  )}
                </span>
                <span>
                  Scenario: {mutationAlpha.liveValidation.environment.currentValidationScenario ??
                    'none selected'}
                </span>
                <small>{mutationAlpha.liveValidation.reason}</small>
                {#if mutationAlpha.liveValidation.developmentHostRefused}
                  <strong>Development host refused â€” live mutation cannot be enabled.</strong>
                {/if}
              </div>

              {#if !mutationAlpha.warningAcknowledged}
                <div class="mutation-warning" role="alert">
                  <strong>This can change three taskbar presentation settings.</strong>
                  <p>
                    Use only in a disposable Windows test environment. Every operation captures
                    pre-state, verifies the effective result, records an audit transaction, and
                    requires separate rollback approval.
                  </p>
                  <button
                    class="action-button"
                    disabled={!mutationAlpha.debugBuild || !mutationAlpha.commandLineOptIn}
                    onclick={() => void acknowledgeMutationAlpha()}
                    >I understand and enable this session</button
                  >
                  <small>{mutationAlpha.reason}</small>
                </div>
              {:else if !mutationAlpha.available}
                <p class="partial-message">{mutationAlpha.reason}</p>
              {:else if !platform?.snapshot}
                <p class="partial-message">Run a fresh inspection before generating a plan.</p>
              {:else}
                {#if mutationProgress}<p class="mutation-progress" aria-live="polite">
                    {mutationProgress}
                  </p>{/if}
                <div class="mutation-grid">
                  {#each mutationOptions as option (option.definition.operationId)}
                    <article class="mutation-card">
                      <div>
                        <strong>{option.definition.title}</strong>
                        <span class="authority-chip">{option.definition.requiredPrivilege}</span>
                      </div>
                      <span class="validation-maturity">{option.validationLabel}</span>
                      <p>{option.definition.description}</p>
                      <dl>
                        <div>
                          <dt>Current</dt>
                          <dd>{option.currentState?.effectiveEnabled ? 'Enabled' : 'Disabled'}</dd>
                        </div>
                        <div>
                          <dt>Authority</dt>
                          <dd>{option.currentState?.authority ?? 'Unknown'}</dd>
                        </div>
                        <div>
                          <dt>Confidence</dt>
                          <dd>{option.currentState?.confidence ?? 'Unknown'}</dd>
                        </div>
                        <div>
                          <dt>Restart</dt>
                          <dd>{option.definition.restartRequirement}</dd>
                        </div>
                      </dl>
                      <label>
                        Reviewed target
                        <select bind:value={mutationTargets[option.definition.operationId]}>
                          {#each option.definition.supportedTargets as target (target)}
                            <option value={target}
                              >{target === 'enabled' ? 'Enabled' : 'Disabled'}</option
                            >
                          {/each}
                        </select>
                      </label>
                      <p><strong>Rollback:</strong> {option.definition.rollbackMethod}</p>
                      <div class="documentation-links">
                        {#each option.definition.documentation as source (source)}
                          <span class="documentation-source">{source}</span>
                        {/each}
                      </div>
                      <button
                        class="action-button"
                        disabled={!option.eligible || Boolean(mutationProgress)}
                        title={option.reason}
                        onclick={() => void reviewMutation(option)}>Generate final plan</button
                      >
                      {#if !option.eligible}<small class="warning-copy">{option.reason}</small>{/if}
                    </article>
                  {/each}
                </div>

                <div class="mutation-history">
                  <h3>Mutation history</h3>
                  {#if mutationHistory.length === 0}
                    <p>No mutation transactions have been created.</p>
                  {:else}
                    {#each mutationHistory as transaction (transaction.transactionId)}
                      <details
                        class:error={transaction.status.includes('failed') ||
                          transaction.status === 'recovery_required'}
                      >
                        <summary>
                          <span
                            ><strong>{transaction.operationId.replaceAll('_', ' ')}</strong><small
                              >{displayTimestamp(transaction.createdAt)}</small
                            ></span
                          >
                          <span class="authority-chip"
                            >{transaction.status.replaceAll('_', ' ')}</span
                          >
                        </summary>
                        <div class="observation-detail">
                          <p>
                            Build {transaction.windowsBuild} · {transaction.edition} · handler {transaction.handlerVersion}
                          </p>
                          <p>Verification: {transaction.verificationResult ?? 'Not completed'}</p>
                          <p>
                            Previous: {transaction.preState?.effectiveEnabled === true
                              ? 'Enabled'
                              : transaction.preState
                                ? 'Disabled'
                                : 'Not captured'}
                          </p>
                          <p>Requested: {transaction.targetState}</p>
                          <p>
                            Rollback: {transaction.rollback.verificationResult ??
                              (transaction.rollback.available ? 'Available' : 'Unavailable')}
                          </p>
                          {#if transaction.recoveryRequirement}<p class="warning-copy">
                              Recovery: {transaction.recoveryRequirement}
                            </p>{/if}
                          {#if transaction.errorSummary}<p class="warning-copy">
                              {transaction.errorSummary}
                            </p>{/if}
                          {#if transaction.rollback.available && transaction.status === 'rollback_available'}
                            <button
                              class="action-button secondary"
                              onclick={() => void rollbackMutation(transaction)}
                              >Review and roll back exact pre-state</button
                            >
                          {/if}
                          {#if transaction.postState && mutationAlpha.liveValidation.available}
                            <fieldset class="validation-evidence-form">
                              <legend>Developer live-validation evidence</legend>
                              <label>
                                Representation
                                <select bind:value={validationEvidence.representation}>
                                  <option value="not_run">Not run</option>
                                  <option value="verified">Verified</option>
                                  <option value="pending_refresh">Pending refresh</option>
                                  <option value="uncertain">Uncertain</option>
                                </select>
                              </label>
                              <label>
                                Production detector
                                <select bind:value={validationEvidence.detector}>
                                  <option value="not_run">Not run</option>
                                  <option value="verified">Verified</option>
                                  <option value="policy_overrode">Policy overrode</option>
                                  <option value="uncertain">Uncertain</option>
                                </select>
                              </label>
                              <label>
                                User-visible taskbar behavior
                                <select bind:value={validationEvidence.userVisibleBehavior}>
                                  <option value="not_run">Not run</option>
                                  <option value="verified">Verified</option>
                                  <option value="pending_refresh">Pending refresh</option>
                                  <option value="ignored_by_shell">Ignored by shell</option>
                                  <option value="policy_overrode">Policy overrode</option>
                                  <option value="reverted_after_sign_in"
                                    >Reverted after sign-in</option
                                  >
                                  <option value="uncertain">Uncertain</option>
                                </select>
                              </label>
                              <label>
                                Windows Settings UI
                                <select bind:value={validationEvidence.settingsUi}>
                                  <option value="not_run">Not run</option>
                                  <option value="verified">Verified</option>
                                  <option value="disagrees">Disagrees</option>
                                  <option value="uncertain">Uncertain</option>
                                </select>
                              </label>
                              <label>
                                Minimum refresh requirement
                                <select bind:value={validationEvidence.refreshRequirement}>
                                  <option value="undetermined">Undetermined</option>
                                  <option value="immediate">Immediate</option>
                                  <option value="taskbar_natural_refresh"
                                    >Natural taskbar refresh</option
                                  >
                                  <option value="settings_app_reopen">Settings reopen</option>
                                  <option value="deslopper_reopen">Deslopper reopen</option>
                                  <option value="sign_out_sign_in">Sign-out/sign-in</option>
                                  <option value="reboot">Reboot</option>
                                  <option value="unsupported_without_explorer_termination"
                                    >Unsupported without Explorer termination</option
                                  >
                                </select>
                              </label>
                              <label class="checkbox-row">
                                <input
                                  type="checkbox"
                                  bind:checked={validationEvidence.lifecycleRefreshCompleted}
                                />
                                Required sign-in/reboot lifecycle was completed
                              </label>
                              <label>
                                Screenshot labels only (comma separated, never paths)
                                <input
                                  bind:value={validationScreenshotLabels}
                                  placeholder="widgets-off.png"
                                />
                              </label>
                              <label>
                                Redacted warnings (one per line)
                                <textarea bind:value={validationWarnings}></textarea>
                              </label>
                              <button
                                class="action-button secondary"
                                onclick={() => void exportValidationEvidence(transaction)}
                                >Export redacted evidence</button
                              >
                            </fieldset>
                          {/if}
                        </div>
                      </details>
                    {/each}
                  {/if}
                </div>
              {/if}
            </section>
          {/if}

          {#if platform?.snapshot}
            <section class="observation-panel" aria-labelledby="observation-title">
              <div class="catalogue-heading">
                <div>
                  <h2 id="observation-title">Latest observations</h2>
                  <p>Structured evidence from the saved read-only inspection.</p>
                </div>
                <span>{platform.snapshot.observations.length} components</span>
              </div>
              <div class="observation-grid">
                {#each platform.snapshot.observations as observation (observation.componentId)}
                  <details class:error={observation.error !== null}>
                    <summary>
                      <span>
                        <strong>{observation.componentId.replaceAll('_', ' ')}</strong>
                        <small>{stateLabel(observation.current)}</small>
                      </span>
                      <span class="authority-chip">{observation.authority}</span>
                    </summary>
                    <div class="observation-detail">
                      <p><strong>Policy:</strong> {observation.policyState ?? 'Not proven'}</p>
                      <p>
                        <strong>Preference:</strong>
                        {observation.preferenceState ?? 'Not proven'}
                      </p>
                      <p>
                        <strong>Provisioning:</strong>
                        {observation.provisioningState ?? 'Not applicable or incomplete'}
                      </p>
                      <p>
                        <strong>Detector:</strong>
                        {observation.detectorStatus.replaceAll('_', ' ')}
                      </p>
                      <p>
                        <strong>Applicability:</strong>
                        {observation.applicability.status.replaceAll('_', ' ')} — {observation
                          .applicability.reason}
                      </p>
                      <p>
                        <strong>Package scope:</strong>
                        {observation.packageCompleteness.replaceAll('_', ' ')}
                      </p>
                      <p>
                        <strong>Authority confidence:</strong>
                        {observation.authorityAttribution.confidence}
                        {observation.authorityAttribution.exactSourceProven
                          ? ' · exact source proven'
                          : ' · source inferred'}
                      </p>
                      {#if observation.controlPrecedence.conflictingEvidence}
                        <p class="warning-copy">
                          Preference and policy evidence conflict; the policy-derived state is
                          effective.
                        </p>
                      {/if}
                      {#each observation.evidence as evidence (evidence.source)}
                        <p class="evidence">
                          <strong>{evidence.source}</strong> — {evidence.detail} ({evidence.confidence}%
                          confidence)
                        </p>
                      {/each}
                      {#each observation.warnings as warning (warning)}<p class="warning-copy">
                          {warning}
                        </p>{/each}
                      {#if observation.packages.length > 0}
                        <div class="package-list">
                          {#each observation.packages as packageRow (packageRow.packageFullName ?? packageRow.packageName)}
                            <article>
                              <strong>{packageRow.packageName}</strong>
                              <span
                                >{packageRow.version ?? 'Version unknown'} · {packageRow.architecture ??
                                  'Architecture unknown'}</span
                              >
                              <small
                                >Current user: {packageRow.currentUser} · Other users: {packageRow.otherUsers}
                                · Provisioning: {packageRow.provisioning}</small
                              >
                            </article>
                          {/each}
                        </div>
                      {/if}
                      <div class="detail-actions">
                        <button
                          class="text-button"
                          onclick={() => void editDesiredState(observation.componentId)}
                          >Edit desired state</button
                        >
                        <button
                          class="text-button"
                          onclick={() => void openTimeline(observation.componentId)}
                          >View timeline</button
                        >
                      </div>
                    </div>
                  </details>
                {/each}
              </div>
            </section>
          {/if}

          {#if desiredComponent}
            <section class="editor-panel" aria-labelledby="desired-editor-title">
              <div class="catalogue-heading">
                <div>
                  <h2 id="desired-editor-title">
                    Desired state · {desiredComponent.replaceAll('_', ' ')}
                  </h2>
                  <p>Rust supplies and validates every allowed state.</p>
                </div>
                <button class="text-button" onclick={() => (desiredComponent = '')}>Close</button>
              </div>
              {#if desiredOptions.length > 0}
                <label
                  >Desired state
                  <select
                    bind:value={desiredStateKey}
                    onchange={() => void validateDesiredEditor()}
                  >
                    {#each desiredOptions as option (option.key)}<option value={option.key}
                        >{option.label}</option
                      >{/each}
                  </select>
                </label>
                <label class="check-row"
                  ><input
                    type="checkbox"
                    bind:checked={desiredPersistent}
                    onchange={() => void validateDesiredEditor()}
                  /> Allow future persistent remediation proposals</label
                >
                <label class="check-row"
                  ><input
                    type="checkbox"
                    bind:checked={desiredApproval}
                    onchange={() => void validateDesiredEditor()}
                  /> Always require approval</label
                >
                <label
                  >Optional note<textarea
                    maxlength="500"
                    bind:value={desiredNote}
                    oninput={() => void validateDesiredEditor()}></textarea></label
                >
                {#if desiredValidation}
                  <div class:invalid={!desiredValidation.valid} class="validation-box">
                    <strong>{desiredValidation.status.replaceAll('_', ' ')}</strong>
                    <p>{desiredValidation.reason}</p>
                    {#each desiredValidation.warnings as warning (warning)}<small>{warning}</small
                      >{/each}
                  </div>
                {/if}
                <div class="detail-actions">
                  <button
                    class="action-button"
                    disabled={!desiredValidation?.valid}
                    onclick={() => void saveDesiredEditor()}>Save desired state</button
                  >
                  <button class="action-button secondary" onclick={() => void clearDesiredEditor()}
                    >Clear saved state</button
                  >
                </div>
              {:else}<p class="partial-message">
                  No desired state can be safely selected from the current observation.
                </p>{/if}
            </section>
          {/if}

          {#if timelineComponent}
            <section class="editor-panel" aria-labelledby="timeline-title">
              <div class="catalogue-heading">
                <div>
                  <h2 id="timeline-title">
                    Component timeline · {timelineComponent.replaceAll('_', ' ')}
                  </h2>
                  <p>Structured history, newest first.</p>
                </div>
                <button class="text-button" onclick={() => (timelineComponent = '')}>Close</button>
              </div>
              <div class="timeline-list">
                {#each selectedTimeline as entry, index (`${timelineComponent}-${index}`)}
                  <details>
                    <summary
                      ><strong>{displayTimestamp(entry.observationTime)}</strong><span
                        >Build {entry.windowsBuild} · {stateLabel(entry.observation.current)}</span
                      ></summary
                    >
                    <div class="observation-detail">
                      <p>Inspection: {entry.inspectionId}</p>
                      <p>
                        Authority: {entry.observation.authority} ({entry.observation
                          .authorityAttribution.confidence})
                      </p>
                      <p>Applicability: {entry.observation.applicability.status}</p>
                      <p>Desired at the time: {entry.desiredState?.validation_status ?? 'None'}</p>
                      <p>
                        Drift: {entry.drift?.classification ?? 'None'}
                        {entry.drift?.cause ?? ''}
                      </p>
                      {#each entry.observation.evidence as evidence (evidence.queryId)}<p>
                          {evidence.source}: {evidence.detail}
                        </p>{/each}
                    </div>
                  </details>
                {/each}
              </div>
            </section>
          {/if}

          <div class="search-and-filter">
            <label class="search-field">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="m20 20-4.25-4.25m1.5-5.25a6.75 6.75 0 1 1-13.5 0 6.75 6.75 0 0 1 13.5 0Z"
                ></path>
              </svg>
              <span class="sr-only">Search the component catalogue</span>
              <input
                type="search"
                value={searchQuery}
                placeholder="Search components, categories, or descriptions"
                oninput={searchChanged}
              />
              {#if searchQuery}
                <button
                  type="button"
                  class="clear-search"
                  aria-label="Clear search"
                  onclick={() => {
                    searchQuery = '';
                    void update({ type: 'search_changed', query: '' });
                  }}>×</button
                >
              {/if}
            </label>

            <div class="filter-row" aria-label="Filter component catalogue">
              {#each filters as filter (filter.value)}
                <button
                  class="filter-chip"
                  class:active={view.selectedFilter === filter.value}
                  aria-pressed={view.selectedFilter === filter.value}
                  onclick={() => void update({ type: 'filter_changed', filter: filter.value })}
                >
                  {#if view.selectedFilter === filter.value}
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                      <path d="m3.25 8.2 3 3L12.75 4.8"></path>
                    </svg>
                  {/if}
                  {filter.label}
                </button>
              {/each}
            </div>
          </div>

          <section class="summary-strip" aria-label="Catalogue summary">
            <div class="summary-copy">
              <span class="summary-icon" aria-hidden="true">
                <svg viewBox="0 0 24 24">
                  <path d="M5 6.5h14M5 12h14M5 17.5h8"></path>
                </svg>
              </span>
              <div>
                <strong>{view.catalogueCount}</strong>
                <span>mock components in catalogue</span>
              </div>
            </div>
            <div class="summary-divider"></div>
            <div class="summary-copy planned">
              <span class="summary-icon" aria-hidden="true">
                <svg viewBox="0 0 24 24">
                  <path d="m6.5 12 3.25 3.25L17.5 7.5"></path>
                </svg>
              </span>
              <div>
                <strong>{view.plannedCount}</strong>
                <span>{view.plannedCount === 1 ? 'planned change' : 'planned changes'}</span>
              </div>
            </div>
            <p>All data is local, temporary, and mock-only.</p>
          </section>

          <div class="catalogue-heading">
            <div>
              <h2>Component catalogue</h2>
              <p>Choose items to assemble a preview plan.</p>
            </div>
            <span>{view.cleanupItems.length} shown</span>
          </div>

          <div class="catalogue" aria-live="polite">
            {#each view.cleanupItems as item (item.id)}
              <article class="cleanup-card" class:planned={item.planned}>
                <div class="card-heading">
                  <span class="component-icon" aria-hidden="true">{item.iconLabel}</span>
                  <div class="component-title">
                    <h3>{item.name}</h3>
                    <p>{item.category}</p>
                  </div>
                  {#if item.planned}
                    <span class="planned-label">
                      <svg viewBox="0 0 16 16" aria-hidden="true">
                        <path d="m3.25 8.2 3 3L12.75 4.8"></path>
                      </svg>
                      Planned
                    </span>
                  {/if}
                  <button
                    class:secondary={item.planned}
                    class="action-button"
                    disabled={busy}
                    onclick={() => void update({ type: 'toggle_planned', componentId: item.id })}
                  >
                    {item.planned ? 'Remove' : 'Add to plan'}
                  </button>
                </div>

                <p class="component-description">{item.description}</p>

                <div class="metadata" aria-label={`${item.name} details`}>
                  <span>{item.status}</span>
                  <span class:warning={item.risk === 'Moderate risk'}>{item.risk}</span>
                  <span>{item.restart}</span>
                  <span>{item.compatibility}</span>
                </div>

                <div class="proposed-action">
                  <span>Possible future actions</span>
                  <p>{item.proposedActions}</p>
                </div>
              </article>
            {:else}
              <section class="empty-state">
                <span class="empty-icon" aria-hidden="true">
                  <svg viewBox="0 0 32 32">
                    <path d="m27 27-5.4-5.4m2-7.1a9.1 9.1 0 1 1-18.2 0 9.1 9.1 0 0 1 18.2 0Z"
                    ></path>
                  </svg>
                </span>
                <h2>No matching components</h2>
                <p>Try another search or choose a different category.</p>
                <button
                  class="action-button"
                  onclick={() => {
                    searchQuery = '';
                    void update({ type: 'search_changed', query: '' });
                  }}>Clear search</button
                >
              </section>
            {/each}
          </div>

          <section class="plan-bar" class:active={view.plannedCount > 0} aria-live="polite">
            <div class="plan-count" aria-hidden="true">{view.plannedCount}</div>
            <div>
              <strong>
                {view.plannedCount === 0
                  ? 'No planned changes'
                  : `${view.plannedCount} ${view.plannedCount === 1 ? 'change' : 'changes'} ready to review`}
              </strong>
              <span>Planning is local and preview-only.</span>
            </div>
            <button
              class="action-button"
              disabled={view.plannedCount === 0 || busy}
              onclick={() => void update({ type: 'open_review' })}
            >
              Review changes
            </button>
          </section>
        </section>
      {:else if view.selectedSection === 'inspection_history'}
        <section class="history-page" aria-labelledby="history-title">
          <header class="page-header">
            <div>
              <p class="eyebrow">Persisted read-only evidence</p>
              <h1 id="history-title">Inspection history</h1>
              <p>
                Compare complete, partial, failed, and cancelled inspections on this machine
                identity.
              </p>
            </div>
            <span class="preview-badge">{inspectionHistory.length} saved</span>
          </header>
          <div class="history-layout">
            <div class="history-list">
              {#each inspectionHistory as inspection (inspection.inspectionId)}
                <button
                  class="history-card"
                  onclick={() => void openInspection(inspection.inspectionId)}
                >
                  <span
                    ><strong>{displayTimestamp(inspection.timestamp)}</strong><small
                      >{inspection.status}</small
                    ></span
                  >
                  <span>Build {inspection.windowsBuild} · {inspection.windowsEdition}</span>
                  <span
                    >{inspection.successfulCount} successful · {inspection.unknownCount} unknown · {inspection.failedCount}
                    failed · {inspection.cancelledCount} cancelled</span
                  >
                  <small
                    >{inspection.durationMs ?? 0} ms · {inspection.driftCount} drift events · {inspection.managementSummary}</small
                  >
                </button>
              {:else}<p class="partial-message">No inspection history has been saved yet.</p>{/each}
            </div>
            {#if selectedInspection}
              <section class="history-detail">
                <div class="catalogue-heading">
                  <div>
                    <h2>Inspection detail</h2>
                    <p>{selectedInspection.id}</p>
                  </div>
                  <button class="text-button" onclick={() => (selectedInspection = null)}
                    >Close</button
                  >
                </div>
                <p>
                  {selectedInspection.platform.edition} · build {selectedInspection.platform.build}
                </p>
                <div class="timeline-list">
                  {#each selectedInspection.observations as observation (observation.componentId)}
                    <details>
                      <summary
                        ><strong>{observation.componentId.replaceAll('_', ' ')}</strong><span
                          >{observation.detectorStatus} · {stateLabel(observation.current)}</span
                        ></summary
                      >
                      <div class="observation-detail">
                        <p>
                          Authority: {observation.authority} ({observation.authorityAttribution
                            .confidence})
                        </p>
                        <p>Applicability: {observation.applicability.status}</p>
                        <p>Packages: {observation.packages.length}</p>
                        {#each observation.evidence as evidence (evidence.queryId)}<p>
                            {evidence.source}: {evidence.detail}
                          </p>{/each}
                      </div>
                    </details>
                  {/each}
                </div>
              </section>
            {/if}
          </div>
        </section>
      {:else if view.selectedSection === 'drift_history'}
        <section class="history-page" aria-labelledby="drift-title">
          <header class="page-header">
            <div>
              <p class="eyebrow">Historical comparison</p>
              <h1 id="drift-title">Drift history</h1>
              <p>
                Meaningful changes, uncertainty transitions, authority refinements, and
                informational servicing.
              </p>
            </div>
            <span class="preview-badge">{filteredDrift.length} shown</span>
          </header>
          <div class="drift-filters">
            <label
              >Status<select bind:value={driftStatusFilter}
                ><option value="active">Active</option><option value="resolved">Resolved</option
                ><option value="all">All</option></select
              ></label
            >
            <label
              >Classification<select bind:value={driftTypeFilter}
                ><option value="all">All</option><option value="PackagePresence"
                  >Package presence</option
                ><option value="PackageProvisioning">Package provisioning</option><option
                  value="Policy">Policy</option
                ><option value="Preference">Preference</option><option value="ManagementAuthority"
                  >Authority</option
                ><option value="Availability">Availability</option><option
                  value="DetectionUncertainty">Detection uncertainty</option
                ><option value="PackageIdentityMigration">Identity migration</option><option
                  value="ResetOrReinstall">Reset/reinstall</option
                ><option value="NormalServicing">Normal servicing</option></select
              ></label
            >
            <label class="check-row"
              ><input type="checkbox" bind:checked={servicingVisible} /> Show informational servicing</label
            >
          </div>
          <div class="drift-list">
            {#each filteredDrift as drift (`${drift.component_id}-${drift.classification}-${drift.resolved}`)}
              <details class:servicing={drift.classification === 'NormalServicing'}>
                <summary
                  ><span
                    ><strong>{drift.component_id.replaceAll('_', ' ')}</strong><small
                      >{drift.classification.replaceAll(/([A-Z])/g, ' $1').trim()} · {drift.resolved
                        ? 'Resolved'
                        : 'Active'}</small
                    ></span
                  ><span>{drift.confidence}</span></summary
                >
                <div class="observation-detail">
                  <p>Previous: {stateLabel(drift.previous)}</p>
                  <p>Current: {stateLabel(drift.current)}</p>
                  <p>Desired: {drift.desired ? stateLabel(drift.desired) : 'None'}</p>
                  <p>Likely cause: {drift.cause} · rule v{drift.inference_rule_version}</p>
                  <p>
                    First detected: {displayTimestamp(drift.first_detected)} · last observed: {displayTimestamp(
                      drift.last_observed
                    )} · {drift.occurrence_count} occurrence(s)
                  </p>
                  {#each drift.supporting_facts as fact (fact)}<p>
                      Evidence: {fact}
                    </p>{/each}{#if drift.alternative_causes.length}<p>
                      Alternatives: {drift.alternative_causes.join(', ')}
                    </p>{/if}
                </div>
              </details>
            {:else}<p class="partial-message">
                No drift events match these filters. Normal servicing is hidden by default.
              </p>{/each}
          </div>
        </section>
      {:else}
        <section class="extensions-page" aria-labelledby="extensions-title">
          <header class="page-header">
            <div>
              <p class="eyebrow">Future milestone</p>
              <h1 id="extensions-title">Extensions</h1>
              <p>Optional Windows experiences, without bloating the core application.</p>
            </div>
            <span class="preview-badge muted">Not available</span>
          </header>

          <section class="coming-soon">
            <div class="extension-art" aria-hidden="true">
              <div class="orbit orbit-one"></div>
              <div class="orbit orbit-two"></div>
              <div class="extension-core">+</div>
              <span class="satellite one"></span>
              <span class="satellite two"></span>
            </div>
            <p class="eyebrow">Designed for trust first</p>
            <h2>A careful extension system is coming later</h2>
            <p class="coming-soon-copy">
              Extensions will let Deslopper add optional experiences without putting unrelated
              features in the core application. Nothing can be installed or executed in this
              preview.
            </p>
            <div class="extension-chips" aria-label="Potential future extension categories">
              <span>Start menu replacements</span>
              <span>Wallpaper engines</span>
              <span>Desktop utilities</span>
              <span>System monitoring</span>
              <span>Productivity tools</span>
            </div>
            <div class="future-label">Coming in a future milestone</div>
          </section>
        </section>
      {/if}
    </main>
  </div>

  {#if issuedMutationPlan}
    <dialog
      bind:this={mutationDialog}
      class="review-dialog mutation-dialog"
      aria-labelledby="mutation-review-title"
    >
      <header>
        <div>
          <p class="eyebrow">Final machine-bound plan</p>
          <h2 id="mutation-review-title">Approve this exact operation</h2>
        </div>
        <button
          aria-label="Cancel mutation before it starts"
          onclick={() => void cancelMutationReview()}>×</button
        >
      </header>
      <div class="review-warning mutation-warning">
        <span aria-hidden="true">!</span>
        <p><strong>Experimental mutation alpha.</strong> {issuedMutationPlan.confirmationText}</p>
      </div>
      <div class="mutation-plan-detail">
        <p>
          <strong>Operation:</strong>
          {issuedMutationPlan.plan.operationId.replaceAll('_', ' ')}
        </p>
        <p>
          <strong>Current:</strong>
          {issuedMutationPlan.plan.currentState.effectiveEnabled ? 'Enabled' : 'Disabled'}
        </p>
        <p><strong>Target:</strong> {issuedMutationPlan.plan.targetState}</p>
        <p><strong>Scope:</strong> Current Windows user · no elevation</p>
        <p><strong>Restart:</strong> {issuedMutationPlan.plan.restartRequirement}</p>
        <p><strong>Rollback:</strong> {issuedMutationPlan.plan.rollbackMethod}</p>
        <p><strong>Fresh until:</strong> {displayTimestamp(issuedMutationPlan.plan.expiresAt)}</p>
        <code>{issuedMutationPlan.plan.planHash}</code>
      </div>
      <footer>
        <span>{mutationProgress || 'Approval consumes this plan once.'}</span>
        <div>
          <button
            class="action-button"
            disabled={Boolean(mutationProgress)}
            onclick={() => void executeReviewedMutation()}>Approve, apply, and verify</button
          >
          <button
            class="action-button secondary"
            disabled={Boolean(mutationProgress)}
            onclick={() => void cancelMutationReview()}>Cancel before mutation</button
          >
        </div>
      </footer>
    </dialog>
  {/if}

  {#if view.reviewOpen}
    <div class="scrim">
      <dialog
        open
        class="review-dialog"
        aria-modal="true"
        aria-labelledby="review-title"
        tabindex="-1"
        bind:this={dialogElement}
      >
        <header class="review-heading">
          <div>
            <p class="eyebrow">Local plan</p>
            <h2 id="review-title">Review planned changes</h2>
            <p>Nothing in this list is an actual system result.</p>
          </div>
          <button
            class="icon-button"
            aria-label="Close review"
            onclick={() => void update({ type: 'close_review' })}>×</button
          >
        </header>

        <div class="review-warning">
          <span aria-hidden="true">!</span>
          <p><strong>Preview only.</strong> Deslopper will not modify Windows in this build.</p>
        </div>

        <div class="review-list">
          {#each view.plannedItems as item (item.id)}
            <article>
              <span class="component-icon" aria-hidden="true">{item.iconLabel}</span>
              <div>
                <h3>{item.name}</h3>
                <p>{item.category}</p>
              </div>
              <button
                class="text-button"
                disabled={busy}
                onclick={() => void update({ type: 'toggle_planned', componentId: item.id })}
                >Remove</button
              >
            </article>
          {/each}
        </div>

        <footer class="review-footer">
          <span>
            {view.plannedCount}
            {view.plannedCount === 1 ? 'item in plan' : 'items in plan'}
          </span>
          <div>
            <button class="action-button" disabled>Apply changes — unavailable</button>
            <button
              class="action-button secondary"
              onclick={() => void update({ type: 'close_review' })}>Done</button
            >
          </div>
        </footer>
      </dialog>
    </div>
  {/if}
{:else}
  <main class="fatal-state">
    <div class="error-symbol" aria-hidden="true">!</div>
    <h1>The local preview did not start</h1>
    <p>{errorMessage || 'The application could not load its in-memory catalogue.'}</p>
    <button class="action-button" onclick={() => void load()}>Try again</button>
  </main>
{/if}

<style>
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .boot-state,
  .fatal-state {
    display: flex;
    width: 100%;
    height: 100%;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
    padding: var(--space-6);
    background: var(--color-background);
  }

  .boot-state > div:last-child {
    display: grid;
    gap: var(--space-1);
  }

  .boot-state strong {
    font-size: var(--type-body);
  }

  .boot-state span,
  .fatal-state p {
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
  }

  .fatal-state {
    flex-direction: column;
    text-align: center;
  }

  .fatal-state h1,
  .fatal-state p {
    margin: 0;
  }

  .error-symbol {
    display: grid;
    width: 56px;
    height: 56px;
    place-items: center;
    border-radius: var(--radius-card);
    color: var(--color-warning);
    background: var(--color-warning-container);
    font-size: var(--type-section);
    font-weight: 800;
  }

  .app-shell {
    display: grid;
    width: 100%;
    height: 100%;
    grid-template-columns: 236px minmax(0, 1fr);
    overflow: hidden;
    background: var(--color-background);
  }

  .navigation {
    display: flex;
    min-height: 0;
    flex-direction: column;
    padding: var(--space-5) var(--space-3) var(--space-4);
    border-right: 1px solid var(--color-outline);
    background: var(--color-surface);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-2) var(--space-6);
  }

  .brand-mark {
    display: grid;
    width: 44px;
    height: 44px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: var(--radius-control);
    color: var(--color-on-primary);
    background: var(--color-primary);
    box-shadow: 0 7px 18px var(--color-shadow-strong);
    font-size: 1.3rem;
    font-weight: 800;
  }

  .brand-copy {
    display: grid;
    gap: 1px;
  }

  .brand-copy strong {
    font-size: 1.12rem;
    letter-spacing: -0.02em;
  }

  .brand-copy span {
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }

  .nav-eyebrow,
  .eyebrow {
    margin: 0;
    color: var(--color-primary);
    font-size: var(--type-label);
    font-weight: 750;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }

  .nav-eyebrow {
    padding: 0 var(--space-3) var(--space-2);
    color: var(--color-text-secondary);
  }

  nav {
    display: grid;
    gap: var(--space-2);
  }

  .nav-item {
    display: grid;
    width: 100%;
    min-height: 64px;
    grid-template-columns: 38px minmax(0, 1fr);
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border: 1px solid transparent;
    border-radius: var(--radius-card);
    text-align: left;
    background: transparent;
    cursor: pointer;
    transition:
      background var(--motion-fast),
      border-color var(--motion-fast);
  }

  .nav-item:hover {
    background: var(--color-surface-tonal);
  }

  .nav-item.active {
    border-color: var(--color-primary-container);
    background: var(--color-primary-container);
  }

  .nav-item > span:last-child {
    display: grid;
    min-width: 0;
    gap: 2px;
  }

  .nav-item strong {
    overflow: hidden;
    font-size: var(--type-supporting);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nav-item small {
    overflow: hidden;
    color: var(--color-text-secondary);
    font-size: 0.68rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nav-icon {
    display: grid;
    width: 36px;
    height: 36px;
    place-items: center;
    border-radius: var(--radius-control);
    color: var(--color-primary);
    background: var(--color-surface-tonal);
    font-weight: 800;
  }

  .nav-item.active .nav-icon {
    color: var(--color-on-primary);
    background: var(--color-primary);
  }

  .navigation-spacer {
    flex: 1;
  }

  .preview-note {
    padding: var(--space-4);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    background: var(--color-surface-tonal);
  }

  .preview-note-heading {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--color-primary);
    font-size: var(--type-supporting);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-pill);
    background: var(--color-primary);
  }

  .preview-note p {
    margin: var(--space-2) 0 0;
    color: var(--color-text-secondary);
    font-size: var(--type-label);
    line-height: 1.45;
  }

  .workspace {
    position: relative;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .cleanup-page,
  .history-page,
  .extensions-page {
    display: flex;
    width: 100%;
    height: 100%;
    min-height: 0;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5) var(--space-6) 104px;
    overflow: hidden;
  }

  .extensions-page {
    padding-bottom: var(--space-6);
  }

  .history-page {
    padding-bottom: var(--space-6);
    overflow: auto;
  }

  .page-header {
    display: flex;
    flex: 0 0 auto;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-5);
  }

  .page-header h1 {
    margin: var(--space-1) 0 var(--space-1);
    font-size: var(--type-title);
    font-weight: 760;
    letter-spacing: -0.045em;
    line-height: 1.04;
  }

  .page-header p:last-child {
    max-width: 720px;
    margin: 0;
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
    line-height: 1.45;
  }

  .preview-badge {
    display: inline-flex;
    min-height: 34px;
    flex: 0 0 auto;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-3);
    border-radius: var(--radius-pill);
    color: var(--color-primary);
    background: var(--color-primary-container);
    font-size: var(--type-label);
    font-weight: 750;
  }

  .preview-badge > span {
    width: 7px;
    height: 7px;
    border-radius: var(--radius-pill);
    background: var(--color-primary);
  }

  .preview-badge.muted {
    color: var(--color-text-secondary);
    background: var(--color-surface-tonal);
  }

  .search-and-filter {
    display: flex;
    flex: 0 0 auto;
    align-items: center;
    gap: var(--space-3);
  }

  .search-field {
    display: flex;
    height: 48px;
    min-width: 260px;
    flex: 1;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-4);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast);
  }

  .search-field:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-container);
  }

  .search-field > svg {
    width: 20px;
    height: 20px;
    flex: 0 0 auto;
    fill: none;
    stroke: var(--color-text-secondary);
    stroke-linecap: round;
    stroke-width: 1.8;
  }

  .search-field input {
    width: 100%;
    min-width: 0;
    padding: 0;
    border: 0;
    color: var(--color-text);
    background: transparent;
    font-size: var(--type-supporting);
  }

  .search-field input::placeholder {
    color: var(--color-text-secondary);
    opacity: 0.88;
  }

  .search-field input::-webkit-search-cancel-button {
    display: none;
  }

  .clear-search,
  .icon-button {
    display: grid;
    width: 32px;
    height: 32px;
    flex: 0 0 auto;
    place-items: center;
    border: 0;
    border-radius: var(--radius-pill);
    color: var(--color-text-secondary);
    background: transparent;
    cursor: pointer;
    font-size: 1.35rem;
  }

  .clear-search:hover,
  .icon-button:hover {
    background: var(--color-surface-tonal);
  }

  .filter-row {
    display: flex;
    gap: var(--space-2);
  }

  .filter-chip {
    display: inline-flex;
    height: 40px;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    padding: 0 var(--space-3);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    cursor: pointer;
    font-size: var(--type-label);
    font-weight: 650;
    transition:
      background var(--motion-fast),
      border-color var(--motion-fast);
  }

  .filter-chip:hover {
    background: var(--color-surface-tonal);
  }

  .filter-chip.active {
    border-color: var(--color-primary);
    color: var(--color-primary);
    background: var(--color-primary-container);
  }

  .filter-chip svg,
  .planned-label svg {
    width: 15px;
    height: 15px;
    fill: none;
    stroke: currentcolor;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-width: 2;
  }

  .summary-strip {
    display: flex;
    min-height: 68px;
    flex: 0 0 auto;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
  }

  .inspection-overview {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    border: 1px solid var(--color-primary-container);
    border-radius: var(--radius-card);
    background: var(--color-primary-container);
  }

  .inspection-overview > div:first-child {
    display: grid;
    gap: 3px;
  }
  .inspection-overview .eyebrow {
    color: var(--color-primary);
  }
  .inspection-overview small {
    color: var(--color-text-secondary);
  }
  .inspection-stats {
    display: flex;
    gap: var(--space-4);
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }
  .inspection-stats span {
    display: grid;
    text-align: center;
  }
  .inspection-stats strong {
    color: var(--color-text);
    font-size: 1.1rem;
  }

  .observation-panel {
    display: grid;
    gap: var(--space-3);
  }
  .observation-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
  }
  .observation-grid details {
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
    background: var(--color-surface);
  }
  .observation-grid details.error {
    border-color: var(--color-warning);
  }
  .observation-grid summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3);
    cursor: pointer;
  }
  .observation-grid summary > span:first-child {
    display: grid;
    gap: 2px;
    text-transform: capitalize;
  }
  .observation-grid summary small {
    color: var(--color-text-secondary);
  }
  .authority-chip {
    padding: 4px 8px;
    border-radius: var(--radius-pill);
    color: var(--color-primary);
    background: var(--color-primary-container);
    font-size: var(--type-label);
  }
  .observation-detail {
    display: grid;
    gap: var(--space-2);
    padding: 0 var(--space-3) var(--space-3);
    border-top: 1px solid var(--color-outline);
  }
  .observation-detail p {
    margin: var(--space-2) 0 0;
    color: var(--color-text-secondary);
    font-size: var(--type-label);
    line-height: 1.45;
  }
  .observation-detail .warning-copy {
    color: var(--color-warning);
  }

  .inspection-actions,
  .detail-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .inspection-progress {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: minmax(160px, 0.35fr) minmax(0, 1fr);
    align-items: center;
    gap: var(--space-3);
  }
  .inspection-progress progress {
    width: 100%;
    height: 12px;
    accent-color: var(--color-primary);
  }
  .inspection-progress > div {
    display: grid;
    gap: 2px;
  }
  .inspection-progress span,
  .inspection-progress small {
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }
  .partial-message,
  .validation-box {
    grid-column: 1 / -1;
    margin: 0;
    padding: var(--space-3);
    border-radius: var(--radius-control);
    color: var(--color-warning);
    background: var(--color-warning-container);
    font-size: var(--type-supporting);
  }
  .package-list {
    display: grid;
    gap: var(--space-2);
  }
  .package-list article {
    display: grid;
    gap: 2px;
    padding: var(--space-2);
    border-radius: var(--radius-small);
    background: var(--color-surface-tonal);
    font-size: var(--type-label);
  }
  .package-list span,
  .package-list small {
    color: var(--color-text-secondary);
  }
  .editor-panel,
  .history-detail {
    display: grid;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
  }
  .editor-panel label,
  .drift-filters label {
    display: grid;
    gap: var(--space-1);
    color: var(--color-text-secondary);
    font-size: var(--type-label);
    font-weight: 650;
  }
  .editor-panel select,
  .editor-panel textarea,
  .drift-filters select {
    min-height: 42px;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
    color: var(--color-text);
    background: var(--color-surface);
    font: inherit;
  }
  .editor-panel textarea {
    min-height: 76px;
    resize: vertical;
  }
  .check-row {
    display: flex !important;
    min-height: 40px;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: var(--space-2) !important;
  }
  .validation-box {
    display: grid;
    gap: var(--space-1);
    color: var(--color-success);
    background: var(--color-success-container);
  }
  .validation-box.invalid {
    color: var(--color-warning);
    background: var(--color-warning-container);
  }
  .validation-box p {
    margin: 0;
  }
  .timeline-list,
  .drift-list,
  .history-list {
    display: grid;
    gap: var(--space-2);
  }
  .timeline-list details,
  .drift-list details {
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
    background: var(--color-surface);
  }
  .timeline-list summary,
  .drift-list summary {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3);
    cursor: pointer;
  }
  .timeline-list summary span,
  .drift-list summary span:first-child {
    display: grid;
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }
  .history-layout {
    display: grid;
    grid-template-columns: minmax(280px, 0.8fr) minmax(360px, 1.2fr);
    align-items: start;
    gap: var(--space-4);
  }
  .history-card {
    display: grid;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-3);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
    text-align: left;
    color: var(--color-text-secondary);
    background: var(--color-surface);
    cursor: pointer;
    font-size: var(--type-label);
  }
  .history-card:hover {
    border-color: var(--color-primary);
    background: var(--color-primary-container);
  }
  .history-card > span:first-child {
    display: flex;
    justify-content: space-between;
    color: var(--color-text);
  }
  .drift-filters {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: var(--space-3);
  }
  .drift-list details.servicing {
    opacity: 0.8;
  }

  .summary-copy {
    display: flex;
    min-width: 158px;
    align-items: center;
    gap: var(--space-3);
  }

  .summary-icon {
    display: grid;
    width: 38px;
    height: 38px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: var(--radius-control);
    color: var(--color-text-secondary);
    background: var(--color-surface-tonal);
  }

  .summary-icon svg {
    width: 21px;
    height: 21px;
    fill: none;
    stroke: currentcolor;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-width: 1.8;
  }

  .summary-copy.planned .summary-icon {
    color: var(--color-primary);
    background: var(--color-primary-container);
  }

  .summary-copy > div {
    display: grid;
  }

  .summary-copy strong {
    font-size: 1.15rem;
  }

  .summary-copy span:last-child {
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }

  .summary-divider {
    width: 1px;
    height: 36px;
    background: var(--color-outline);
  }

  .summary-strip > p {
    margin: 0 0 0 auto;
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }

  .catalogue-heading {
    display: flex;
    flex: 0 0 auto;
    align-items: end;
    justify-content: space-between;
  }

  .catalogue-heading h2,
  .catalogue-heading p {
    margin: 0;
  }

  .catalogue-heading h2 {
    font-size: var(--type-section);
    letter-spacing: -0.025em;
  }

  .catalogue-heading p,
  .catalogue-heading > span {
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }

  .catalogue {
    min-height: 0;
    flex: 1;
    padding: 1px var(--space-1) var(--space-4) 1px;
    overflow: auto;
    scrollbar-color: var(--color-outline) transparent;
    scrollbar-width: thin;
  }

  .cleanup-card {
    display: grid;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    background: var(--color-surface-elevated);
    box-shadow: var(--shadow-card);
    transition:
      border-color var(--motion-standard),
      background var(--motion-standard);
  }

  .cleanup-card + .cleanup-card {
    margin-top: var(--space-3);
  }

  .cleanup-card.planned {
    border-color: var(--color-primary);
    background: var(--color-primary-container);
  }

  .card-heading {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: var(--space-3);
  }

  .component-icon {
    display: grid;
    width: 42px;
    height: 42px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: var(--radius-control);
    color: var(--color-primary);
    background: var(--color-surface-tonal);
    font-weight: 800;
  }

  .cleanup-card.planned .component-icon {
    color: var(--color-on-primary);
    background: var(--color-primary);
  }

  .component-title {
    min-width: 0;
    flex: 1;
  }

  .component-title h3,
  .component-title p {
    overflow: hidden;
    margin: 0;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .component-title h3 {
    font-size: 1.05rem;
    letter-spacing: -0.015em;
  }

  .component-title p {
    margin-top: 2px;
    color: var(--color-primary);
    font-size: var(--type-label);
    font-weight: 650;
  }

  .planned-label {
    display: inline-flex;
    height: 30px;
    align-items: center;
    gap: var(--space-1);
    padding: 0 var(--space-3);
    border-radius: var(--radius-pill);
    color: var(--color-success);
    background: var(--color-success-container);
    font-size: var(--type-label);
    font-weight: 750;
  }

  .action-button {
    min-height: 42px;
    padding: 0 var(--space-4);
    border: 1px solid var(--color-primary);
    border-radius: var(--radius-pill);
    color: var(--color-on-primary);
    background: var(--color-primary);
    cursor: pointer;
    font-size: var(--type-supporting);
    font-weight: 700;
    transition:
      background var(--motion-fast),
      border-color var(--motion-fast),
      opacity var(--motion-fast);
  }

  .action-button:hover:not(:disabled) {
    border-color: var(--color-primary-hover);
    background: var(--color-primary-hover);
  }

  .action-button.secondary {
    color: var(--color-primary);
    background: var(--color-surface);
  }

  .cleanup-card.planned .action-button.secondary {
    background: transparent;
  }

  .action-button.secondary:hover:not(:disabled) {
    background: var(--color-primary-container-hover);
  }

  .action-button:disabled {
    border-color: var(--color-disabled);
    color: var(--color-text-secondary);
    background: var(--color-surface-tonal);
    cursor: not-allowed;
    opacity: 0.68;
  }

  .component-description {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
    line-height: 1.45;
  }

  .metadata {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .metadata span {
    display: inline-flex;
    min-height: 27px;
    align-items: center;
    padding: 0 var(--space-3);
    border-radius: var(--radius-pill);
    color: var(--color-text-secondary);
    background: var(--color-surface-tonal);
    font-size: 0.7rem;
    font-weight: 650;
  }

  .cleanup-card.planned .metadata span {
    background: var(--color-surface);
  }

  .metadata span.warning {
    color: var(--color-warning);
    background: var(--color-warning-container);
  }

  .proposed-action {
    display: grid;
    grid-template-columns: 140px minmax(0, 1fr);
    gap: var(--space-3);
    padding-top: var(--space-3);
    border-top: 1px solid var(--color-outline);
  }

  .proposed-action span {
    color: var(--color-text);
    font-size: var(--type-label);
    font-weight: 750;
  }

  .proposed-action p {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: var(--type-label);
    line-height: 1.45;
  }

  .empty-state {
    display: flex;
    min-height: 260px;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
    border: 1px dashed var(--color-outline-strong);
    border-radius: var(--radius-card);
    text-align: center;
    background: var(--color-surface);
  }

  .empty-state h2,
  .empty-state p {
    margin: 0;
  }

  .empty-state h2 {
    margin-top: var(--space-3);
    font-size: var(--type-section);
  }

  .empty-state p {
    margin: var(--space-1) 0 var(--space-4);
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
  }

  .empty-icon {
    display: grid;
    width: 58px;
    height: 58px;
    place-items: center;
    border-radius: var(--radius-card);
    color: var(--color-primary);
    background: var(--color-primary-container);
  }

  .empty-icon svg {
    width: 30px;
    height: 30px;
    fill: none;
    stroke: currentcolor;
    stroke-linecap: round;
    stroke-width: 2;
  }

  .plan-bar {
    position: absolute;
    right: var(--space-6);
    bottom: var(--space-5);
    left: var(--space-6);
    display: grid;
    min-height: 66px;
    grid-template-columns: 40px minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    background: var(--color-surface);
    box-shadow: var(--shadow-floating);
  }

  .plan-bar.active {
    border-color: var(--color-primary);
    background: var(--color-primary-container);
  }

  .plan-count {
    display: grid;
    width: 40px;
    height: 40px;
    place-items: center;
    border-radius: var(--radius-pill);
    color: var(--color-on-primary);
    background: var(--color-disabled);
    font-weight: 800;
  }

  .plan-bar.active .plan-count {
    background: var(--color-primary);
  }

  .plan-bar > div:nth-child(2) {
    display: grid;
    gap: 1px;
  }

  .plan-bar strong {
    font-size: var(--type-supporting);
  }

  .plan-bar span {
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }

  .error-banner {
    position: absolute;
    z-index: 5;
    top: var(--space-4);
    right: var(--space-6);
    left: var(--space-6);
    display: grid;
    min-height: 52px;
    grid-template-columns: 28px minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-control);
    color: var(--color-warning);
    background: var(--color-warning-container);
    box-shadow: var(--shadow-floating);
  }

  .error-banner > span {
    display: grid;
    width: 28px;
    height: 28px;
    place-items: center;
    border-radius: var(--radius-pill);
    color: var(--color-on-primary);
    background: var(--color-warning);
    font-weight: 800;
  }

  .error-banner p {
    margin: 0;
    font-size: var(--type-supporting);
    font-weight: 650;
  }

  .error-banner button {
    min-height: 34px;
    padding: 0 var(--space-3);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-pill);
    color: var(--color-warning);
    background: transparent;
    cursor: pointer;
    font-size: var(--type-label);
    font-weight: 750;
  }

  .coming-soon {
    display: flex;
    min-height: 0;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
    overflow: auto;
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-panel);
    text-align: center;
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
  }

  .extension-art {
    position: relative;
    width: 168px;
    height: 168px;
    flex: 0 0 auto;
    margin-bottom: var(--space-4);
  }

  .orbit {
    position: absolute;
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-pill);
  }

  .orbit-one {
    inset: 8px 28px;
    transform: rotate(42deg);
  }

  .orbit-two {
    inset: 28px 8px;
    transform: rotate(-42deg);
  }

  .extension-core {
    position: absolute;
    top: 45px;
    left: 45px;
    display: grid;
    width: 78px;
    height: 78px;
    place-items: center;
    border: 10px solid var(--color-primary-container);
    border-radius: var(--radius-panel);
    color: var(--color-on-primary);
    background: var(--color-primary);
    box-shadow: var(--shadow-floating);
    font-size: 2.5rem;
    font-weight: 400;
  }

  .satellite {
    position: absolute;
    width: 24px;
    height: 24px;
    border: 5px solid var(--color-surface);
    border-radius: var(--radius-pill);
  }

  .satellite.one {
    top: 20px;
    left: 18px;
    background: var(--color-warning-container);
  }

  .satellite.two {
    right: 14px;
    bottom: 24px;
    background: var(--color-success-container);
  }

  .coming-soon h2 {
    margin: var(--space-2) 0 var(--space-2);
    font-size: clamp(1.5rem, 3vw, 2rem);
    letter-spacing: -0.035em;
  }

  .coming-soon-copy {
    max-width: 650px;
    margin: 0;
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
    line-height: 1.55;
  }

  .extension-chips {
    display: flex;
    max-width: 720px;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--space-2);
    margin: var(--space-5) 0;
  }

  .extension-chips span,
  .future-label {
    display: inline-flex;
    min-height: 34px;
    align-items: center;
    padding: 0 var(--space-3);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-pill);
    color: var(--color-text-secondary);
    background: var(--color-surface-tonal);
    font-size: var(--type-label);
    font-weight: 650;
  }

  .future-label {
    border-color: var(--color-primary-container);
    color: var(--color-primary);
    background: var(--color-primary-container);
    font-weight: 750;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .scrim {
    position: fixed;
    z-index: 20;
    inset: 0;
    display: grid;
    place-items: center;
    padding: var(--space-6);
    background: var(--color-scrim);
  }

  .review-dialog {
    display: flex;
    width: min(680px, 100%);
    max-height: min(620px, 100%);
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-panel);
    background: var(--color-surface-elevated);
    box-shadow: var(--shadow-floating);
  }

  .review-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .review-heading h2,
  .review-heading p:last-child {
    margin: 0;
  }

  .review-heading h2 {
    margin: var(--space-1) 0;
    font-size: 1.65rem;
    letter-spacing: -0.035em;
  }

  .review-heading p:last-child {
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
  }

  .review-warning {
    display: grid;
    min-height: 58px;
    grid-template-columns: 30px minmax(0, 1fr);
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-control);
    color: var(--color-warning);
    background: var(--color-warning-container);
  }

  .review-warning > span {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border-radius: var(--radius-pill);
    color: var(--color-on-primary);
    background: var(--color-warning);
    font-weight: 800;
  }

  .review-warning p {
    margin: 0;
    font-size: var(--type-supporting);
  }

  .review-list {
    display: grid;
    min-height: 0;
    gap: var(--space-2);
    overflow: auto;
  }

  .review-list article {
    display: grid;
    min-height: 68px;
    grid-template-columns: 42px minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3);
    border-radius: var(--radius-control);
    background: var(--color-surface-tonal);
  }

  .review-list h3,
  .review-list p {
    margin: 0;
  }

  .review-list h3 {
    font-size: var(--type-body);
  }

  .review-list p {
    margin-top: 2px;
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }

  .text-button {
    min-height: 38px;
    padding: 0 var(--space-3);
    border: 0;
    border-radius: var(--radius-pill);
    color: var(--color-primary);
    background: transparent;
    cursor: pointer;
    font-size: var(--type-supporting);
    font-weight: 750;
  }

  .text-button:hover:not(:disabled) {
    background: var(--color-primary-container);
  }

  .text-button:disabled {
    color: var(--color-disabled);
    cursor: not-allowed;
  }

  .review-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding-top: var(--space-4);
    border-top: 1px solid var(--color-outline);
  }

  .review-footer > span {
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
  }

  .review-footer > div {
    display: flex;
    gap: var(--space-2);
  }

  @media (max-width: 1060px) {
    .cleanup-page,
    .history-page,
    .extensions-page {
      padding-right: var(--space-5);
      padding-left: var(--space-5);
    }

    .plan-bar {
      right: var(--space-5);
      left: var(--space-5);
    }

    .search-and-filter {
      align-items: stretch;
      flex-direction: column;
      gap: var(--space-2);
    }

    .filter-row {
      overflow-x: auto;
    }

    .summary-strip > p {
      display: none;
    }

    .inspection-overview {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .inspection-stats {
      grid-column: 1 / -1;
      grid-row: 2;
    }
    .history-layout {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 840px) {
    .app-shell {
      grid-template-columns: 84px minmax(0, 1fr);
    }

    .navigation {
      padding-right: var(--space-2);
      padding-left: var(--space-2);
    }

    .brand {
      justify-content: center;
      padding-right: 0;
      padding-left: 0;
    }

    .brand-copy,
    .nav-eyebrow,
    .nav-item > span:last-child,
    .preview-note p {
      display: none;
    }

    .nav-item {
      display: grid;
      grid-template-columns: 1fr;
      justify-items: center;
      padding: var(--space-2);
    }

    .preview-note {
      display: grid;
      place-items: center;
      padding: var(--space-3);
    }

    .preview-note-heading strong {
      display: none;
    }

    .cleanup-page,
    .history-page,
    .extensions-page {
      padding: var(--space-4) var(--space-4) 96px;
    }

    .plan-bar {
      right: var(--space-4);
      bottom: var(--space-4);
      left: var(--space-4);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    *,
    *::before,
    *::after {
      scroll-behavior: auto !important;
      transition-duration: 0.01ms !important;
    }
  }
  .mutation-alpha-panel {
    display: grid;
    gap: var(--space-5);
    padding: var(--space-5);
    border: 1px solid color-mix(in srgb, var(--color-warning) 42%, var(--color-outline));
    border-radius: var(--radius-card);
    background: color-mix(in srgb, var(--color-warning) 6%, var(--color-surface));
  }

  .preview-badge.mutation {
    color: var(--color-warning-strong, #7a4d00);
    background: color-mix(in srgb, var(--color-warning) 16%, var(--color-surface));
  }

  .mutation-warning {
    display: grid;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-control);
    background: var(--color-surface);
  }

  .environment-banner {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: var(--space-2) var(--space-4);
    padding: var(--space-4);
    border: 2px solid var(--color-primary);
    border-radius: var(--radius-control);
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
  }

  .environment-banner > strong,
  .environment-banner > small {
    grid-column: 1 / -1;
  }

  .environment-banner.environment-refused {
    border-color: var(--color-error);
    background: color-mix(in srgb, var(--color-error) 10%, var(--color-surface));
    color: var(--color-error);
  }

  .validation-maturity {
    color: var(--color-warning-strong, #7a4d00);
    font-size: var(--type-label);
    font-weight: 700;
  }

  .mutation-warning p,
  .mutation-warning small,
  .mutation-card p,
  .mutation-history p {
    margin: 0;
    color: var(--color-text-secondary);
  }

  .mutation-progress {
    margin: 0;
    padding: var(--space-3);
    border-radius: var(--radius-control);
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
  }

  .mutation-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-4);
  }

  .mutation-card {
    display: grid;
    align-content: start;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    background: var(--color-surface);
  }

  .mutation-card > div:first-child {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .mutation-card dl {
    display: grid;
    gap: var(--space-2);
    margin: 0;
  }

  .mutation-card dl div {
    display: grid;
    grid-template-columns: 90px 1fr;
    gap: var(--space-2);
  }

  .mutation-card dt {
    color: var(--color-text-secondary);
  }

  .mutation-card dd {
    margin: 0;
  }

  .documentation-links {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .documentation-source {
    overflow-wrap: anywhere;
    color: var(--color-primary);
    font-size: var(--type-label);
  }

  .mutation-history {
    display: grid;
    gap: var(--space-3);
  }

  .mutation-history details {
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
    background: var(--color-surface);
  }

  .mutation-history summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3);
    cursor: pointer;
  }

  .mutation-history summary span:first-child {
    display: grid;
  }

  .validation-evidence-form {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
    margin-top: var(--space-4);
    padding: var(--space-4);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
  }

  .validation-evidence-form legend {
    font-weight: 700;
  }

  .validation-evidence-form label {
    display: grid;
    gap: var(--space-1);
  }

  .validation-evidence-form .checkbox-row,
  .validation-evidence-form button {
    grid-column: 1 / -1;
  }

  .validation-evidence-form .checkbox-row {
    display: flex;
    align-items: center;
  }

  .mutation-dialog {
    max-width: 680px;
  }

  .mutation-plan-detail {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-5);
  }

  .mutation-plan-detail p {
    margin: 0;
  }

  .mutation-plan-detail code {
    overflow-wrap: anywhere;
    color: var(--color-text-secondary);
    font-size: 0.72rem;
  }

  @media (max-width: 1120px) {
    .mutation-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
