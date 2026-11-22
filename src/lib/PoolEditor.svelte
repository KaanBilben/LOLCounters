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
    onsave,
    dirty = $bindable(false)
  }: {
    champions: Champion[];
    champByKey: Map<number, Champion>;
    saved: Settings;
    welcome?: boolean;
    /** false while the view is hidden (keeps the draft, disables shortcuts) */
    active?: boolean;
    onsave: (s: Settings) => Promise<void>;
    dirty?: boolean;
  } = $props();

  type Pools = Record<Lane, number[]>;
  const toPools = (s: Settings): Pools =>
    Object.fromEntries(LANES.map((l) => [l, [...(s.pools[l] ?? [])]])) as Pools;

  let draft = $state<Pools>(untrack(() => toPools(saved)));
  let lane = $state<Lane>('top');
  let query = $state('');
  let status = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let saveError = $state('');
  let savedTimer: ReturnType<typeof setTimeout> | undefined;

  const same = (a: number[], b: number[]) => a.length === b.length && a.every((v, i) => v === b[i]);
  const isDirty = $derived(LANES.some((l) => !same(draft[l], saved.pools[l] ?? [])));
  $effect(() => {
    dirty = isDirty;
  });

  const pool = $derived(draft[lane]);
  const selected = $derived(new Map(pool.map((k, i) => [k, i + 1])));

  function toggle(c: Champion) {
    const i = draft[lane].indexOf(c.key);
    if (i >= 0) draft[lane].splice(i, 1);
    else draft[lane].push(c.key);
    if (status === 'saved' || status === 'error') status = 'idle';
  }

  function move(from: number, to: number) {
    const arr = draft[lane];
    if (to < 0 || to >= arr.length || from === to) return;
    const [k] = arr.splice(from, 1);
    arr.splice(to, 0, k);
    if (status === 'saved') status = 'idle';
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
    if (status === 'saved') status = 'idle';
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

  async function save() {
    if (!isDirty || status === 'saving') return;
    status = 'saving';
    saveError = '';
    const settings: Settings = { pools: toPools({ pools: draft }) };
    try {
      await onsave(settings);
      status = 'saved';
      clearTimeout(savedTimer);
      savedTimer = setTimeout(() => {
        if (status === 'saved') status = 'idle';
      }, 2200);
    } catch (e) {
      status = 'error';
      saveError = errorText(e);
    }
  }

  function revert() {
    draft = toPools(saved);
    status = 'idle';
  }

  function onkeydown(e: KeyboardEvent) {
    if (active && (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
      e.preventDefault();
      save();
    }
  }

  onDestroy(() => clearTimeout(savedTimer));
</script>

<svelte:window {onkeydown} />

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
    <div class="state" aria-live="polite">
      {#if status === 'error'}
        <span class="dot err" aria-hidden="true"></span>
        <span class="err-text">Kaydedilemedi: {saveError}</span>
      {:else if isDirty}
        <span class="dot pending" aria-hidden="true"></span>
        <span>Kaydedilmemiş değişiklikler</span>
      {:else}
        <span class="dot ok" aria-hidden="true"></span>
        <span class="muted">Tüm değişiklikler kayıtlı</span>
      {/if}
    </div>
    <div class="actions">
      {#if isDirty && status !== 'saving'}
        <button type="button" class="btn btn-sm ghost" onclick={revert}>Vazgeç</button>
      {/if}
      <button
        type="button"
        class="btn btn-primary save"
        class:done={status === 'saved' && !isDirty}
        disabled={!isDirty || status === 'saving'}
        onclick={save}
        use:tooltip={isDirty ? 'Ctrl+S' : null}
      >
        {#if status === 'saving'}
          Kaydediliyor…
        {:else if status === 'saved' && !isDirty}
          Kaydedildi ✓
        {:else}
          Kaydet
        {/if}
      </button>
    </div>
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
    max-height: 104px;
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
    padding: 10px 14px;
    border: 1px solid var(--border);
    border-top-color: var(--gold-dark);
    border-radius: var(--radius);
    background: rgba(1, 10, 19, 0.85);
  }
  .state {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    font-size: 13px;
  }
  .err-text {
    color: var(--hard);
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
  .dot.pending {
    background: var(--gold);
    box-shadow: 0 0 8px rgba(200, 170, 110, 0.6);
  }
  .dot.ok {
    background: var(--good);
    opacity: 0.7;
  }
  .dot.err {
    background: var(--hard);
  }
  .actions {
    display: flex;
    gap: 8px;
    flex: none;
  }
  .ghost {
    border-color: transparent;
    background: transparent;
    color: var(--muted);
  }
  .ghost:hover:not(:disabled) {
    border-color: var(--border);
    box-shadow: none;
  }
  .save {
    min-width: 136px;
  }
  .save.done:disabled {
    border-color: rgba(10, 207, 131, 0.45);
    color: var(--good);
    background: rgba(10, 207, 131, 0.08);
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
      max-height: 68px;
    }
    .browse {
      padding: 10px 12px;
    }
    .savebar {
      padding: 6px 12px;
    }
  }
</style>
