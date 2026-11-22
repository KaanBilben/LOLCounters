<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { Champion } from './types';
  import { searchChampions } from './util';

  let {
    champions,
    onpick,
    query = $bindable(''),
    selected,
    placeholder = 'Şampiyon ara',
    label = 'Şampiyon ara',
    large = false,
    autofocus = false,
    toolbar
  }: {
    champions: Champion[];
    onpick: (c: Champion) => void;
    query?: string;
    /** champion key -> 1-based order number (shown as a badge) */
    selected?: Map<number, number>;
    placeholder?: string;
    label?: string;
    large?: boolean;
    autofocus?: boolean;
    /** extra controls shown to the right of the search box (e.g. the lane filter) */
    toolbar?: Snippet;
  } = $props();

  const results = $derived(searchChampions(champions, query));
  let input: HTMLInputElement | undefined = $state();

  // re-runs when `autofocus` turns true (e.g. its view becomes visible)
  $effect(() => {
    if (autofocus) input?.focus();
  });

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && query.trim() !== '' && results.length > 0) {
      e.preventDefault();
      onpick(results[0]);
    } else if (e.key === 'Escape' && query) {
      e.preventDefault();
      query = '';
    }
  }
</script>

<div class="picker" class:large>
  <div class="top">
    <div class="search">
      <svg class="glass" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="10.5" cy="10.5" r="6.5" />
        <path d="m15.5 15.5 5 5" />
      </svg>
      <input
        bind:this={input}
        bind:value={query}
        {placeholder}
        aria-label={label}
        type="text"
        spellcheck="false"
        autocomplete="off"
        {onkeydown}
      />
      {#if query}
        <button
          class="clear"
          type="button"
          aria-label="Aramayı temizle"
          onclick={() => {
            query = '';
            input?.focus();
          }}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      {:else if large}
        <span class="hint" aria-hidden="true">Adını yaz, Enter ile seç</span>
      {/if}
    </div>
    {@render toolbar?.()}
  </div>

  <div class="grid-wrap">
    {#if results.length === 0}
      <p class="empty">“{query}” ile eşleşen şampiyon yok.</p>
    {:else}
      <ul class="grid" aria-label="Şampiyonlar">
        {#each results as c, i (c.key)}
          {@const order = selected?.get(c.key)}
          <li>
            <button
              type="button"
              class="tile"
              class:selected={order !== undefined}
              class:first={i === 0 && query.trim() !== ''}
              aria-pressed={selected ? order !== undefined : undefined}
              aria-label={order !== undefined ? `${c.name}, havuzda ${order}. sırada` : c.name}
              onclick={() => onpick(c)}
            >
              <span class="portrait">
                <img src={c.icon} alt="" width="56" height="56" loading="lazy" decoding="async" />
                {#if order !== undefined}
                  <span class="order" aria-hidden="true">{order}</span>
                {/if}
              </span>
              <span class="name">{c.name}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
    flex: 1;
  }

  .top {
    display: flex;
    gap: 10px;
    flex: none;
  }
  .search {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
  }
  .glass {
    position: absolute;
    left: 12px;
    width: 16px;
    height: 16px;
    fill: none;
    stroke: var(--muted);
    stroke-width: 2;
    stroke-linecap: round;
    pointer-events: none;
    transition: stroke var(--t-fast) var(--ease);
  }
  input {
    width: 100%;
    height: 38px;
    padding: 0 40px 0 38px;
    border: 1px solid #2a2f36;
    border-radius: 2px;
    background: rgba(1, 10, 19, 0.75);
    color: var(--text);
    font-size: 14px;
    transition:
      border-color var(--t-fast) var(--ease),
      box-shadow var(--t-fast) var(--ease);
  }
  input::placeholder {
    color: var(--faint);
  }
  input:hover {
    border-color: var(--gold-dark);
  }
  input:focus,
  input:focus-visible {
    outline: none;
    border-color: var(--gold-mid);
    box-shadow: 0 0 0 1px var(--gold-dark), 0 0 16px rgba(200, 170, 110, 0.1);
  }
  .search:focus-within .glass {
    stroke: var(--gold);
  }
  .large input {
    height: 50px;
    font-size: 17px;
    padding-left: 46px;
  }
  .large .glass {
    left: 16px;
    width: 20px;
    height: 20px;
  }
  .clear {
    position: absolute;
    right: 6px;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 2px;
    color: var(--muted);
    transition:
      color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .clear:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.06);
  }
  .clear svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linecap: round;
  }
  .hint {
    position: absolute;
    right: 16px;
    font-size: 12px;
    color: var(--faint);
    pointer-events: none;
  }

  .grid-wrap {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin-right: -6px;
    padding-right: 6px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(78px, 1fr));
    gap: 6px;
    padding: 2px;
  }
  .large .grid {
    grid-template-columns: repeat(auto-fill, minmax(84px, 1fr));
  }

  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 8px 4px 7px;
    border: 1px solid transparent;
    border-radius: var(--radius);
    transition:
      background var(--t-fast) var(--ease),
      border-color var(--t-fast) var(--ease);
  }
  .tile:hover {
    background: rgba(200, 170, 110, 0.06);
    border-color: rgba(120, 90, 40, 0.45);
  }
  .tile.first {
    border-color: rgba(10, 200, 185, 0.35);
    background: var(--teal-soft);
  }
  .portrait {
    position: relative;
    width: 56px;
    height: 56px;
    border: 1px solid var(--gold-dark);
    border-radius: 2px;
    overflow: hidden;
    background: #0b111a;
    transition:
      border-color var(--t-fast) var(--ease),
      box-shadow var(--t-fast) var(--ease),
      filter var(--t-fast) var(--ease);
  }
  .portrait img {
    width: 100%;
    height: 100%;
    transform: scale(1.12);
  }
  .tile:hover .portrait {
    border-color: var(--gold-mid);
  }
  .tile.selected .portrait {
    border: 2px solid var(--gold);
    box-shadow: 0 0 12px rgba(200, 170, 110, 0.35);
  }
  .order {
    position: absolute;
    top: 0;
    right: 0;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    display: grid;
    place-items: center;
    background: var(--gold);
    color: #010a13;
    font-size: 11px;
    font-weight: 700;
    border-bottom-left-radius: 3px;
  }
  .name {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--muted);
    transition: color var(--t-fast) var(--ease);
  }
  .tile:hover .name,
  .tile.selected .name {
    color: var(--text);
  }

  .empty {
    padding: 32px 8px;
    text-align: center;
    color: var(--muted);
  }
</style>
