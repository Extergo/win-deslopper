<script lang="ts">
  import { onDestroy, onMount } from 'svelte';

  import {
    createBackendClient,
    type MutationOperationId,
    type MutationTarget,
    type MutationTransaction
  } from './backend';
  import {
    MutationAlphaWorkflow,
    canUndo,
    formatEpoch,
    redactIdentifier,
    representationText,
    rollbackText
  } from './mutation-alpha';

  export let buildMode: string;
  export let sourceInspectionId: string | null;

  const backend = createBackendClient();
  let workflow = new MutationAlphaWorkflow(backend);
  let warningConfirmed = false;
  let pollHandle: number | null = null;

  const durableStages = [
    { label: 'Pre-write revalidation', steps: ['validate_plan'] },
    { label: 'Pre-state capture', steps: ['capture_pre_state'] },
    { label: 'Write started', steps: ['apply'] },
    { label: 'Registry/package readback', steps: ['verify', 'handler_verified'] },
    { label: 'Detector verification', steps: ['effective_reinspection_verified'] }
  ];
  const lifecycleStates = [
    'plan_ready',
    'awaiting_approval',
    'approved',
    'executing',
    'verifying',
    'applied',
    'rolling_back',
    'rolled_back',
    'failed',
    'recovery_required'
  ];

  onMount(() => void initialise());
  onDestroy(stopPolling);

  async function initialise(): Promise<void> {
    await workflow.initialise(buildMode);
    workflow = workflow;
  }

  async function acknowledgeWarning(): Promise<void> {
    if (!warningConfirmed) return;
    await workflow.acknowledgeWarning();
    warningConfirmed = false;
    workflow = workflow;
  }

  async function createPlan(
    operationId: MutationOperationId,
    target: MutationTarget
  ): Promise<void> {
    await workflow.createPlan(operationId, target, sourceInspectionId);
    workflow = workflow;
  }

  async function execute(): Promise<void> {
    startPolling();
    await workflow.execute();
    stopPolling();
    workflow = workflow;
  }

  async function undo(): Promise<void> {
    startPolling();
    await workflow.undo();
    stopPolling();
    workflow = workflow;
  }

  function startPolling(): void {
    stopPolling();
    pollHandle = window.setInterval(() => {
      void workflow.refreshCurrentTransaction().then(() => (workflow = workflow));
    }, 300);
  }

  function stopPolling(): void {
    if (pollHandle !== null) window.clearInterval(pollHandle);
    pollHandle = null;
  }

  function stageStatus(stepTypes: string[]): string {
    const step = workflow.transaction?.steps
      .filter((candidate) => stepTypes.includes(candidate.stepType))
      .at(-1);
    return step?.status ?? 'pending';
  }

  function operationTitle(operationId: MutationOperationId): string {
    return (
      workflow.options.find((option) => option.definition.operationId === operationId)?.definition
        .title ?? operationId.replaceAll('_', ' ')
    );
  }

  function transactionNotes(transaction: MutationTransaction): string[] {
    return transaction.steps.flatMap((step) => [
      ...step.redactedEvidence,
      ...(step.errorSummary ? [step.errorSummary] : [])
    ]);
  }
</script>

