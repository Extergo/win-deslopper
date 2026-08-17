<script lang="ts">
  export let label: string;
  export let options: Array<{ value: string; label: string }> = [];
  export let className = '';
</script>

<label class={['ds-field', className].filter(Boolean).join(' ')}>
  <span>{label}</span>
  <select class="ds-select" {...$$restProps}>
    {#each options as option (option.value)}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
</label>
