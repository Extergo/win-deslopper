<script lang="ts">
  export let tone: 'tonal' | 'accent' | 'brand' = 'tonal';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let className = '';
</script>

<span
  aria-hidden="true"
  class={[
    'ds-glyph',
    tone !== 'tonal' && `ds-glyph--${tone}`,
    size !== 'md' && `ds-glyph--${size}`,
    className
  ]
    .filter(Boolean)
    .join(' ')}
  {...$$restProps}
>
  <slot />
</span>
