<script lang="ts">
  export let steps: Array<{ label: string; value: string | number }> = [];
  export let className = '';
</script>

<div class={['ds-statechange', className].filter(Boolean).join(' ')} {...$$restProps}>
  {#each steps as step, i (step.label)}
    <span>
      <small>{step.label}</small>
      <strong>{step.value}</strong>
    </span>
    {#if i < steps.length - 1}
      <span aria-hidden="true">→</span>
    {/if}
  {/each}
</div>
