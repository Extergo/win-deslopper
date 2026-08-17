<script lang="ts">
  export let className = '';
</script>

<span class={['ds-eyebrow', className].filter(Boolean).join(' ')} {...$$restProps}>
  <slot />
</span>
