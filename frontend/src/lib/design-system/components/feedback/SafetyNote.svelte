<script lang="ts">
  export let title: string;
  export let internal = false;
  export let className = '';
</script>

<section
  class={['ds-safetynote', internal && 'ds-safetynote--internal', className]
    .filter(Boolean)
    .join(' ')}
  {...$$restProps}
>
  <strong><span aria-hidden="true">●</span> {title}</strong>
  {#if $$slots.default}<p><slot /></p>{/if}
</section>
