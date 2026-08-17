<script lang="ts">
  export let eyebrow = '';
  export let title: string;
  export let footnote = '';
  export let onDismiss: (() => void) | undefined = undefined;
  export let className = '';
</script>

<div
  class="ds-scrim"
  role="button"
  tabindex="0"
  onclick={onDismiss}
  onkeydown={(event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') onDismiss?.();
  }}
>
  <div
    role="dialog"
    aria-modal="true"
    class={['ds-dialog', className].filter(Boolean).join(' ')}
    onclick={(event: MouseEvent) => event.stopPropagation()}
    {...$$restProps}
  >
    {#if eyebrow}<span class="ds-eyebrow">{eyebrow}</span>{/if}
    <h2 class="ds-h2">{title}</h2>
    <slot />
    {#if $$slots.actions}
      <div class="ds-dialog__actions">
        <slot name="actions" />
      </div>
    {/if}
    {#if footnote}
      <small style="color: var(--color-text-secondary)">{footnote}</small>
    {/if}
  </div>
</div>
