<script lang="ts">
  export let label: string;
  export let className = '';
</script>

<label class={['ds-check', className].filter(Boolean).join(' ')}>
  <input type="checkbox" {...$$restProps} />
  {label}
</label>
