<script lang="ts">
  export let label: string;
  export let hint = '';
  export let multiline = false;
  export let className = '';
</script>

<label class={['ds-field', className].filter(Boolean).join(' ')}>
  <span>{label}</span>
  {#if multiline}
    <textarea class="ds-textarea" {...$$restProps}></textarea>
  {:else}
    <input class="ds-input" {...$$restProps} />
  {/if}
  {#if hint}<small>{hint}</small>{/if}
</label>
