<script lang="ts">
  import type { Lane } from './types';

  let { lane, size = 18 }: { lane: Lane; size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="currentColor"
  aria-hidden="true"
  focusable="false"
>
  {#if lane === 'top'}
    <path d="M3 3h15l-3.5 3.5H6.5v8L3 18z" />
    <rect x="10" y="10" width="7" height="7" rx="0.5" opacity="0.45" />
    <path d="M21 6v15H6l3-3h9V9z" opacity="0.22" />
  {:else if lane === 'jungle'}
    <path
      d="M12 2.5c3.9 2.6 6.4 6.3 6.4 10.2a6.4 6.4 0 0 1-12.8 0C5.6 8.8 8.1 5.1 12 2.5zm0 3.2c-2.4 2.1-3.9 4.6-3.9 7a3.9 3.9 0 0 0 2.65 3.7V12l1.25-1.6 1.25 1.6v4.4a3.9 3.9 0 0 0 2.65-3.7c0-2.4-1.5-4.9-3.9-7z"
    />
    <rect x="11.1" y="17.5" width="1.8" height="4" rx="0.6" />
  {:else if lane === 'mid'}
    <path d="M17 3h4v4L7 21H3v-4z" />
    <path d="M3 3h9l-3 3H6v3l-3 3z" opacity="0.35" />
    <path d="M21 21h-9l3-3h3v-3l3-3z" opacity="0.35" />
  {:else if lane === 'adc'}
    <path d="M21 21H6l3.5-3.5h8v-8L21 6z" />
    <rect x="7" y="7" width="7" height="7" rx="0.5" opacity="0.45" />
    <path d="M3 18V3h15l-3 3H6v9z" opacity="0.22" />
  {:else}
    <path
      d="M12 2.5 20 5.6v5.8c0 4.9-3.3 8.6-8 10.1-4.7-1.5-8-5.2-8-10.1V5.6zm-1.3 5v3.2H7.5v2.6h3.2v3.2h2.6v-3.2h3.2v-2.6h-3.2V7.5z"
    />
  {/if}
</svg>

<style>
  svg {
    display: block;
    flex: none;
  }
</style>
