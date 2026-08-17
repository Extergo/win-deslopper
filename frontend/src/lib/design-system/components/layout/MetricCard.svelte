<script lang="ts">
  export let label: string;
  export let value: string | number;
  export let note = '';
  export let accent = false;
  export let className = '';
</script>

<article
  class={['ds-metric', accent && 'ds-metric--accent', className].filter(Boolean).join(' ')}
  {...$$restProps}
>
  <span>{label}</span>
  <strong>{value}</strong>
  {#if note}<small>{note}</small>{/if}
</article>
