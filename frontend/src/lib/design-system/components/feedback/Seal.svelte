<script lang="ts">
  export let mark = '✓';
  export let title: string;
  export let note = '';
  export let className = '';
</script>

<div class={['ds-seal', className].filter(Boolean).join(' ')} {...$$restProps}>
  <span aria-hidden="true">{mark}</span>
  <strong>{title}</strong>
  {#if note}<small>{note}</small>{/if}
</div>
