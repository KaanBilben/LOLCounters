<script lang="ts">
  import { tooltip } from './tooltip';

  // Square/round icon from Data Dragon with a themed frame, a custom tooltip and
  // a quiet placeholder if the image cannot be loaded (no broken-image glyph).
  let {
    src,
    name,
    size = 32,
    shape = 'square',
    crop = false,
    tip = true,
    focusable = true
  }: {
    src: string;
    name: string;
    size?: number;
    shape?: 'square' | 'round';
    crop?: boolean;
    tip?: boolean;
    /** false inside an interactive parent (e.g. a rune card): mouse tooltip only, no tab stop. */
    focusable?: boolean;
  } = $props();

  let failed = $state(false);
  $effect(() => {
    void src;
    failed = false;
  });
</script>

<!-- focusable so keyboard users can open the name tooltip, too -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<span
  class="icon {shape}"
  class:crop
  style:width="{size}px"
  style:height="{size}px"
  tabindex={tip && focusable ? 0 : undefined}
  role={tip && focusable ? 'img' : undefined}
  aria-label={tip && focusable ? name : undefined}
  use:tooltip={tip ? name : null}
>
  {#if failed}
    <span class="ph" aria-label={name} role="img">{name.slice(0, 1)}</span>
  {:else}
    <img
      {src}
      alt={name}
      width={size}
      height={size}
      loading="lazy"
      decoding="async"
      onerror={() => (failed = true)}
    />
  {/if}
</span>

<style>
  .icon {
    position: relative;
    display: inline-flex;
    flex: none;
    overflow: hidden;
    background: #0b111a;
    border: 1px solid var(--gold-dark);
  }
  .square {
    border-radius: 3px;
  }
  .round {
    border-radius: 50%;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .crop img {
    transform: scale(1.12);
  }
  .ph {
    display: grid;
    place-items: center;
    width: 100%;
    height: 100%;
    color: var(--muted);
    font-size: 12px;
    font-weight: 600;
  }
</style>
