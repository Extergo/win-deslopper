<script lang="ts">
  export let status: string;
  export let completed = 0;
  export let total = 1;
  export let detail = '';
  export let note = '';
  export let className = '';
</script>

<section
  class={['ds-progresspanel', className].filter(Boolean).join(' ')}
  aria-live="polite"
  {...$$restProps}
>
  <div class="ds-sectionheading">
    <strong>{status}</strong>
    <span>{completed} of {total}</span>
  </div>
  <progress class="ds-progress" max={total || 1} value={completed}></progress>
  {#if detail}
    <p class="ds-p" style="font-size: var(--type-supporting)">{detail}</p>
  {/if}
  {#if note}
    <div class="ds-callout" style="margin-top: var(--space-3)">
      <span>{note}</span>
    </div>
  {/if}
</section>
