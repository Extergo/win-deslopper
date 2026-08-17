<script lang="ts">
  const TONES: Record<string, string> = {
    Observed: 'observed',
    'Externally managed': 'managed',
    'Permission limited': 'limited',
    'Failed inspection': 'failed',
    Unknown: 'unknown'
  };

  export let text = '';
  export let tone: 'observed' | 'unknown' | 'limited' | 'failed' | 'managed' | undefined =
    undefined;
  export let className = '';

  $: key = tone ?? TONES[text] ?? '';
</script>

<span
  class={['ds-status', key && `ds-status--${key}`, className].filter(Boolean).join(' ')}
  {...$$restProps}
>
  {#if $$slots.default}<slot />{:else}{text}{/if}
</span>