<section class="mutation-page" aria-labelledby="mutation-alpha-title">
  <header class="mutation-header">
    <div>
      <span class="eyebrow">Experimental local workflow</span>
      <h1 id="mutation-alpha-title">Experimental Apply &amp; Undo</h1>
      <p>
        This internal interface uses the closed Mutation Alpha broker. A warning acknowledgement,
        target approval, operation scope, reviewed plan, and one-time approval are separate gates.
      </p>
    </div>
    <span class="internal-badge">Internal Mutation Alpha</span>
  </header>

  {#if workflow.error}
    <section class="mutation-error" role="alert">
      <strong>{workflow.error.summary}</strong>
      <details>
        <summary>Technical detail</summary>
        <p>{workflow.error.code}: {workflow.error.technical}</p>
      </details>
    </section>
  {/if}

  {#if workflow.busy && !workflow.status}
    <section class="alpha-surface" aria-live="polite">Loading internal safety status…</section>
  {:else if workflow.status}
    <section class="alpha-surface status-card" aria-labelledby="alpha-status-title">
      <div class="section-heading">
        <div>
          <span class="status-kicker">Feature-gated status</span>
          <h2 id="alpha-status-title">Mutation Alpha safety gates</h2>
        </div>
        <span class:ready={workflow.status.available} class="status-pill">
          {workflow.status.available ? 'Eligible for reviewed planning' : 'Blocked'}
        </span>
      </div>

      {#if workflow.status.liveValidation.developmentHostRefused}
        <div class="permanent-block" role="status">
          <strong>This development machine is permanently blocked from live mutation.</strong>
          <span
            >No executable controls are available. The development-host denial has no bypass.</span
          >
        </div>
      {:else}
        <p class="backend-reason">{workflow.status.reason}</p>
      {/if}

      <dl class="status-grid">
        <div>
          <dt>Build mode</dt>
          <dd>{buildMode}</dd>
        </div>
        <div>
          <dt>Environment</dt>
          <dd>{workflow.status.liveValidation.environment.computerName}</dd>
        </div>
        <div>
          <dt>Machine identity prefix</dt>
          <dd>{workflow.status.liveValidation.environment.machineIdPrefix}</dd>
        </div>
        <div>
          <dt>Platform</dt>
          <dd>
            {workflow.status.liveValidation.environment.edition} · build
            {workflow.status.liveValidation.environment.build}
          </dd>
        </div>
        <div>
          <dt>VM detection (informational)</dt>
          <dd>
            {workflow.status.liveValidation.environment.virtualMachineDetection.replaceAll(
              '_',
              ' '
            )}
          </dd>
        </div>
        <div>
          <dt>Scenario</dt>
          <dd>
            {workflow.status.liveValidation.environment.currentValidationScenario ?? 'Not selected'}
          </dd>
        </div>
        <div>
          <dt>Target eligibility</dt>
          <dd>{workflow.status.liveValidation.available ? 'Validated' : 'Not eligible'}</dd>
        </div>
        <div>
          <dt>Development-host denial</dt>
          <dd>
            {workflow.status.liveValidation.developmentHostRefused ? 'Active' : 'Not triggered'}
          </dd>
        </div>
        <div>
          <dt>Approval / manifest</dt>
          <dd>
            {workflow.status.liveValidation.targetApprovalLoaded
              ? 'Approval loaded'
              : 'No valid approval'}
            · {workflow.status.liveValidation.manifestLoaded
              ? 'manifest loaded'
              : 'manifest missing'}
          </dd>
        </div>
        <div>
          <dt>Expiration</dt>
          <dd>{formatEpoch(workflow.status.liveValidation.approvalExpiresAtEpochMs)}</dd>
        </div>
        <div>
          <dt>Allowances</dt>
          <dd>
            {workflow.status.liveValidation.maximumPlans} plan(s) ·
            {workflow.status.liveValidation.maximumExecutions} execution(s)
          </dd>
        </div>
        <div>
          <dt>Automatic repair</dt>
          <dd>Disabled</dd>
        </div>
        <div>
          <dt>Warning acknowledgement</dt>
          <dd>
            {workflow.status.warningAcknowledged
              ? 'Acknowledged for this process'
              : 'Not acknowledged'}
          </dd>
        </div>
      </dl>

      {#if !workflow.status.liveValidation.developmentHostRefused && !workflow.status.warningAcknowledged}
        <div class="warning-box">
          <strong>Read before continuing</strong>
          <p>{workflow.status.warningText}</p>
          <label class="check-row">
            <input type="checkbox" bind:checked={warningConfirmed} />
            <span>I understand that acknowledgement is not target or operation authorization.</span>
          </label>
          <button
            class="primary"
            disabled={!warningConfirmed || workflow.busy}
            onclick={() => void acknowledgeWarning()}>Acknowledge warning</button
          >
        </div>
      {/if}
    </section>

    {#if !workflow.status.liveValidation.developmentHostRefused && workflow.status.available}
      <section class="alpha-surface" aria-labelledby="authorized-options-title">
        <div class="section-heading">
          <div>
            <span class="status-kicker">Backend-authorized scope only</span>
            <h2 id="authorized-options-title">Available operations</h2>
          </div>
          <span class="count-pill">{workflow.options.length} returned</span>
        </div>
        {#if workflow.options.length === 0}
          <div class="empty-state">
            <strong>No authorized options</strong>
            <p>{workflow.status.liveValidation.reason}</p>
          </div>
        {:else}
          <div class="option-list">
            {#each workflow.options as option (option.definition.operationId)}
              <article class="option-card">
                <div>
                  <h3>{option.definition.title}</h3>
                  <p>{option.definition.description}</p>
                  <small>{option.validationLabel} · {option.reason}</small>
                </div>
                <div class="target-actions">
                  {#each option.definition.supportedTargets as target (target)}
                    <button
                      class="secondary"
                      disabled={!option.eligible || workflow.busy}
                      onclick={() => void createPlan(option.definition.operationId, target)}
                      >Review {target}</button
                    >
                  {/each}
                </div>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    {/if}

    {#if workflow.issuedPlan}
      <section class="alpha-surface plan-review" aria-labelledby="plan-review-title">
        <div class="section-heading">
          <div>
            <span class="status-kicker">Plan ready · Awaiting explicit approval</span>
            <h2 id="plan-review-title">Review the exact one-operation plan</h2>
          </div>
          <span class="status-pill">{workflow.phase.replaceAll('_', ' ')}</span>
        </div>
        <div class="lifecycle-strip" aria-label="Mutation lifecycle states">
          {#each lifecycleStates as state (state)}
            <span class:active={workflow.phase === state}>{state.replaceAll('_', ' ')}</span>
          {/each}
        </div>
        <p>{workflow.issuedPlan.confirmationText}</p>
        <dl class="review-grid">
          <div>
            <dt>Component</dt>
            <dd>{operationTitle(workflow.issuedPlan.plan.operationId)}</dd>
          </div>
          <div>
            <dt>Current state</dt>
            <dd>
              {workflow.issuedPlan.plan.currentState.effectiveEnabled ? 'Enabled' : 'Disabled'}
            </dd>
          </div>
          <div>
            <dt>Exact current representation</dt>
            <dd>{representationText(workflow.issuedPlan.plan.currentState.representation)}</dd>
          </div>
          <div>
            <dt>Target state</dt>
            <dd>{workflow.issuedPlan.plan.targetState}</dd>
          </div>
          <div>
            <dt>Exact proposed representation</dt>
            <dd>{representationText(workflow.issuedPlan.proposedRepresentation)}</dd>
          </div>
          <div>
            <dt>Scope</dt>
            <dd>Current Windows user</dd>
          </div>
          <div>
            <dt>Authority</dt>
            <dd>{workflow.issuedPlan.plan.authority}</dd>
          </div>
          <div>
            <dt>Restart / sign-out</dt>
            <dd>{workflow.issuedPlan.plan.restartRequirement}</dd>
          </div>
          <div>
            <dt>Rollback representation</dt>
            <dd>{representationText(workflow.issuedPlan.plan.currentState.representation)}</dd>
          </div>
          <div>
            <dt>Expiration</dt>
            <dd>{formatEpoch(workflow.issuedPlan.plan.expiresAt)}</dd>
          </div>
          <div>
            <dt>Number of operations</dt>
            <dd>1</dd>
          </div>
          <div>
            <dt>Elevation required</dt>
            <dd>No</dd>
          </div>
          <div>
            <dt>Explorer restart required</dt>
            <dd>No</dd>
          </div>
          <div>
            <dt>Automatic repair</dt>
            <dd>Disabled</dd>
          </div>
          <div>
            <dt>Plan ID</dt>
            <dd>{redactIdentifier(workflow.issuedPlan.plan.planId)}</dd>
          </div>
          <div>
            <dt>One-time nonce</dt>
            <dd>{redactIdentifier(workflow.issuedPlan.approvalNonce)}</dd>
          </div>
        </dl>

        {#if !workflow.planConsumed}
          <div class="approval-box">
            <label for="approval-phrase">
              Type <code>{workflow.issuedPlan.approvalPhrase}</code> exactly
            </label>
            <input
              id="approval-phrase"
              autocomplete="off"
              spellcheck="false"
              bind:value={workflow.approvalInput}
            />
            <div class="button-row">
              <button
                class="primary"
                disabled={!workflow.approvalMatches() || workflow.busy}
                onclick={() => void execute()}>Approve and execute</button
              >
              <button
                class="secondary"
                disabled={workflow.busy}
                onclick={() => void workflow.cancelPlan().then(() => (workflow = workflow))}
                >Cancel plan</button
              >
            </div>
            <small>Creating a plan did not execute it. This exact phrase is case-sensitive.</small>
          </div>
        {/if}
      </section>
    {/if}

    {#if workflow.transaction}
      <section class="alpha-surface" aria-labelledby="transaction-title" aria-live="polite">
        <div class="section-heading">
          <div>
            <span class="status-kicker">Durable broker journal</span>
            <h2 id="transaction-title">Execution and verification</h2>
          </div>
          <span class="status-pill">{workflow.transaction.status.replaceAll('_', ' ')}</span>
        </div>
        <ol class="progress-list">
          {#each durableStages as stage (stage.label)}
            <li>
              <span aria-hidden="true">{stageStatus(stage.steps) === 'pending' ? '○' : '●'}</span>
              <span><strong>{stage.label}</strong><small>{stageStatus(stage.steps)}</small></span>
            </li>
          {/each}
          <li>
            <span aria-hidden="true">{workflow.visualReport ? '●' : '○'}</span>
            <span
              ><strong>Visual confirmation required</strong><small
                >{workflow.visualReport?.replaceAll('_', ' ') ?? 'pending user report'}</small
              ></span
            >
          </li>
        </ol>
        <dl class="review-grid compact">
          <div>
            <dt>Transaction state</dt>
            <dd>{workflow.transaction.status}</dd>
          </div>
          <div>
            <dt>Effective-state result</dt>
            <dd>{workflow.transaction.verificationResult ?? 'Not yet verified'}</dd>
          </div>
          <div>
            <dt>Applied representation</dt>
            <dd>{representationText(workflow.transaction.postState?.representation)}</dd>
          </div>
          <div>
            <dt>Recovery</dt>
            <dd>{workflow.transaction.recoveryRequirement ?? 'Not required'}</dd>
          </div>
        </dl>

        {#if workflow.phase === 'applied'}
          <div class="visual-box">
            <strong>Did the visible taskbar behavior change?</strong>
            <p>
              This is a manual observation. It does not trigger another write and is exported only
              through the existing validation-evidence flow after exact rollback.
            </p>
            <div class="button-row">
              <button
                class="secondary"
                onclick={() => {
                  workflow.recordVisualReport('changed_as_expected');
                  workflow = workflow;
                }}>Changed as expected</button
              >
              <button
                class="secondary"
                onclick={() => {
                  workflow.recordVisualReport('did_not_change');
                  workflow = workflow;
                }}>Did not change</button
              >
            </div>
          </div>
        {/if}

        {#if canUndo(workflow.transaction)}
          <div class="undo-box">
            <strong>Transaction-bound Undo</strong>
            <p>{rollbackText(workflow.transaction.preState)}</p>
            <button class="primary" disabled={workflow.busy} onclick={() => void undo()}
              >Undo</button
            >
          </div>
        {:else if workflow.transaction.status === 'rolled_back'}
          <div class="undo-box success">
            <strong>Exact original state restored</strong>
            <p>
              {representationText(workflow.transaction.rollbackState?.representation)} ·
              {workflow.transaction.rollback.verificationResult ?? 'verification unavailable'}
            </p>
            {#if workflow.visualReport && !workflow.evidence}
              <button
                class="secondary"
                disabled={workflow.busy}
                onclick={() =>
                  void workflow.exportVisualEvidence().then(() => (workflow = workflow))}
                >Save validation evidence</button
              >
            {:else if workflow.evidence}
              <small>Validation evidence saved locally: {workflow.evidence.finalResult}</small>
            {/if}
          </div>
        {/if}
      </section>
    {/if}

    <section class="alpha-surface" aria-labelledby="mutation-history-title">
      <div class="section-heading">
        <div>
          <span class="status-kicker">Local only · identifiers redacted</span>
          <h2 id="mutation-history-title">Mutation transaction history</h2>
        </div>
        <span class="count-pill">{workflow.history.length}</span>
      </div>
      {#if workflow.history.length === 0}
        <div class="empty-state"><strong>No local mutation transactions</strong></div>
      {:else}
        <div class="history-list">
          {#each workflow.history as item (item.transactionId)}
            <details class="history-row">
              <summary>
                <span>
                  <strong>{operationTitle(item.operationId)} → {item.targetState}</strong>
                  <small>{formatEpoch(item.createdAt)}</small>
                </span>
                <span>{item.status.replaceAll('_', ' ')}</span>
              </summary>
              <dl class="review-grid compact">
                <div>
                  <dt>Original</dt>
                  <dd>{representationText(item.preState?.representation)}</dd>
                </div>
                <div>
                  <dt>Applied</dt>
                  <dd>{representationText(item.postState?.representation)}</dd>
                </div>
                <div>
                  <dt>Apply result</dt>
                  <dd>{item.verificationResult ?? item.status}</dd>
                </div>
                <div>
                  <dt>Verification</dt>
                  <dd>{item.verificationResult ?? 'Not available'}</dd>
                </div>
                <div>
                  <dt>Rollback</dt>
                  <dd>{item.rollback.result ?? 'Not attempted'}</dd>
                </div>
                <div>
                  <dt>Final state</dt>
                  <dd>{item.status}</dd>
                </div>
              </dl>
              {#if item.errorSummary}<p class="history-error">{item.errorSummary}</p>{/if}
              {#if transactionNotes(item).length > 0}
                <ul class="history-notes">
                  {#each transactionNotes(item) as note, index (`${item.transactionId}-${index}`)}
                    <li>{note}</li>
                  {/each}
                </ul>
              {/if}
            </details>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</section>

<style>
  .mutation-page {
    display: grid;
    gap: var(--space-4);
  }
  .mutation-header,
  .section-heading,
  .option-card,
  .history-row summary {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    align-items: flex-start;
  }
  .mutation-header {
    margin-bottom: var(--space-2);
  }
  .mutation-header p {
    max-width: 760px;
  }
  h1,
  h2,
  h3,
  p {
    margin-top: 0;
  }
  h1 {
    margin-bottom: var(--space-3);
    font-size: var(--type-title);
    line-height: 1.08;
  }
  h2 {
    margin-bottom: var(--space-2);
    font-size: var(--type-section);
  }
  h3 {
    margin-bottom: var(--space-2);
  }
  p,
  small {
    color: var(--color-text-secondary);
    line-height: 1.5;
  }
  .eyebrow,
  .status-kicker {
    display: block;
    margin-bottom: var(--space-2);
    color: var(--color-primary);
    font-size: var(--type-label);
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .internal-badge,
  .status-pill,
  .count-pill {
    flex: 0 0 auto;
    padding: 8px 13px;
    border-radius: var(--radius-pill);
    color: var(--color-warning);
    background: var(--color-warning-container);
    font-size: var(--type-supporting);
    font-weight: 750;
  }
  .status-pill.ready {
    color: var(--color-success);
    background: var(--color-success-container);
  }
  .count-pill {
    color: var(--color-primary);
    background: var(--color-primary-container);
  }
  .alpha-surface {
    padding: clamp(20px, 3vw, 30px);
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-card);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
  }
  .mutation-error,
  .permanent-block,
  .warning-box,
  .approval-box,
  .visual-box,
  .undo-box,
  .empty-state {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-4);
    border-radius: var(--radius-control);
  }
  .mutation-error,
  .permanent-block {
    background: var(--color-warning-container);
  }
  .mutation-error p,
  .permanent-block span {
    margin: 0;
  }
  .backend-reason {
    margin: var(--space-3) 0;
    padding: var(--space-3);
    border-radius: var(--radius-control);
    background: var(--color-surface-tonal);
  }
  .status-grid,
  .review-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1px;
    overflow: hidden;
    border: 1px solid var(--color-outline);
    border-radius: var(--radius-control);
    background: var(--color-outline);
  }
  .status-grid > div,
  .review-grid > div {
    display: grid;
    gap: 4px;
    padding: var(--space-3);
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
  .warning-box,
  .approval-box,
  .visual-box,
  .undo-box {
    margin-top: var(--space-4);
    background: var(--color-primary-container);
  }
  .undo-box.success {
    background: var(--color-success-container);
  }
  .check-row {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
  }
  .option-list,
  .history-list,
  .progress-list {
    display: grid;
    gap: var(--space-3);
  }
  .option-card,
  .history-row {
    padding: var(--space-4);
    border-radius: var(--radius-control);
    background: var(--color-surface-tonal);
  }
  .option-card p {
    margin-bottom: var(--space-2);
  }
  .target-actions,
  .button-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .lifecycle-strip {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }
  .lifecycle-strip span {
    padding: 5px 9px;
    border-radius: var(--radius-pill);
    color: var(--color-text-secondary);
    background: var(--color-surface-tonal);
    font-size: var(--type-label);
  }
  .lifecycle-strip span.active {
    color: var(--color-primary);
    background: var(--color-primary-container);
    font-weight: 800;
  }
  button.primary,
  button.secondary {
    min-height: 40px;
    padding: 9px 15px;
    border-radius: var(--radius-control);
    font-weight: 750;
    cursor: pointer;
  }
  button.primary {
    border: 1px solid var(--color-primary);
    color: var(--color-on-primary);
    background: var(--color-primary);
  }
  button.secondary {
    border: 1px solid var(--color-outline);
    background: var(--color-surface);
  }
  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
  .approval-box input {
    width: 100%;
    min-height: 44px;
    padding: 9px 12px;
    border: 1px solid var(--color-outline-strong);
    border-radius: var(--radius-control);
    background: var(--color-surface);
  }
  code {
    user-select: all;
  }
  .progress-list {
    padding: 0;
    list-style: none;
  }
  .progress-list li {
    display: grid;
    grid-template-columns: 24px 1fr;
    gap: var(--space-2);
    align-items: start;
  }
  .progress-list li > span:last-child,
  .history-row summary > span:first-child {
    display: grid;
    gap: 2px;
  }
  .review-grid.compact {
    margin-top: var(--space-3);
  }
  .history-row summary {
    cursor: pointer;
  }
  .history-error {
    margin: var(--space-3) 0 0;
  }
  .history-notes {
    color: var(--color-text-secondary);
    font-size: var(--type-supporting);
  }
  @media (max-width: 760px) {
    .mutation-header,
    .section-heading,
    .option-card {
      display: grid;
    }
    .status-grid,
    .review-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
