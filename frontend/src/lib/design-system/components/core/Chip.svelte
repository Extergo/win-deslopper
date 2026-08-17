<script lang="ts">
  export let active = false;
  export let className = '';
</script>

<button
  type="button"
  aria-pressed={active}
  class={['ds-chip', className].filter(Boolean).join(' ')}
  {...$$restProps}
>
  <slot />
</button>
