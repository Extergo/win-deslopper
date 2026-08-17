<script lang="ts">
  export let tone: 'accent' | 'neutral' | 'success' | 'warning' | 'danger' = 'accent';
  export let className = '';
</script>

<span
  class={['ds-badge', tone !== 'accent' && `ds-badge--${tone}`, className]
    .filter(Boolean)
    .join(' ')}
  {...$$restProps}
>
  <slot />
</span>
