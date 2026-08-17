<script lang="ts">
  export let facts: Array<{ term: string; value: string }> = [];
  export let mono = false;
  export let className = '';
</script>

<dl
  class={['ds-facts', mono && 'ds-facts--mono', className].filter(Boolean).join(' ')}
  {...$$restProps}
>
  {#each facts as fact (fact.term)}
    <div>
      <dt>{fact.term}</dt>
      <dd>{fact.value}</dd>
    </div>
  {/each}
</dl>
