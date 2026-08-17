<script lang="ts">
  export let variant: 'primary' | 'secondary' | 'text' | 'danger' = 'secondary';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let block = false;
  export let className = '';

  $: cls = [
    'ds-btn',
    `ds-btn--${variant}`,
    size !== 'md' && `ds-btn--${size}`,
    block && 'ds-btn--block',
    className
  ]
    .filter(Boolean)
    .join(' ');
</script>

<button type="button" class={cls} {...$$restProps}>
  <slot />
</button>
