<script lang="ts">
  export let tone: 'default' | 'tonal' | 'accent' = 'default';
  export let flush = false;
  export let panel = false;
  export let as = 'section';
  export let className = '';

  $: cls = [
    panel ? 'ds-panel' : 'ds-surface',
    !panel && tone !== 'default' && `ds-surface--${tone}`,
    !panel && flush && 'ds-surface--flush',
    className
  ]
    .filter(Boolean)
    .join(' ');
</script>

<svelte:element this={as} class={cls} {...$$restProps}>
  <slot />
</svelte:element>
