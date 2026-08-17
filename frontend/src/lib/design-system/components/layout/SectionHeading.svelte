<script lang="ts">
  export let eyebrow = '';
  export let title: string;
  export let description = '';
  export let level: 2 | 3 = 2;
  export let id: string | undefined = undefined;
  export let className = '';

  $: tag = `h${level}`;
</script>

<div class={['ds-sectionheading', className].filter(Boolean).join(' ')} {...$$restProps}>
  <div>
    {#if eyebrow}<span class="ds-eyebrow">{eyebrow}</span>{/if}
    <svelte:element this={tag} {id} class={level === 2 ? 'ds-h2' : 'ds-h3'}>{title}</svelte:element>
    {#if description}<p class="ds-p">{description}</p>{/if}
  </div>
  <slot name="trailing" />
</div>
