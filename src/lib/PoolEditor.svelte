<script lang="ts">
  import { onDestroy, tick, untrack } from 'svelte';
  import ChampionPicker from './ChampionPicker.svelte';
  import LaneIcon from './LaneIcon.svelte';
  import { tooltip } from './tooltip';
  import type { Champion, Lane, Settings } from './types';
  import { errorText } from './api';
  import { LANES, LANE_NAMES } from './util';

  let {
    champions,
    champByKey,
    saved,
    welcome = false,
    active = true,
    onsave
  }: {
    champions: Champion[];
    champByKey: Map<number, Champion>;
    saved: Settings;
    welcome?: boolean;
    /** false while the view is hidden (keeps the draft, disables shortcuts) */
    active?: boolean;
    onsave: (s: Settings) => Promise<void>;
  } = $props();

  type Pools = Record<Lane, number[]>;
  const toPools = (s: Settings): Pools =>
    Object.fromEntries(LANES.map((l) => [l, [...(s.pools[l] ?? [])]])) as Pools;

  let draft = $state<Pools>(untrack(() => toPools(saved)));
  let lane = $state<Lane>('top');
  let query = $state('');

  // --- autosave: every change is written SAVE_DELAY ms after the last one
  // (one-shot debounce, no polling); flushed at once when the view is hidden,
  // the page goes away or the component is destroyed.
  const SAVE_DELAY = 500;
  let status = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let saveError = $state('');
  let pending = false; // a change not yet handed to onsave
  let inFlight = false;
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  function changed() {
    pending = true;
    status = 'saving';
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flush, SAVE_DELAY);
  }

  async function flush() {
    clearTimeout(saveTimer);
    saveTimer = undefined;
    // a save already running writes the newer draft when it finishes
    if (!pending || inFlight) return;
    pending = false;
    inFlight = true;
    status = 'saving';
    const settings: Settings = { ...saved, pools: toPools({ pools: draft }) };
    try {
      await onsave(settings);
      // "Kaydedildi" fades out with a CSS animation, then goes back to idle
      if (!pending) {
        status = 'saved';
        saveError = '';
      }
    } catch (e) {
      pending = true;
      status = 'error';
      saveError = errorText(e);
    } finally {
      inFlight = false;
    }
    // changed during the save: write the newer draft now (unless its own
    // debounce is still waiting, or the save failed and waits for "Tekrar dene")
    if (pending && saveTimer === undefined && status !== 'error') flush();
  }

  function retry() {
    pending = true;
    flush();
  }

  // leaving the view: write at once so Counter sees the new pool
  $effect(() => {
    if (!active) untrack(flush);
  });

  const pool = $derived(draft[lane]);
  const selected = $derived(new Map(pool.map((k, i) => [k, i + 1])));

  function toggle(c: Champion) {
    const i = draft[lane].indexOf(c.key);
    if (i >= 0) draft[lane].splice(i, 1);
    else draft[lane].push(c.key);
    changed();
  }

  function move(from: number, to: number) {
    const arr = draft[lane];
    if (to < 0 || to >= arr.length || from === to) return;
    const [k] = arr.splice(from, 1);
    arr.splice(to, 0, k);
    changed();
  }

  let listEl: HTMLOListElement | undefined = $state();

  async function step(index: number, dir: -1 | 1) {
    const key = draft[lane][index];
    move(index, index + dir);
    await tick();
    const chip = listEl?.querySelector<HTMLElement>(`[data-key="${key}"]`);
    const want = chip?.querySelector<HTMLButtonElement>(dir < 0 ? '.left' : '.right');
    const other = chip?.querySelector<HTMLButtonElement>(dir < 0 ? '.right' : '.left');
    (want && !want.disabled ? want : other)?.focus();
  }

  let browseEl: HTMLDivElement | undefined = $state();

  async function remove(index: number) {
    draft[lane].splice(index, 1);
    changed();
    // keep keyboard focus in the list: the × of the chip now at this place
    await tick();
    const left = draft[lane];
    if (left.length === 0) {
      browseEl?.querySelector<HTMLInputElement>('input')?.focus();
      return;
    }
    const key = left[Math.min(index, left.length - 1)];
    listEl?.querySelector<HTMLButtonElement>(`[data-key="${key}"] .x`)?.focus();
  }

  // --- HTML5 drag and drop ---
  let dragFrom = $state<number | null>(null);
  let dropAt = $state<number | null>(null);

  function dragstart(e: DragEvent, i: number) {
    dragFrom = i;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', String(i));
    }
  }
  function dragover(e: DragEvent, i: number) {
    if (dragFrom === null) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    dropAt = e.clientX < r.left + r.width / 2 ? i : i + 1;
  }
  function drop(e: DragEvent) {
    e.preventDefault();
    if (dragFrom !== null && dropAt !== null) {
      move(dragFrom, dropAt > dragFrom ? dropAt - 1 : dropAt);
    }
    dragend();
  }
  function dragend() {
    dragFrom = null;
    dropAt = null;
  }
  const showDrop = (at: number) =>
    dragFrom !== null && dropAt === at && at !== dragFrom && at !== dragFrom + 1;

  function onkeydown(e: KeyboardEvent) {
    // Ctrl+S: save now instead of waiting for the debounce
    if (active && (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
      e.preventDefault();
      if (status === 'error') retry();
      else flush();
    }
  }

  function onvisibilitychange() {
    if (document.visibilityState === 'hidden') flush();
  }

  onDestroy(flush);
</script>

<svelte:window {onkeydown} onpagehide={flush} />
<svelte:document {onvisibilitychange} />

<section class="editor" aria-labelledby="pool-title">
  <h1 id="pool-title" class="sr-only">Havuzum</h1>

  {#if welcome}
    <div class="welcome">
      <span class="welcome-mark" aria-hidden="true"></span>
      <div>
        <p class="welcome-title">Hoş geldin</p>
        <p class="welcome-text">
          Her lane için oynadığın şampiyonları tercih sırasıyla seç. İlk sıradaki en çok oynamak
          istediğin şampiyondur.
        </p>
      </div>
    </div>
  {/if}

  <div class="lanes" role="tablist" aria-label="Lane">
    {#each LANES as l (l)}
      {@const count = draft[l].length}
      <button
        type="button"
        role="tab"
        class="lane"
        class:on={lane === l}
        aria-selected={lane === l}
        onclick={() => (lane = l)}
      >
        <LaneIcon lane={l} size={18} />
        <span>{LANE_NAMES[l]}</span>
        <span class="count" class:has={count > 0} aria-label="{count} şampiyon">{count}</span>
      </button>
    {/each}
  </div>

  <div class="panel pool">
    <div class="pool-head">
      <h2 class="section-title">Havuzun · {LANE_NAMES[lane]}</h2>
      {#if pool.length > 1}
        <span class="muted small">Sürükleyerek ya da oklarla sırala</span>
      {/if}
    </div>

    {#if pool.length === 0}
      <p class="pool-empty">
        Bu lane için henüz şampiyon yok. Aşağıdaki listeden tıklayarak ekle; tıklama sırası tercih
        sırasıdır.
      </p>
    {:else}
      <ol
        class="chips"
        bind:this={listEl}
        aria-label="{LANE_NAMES[lane]} havuzu, tercih sırasıyla"
        ondragover={(e) => {
          if (dragFrom !== null) e.preventDefault();
        }}
        ondrop={drop}
      >
        {#each pool as key, i (key)}
          {@const c = champByKey.get(key)}
          <li
            class="chip"
            class:dragging={dragFrom === i}
            class:drop-before={showDrop(i)}
            class:drop-after={i === pool.length - 1 && showDrop(pool.length)}
            data-key={key}
            draggable="true"
            ondragstart={(e) => dragstart(e, i)}
            ondragover={(e) => dragover(e, i)}
            ondragend={dragend}
          >
            <span class="num" aria-hidden="true">{i + 1}</span>
            {#if c}
              <img src={c.icon} alt="" width="28" height="28" />
            {:else}
              <span class="noimg" aria-hidden="true"></span>
            {/if}
            <span class="cname">{c?.name ?? `#${key}`}</span>
            <span class="ctrls">
              <button
                type="button"
                class="ctrl left"
                disabled={i === 0}
                aria-label="{c?.name ?? key}: öne al"
                use:tooltip={'Öne al'}
                onclick={() => step(i, -1)}>←</button
              >
              <button
                type="button"
                class="ctrl right"
                disabled={i === pool.length - 1}
                aria-label="{c?.name ?? key}: geri al"
                use:tooltip={'Geri al'}
                onclick={() => step(i, 1)}>→</button
              >
              <button
                type="button"
                class="ctrl x"
                aria-label="{c?.name ?? key}: havuzdan çıkar"
                use:tooltip={'Havuzdan çıkar'}
                onclick={() => remove(i)}>×</button
              >
            </span>
          </li>
        {/each}
      </ol>
    {/if}
  </div>

  <div class="panel browse" bind:this={browseEl}>
    <ChampionPicker
      {champions}
      bind:query
      {selected}
      onpick={toggle}
      placeholder="Şampiyon ara (ad ya da kısaltma)"
      label="Havuza eklemek için şampiyon ara"
    />
  </div>

  <div class="savebar">
    <div class="state" role="status" aria-live="polite">
      {#if status === 'error'}
        <span class="msg err-text">
          <span class="dot err" aria-hidden="true"></span>
          <span class="clip">Kaydedilemedi: {saveError}</span>
        </span>
      {:else if status === 'saving'}
        <span class="msg">
          <span class="dot busy" aria-hidden="true"></span>
          <span>Kaydediliyor…</span>
        </span>
      {:else if status === 'saved'}
        <span
          class="msg ok-msg"
          onanimationend={() => {
            if (status === 'saved') status = 'idle';
          }}
        >
          <span class="dot ok" aria-hidden="true"></span>
          <span>Kaydedildi ✓</span>
        </span>
      {:else}
        <span class="msg muted idle-msg">
          <span class="dot idle" aria-hidden="true"></span>
          <span>Değişiklikler otomatik kaydedilir</span>
        </span>
      {/if}
    </div>
    {#if status === 'error'}
      <button type="button" class="btn btn-sm" onclick={retry} use:tooltip={'Ctrl+S'}>
        Tekrar dene
      </button>
    {/if}
  </div>
</section>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100%;
    min-height: 0;
  }

  .welcome {
    display: flex;
    gap: 14px;
    align-items: center;
    padding: 12px 16px;
    border: 1px solid var(--gold-dark);
    border-left: 3px solid var(--gold);
    border-radius: var(--radius);
    background: linear-gradient(90deg, rgba(200, 170, 110, 0.1), rgba(200, 170, 110, 0.02));
    flex: none;
  }
  .welcome-mark {
    width: 10px;
    height: 10px;
    flex: none;
    transform: rotate(45deg);
    border: 2px solid var(--gold);
  }
  .welcome-title {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--gold);
  }
  .welcome-text {
    color: var(--text);
  }

  .lanes {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 6px;
    flex: none;
  }
  .lane {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 40px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 2px;
    background: rgba(10, 20, 40, 0.7);
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
    transition:
      color var(--t-fast) var(--ease),
      border-color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .lane:hover {
    color: var(--text);
    border-color: var(--gold-dark);
  }
  .lane.on {
    color: var(--gold-light);
    border-color: var(--gold-mid);
    background: linear-gradient(180deg, rgba(200, 170, 110, 0.14), rgba(200, 170, 110, 0.03));
    box-shadow: inset 0 -2px 0 var(--gold);
  }
  .lane.on :global(svg) {
    color: var(--gold);
  }
  .count {
    min-width: 20px;
    height: 18px;
    padding: 0 6px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    background: #1a1f26;
    color: var(--faint);
    font-size: 11px;
    font-weight: 700;
  }
  .count.has {
    background: rgba(200, 170, 110, 0.16);
    color: var(--gold);
  }

  .pool {
    flex: none;
    padding: 12px 14px 14px;
  }
  .pool-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 10px;
  }
  .small {
    font-size: 12px;
  }
  .pool-empty {
    padding: 14px;
    border: 1px dashed #2a2f36;
    border-radius: var(--radius);
    color: var(--muted);
    text-align: center;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    max-height: 92px; /* tam 2 satır çip: 2 + 40 + 8 + 40 + 2 */
    overflow-y: auto;
    padding: 2px;
  }
  .chip {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 4px 0 4px;
    border: 1px solid var(--gold-dark);
    border-radius: 3px;
    background: linear-gradient(180deg, #151f30, #0d1626);
    cursor: grab;
    transition:
      border-color var(--t-fast) var(--ease),
      opacity var(--t-fast) var(--ease);
  }
  .chip:hover {
    border-color: var(--gold-mid);
  }
  .chip.dragging {
    opacity: 0.4;
    cursor: grabbing;
  }
  .chip.drop-before::before,
  .chip.drop-after::after {
    content: '';
    position: absolute;
    top: 4px;
    bottom: 4px;
    width: 3px;
    border-radius: 2px;
    background: var(--teal);
    box-shadow: 0 0 8px var(--teal);
  }
  .chip.drop-before::before {
    left: -6px;
  }
  .chip.drop-after::after {
    right: -6px;
  }
  .num {
    min-width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
    border-radius: 2px;
    background: var(--gold);
    color: #010a13;
    font-size: 11px;
    font-weight: 700;
  }
  .chip img,
  .noimg {
    width: 28px;
    height: 28px;
    border-radius: 2px;
    border: 1px solid var(--gold-dark);
    background: #0b111a;
  }
  .cname {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
  }
  .ctrls {
    display: flex;
    gap: 2px;
    margin-left: 2px;
  }
  .ctrl {
    width: 22px;
    height: 26px;
    display: grid;
    place-items: center;
    border-radius: 2px;
    color: var(--muted);
    font-size: 13px;
    line-height: 1;
    transition:
      color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .ctrl:hover:not(:disabled) {
    color: var(--text);
    background: rgba(255, 255, 255, 0.07);
  }
  .ctrl:disabled {
    color: #34383e;
  }
  .ctrl.x {
    font-size: 17px;
  }
  .ctrl.x:hover {
    color: var(--hard);
    background: rgba(232, 64, 87, 0.12);
  }

  .browse {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding: 14px;
  }

  .savebar {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    min-height: 40px;
    padding: 5px 14px;
    border: 1px solid var(--border);
    border-top-color: var(--gold-dark);
    border-radius: var(--radius);
    background: rgba(1, 10, 19, 0.85);
  }
  /* the messages share one grid cell, so the fading "Kaydedildi" never pushes the next one */
  .state {
    display: grid;
    min-width: 0;
    font-size: 13px;
  }
  .msg {
    grid-area: 1 / 1;
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  /* shown for 2 s, then fades; animationend returns the bar to idle */
  .ok-msg {
    color: var(--good);
    animation: saved-out 450ms var(--ease) 2s forwards;
  }
  .idle-msg {
    animation: msg-in 250ms var(--ease);
  }
  @keyframes saved-out {
    to {
      opacity: 0;
    }
  }
  @keyframes msg-in {
    from {
      opacity: 0;
    }
  }
  .err-text {
    color: var(--hard);
  }
  .clip {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dot {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
  }
  .dot.busy {
    background: var(--gold);
    box-shadow: 0 0 8px rgba(200, 170, 110, 0.6);
    animation: pulse 0.8s var(--ease) infinite alternate;
  }
  .dot.ok {
    background: var(--good);
    box-shadow: 0 0 8px rgba(10, 207, 131, 0.5);
  }
  .dot.idle {
    background: var(--good);
    opacity: 0.45;
  }
  .dot.err {
    background: var(--hard);
  }
  @keyframes pulse {
    from {
      opacity: 0.35;
    }
    to {
      opacity: 1;
    }
  }

  /* short windows (down to the 620px minimum): give the champion list room */
  @media (max-height: 720px) {
    .editor {
      gap: 8px;
    }
    .welcome {
      padding: 7px 12px;
      gap: 10px;
    }
    .welcome-title {
      display: none;
    }
    .welcome-text {
      font-size: 12.5px;
    }
    .lane {
      height: 36px;
    }
    .pool {
      padding: 8px 12px 10px;
    }
    .pool-head {
      margin-bottom: 6px;
    }
    .pool-empty {
      padding: 8px 12px;
    }
    .chips {
      max-height: 92px;
    }
    .browse {
      padding: 10px 12px;
    }
    .savebar {
      min-height: 36px;
      padding: 3px 12px;
    }
  }
</style>
