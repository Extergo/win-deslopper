<script lang="ts">
  import Glyph from '../core/Glyph.svelte';

  export let glyph = '';
  export let title: string;
  export let subtitle = '';
  export let selected = false;
  export let compact = false;
  export let className = '';
</script>

<button
  type="button"
  aria-pressed={selected}
  class={['ds-listrow', compact && 'ds-listrow--compact', className].filter(Boolean).join(' ')}
  {...$$restProps}
>
  {#if !compact}
    <Glyph tone="accent">{glyph}</Glyph>
  {/if}
  <span>
    <strong>{title}</strong>
    {#if subtitle}<small>{subtitle}</small>{/if}
  </span>
  <span class="ds-status"><slot name="status" /></span>
</button>
