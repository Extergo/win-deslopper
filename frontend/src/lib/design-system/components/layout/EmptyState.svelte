<script lang="ts">
  export let title: string;
  export let large = false;
  export let className = '';
</script>

<div
  class={['ds-empty', large && 'ds-empty--large', className].filter(Boolean).join(' ')}
  {...$$restProps}
>
  <strong>{title}</strong>
  {#if $$slots.default}<p class="ds-p"><slot /></p>{/if}
  {#if $$slots.action}
    <div class="ds-row" style="justify-content: center; margin-top: var(--space-4)">
      <slot name="action" />
    </div>
  {/if}
</div>
