<script lang="ts">
  export let summary: string;
  export let meta = '';
  export let inline = false;
  export let className = '';
</script>

<details
  class={['ds-disclosure', inline && 'ds-disclosure--inline', className].filter(Boolean).join(' ')}
  {...$$restProps}
>
  <summary>
    <span>
      <strong>{summary}</strong>
      {#if meta}<small>{meta}</small>{/if}
    </span>
    {#if $$slots.trailing}
      <span class="ds-status"><slot name="trailing" /></span>
    {/if}
  </summary>
  <div class="ds-disclosure__body">
    <slot />
  </div>
</details>
