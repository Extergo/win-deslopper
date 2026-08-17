<script lang="ts">
  export let eyebrow = '';
  export let title: string;
  export let description = '';
  export let id: string | undefined = undefined;
  export let className = '';
</script>

<header class={['ds-pageheader', className].filter(Boolean).join(' ')} {...$$restProps}>
  <div>
    {#if eyebrow}<span class="ds-eyebrow">{eyebrow}</span>{/if}
    <h1 {id} class="ds-h1">{title}</h1>
    {#if description}<p class="ds-p">{description}</p>{/if}
  </div>
  <slot name="trailing" />
</header>
