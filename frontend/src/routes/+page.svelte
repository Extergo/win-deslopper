<script lang="ts">
  import { onMount } from 'svelte';

  import {
    createBackendClient,
    describeCommandError,
    type ComponentTimelineEntry,
    type DesiredState,
    type DesiredStateRequest,
    type DesiredStateValidation,
    type DetectionObservation,
    type DriftEvent,
    type InspectionHistoryItem,
    type InspectionProgress,
    type PlatformDashboard,
    type ProductComponent,
    type ProductInfo,
    type SnapshotComparison
  } from '$lib/backend';
  import MutationAlphaPanel from '$lib/MutationAlphaPanel.svelte';
  import {
    installInspectionProgressListener,
    isTerminalInspectionPhase
  } from '$lib/inspection-progress';
  import {
    dashboardStatement,
    diagnosticsCategories,
    filterComponents,
    inspectionFreshness,
    inspectionProgressDetail,
    observationStatus,
    onboardingPrinciples,
    previewIsNonExecutable,
    stateText,
    type ComponentFilter,
    type ProductSection
  } from '$lib/product-ui';
  import '$lib/theme.css';

  const backend = createBackendClient();
  const productSections: Array<{
    id: ProductSection;
    label: string;
    glyph: string;
    description: string;
  }> = [
    { id: 'overview', label: 'Overview', glyph: 'O', description: 'Current system summary' },
    { id: 'components', label: 'Components', glyph: 'C', description: '20 observed surfaces' },
    {
      id: 'desired',
      label: 'Desired states',
      glyph: 'P',
      description: 'Preview-only preferences'
    },
    { id: 'drift', label: 'Drift', glyph: 'D', description: 'Changes over time' },
    { id: 'history', label: 'History', glyph: 'H', description: 'Saved inspections' },
    {
      id: 'settings',
      label: 'Settings & About',
      glyph: 'S',
      description: 'Privacy and local data'
    }
  ];
  const componentFilters: Array<{ id: ComponentFilter; label: string }> = [
    { id: 'all', label: 'All' },
    { id: 'changed', label: 'Changed' },
    { id: 'desired_differs', label: 'Desired differs' },
    { id: 'managed', label: 'Managed externally' },
    { id: 'permission_limited', label: 'Permission limited' },
    { id: 'unknown', label: 'Unknown' },
    { id: 'failed', label: 'Failed' },
    { id: 'package', label: 'Package-backed' },
    { id: 'policy', label: 'Policy-backed' },
    { id: 'preference', label: 'User preference' }
  ];

  let loading = true;
  let errorMessage = '';
  let progressError = '';
  let activeSection: ProductSection = 'overview';
  let onboardingVisible = false;
  let platform: PlatformDashboard | null = null;
  let productInfo: ProductInfo | null = null;
  let catalogue: ProductComponent[] = [];
  let inspectionHistory: InspectionHistoryItem[] = [];
  let driftHistory: DriftEvent[] = [];
  let inspectionProgress: InspectionProgress | null = null;
  let inspectionRunning = false;
  let componentFilter: ComponentFilter = 'all';
  let componentQuery = '';
  let selectedComponentId = '';
  let selectedTimeline: ComponentTimelineEntry[] = [];
  let desiredOptions: Array<{ key: string; label: string; scope: string; description: string }> =
    [];
  let desiredStateKey = '';
  let desiredNote = '';
  let desiredValidation: DesiredStateValidation | null = null;
  let preview: Record<string, unknown> | null = null;
  let comparePrevious = '';
  let compareCurrent = '';
  let comparison: SnapshotComparison | null = null;
  let driftStatusFilter = 'active';
  let driftComponentFilter = 'all';
  let diagnosticsReviewVisible = false;
  let diagnosticsIncludeHistory = true;
  let diagnosticsIncludeErrors = true;
  let diagnosticsStatus = '';
  let clearingHistory = false;

  $: observations = platform?.snapshot?.observations ?? [];
  $: sections =
    productInfo?.buildMode === 'internal mutation-alpha compile'
      ? [
          ...productSections,
          {
            id: 'mutation_alpha' as const,
            label: 'Experimental Apply & Undo',
            glyph: 'X',
            description: 'Internal Mutation Alpha'
          }
        ]
      : productSections;
  $: desiredStates = platform?.desiredStates ?? [];
  $: filteredComponents = filterComponents(
    catalogue,
    observations,
    desiredStates,
    driftHistory,
    componentFilter,
    componentQuery
  );
  $: selectedComponent = catalogue.find((item) => item.componentId === selectedComponentId);
  $: selectedObservation = observations.find((item) => item.componentId === selectedComponentId);
  $: selectedDesired = desiredStates.find((item) => item.component_id === selectedComponentId);
  $: selectedDrift = driftHistory.filter((item) => item.component_id === selectedComponentId);
  $: freshness = inspectionFreshness(platform?.snapshot?.timestamp);
  $: visibleDrift = driftHistory.filter((event) => {
    if (driftStatusFilter === 'active' && event.resolved) return false;
    if (driftStatusFilter === 'resolved' && !event.resolved) return false;
    if (driftStatusFilter === 'unreviewed' && event.reviewed) return false;
    return driftComponentFilter === 'all' || event.component_id === driftComponentFilter;
  });

  onMount(() => {
    onboardingVisible = window.localStorage.getItem('deslopper-onboarding-complete') !== 'true';
    const removeProgressListener = installInspectionProgressListener(backend, {
      onProgress: handleInspectionProgress,
      onError: (message) => (progressError = message)
    });
    void backend.getRunningInspectionState().then((running) => {
      inspectionProgress = running;
      inspectionRunning = Boolean(running && !isTerminalInspectionPhase(running.phase));
    });
    void loadProduct();
    return removeProgressListener;
  });

  async function loadProduct(): Promise<void> {
    loading = true;
    errorMessage = '';
    try {
      [productInfo, catalogue, platform, inspectionHistory, driftHistory] = await Promise.all([
        backend.getProductInfo(),
        backend.getProductComponentCatalogue(),
        backend.getPlatformDashboard(),
        backend.getInspectionHistory(),
        backend.getDriftHistory()
      ]);
      if (platform.snapshot) onboardingVisible = false;
      initialiseComparison();
    } catch (error) {
      errorMessage = describeCommandError(error);
    } finally {
      loading = false;
    }
  }

  async function refreshSavedData(): Promise<void> {
    [platform, inspectionHistory, driftHistory] = await Promise.all([
      backend.getPlatformDashboard(),
      backend.getInspectionHistory(),
      backend.getDriftHistory()
    ]);
    initialiseComparison();
  }

  function initialiseComparison(): void {
    if (inspectionHistory.length >= 2 && (!comparePrevious || !compareCurrent)) {
      compareCurrent = inspectionHistory[0].inspectionId;
      comparePrevious = inspectionHistory[1].inspectionId;
    }
  }

  function handleInspectionProgress(value: InspectionProgress): void {
    inspectionProgress = value;
    inspectionRunning = !isTerminalInspectionPhase(value.phase);
    if (!inspectionRunning) void refreshSavedData();
  }

  async function runInspection(): Promise<void> {
    errorMessage = '';
    completeOnboarding();
    try {
      inspectionProgress = await backend.startInspection();
      inspectionRunning = true;
      activeSection = 'overview';
    } catch (error) {
      errorMessage = describeCommandError(error);
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

  function completeOnboarding(): void {
    window.localStorage.setItem('deslopper-onboarding-complete', 'true');
    onboardingVisible = false;
  }

  function skipOnboarding(): void {
    completeOnboarding();
    activeSection = 'overview';
  }

  function modalDialog(node: HTMLDialogElement): { destroy: () => void } {
    node.showModal();
    return {
      destroy: () => {
        if (node.open) node.close();
      }
    };
  }

  async function openComponent(componentId: string): Promise<void> {
    selectedComponentId = componentId;
    preview = null;
    desiredValidation = null;
    desiredOptions = observationFor(componentId)
      ? await backend.getAllowedDesiredStateOptions(componentId)
      : [];
    desiredStateKey = desiredOptions[0]?.key ?? '';
    desiredNote = desiredFor(componentId)?.note ?? '';
    selectedTimeline = platform?.snapshot ? await backend.getComponentTimeline(componentId) : [];
  }

  function desiredRequest(): DesiredStateRequest {
    const option = desiredOptions.find((item) => item.key === desiredStateKey);
    return {
      componentId: selectedComponentId,
      stateKey: desiredStateKey,
      scope: option?.scope ?? 'effective',
      persistentRemediation: false,
      alwaysRequireApproval: true,
      note: desiredNote.trim() || null
    };
  }

  async function validateDesired(): Promise<void> {
    if (!desiredStateKey) return;
    desiredValidation = await backend.validateDesiredState(desiredRequest());
  }

  async function saveDesired(): Promise<void> {
    await validateDesired();
    if (!desiredValidation?.valid) return;
    platform = await backend.saveDesiredState(desiredRequest());
    preview = await backend.generatePreviewPlan(selectedComponentId);
  }

  async function clearDesired(): Promise<void> {
    platform = await backend.clearDesiredState(selectedComponentId);
    preview = null;
    desiredValidation = null;
  }

  async function generatePreview(): Promise<void> {
    try {
      preview = await backend.generatePreviewPlan(selectedComponentId);
    } catch (error) {
      errorMessage = describeCommandError(error);
    }
  }

  async function reviewDrift(event: DriftEvent): Promise<void> {
    driftHistory = await backend.acknowledgeDriftEvent(event.component_id, event.classification);
  }

  async function compareSnapshots(): Promise<void> {
    try {
      comparison = await backend.compareInspections(comparePrevious, compareCurrent);
    } catch (error) {
      comparison = null;
      errorMessage = describeCommandError(error);
    }
  }

  async function updateRetention(event: Event): Promise<void> {
    const days = Number((event.currentTarget as HTMLSelectElement).value);
    try {
      platform = await backend.setHistoryRetention(days);
      await refreshSavedData();
    } catch (error) {
      errorMessage = describeCommandError(error);
    }
  }

  async function clearHistory(): Promise<void> {
    if (
      !window.confirm(
        'Clear Deslopper inspection history, drift, desired states, and preview records? Windows settings will not be changed.'
      )
    )
      return;
    clearingHistory = true;
    try {
      platform = await backend.clearLocalHistory(true);
      inspectionHistory = [];
      driftHistory = [];
      selectedComponentId = '';
      comparison = null;
    } catch (error) {
      errorMessage = describeCommandError(error);
    } finally {
      clearingHistory = false;
    }
  }

  async function exportDiagnostics(): Promise<void> {
    diagnosticsStatus = 'Preparing a redacted local file...';
    try {
      const bundle = await backend.generateDiagnosticsExport({
        includeHistorySummary: diagnosticsIncludeHistory,
        includeRedactedErrors: diagnosticsIncludeErrors
      });
      const content = `${JSON.stringify(bundle, null, 2)}\n`;
      const suggestedName = `deslopper-diagnostics-${new Date().toISOString().slice(0, 10)}.json`;
      const savePicker = (
        window as Window & {
          showSaveFilePicker?: (options: Record<string, unknown>) => Promise<{
            createWritable: () => Promise<{
              write: (value: string) => Promise<void>;
              close: () => Promise<void>;
            }>;
          }>;
        }
      ).showSaveFilePicker;
      if (savePicker) {
        const handle = await savePicker({
          suggestedName,
          types: [
            {
              description: 'JSON diagnostics',
              accept: { 'application/json': ['.json'] }
            }
          ]
        });
        const writable = await handle.createWritable();
        await writable.write(content);
        await writable.close();
      } else {
        const url = URL.createObjectURL(new Blob([content], { type: 'application/json' }));
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.download = suggestedName;
        anchor.click();
        URL.revokeObjectURL(url);
      }
      diagnosticsStatus = 'Diagnostics saved locally. Nothing was uploaded.';
      diagnosticsReviewVisible = false;
    } catch (error) {
      diagnosticsStatus = `Export was not completed. ${describeCommandError(error)}`;
    }
  }

  function displayTime(value: string): string {
    const numeric = Number(value);
    return Number.isFinite(numeric) ? new Date(numeric).toLocaleString() : value;
  }

  function componentName(id: string): string {
    return catalogue.find((item) => item.componentId === id)?.name ?? id.replaceAll('_', ' ');
  }

  function desiredFor(componentId: string): DesiredState | undefined {
    return desiredStates.find((item) => item.component_id === componentId);
  }

  function observationFor(componentId: string): DetectionObservation | undefined {
    return observations.find((item) => item.componentId === componentId);
  }
</script>

<svelte:head>
  <title>Deslopper - Read-Only Product Alpha</title>
  <meta
    name="description"
    content="A privacy-conscious, read-only Windows configuration inspector."
  />
</svelte:head>

{#if loading}
  <main class="boot" aria-live="polite">
    <div class="logo" aria-hidden="true">D</div>
    <div>
      <strong>Opening Deslopper</strong><span>Loading your local inspection history...</span>
    </div>
  </main>
{:else if platform && productInfo}
  <div class="shell">
    <aside class="sidebar" aria-label="Primary navigation">
      <div class="brand">
        <div class="logo" aria-hidden="true">D</div>
        <div>
          <strong>Deslopper</strong><span
            >{productInfo.buildMode === 'internal mutation-alpha compile'
              ? 'Internal Mutation Alpha'
              : 'Read-Only Product Alpha'}</span
          >
        </div>
      </div>
      <nav>
        {#each sections as section (section.id)}
          <button
            class:active={activeSection === section.id}
            aria-current={activeSection === section.id ? 'page' : undefined}
            onclick={() => (activeSection = section.id)}
          >
            <span class="nav-glyph" aria-hidden="true">{section.glyph}</span>
            <span><strong>{section.label}</strong><small>{section.description}</small></span>
          </button>
        {/each}
      </nav>
      <div class="sidebar-spacer"></div>
      {#if productInfo.buildMode === 'internal mutation-alpha compile'}
        <section class="safety-note internal" aria-label="Build safety status">
          <strong><span aria-hidden="true">●</span> Internal build</strong>
          <p>Mutation remains unavailable unless every backend safety gate passes.</p>
        </section>
      {:else}
        <section class="safety-note" aria-label="Build safety status">
          <strong><span aria-hidden="true">●</span> Read-only</strong>
          <p>I inspect and explain. I do not apply Windows changes in this build.</p>
        </section>
      {/if}
    </aside>

    <main class="workspace" aria-busy={inspectionRunning}>
      {#if errorMessage}
        <section class="error-banner" role="alert">
          <div>
            <strong>Deslopper could not complete that local action.</strong>
            <p>{errorMessage}</p>
          </div>
          <button onclick={() => void loadProduct()}>Retry safely</button>
        </section>
      {/if}
      {#if progressError}
        <section class="notice" role="status">
          <strong>Live progress is unavailable.</strong><span>{progressError}</span>
        </section>
      {/if}

      {#if activeSection === 'overview'}
        <section aria-labelledby="overview-title">
          <header class="hero">
            <div>
              <span class="eyebrow">Local Windows configuration</span>
              <h1 id="overview-title">A clear view of what Windows is doing.</h1>
              <p>{dashboardStatement(platform)}</p>
              <div class="hero-actions">
                <button
                  class="primary"
                  disabled={inspectionRunning}
                  onclick={() => void runInspection()}
                  >{platform.snapshot ? 'Run a fresh inspection' : 'Start first inspection'}</button
                >
                {#if inspectionRunning && inspectionProgress?.cancellationAvailable}<button
                    class="secondary"
                    onclick={() => void cancelInspection()}>Cancel safely</button
                  >{/if}
              </div>
            </div>
            <div class="read-only-seal">
              <span aria-hidden="true">✓</span><strong>Read-only Product Alpha</strong><small
                >No UAC. No silent changes. Local history.</small
              >
            </div>
          </header>

          {#if inspectionProgress}
            <section class="progress-panel" aria-live="polite">
              <div class="progress-heading">
                <strong>{inspectionProgress.status}</strong><span
                  >{inspectionProgress.completedWork} of {inspectionProgress.totalWork}</span
                >
              </div>
              <progress
                max={inspectionProgress.totalWork || 1}
                value={inspectionProgress.completedWork}
              ></progress>
              <p>
                {inspectionProgressDetail(
                  inspectionProgress,
                  inspectionProgress.currentComponent
                    ? componentName(inspectionProgress.currentComponent)
                    : undefined
                )} · {inspectionProgress.warningCount} warnings · {inspectionProgress.errorCount} failures
              </p>
              {#if inspectionProgress.phase === 'cancelled'}<p class="plain-status">
                  Cancelled. Completed evidence was preserved; queued checks were not treated as
                  successful.
                </p>{/if}
              {#if inspectionProgress.phase === 'completed_with_partial_failures'}<p
                  class="plain-status"
                >
                  Completed with partial evidence. Unknown and permission-limited results remain
                  explicit.
                </p>{/if}
            </section>
          {/if}

          <section class="metrics" aria-label="Configuration summary">
            <article>
              <span>Last inspection</span><strong
                >{platform.snapshot ? displayTime(platform.snapshot.timestamp) : 'Not yet'}</strong
              ><small>{freshness}</small>
            </article>
            <article>
              <span>Detected drift</span><strong>{platform.driftCount}</strong><small
                >Unresolved configuration changes</small
              >
            </article>
            <article>
              <span>Managed externally</span><strong>{platform.managedCount}</strong><small
                >Policy remains authoritative</small
              >
            </article>
            <article>
              <span>Unknown or limited</span><strong
                >{platform.unknownCount + platform.permissionLimitedCount}</strong
              ><small>Not interpreted as absence</small>
            </article>
            <article>
              <span>Failed detectors</span><strong>{platform.failedCount}</strong><small
                >Retryable, scoped failures</small
              >
            </article>
          </section>

          <div class="overview-grid">
            <section class="surface">
              <div class="section-heading">
                <div>
                  <span class="eyebrow">Current evidence</span>
                  <h2>Configuration at a glance</h2>
                </div>
                <button class="text-button" onclick={() => (activeSection = 'components')}
                  >Browse all 20</button
                >
              </div>
              {#if platform.snapshot}
                <div class="compact-list">
                  {#each catalogue.slice(0, 6) as component (component.componentId)}
                    <button
                      onclick={() => {
                        void openComponent(component.componentId);
                        activeSection = 'components';
                      }}
                    >
                      <span
                        ><strong>{component.name}</strong><small>{component.category}</small></span
                      >
                      <span class="status-text"
                        >{observationStatus(observationFor(component.componentId))}</span
                      >
                    </button>
                  {/each}
                </div>
              {:else}
                <div class="empty">
                  <strong>No inspection yet</strong>
                  <p>
                    Start a read-only inspection to populate all 20 component views. You can cancel
                    safely while supported.
                  </p>
                </div>
              {/if}
            </section>
            <section class="surface trust-card">
              <span class="eyebrow">What I can do</span>
              <h2>Observe first, explain clearly</h2>
              <ul>
                <li>Distinguish your choices from Windows servicing and administrator policy.</li>
                <li>Keep snapshots and meaningful drift history locally.</li>
                <li>Save desired states and generate non-executable previews.</li>
                <li>Export a privacy-reviewed diagnostic file only when you ask.</li>
              </ul>
              <div class="boundary">
                <strong>Automatic restoration is not available in this build.</strong><span
                  >Preview records create no approval nonce or mutation transaction.</span
                >
              </div>
            </section>
          </div>
        </section>
      {:else if activeSection === 'components'}
        <section aria-labelledby="components-title">
          <header class="page-header">
            <div>
              <span class="eyebrow">20 registered components</span>
              <h1 id="components-title">Browse what Deslopper observes</h1>
              <p>
                Technical evidence stays secondary to the conclusion, its confidence, and its
                limits.
              </p>
            </div>
            <span class="badge">{filteredComponents.length} shown</span>
          </header>
          <div class="component-tools">
            <label class="search"
              ><span class="sr-only">Search components</span><input
                bind:value={componentQuery}
                placeholder="Search name, purpose, or category"
              /></label
            >
            <div class="filter-row" aria-label="Component filters">
              {#each componentFilters as filter (filter.id)}<button
                  class:active={componentFilter === filter.id}
                  aria-pressed={componentFilter === filter.id}
                  onclick={() => (componentFilter = filter.id)}>{filter.label}</button
                >{/each}
            </div>
          </div>
          <div class="component-layout">
            <div class="component-list" aria-live="polite">
              {#each filteredComponents as component (component.componentId)}
                {@const observation = observationFor(component.componentId)}
                <button
                  class:selected={selectedComponentId === component.componentId}
                  onclick={() => void openComponent(component.componentId)}
                >
                  <span class="component-icon" aria-hidden="true">{component.name.slice(0, 1)}</span
                  >
                  <span
                    ><strong>{component.name}</strong><small
                      >{component.category} · {stateText(observation?.current)}</small
                    ></span
                  >
                  <span class="status-text">{observationStatus(observation)}</span>
                </button>
              {:else}<div class="empty">
                  <strong>No components match</strong>
                  <p>Try a broader filter or clear the search.</p>
                </div>{/each}
            </div>
            <section class="component-detail" aria-live="polite">
              {#if selectedComponent}
                <div class="detail-heading">
                  <div>
                    <span class="eyebrow">{selectedComponent.category}</span>
                    <h2>{selectedComponent.name}</h2>
                    <p>{selectedComponent.purpose}</p>
                  </div>
                  <span class="badge">{observationStatus(selectedObservation)}</span>
                </div>
                <dl class="facts">
                  <div>
                    <dt>Observed state</dt>
                    <dd>{stateText(selectedObservation?.current)}</dd>
                  </div>
                  <div>
                    <dt>Evidence freshness</dt>
                    <dd>
                      {selectedObservation
                        ? displayTime(selectedObservation.detectedAt)
                        : 'Not inspected'}
                    </dd>
                  </div>
                  <div>
                    <dt>Authority</dt>
                    <dd>
                      {selectedObservation?.authority ?? 'Unknown'} · {selectedObservation
                        ?.authorityAttribution.confidence ?? 'Unknown'} confidence
                    </dd>
                  </div>
                  <div>
                    <dt>Applicability</dt>
                    <dd>
                      {selectedObservation?.applicability.status.replaceAll('_', ' ') ?? 'Unknown'}
                    </dd>
                  </div>
                  <div>
                    <dt>Completeness</dt>
                    <dd>
                      {selectedObservation?.packageCompleteness.replaceAll('_', ' ') ??
                        'Not package-backed'}
                    </dd>
                  </div>
                  <div>
                    <dt>Desired state</dt>
                    <dd>{selectedDesired ? stateText(selectedDesired.state) : 'Not configured'}</dd>
                  </div>
                  <div>
                    <dt>Drift</dt>
                    <dd>
                      {selectedDrift.filter((item) => !item.resolved).length
                        ? `${selectedDrift.filter((item) => !item.resolved).length} unresolved event(s)`
                        : 'No unresolved drift'}
                    </dd>
                  </div>
                  <div>
                    <dt>Saved timeline</dt>
                    <dd>{selectedTimeline.length} comparable observation(s)</dd>
                  </div>
                  <div>
                    <dt>Restart / sign-out</dt>
                    <dd>{selectedComponent.restart.replaceAll('_', ' ')}</dd>
                  </div>
                </dl>
                <section class="explanation">
                  <h3>Why I reached this conclusion</h3>
                  <p>
                    {selectedObservation?.applicability.reason ??
                      'Run an inspection to collect current evidence.'}
                  </p>
                  {#if selectedObservation?.warnings.length}<ul>
                      {#each selectedObservation.warnings as warning (warning)}<li>
                          {warning}
                        </li>{/each}
                    </ul>{/if}{#if selectedObservation?.error}<p class="error-copy">
                      This component failed inspection: {selectedObservation.error}
                    </p>{/if}
                </section>
                <div class="tradeoffs">
                  <article>
                    <h3>What you gain</h3>
                    <p>{selectedComponent.benefit}</p>
                  </article>
                  <article>
                    <h3>What to consider</h3>
                    <p>{selectedComponent.gamingNotes} {selectedComponent.enterpriseNotes}</p>
                  </article>
                </div>
                <section class="support-strip" aria-label="Product support levels">
                  <span><strong>Observe</strong> Supported</span><span
                    ><strong>Preview</strong>
                    {desiredOptions.length
                      ? 'Available with complete evidence'
                      : 'Unavailable for current evidence'}</span
                  ><span><strong>Apply</strong> Unavailable in Product Alpha</span>
                </section>
                <section class="desired-editor">
                  <div class="section-heading">
                    <div>
                      <h3>Desired state and preview</h3>
                      <p>Saving a preference changes only Deslopper's local database.</p>
                    </div>
                    {#if selectedDesired}<button
                        class="text-button danger"
                        onclick={() => void clearDesired()}>Clear desired state</button
                      >{/if}
                  </div>
                  {#if desiredOptions.length}
                    <label
                      >Desired state<select
                        bind:value={desiredStateKey}
                        onchange={() => void validateDesired()}
                        >{#each desiredOptions as option (option.key)}<option value={option.key}
                            >{option.label}</option
                          >{/each}</select
                      ></label
                    >
                    <label
                      >Optional local note<textarea
                        bind:value={desiredNote}
                        maxlength="500"
                        placeholder="Why this state matters to you"></textarea></label
                    >
                    {#if desiredValidation}<div
                        class="validation"
                        class:invalid={!desiredValidation.valid}
                      >
                        <strong>{desiredValidation.status.replaceAll('_', ' ')}</strong>
                        <p>{desiredValidation.reason}</p>
                        {#each desiredValidation.warnings as warning (warning)}<small
                            >{warning}</small
                          >{/each}
                      </div>{/if}
                    <div class="button-row">
                      <button class="primary" onclick={() => void saveDesired()}
                        >Save and generate preview</button
                      >{#if selectedDesired}<button
                          class="secondary"
                          onclick={() => void generatePreview()}>Regenerate preview</button
                        >{/if}
                    </div>
                  {:else}<p class="plain-status">
                      A preview is unavailable until current applicability and evidence are
                      sufficient. Unknown does not mean broken.
                    </p>{/if}
                </section>
                {#if preview}
                  <section class="preview-card">
                    <div>
                      <span class="eyebrow">Preview only · not yet applied</span>
                      <h3>What would be reviewed</h3>
                    </div>
                    <dl class="facts">
                      <div>
                        <dt>Current</dt>
                        <dd>{stateText(preview.currentState as Record<string, unknown>)}</dd>
                      </div>
                      <div>
                        <dt>Desired</dt>
                        <dd>{stateText(preview.desiredState as Record<string, unknown>)}</dd>
                      </div>
                      <div>
                        <dt>Required authority</dt>
                        <dd>{String(preview.authority ?? 'Unknown')}</dd>
                      </div>
                      <div>
                        <dt>Expected representation</dt>
                        <dd>{String(preview.mechanism ?? 'Research required')}</dd>
                      </div>
                      <div>
                        <dt>Restart</dt>
                        <dd>{String(preview.restart ?? 'Unknown')}</dd>
                      </div>
                      <div>
                        <dt>Rollback concept</dt>
                        <dd>{String(preview.rollback ?? 'A verified source would be required')}</dd>
                      </div>
                    </dl>
                    <div class="boundary">
                      <strong
                        >{previewIsNonExecutable(preview)
                          ? 'This preview cannot execute.'
                          : 'Preview safety state is incomplete.'}</strong
                      ><span
                        >{String(
                          preview.cannotExecute ??
                            'Automatic restoration is not available in this build.'
                        )}</span
                      >
                    </div>
                  </section>
                {/if}
                <details class="technical">
                  <summary>Redacted technical evidence</summary>{#if selectedObservation}<p>
                      Detector: {selectedObservation.detectorStatus} · official support: {selectedComponent.support}
                    </p>
                    {#each selectedObservation.evidence as evidence (evidence.queryId + evidence.source)}<article
                      >
                        <strong>{evidence.source}</strong><span>{evidence.detail}</span><small
                          >Confidence {evidence.confidence}/100</small
                        >
                      </article>{/each}{#if selectedObservation.packages.length}<p>
                        {selectedObservation.packages.length} exact package observation(s); install paths
                        are not stored.
                      </p>{/if}{:else}<p>No evidence has been collected.</p>{/if}
                  <p>Reference: {selectedComponent.documentation}</p>
                </details>
              {:else}<div class="empty detail-empty">
                  <strong>Select a component</strong>
                  <p>
                    Choose any of the 20 registered components to review its purpose, observed
                    state, authority, applicability, desired state, and redacted evidence.
                  </p>
                </div>{/if}
            </section>
          </div>
        </section>
      {:else if activeSection === 'desired'}
        <section aria-labelledby="desired-title">
          <header class="page-header">
            <div>
              <span class="eyebrow">Local preferences</span>
              <h1 id="desired-title">Desired states and previews</h1>
              <p>
                Compare what you want with what Deslopper observed. Nothing here can apply a Windows
                change.
              </p>
            </div>
            <span class="badge">{desiredStates.length} saved</span>
          </header>
          {#if desiredStates.length}
            <div class="card-grid">
              {#each desiredStates as desired (desired.component_id)}{@const observation =
                  observationFor(desired.component_id)}
                <article class="surface">
                  <span class="eyebrow">{desired.validation_status.replaceAll('_', ' ')}</span>
                  <h2>{componentName(desired.component_id)}</h2>
                  <dl class="facts">
                    <div>
                      <dt>Observed</dt>
                      <dd>{stateText(observation?.current)}</dd>
                    </div>
                    <div>
                      <dt>Desired</dt>
                      <dd>{stateText(desired.state)}</dd>
                    </div>
                    <div>
                      <dt>Saved for</dt>
                      <dd>{desired.selected_edition} build {desired.selected_build}</dd>
                    </div>
                    <div>
                      <dt>Revision</dt>
                      <dd>{desired.revision}</dd>
                    </div>
                  </dl>
                  <div class="boundary">
                    <strong>Not yet applied</strong><span
                      >Automatic restoration is not available in this build.</span
                    >
                  </div>
                  <button
                    class="secondary"
                    onclick={() => {
                      void openComponent(desired.component_id);
                      activeSection = 'components';
                    }}>Review or regenerate preview</button
                  >
                </article>{/each}
            </div>
          {:else}<div class="large-empty">
              <strong>No desired states yet</strong>
              <p>
                Inspect Windows, open a component, and save a supported preference. Deslopper will
                validate the evidence and create only a non-executable preview.
              </p>
              <button class="primary" onclick={() => (activeSection = 'components')}
                >Browse components</button
              >
            </div>{/if}
        </section>
      {:else if activeSection === 'drift'}
        <section aria-labelledby="drift-title">
          <header class="page-header">
            <div>
              <span class="eyebrow">History-aware evidence</span>
              <h1 id="drift-title">Configuration drift</h1>
              <p>
                Meaningful state, policy, applicability, and uncertainty changes. Normal package
                version servicing is not classified as drift.
              </p>
            </div>
            <span class="badge">{visibleDrift.length} shown</span>
          </header>
          <div class="control-row">
            <label
              >Status<select bind:value={driftStatusFilter}
                ><option value="active">Active</option><option value="unreviewed">Unreviewed</option
                ><option value="resolved">Resolved</option><option value="all">All</option></select
              ></label
            ><label
              >Component<select bind:value={driftComponentFilter}
                ><option value="all">All components</option
                >{#each catalogue as component (component.componentId)}<option
                    value={component.componentId}>{component.name}</option
                  >{/each}</select
              ></label
            >
          </div>
          <div class="drift-list">
            {#each visibleDrift as event (`${event.component_id}-${event.classification}-${event.first_detected}`)}<details
                class="surface"
              >
                <summary
                  ><span
                    ><strong>{componentName(event.component_id)}</strong><small
                      >{event.classification.replaceAll(/([A-Z])/g, ' $1').trim()} · {event.resolved
                        ? 'Resolved'
                        : event.reviewed
                          ? 'Reviewed'
                          : 'Needs review'}</small
                    ></span
                  ><span class="status-text">{event.confidence} confidence</span></summary
                >
                <div class="drift-detail">
                  <div class="state-change">
                    <span><small>Previous</small><strong>{stateText(event.previous)}</strong></span
                    ><span aria-hidden="true">→</span><span
                      ><small>Current</small><strong>{stateText(event.current)}</strong></span
                    ><span aria-hidden="true">→</span><span
                      ><small>Desired</small><strong
                        >{event.desired ? stateText(event.desired) : 'Not set'}</strong
                      ></span
                    >
                  </div>
                  <p><strong>Likely cause:</strong> {event.cause}</p>
                  {#each event.supporting_facts as fact (fact)}<p>
                      Evidence: {fact}
                    </p>{/each}{#if event.alternative_causes.length}<p>
                      Other plausible causes: {event.alternative_causes.join(', ')}
                    </p>{/if}
                  <p>
                    Related inspections: {event.previous_inspection_id ?? 'unknown'} → {event.current_inspection_id ??
                      'unknown'}
                  </p>
                  {#if event.returned_to_desired}<p class="success-copy">
                      A later inspection found that this component returned to its desired value.
                    </p>{/if}{#if !event.reviewed}<button
                      class="secondary"
                      onclick={() => void reviewDrift(event)}>Mark reviewed locally</button
                    >{/if}
                </div>
              </details>{:else}<div class="large-empty">
                <strong>No drift events match</strong>
                <p>
                  Drift appears only after comparable saved inspections. Failed and cancelled
                  detector results are not treated as proof of change.
                </p>
              </div>{/each}
          </div>
        </section>
      {:else if activeSection === 'history'}
        <section aria-labelledby="history-title">
          <header class="page-header">
            <div>
              <span class="eyebrow">Persisted locally</span>
              <h1 id="history-title">Inspection history</h1>
              <p>
                Review duration, Windows build, completion state, and component-level differences.
              </p>
            </div>
            <span class="badge">{inspectionHistory.length} saved</span>
          </header>
          {#if inspectionHistory.length >= 2}<section class="compare surface">
              <div class="section-heading">
                <div>
                  <h2>Compare snapshots</h2>
                  <p>Choose two inspections from this machine identity.</p>
                </div>
                <button class="primary" onclick={() => void compareSnapshots()}>Compare</button>
              </div>
              <div class="control-row">
                <label
                  >Previous<select bind:value={comparePrevious}
                    >{#each inspectionHistory as item (item.inspectionId)}<option
                        value={item.inspectionId}
                        >{displayTime(item.timestamp)} · build {item.windowsBuild}</option
                      >{/each}</select
                  ></label
                ><label
                  >Current<select bind:value={compareCurrent}
                    >{#each inspectionHistory as item (item.inspectionId)}<option
                        value={item.inspectionId}
                        >{displayTime(item.timestamp)} · build {item.windowsBuild}</option
                      >{/each}</select
                  ></label
                >
              </div>
              {#if comparison}<div class="comparison-results">
                  <strong>{comparison.changes.length} component-level change(s)</strong
                  >{#each comparison.changes as change (change.componentId)}<article>
                      <span
                        ><strong>{componentName(change.componentId)}</strong><small
                          >{change.previousStatus} → {change.currentStatus}</small
                        ></span
                      ><span
                        >{stateText(change.previousState)} → {stateText(change.currentState)}</span
                      >
                      <p>{change.explanation}</p>
                    </article>{/each}{#if comparison.changes.length === 0}<p>
                      No state, authority, applicability, or detector-status differences were found.
                    </p>{/if}
                </div>{/if}
            </section>{/if}
          <div class="history-list">
            {#each inspectionHistory as item (item.inspectionId)}<article class="surface">
                <div>
                  <strong>{displayTime(item.timestamp)}</strong><span class="status-text"
                    >{item.status}</span
                  >
                </div>
                <p>{item.windowsEdition} · build {item.windowsBuild} · {item.durationMs ?? 0} ms</p>
                <div class="count-row">
                  <span>{item.successfulCount} successful</span><span
                    >{item.unknownCount} unknown</span
                  ><span>{item.failedCount} failed</span><span>{item.cancelledCount} cancelled</span
                  ><span>{item.driftCount} drift</span>
                </div>
                <small>Trigger: user-requested local inspection · {item.managementSummary}</small>
              </article>{:else}<div class="large-empty">
                <strong>No saved inspections</strong>
                <p>
                  Your first completed or cancelled inspection will appear here and persist after
                  restart.
                </p>
              </div>{/each}
          </div>
        </section>
      {:else if activeSection === 'settings'}
        <section aria-labelledby="settings-title">
          <header class="page-header">
            <div>
              <span class="eyebrow">Local product controls</span>
              <h1 id="settings-title">Settings & About</h1>
              <p>
                Manage Deslopper's own history and diagnostics. These actions do not alter Windows
                configuration.
              </p>
            </div>
            <span class="badge">v{productInfo.version}</span>
          </header>
          <div class="settings-grid">
            <section class="surface">
              <h2>About Deslopper</h2>
              <dl class="facts">
                <div>
                  <dt>Product</dt>
                  <dd>{productInfo.productName}</dd>
                </div>
                <div>
                  <dt>Release</dt>
                  <dd>{productInfo.releaseLabel}</dd>
                </div>
                <div>
                  <dt>Version</dt>
                  <dd>{productInfo.version}</dd>
                </div>
                <div>
                  <dt>Build mode</dt>
                  <dd>{productInfo.buildMode}</dd>
                </div>
                <div>
                  <dt>Mutation availability</dt>
                  <dd>{productInfo.mutationAvailability}</dd>
                </div>
                <div>
                  <dt>Database schema</dt>
                  <dd>v{productInfo.databaseSchemaVersion}</dd>
                </div>
                <div>
                  <dt>Database location</dt>
                  <dd>{productInfo.databaseLocation}</dd>
                </div>
                <div>
                  <dt>Windows support</dt>
                  <dd>{productInfo.supportedWindows}</dd>
                </div>
              </dl>
              <p class="plain-status">
                Documentation is included with the repository. No network access is required to
                understand this build.
              </p>
            </section>
            <section class="surface">
              <h2>Inspection history</h2>
              <label
                >Retention<select
                  value={platform.historyRetentionDays}
                  onchange={(event) => void updateRetention(event)}
                  ><option value="30">30 days</option><option value="90">90 days</option><option
                    value="180">180 days</option
                  ><option value="365">365 days</option><option value="0"
                    >Keep until I clear it</option
                  ></select
                ></label
              >
              <p>
                Retention applies only to Deslopper's local read-only history. Existing databases
                migrate without a destructive reset.
              </p>
              <button
                class="danger-button"
                disabled={clearingHistory || inspectionHistory.length === 0}
                onclick={() => void clearHistory()}
                >{clearingHistory ? 'Clearing...' : 'Clear local history'}</button
              >
            </section>
            <section class="surface">
              <h2>Privacy-safe diagnostics</h2>
              <p>
                Create a user-controlled JSON file for a bug report. Deslopper never uploads it or
                chooses a repository destination.
              </p>
              <button class="primary" onclick={() => (diagnosticsReviewVisible = true)}
                >Review export contents</button
              >{#if diagnosticsStatus}<p class="plain-status" aria-live="polite">
                  {diagnosticsStatus}
                </p>{/if}
            </section>
            <section class="surface">
              <h2>Privacy model</h2>
              <ul>
                <li>Inspection data stays local by default.</li>
                <li>
                  Profile paths, account identities, sync-root names, and raw command output are
                  excluded or redacted.
                </li>
                <li>Machine identity and the development-host denylist never enter diagnostics.</li>
                <li>
                  The normal capability set has no filesystem, shell, network, updater, or mutation
                  permission.
                </li>
              </ul>
            </section>
          </div>
        </section>
      {:else if productInfo.buildMode === 'internal mutation-alpha compile'}
        <MutationAlphaPanel
          buildMode={productInfo.buildMode}
          sourceInspectionId={platform.snapshot?.id ?? null}
        />
      {/if}
    </main>
  </div>

  {#if onboardingVisible}
    <div class="scrim">
      <dialog use:modalDialog class="onboarding" aria-labelledby="onboarding-title">
        <div class="logo" aria-hidden="true">D</div>
        <span class="eyebrow">Welcome to the Product Alpha</span>
        <h1 id="onboarding-title">Understand Windows before deciding what you want.</h1>
        <p>
          Deslopper gives you a careful local record of supported settings, packages, policy,
          uncertainty, and change over time.
        </p>
        <ul>
          {#each onboardingPrinciples as principle (principle)}<li>
              <span aria-hidden="true">✓</span>{principle}
            </li>{/each}
        </ul>
        <div class="onboarding-actions">
          <button class="primary" onclick={() => void runInspection()}
            >Start first inspection</button
          ><button class="secondary" onclick={skipOnboarding}>Skip to empty dashboard</button>
        </div>
        <small
          >You can cancel supported checks safely. A complete inspection usually takes under a
          minute.</small
        >
      </dialog>
    </div>
  {/if}

  {#if diagnosticsReviewVisible}
    <div class="scrim">
      <dialog use:modalDialog class="diagnostics-review" aria-labelledby="diagnostics-title">
        <span class="eyebrow">Review before writing</span>
        <h2 id="diagnostics-title">Diagnostics export contents</h2>
        <p>The JSON file will include:</p>
        <ul>
          {#each diagnosticsCategories as category (category)}<li>{category}</li>{/each}
        </ul>
        <label class="check"
          ><input type="checkbox" bind:checked={diagnosticsIncludeHistory} /> Include aggregate history
          counts</label
        ><label class="check"
          ><input type="checkbox" bind:checked={diagnosticsIncludeErrors} /> Include redacted detector
          errors</label
        >
        <div class="boundary">
          <strong>Always excluded</strong><span
            >Username, email, hostname, machine identity, denylist, approval manifests, full profile
            paths, OneDrive identity, recovery data, credentials, and unredacted command output.</span
          >
        </div>
        <p>Your system file picker chooses the destination. No upload or network request occurs.</p>
        <div class="button-row">
          <button class="primary" onclick={() => void exportDiagnostics()}
            >Choose destination and save</button
          ><button class="secondary" onclick={() => (diagnosticsReviewVisible = false)}
            >Cancel</button
          >
        </div>
      </dialog>
    </div>
  {/if}
{:else}
  <main class="fatal" role="alert">
    <div class="logo" aria-hidden="true">!</div>
    <h1>Deslopper could not open its local workspace.</h1>
    <p>{errorMessage || 'The local database or product contract is unavailable.'}</p>
    <button class="primary" onclick={() => void loadProduct()}>Retry safely</button>
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
  :global(button),
  :global(input),
  :global(select),
  :global(textarea) {
    font: inherit;
  }
  :global(button) {
    min-height: 42px;
  }
  :global(select),
  :global(input),
  :global(textarea) {
    width: 100%;
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
    color: var(--color-text);
    background: var(--color-surface);
    padding: 11px 13px;
  }
  :global(textarea) {
    min-height: 78px;
    resize: vertical;
  }
  :global(label) {
    display: grid;
    gap: var(--space-2);
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
    font-weight: 650;
  }
  .shell {
    display: grid;
    grid-template-columns: 244px minmax(0, 1fr);
    width: 100vw;
    height: 100vh;
    background: var(--color-background);
  }
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-5) var(--space-4);
    border-right: 1px solid var(--color-outline);
    background: var(--color-surface);
    overflow-y: auto;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-2);
  }
  .brand > div:last-child,
  .boot > div:last-child {
    display: grid;
    gap: 2px;
  }
  .brand span,
  .boot span {
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }
  .logo {
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    flex: 0 0 auto;
    border-radius: 14px;
    color: var(--color-on-primary);
    background: var(--color-primary);
    font-weight: 800;
  }
  nav {
    display: grid;
    gap: var(--space-1);
  }
  nav button {
    display: grid;
    grid-template-columns: 36px 1fr;
    gap: var(--space-3);
    align-items: center;
    width: 100%;
    padding: 8px 10px;
    border: 0;
    border-radius: var(--radius-control);
    text-align: left;
    color: var(--color-text-secondary);
    background: transparent;
    cursor: pointer;
  }
  nav button:hover {
    background: var(--color-surface-tonal);
  }
  nav button.active {
    color: var(--color-text);
    background: var(--color-primary-container);
  }
  nav button.active::before {
    content: '';
    position: absolute;
    width: 4px;
    height: 28px;
    margin-left: -10px;
    border-radius: 8px;
    background: var(--color-primary);
  }
  nav button span:last-child {
    display: grid;
    gap: 1px;
  }
  nav small {
    color: var(--color-text-secondary);
    font-size: 11px;
    font-weight: 450;
  }
  .nav-glyph {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 12px;
    background: var(--color-surface-tonal);
    font-weight: 750;
  }
  .sidebar-spacer {
    flex: 1;
  }
  .safety-note {
    padding: var(--space-4);
    border-radius: var(--radius-card);
    background: var(--color-success-container);
  }
  .safety-note strong {
    color: var(--color-success);
  }
  .safety-note p {
    margin: var(--space-2) 0 0;
    font-size: var(--type-supporting);
    line-height: 1.45;
  }
  .workspace {
    min-width: 0;
    padding: clamp(20px, 3vw, 42px);
    overflow: auto;
  }
  .hero,
  .page-header {
    display: flex;
    justify-content: space-between;
    gap: var(--space-6);
    align-items: flex-start;
    margin-bottom: var(--space-5);
  }
  .hero {
    padding: clamp(24px, 4vw, 44px);
    border-radius: var(--radius-panel);
    background: linear-gradient(135deg, var(--color-primary-container), var(--color-surface));
    box-shadow: var(--shadow-card);
  }
  h1,
  h2,
  h3,
  p {
    margin-top: 0;
  }
  h1 {
    max-width: 820px;
    margin-bottom: var(--space-3);
    font-size: var(--type-title);
    line-height: 1.08;
    letter-spacing: -0.035em;
  }
  h2 {
    margin-bottom: var(--space-2);
    font-size: var(--type-section);
  }
  h3 {
    margin-bottom: var(--space-2);
    font-size: 1rem;
  }
  p {
    color: var(--color-text-secondary);
    line-height: 1.55;
  }
  .eyebrow {
    display: block;
    margin-bottom: var(--space-2);
    color: var(--color-primary);
    font-size: var(--type-label);
    font-weight: 800;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }
  .hero-actions,
  .button-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }
  button.primary,
  button.secondary,
  .danger-button,
  .text-button {
    padding: 10px 16px;
    border-radius: var(--radius-control);
    font-weight: 750;
    cursor: pointer;
  }
  button.primary {
    border: 1px solid var(--color-primary);
    color: var(--color-on-primary);
    background: var(--color-primary);
  }
  button.primary:hover {
    background: var(--color-primary-hover);
  }
  button.secondary {
    border: 1px solid var(--color-outline);
    background: var(--color-surface);
  }
  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
  .text-button {
    min-height: 36px;
    padding: 6px 10px;
    border: 0;
    color: var(--color-primary);
    background: transparent;
  }
  .text-button.danger {
    color: #8c2f35;
  }
  .danger-button {
    border: 1px solid #b44a52;
    color: #8c2f35;
    background: #fff5f5;
  }
  .read-only-seal {
    display: grid;
    justify-items: center;
    gap: var(--space-2);
    min-width: 210px;
    padding: var(--space-5);
    border-radius: var(--radius-card);
    text-align: center;
    background: rgb(255 255 255 / 75%);
  }
  .read-only-seal > span {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    color: var(--color-success);
    background: var(--color-success-container);
    font-size: 1.4rem;
  }
  .read-only-seal small {
    color: var(--color-text-secondary);
  }
  .metrics {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: var(--space-3);
    margin-bottom: var(--space-5);
  }
  .metrics article,
  .surface {
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
  }
  .metrics article {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-4);
  }
  .metrics span,
  .metrics small {
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }
  .metrics strong {
    font-size: 1.2rem;
  }
  .overview-grid,
  .settings-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.35fr) minmax(300px, 0.65fr);
    gap: var(--space-4);
  }
  .surface {
    padding: var(--space-5);
  }
  .section-heading,
  .detail-heading,
  .progress-heading {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    align-items: flex-start;
  }
  .compact-list {
    display: grid;
    gap: var(--space-2);
  }
  .compact-list button {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: 12px;
    border: 0;
    border-radius: var(--radius-control);
    text-align: left;
    background: var(--color-surface-tonal);
    cursor: pointer;
  }
  .compact-list button > span:first-child {
    display: grid;
    gap: 2px;
  }
  .compact-list small {
    color: var(--color-text-secondary);
  }
  .status-text {
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
    font-weight: 700;
  }
  .trust-card ul,
  .settings-grid ul,
  .onboarding ul {
    display: grid;
    gap: var(--space-3);
    padding-left: 22px;
    color: var(--color-text-secondary);
    line-height: 1.5;
  }
  .boundary,
  .plain-status,
  .validation,
  .notice {
    display: grid;
    gap: 4px;
    padding: var(--space-4);
    border-radius: var(--radius-control);
    background: var(--color-primary-container);
  }
  .boundary span,
  .plain-status,
  .validation p,
  .validation small {
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
  }
  .progress-panel {
    margin-bottom: var(--space-4);
    padding: var(--space-4);
    border-radius: var(--radius-card);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
  }
  progress {
    width: 100%;
    height: 9px;
    margin: var(--space-3) 0;
    accent-color: var(--color-primary);
  }
  .page-header {
    align-items: center;
  }
  .page-header p {
    max-width: 760px;
    margin-bottom: 0;
  }
  .badge {
    flex: 0 0 auto;
    padding: 8px 13px;
    border-radius: var(--radius-pill);
    color: var(--color-primary);
    background: var(--color-primary-container);
    font-size: var(--type-supporting);
    font-weight: 750;
  }
  .component-tools {
    display: grid;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }
  .search {
    max-width: 540px;
  }
  .filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .filter-row button {
    min-height: 36px;
    padding: 7px 12px;
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    cursor: pointer;
  }
  .filter-row button.active {
    border-color: var(--color-primary);
    background: var(--color-primary-container);
    font-weight: 750;
  }
  .component-layout {
    display: grid;
    grid-template-columns: minmax(280px, 0.75fr) minmax(440px, 1.25fr);
    gap: var(--space-4);
    align-items: start;
  }
  .component-list {
    display: grid;
    gap: var(--space-2);
    max-height: calc(100vh - 260px);
    overflow-y: auto;
    padding-right: var(--space-1);
  }
  .component-list > button {
    display: grid;
    grid-template-columns: 42px 1fr auto;
    gap: var(--space-3);
    align-items: center;
    width: 100%;
    padding: 12px;
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    text-align: left;
    background: var(--color-surface);
    cursor: pointer;
  }
  .component-list > button.selected {
    border-color: var(--color-primary);
    background: var(--color-primary-container);
  }
  .component-list > button > span:nth-child(2) {
    display: grid;
    gap: 3px;
  }
  .component-list small {
    color: var(--color-text-secondary);
  }
  .component-icon {
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: 14px;
    color: var(--color-primary);
    background: var(--color-primary-container);
    font-weight: 800;
  }
  .component-detail {
    min-height: 500px;
    padding: clamp(20px, 3vw, 34px);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-panel);
    background: var(--color-surface);
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1px;
    margin: var(--space-4) 0;
    overflow: hidden;
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
    background: var(--color-outline);
  }
  .facts > div {
    display: grid;
    gap: 5px;
    padding: 12px;
    background: var(--color-surface);
  }
  dt {
    color: var(--color-text-secondary);
    font-size: var(--type-label);
  }
  dd {
    margin: 0;
    line-height: 1.4;
  }
  .explanation,
  .desired-editor,
  .preview-card,
  .technical {
    margin-top: var(--space-4);
    padding-top: var(--space-4);
    border-top: 1px solid var(--color-outline);
  }
  .tradeoffs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
  }
  .tradeoffs article {
    padding: var(--space-4);
    border-radius: var(--radius-control);
    background: var(--color-surface-tonal);
  }
  .support-strip {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-4);
  }
  .support-strip span {
    display: grid;
    gap: 2px;
    padding: 9px 12px;
    border-radius: var(--radius-control);
    background: var(--color-surface-tonal);
    font-size: var(--type-supporting);
  }
  .desired-editor {
    display: grid;
    gap: var(--space-3);
  }
  .validation.invalid {
    background: var(--color-warning-container);
  }
  .error-copy {
    color: #8c2f35;
  }
  .success-copy {
    color: var(--color-success);
    font-weight: 700;
  }
  .technical summary,
  details > summary {
    cursor: pointer;
  }
  .technical article {
    display: grid;
    gap: 4px;
    padding: var(--space-3) 0;
    border-top: 1px solid var(--color-outline);
  }
  .technical span,
  .technical small {
    color: var(--color-text-secondary);
  }
  .empty,
  .large-empty {
    text-align: center;
    color: var(--color-text-secondary);
  }
  .empty {
    padding: var(--space-5);
  }
  .large-empty {
    max-width: 660px;
    margin: 10vh auto;
    padding: var(--space-7);
    border: 1px dashed var(--color-outline-strong);
    border-radius: var(--radius-panel);
    background: var(--color-surface);
  }
  .detail-empty {
    margin-top: 18vh;
  }
  .card-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
  }
  .card-grid article {
    display: grid;
    align-content: start;
    gap: var(--space-3);
  }
  .control-row {
    display: flex;
    gap: var(--space-3);
    flex-wrap: wrap;
    margin-bottom: var(--space-4);
  }
  .control-row label {
    min-width: 220px;
    flex: 1;
  }
  .drift-list,
  .history-list {
    display: grid;
    gap: var(--space-3);
  }
  details.surface {
    padding: 0;
  }
  details.surface > summary {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4);
    list-style-position: inside;
  }
  details.surface > summary > span:first-child {
    display: inline-grid;
    gap: 3px;
    margin-left: var(--space-2);
  }
  details.surface small {
    color: var(--color-text-secondary);
  }
  .drift-detail {
    padding: 0 var(--space-5) var(--space-5);
  }
  .state-change {
    display: grid;
    grid-template-columns: 1fr auto 1fr auto 1fr;
    gap: var(--space-3);
    align-items: center;
    padding: var(--space-4);
    border-radius: var(--radius-control);
    background: var(--color-surface-tonal);
  }
  .state-change span:not([aria-hidden]) {
    display: grid;
    gap: 4px;
  }
  .compare {
    margin-bottom: var(--space-4);
  }
  .comparison-results {
    display: grid;
    gap: var(--space-2);
    padding-top: var(--space-4);
    border-top: 1px solid var(--color-outline);
  }
  .comparison-results article {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
    padding: var(--space-3);
    border-radius: var(--radius-control);
    background: var(--color-surface-tonal);
  }
  .comparison-results article span {
    display: grid;
    gap: 3px;
  }
  .comparison-results article p {
    grid-column: 1 / -1;
    margin: 0;
  }
  .history-list article > div:first-child {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
  }
  .count-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin: var(--space-3) 0;
  }
  .count-row span {
    padding: 6px 9px;
    border-radius: var(--radius-pill);
    background: var(--color-surface-tonal);
    font-size: var(--type-label);
  }
  .settings-grid .surface {
    display: grid;
    align-content: start;
    gap: var(--space-3);
  }
  .check {
    display: flex;
    grid-template-columns: none;
    gap: var(--space-2);
    align-items: center;
  }
  .check input {
    width: auto;
  }
  .error-banner {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
    padding: var(--space-4);
    border-radius: var(--radius-card);
    background: var(--color-warning-container);
  }
  .error-banner p {
    margin: 3px 0 0;
  }
  .notice {
    margin-bottom: var(--space-4);
  }
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: grid;
    place-items: center;
    padding: var(--space-4);
    background: var(--color-scrim);
  }
  .onboarding,
  .diagnostics-review {
    width: min(680px, 100%);
    max-height: calc(100vh - 32px);
    overflow-y: auto;
    padding: clamp(24px, 5vw, 44px);
    border-radius: var(--radius-panel);
    background: var(--color-surface);
    box-shadow: var(--shadow-floating);
  }
  .onboarding .logo {
    margin-bottom: var(--space-5);
  }
  .onboarding li {
    display: grid;
    grid-template-columns: 24px 1fr;
    gap: var(--space-2);
  }
  .onboarding li span {
    color: var(--color-success);
    font-weight: 800;
  }
  .onboarding-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    margin: var(--space-5) 0 var(--space-3);
  }
  .diagnostics-review {
    display: grid;
    gap: var(--space-3);
  }
  .boot,
  .fatal {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
    width: 100vw;
    height: 100vh;
    padding: var(--space-5);
    background: var(--color-background);
  }
  .fatal {
    flex-direction: column;
    text-align: center;
  }
  @media (max-width: 1050px) {
    .metrics {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
    .component-layout {
      grid-template-columns: minmax(260px, 0.75fr) minmax(390px, 1.25fr);
    }
  }
  @media (max-width: 820px) {
    .shell {
      grid-template-columns: 82px minmax(0, 1fr);
    }
    .brand > div:last-child,
    nav button > span:last-child,
    .safety-note p {
      display: none;
    }
    .sidebar {
      padding-inline: var(--space-3);
    }
    nav button {
      grid-template-columns: 1fr;
      justify-items: center;
    }
    .safety-note {
      padding: var(--space-3);
      text-align: center;
    }
    .overview-grid,
    .settings-grid,
    .component-layout {
      grid-template-columns: 1fr;
    }
    .component-list {
      max-height: 360px;
    }
    .hero {
      display: grid;
    }
    .read-only-seal {
      min-width: 0;
    }
  }
  @media (max-width: 680px) {
    .workspace {
      padding: var(--space-4);
    }
    .metrics,
    .card-grid {
      grid-template-columns: 1fr;
    }
    .page-header,
    .section-heading {
      display: grid;
    }
    .facts,
    .tradeoffs {
      grid-template-columns: 1fr;
    }
    .state-change {
      grid-template-columns: 1fr;
    }
    .state-change span[aria-hidden] {
      transform: rotate(90deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    *,
    *::before,
    *::after {
      scroll-behavior: auto !important;
      transition-duration: 0.01ms !important;
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
    }
  }
</style>
