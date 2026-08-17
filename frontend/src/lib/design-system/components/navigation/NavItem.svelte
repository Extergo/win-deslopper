<script lang="ts">
  import Glyph from '../core/Glyph.svelte';

  export let glyph = '';
  export let label: string;
  export let description = '';
  export let active = false;
  export let className = '';
</script>

<button
  type="button"
  aria-current={active ? 'page' : undefined}
  class={['ds-navitem', className].filter(Boolean).join(' ')}
  {...$$restProps}
>
  <Glyph size="sm">{glyph}</Glyph>
  <span>
    <strong>{label}</strong>
    {#if description}<small>{description}</small>{/if}
  </span>
</button>
