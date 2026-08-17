<script lang="ts">
  export let label = 'Search';
  export let className = '';
</script>

<label class={['ds-search', className].filter(Boolean).join(' ')}>
  <span class="ds-sr">{label}</span>
  <input class="ds-input" type="search" {...$$restProps} />
</label>
