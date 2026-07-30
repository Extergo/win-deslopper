<script lang="ts">
  import { onMount, tick } from 'svelte';

  import {
    createBackendClient,
    describeCommandError,
    type AppAction,
    type AppView,
    type CatalogueFilter,
    type NavigationDestination
  } from '$lib/backend';
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
      value: 'extensions',
      label: 'Extensions',
      icon: '+',
      description: 'Coming in a future milestone'
    }
  ];

  let view: AppView | null = null;
  let loading = true;
  let pendingRequests = 0;
  let latestRequest = 0;
  let searchQuery = '';
  let errorMessage = '';
  let dialogElement: HTMLElement | undefined;

  $: busy = pendingRequests > 0;

  onMount(() => {
    void load();
  });

  async function load(): Promise<void> {
    loading = true;
    errorMessage = '';

    try {
      view = await backend.getAppView();
    } catch (error) {
      errorMessage = describeCommandError(error);
    } finally {
      loading = false;
    }
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
          <strong>Preview mode</strong>
        </div>
        <p>No administrator access or Windows changes.</p>
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
                Review optional Windows components and promotional features. This preview never
                changes your system.
              </p>
            </div>
            <span class="preview-badge">
              <span aria-hidden="true"></span>
              Preview only
            </span>
          </header>

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
</style>
