<script lang="ts">
  export let eyebrow = '';
  export let title: string;
  export let statement = '';
  export let id: string | undefined = undefined;
  export let className = '';
</script>

<header class={['ds-hero', className].filter(Boolean).join(' ')} {...$$restProps}>
  <div>
    {#if eyebrow}<span class="ds-eyebrow">{eyebrow}</span>{/if}
    <h1 {id} class="ds-h1">{title}</h1>
    {#if statement}<p class="ds-p">{statement}</p>{/if}
    {#if $$slots.actions}
      <div class="ds-row" style="margin-top: var(--space-4)">
        <slot name="actions" />
      </div>
    {/if}
  </div>
  <slot name="trailing" />
</header>
