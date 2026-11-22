<script lang="ts">
  import { tick } from 'svelte';
  import LaneIcon from './LaneIcon.svelte';
  import { tooltip } from './tooltip';
  import type { Lane } from './types';
  import { LANE_NAMES } from './util';

  let {
    lane,
    all = $bindable(false),
    searching = false,
    large = false
  }: {
    lane: Lane;
    /** false: only this lane's champions, true: every champion */
    all?: boolean;
    /** the search box has text (the search always covers every champion) */
    searching?: boolean;
    large?: boolean;
  } = $props();

  let el: HTMLDivElement | undefined = $state();

  // radio group keys: arrows / Home / End switch between the two options
  async function onkeydown(e: KeyboardEvent) {
    let next: boolean | undefined;
    if (['ArrowRight', 'ArrowDown', 'ArrowLeft', 'ArrowUp'].includes(e.key)) next = !all;
    else if (e.key === 'Home') next = false;
    else if (e.key === 'End') next = true;
    if (next === undefined) return;
    e.preventDefault();
    all = next;
    await tick();
    el?.querySelector<HTMLButtonElement>(`[data-all="${next}"]`)?.focus();
  }
</script>

<div
  class="filter"
  class:large
  class:searching
  role="radiogroup"
  aria-label="Gösterilen şampiyonlar"
  bind:this={el}
  use:tooltip={searching ? 'Arama bütün şampiyonlarda yapılır' : null}
>
  <button
    type="button"
    role="radio"
    class="opt"
    class:on={!all}
    data-all="false"
    tabindex={all ? -1 : 0}
    aria-checked={!all}
    onclick={() => (all = false)}
    {onkeydown}
  >
    <LaneIcon {lane} size={large ? 17 : 15} />
    <span>{LANE_NAMES[lane]} şampiyonları</span>
  </button>
  <button
    type="button"
    role="radio"
    class="opt"
    class:on={all}
    data-all="true"
    tabindex={all ? 0 : -1}
    aria-checked={all}
    onclick={() => (all = true)}
    {onkeydown}
  >
    Hepsi
  </button>
</div>

<style>
  .filter {
    display: flex;
    flex: none;
    gap: 2px;
    padding: 3px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: rgba(1, 10, 19, 0.7);
    transition: opacity var(--t-fast) var(--ease);
  }
  .filter.searching {
    opacity: 0.55;
  }
  .filter.searching:hover,
  .filter.searching:focus-within {
    opacity: 0.8;
  }

  .opt {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 12px;
    border-radius: 2px;
    color: var(--muted);
    font-size: 12.5px;
    font-weight: 600;
    white-space: nowrap;
    transition:
      color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease),
      box-shadow var(--t-fast) var(--ease);
  }
  .large .opt {
    height: 42px;
    padding: 0 14px;
    font-size: 13px;
  }
  .opt:hover:not(.on) {
    color: var(--text);
    background: rgba(255, 255, 255, 0.04);
  }
  .opt.on {
    color: var(--gold-light);
    background: linear-gradient(180deg, rgba(200, 170, 110, 0.2), rgba(200, 170, 110, 0.07));
    box-shadow: inset 0 0 0 1px var(--gold-mid);
  }
  .opt.on :global(svg) {
    color: var(--gold);
  }
  .opt:focus-visible {
    outline-offset: -2px;
  }
</style>
