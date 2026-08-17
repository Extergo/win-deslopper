<script lang="ts">
  export let label: string;
  export let value: string | number | undefined = undefined;
  export let className = '';
</script>

{#if value === undefined}
  <span class={['ds-countchip', className].filter(Boolean).join(' ')} {...$$restProps}>
    <slot>{label}</slot>
  </span>
{:else}
  <span class={['ds-supportchip', className].filter(Boolean).join(' ')} {...$$restProps}>
    <strong>{value}</strong>
    <small>{label}</small>
  </span>
{/if}
