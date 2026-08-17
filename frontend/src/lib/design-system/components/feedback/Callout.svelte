<script lang="ts">
  export let tone: 'info' | 'success' | 'warning' | 'danger' = 'info';
  export let title = '';
  export let note = '';
  export let banner = false;
  export let className = '';
</script>

<div
  class={[
    'ds-callout',
    tone !== 'info' && `ds-callout--${tone}`,
    banner && 'ds-callout--banner',
    className
  ]
    .filter(Boolean)
    .join(' ')}
  {...$$restProps}
>
  <div>
    {#if title}<strong>{title}</strong>{/if}
    {#if $$slots.default}<p><slot /></p>{/if}
    {#if note}<small>{note}</small>{/if}
  </div>
  <slot name="trailing" />
</div>
