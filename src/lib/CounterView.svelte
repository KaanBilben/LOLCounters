<script lang="ts">
  import { tick, untrack } from 'svelte';
  import { errorText, recommend } from './api';
  import ChampionPicker from './ChampionPicker.svelte';
  import GameIcon from './GameIcon.svelte';
  import LaneFilter from './LaneFilter.svelte';
  import LaneIcon from './LaneIcon.svelte';
  import RecommendationCard from './RecommendationCard.svelte';
  import { inLane, watchRoles } from './roles';
  import { tooltip } from './tooltip';
  import type { Champion, Lane, Recommendation, Roles, Settings, StaticPayload } from './types';
  import { LANES, LANE_NAMES } from './util';

  let {
    data,
    champByKey,
    settings,
    active = true,
    ongopool
  }: {
    data: StaticPayload;
    champByKey: Map<number, Champion>;
    settings: Settings;
    /** true while the Counter view is visible (focuses the enemy search) */
    active?: boolean;
    ongopool: () => void;
  } = $props();

  const poolOf = (l: Lane) => settings.pools[l] ?? [];
  const usable = $derived(LANES.filter((l) => poolOf(l).length > 0));

  let chosenLane = $state<Lane | null>(null);
  const lane = $derived(
    chosenLane && usable.includes(chosenLane) ? chosenLane : (usable[0] ?? null)
  );

  let enemy = $state<number | null>(null);
  let changing = $state(false);
  let query = $state('');
  const enemyChamp = $derived(enemy === null ? undefined : champByKey.get(enemy));
  const showPicker = $derived(enemy === null || changing);

  // enemy list: champions played in my lane unless "Hepsi"; a search covers every champion.
  // Without lane data (still loading or failed) every champion is shown.
  let roles = $state.raw<Roles | null>(null);
  let showAll = $state(false);
  // asked again on window focus if it failed (no timer)
  $effect(() => watchRoles((r) => (roles = r)));
  const searching = $derived(query.trim() !== '');
  const enemyChoices = $derived(
    roles && lane && !showAll && !searching ? inLane(data.champions, roles, lane) : data.champions
  );

  type Result =
    | { status: 'loading' }
    | { status: 'error'; detail: string }
    | { status: 'ok'; key: string; recs: Recommendation[] };
  let result = $state<Result | null>(null);

  // results per lane + enemy + pool contents, kept for the session.
  // Only lists without fetch errors are kept, so a failed champion is asked
  // again next time (the backend has its own memory + disk cache, so a repeat
  // request is cheap).
  const cache = new Map<string, Recommendation[]>();
  let requestId = 0;

  const reqKey = $derived(
    lane && enemy !== null ? `${lane}|${enemy}|${poolOf(lane).join(',')}` : null
  );

  $effect(() => {
    const key = reqKey;
    const l = lane;
    const e = enemy;
    if (key === null || l === null || e === null) {
      result = null;
      return;
    }
    untrack(() => run(key, l, e));
  });

  async function run(key: string, l: Lane, e: number) {
    const hit = cache.get(key);
    const id = ++requestId;
    if (hit) {
      result = { status: 'ok', key, recs: hit };
      return;
    }
    result = { status: 'loading' };
    try {
      const recs = await recommend(l, e);
      if (recs.every((r) => r.error === null)) cache.set(key, recs);
      if (id === requestId) result = { status: 'ok', key, recs };
    } catch (err) {
      if (id === requestId) result = { status: 'error', detail: errorText(err) };
    }
  }

  function retry() {
    if (reqKey && lane && enemy !== null) run(reqKey, lane, enemy);
  }

  let changeBtn: HTMLButtonElement | undefined = $state();

  async function pickEnemy(c: Champion) {
    enemy = c.key;
    changing = false;
    query = '';
    await tick();
    changeBtn?.focus();
  }

  function selectLane(l: Lane) {
    if (poolOf(l).length > 0) chosenLane = l;
  }

  // radio group keys: arrows move (and select) across enabled lanes only
  let segEl: HTMLDivElement | undefined = $state();

  async function laneKey(e: KeyboardEvent) {
    if (!lane) return;
    const i = usable.indexOf(lane);
    let next: Lane | undefined;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = usable[(i + 1) % usable.length];
    else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp')
      next = usable[(i - 1 + usable.length) % usable.length];
    else if (e.key === 'Home') next = usable[0];
    else if (e.key === 'End') next = usable[usable.length - 1];
    if (!next) return;
    e.preventDefault();
    chosenLane = next;
    await tick();
    segEl?.querySelector<HTMLButtonElement>(`[data-lane="${next}"]`)?.focus();
  }
</script>

<section class="counter" aria-labelledby="counter-title">
  <h1 id="counter-title" class="sr-only">Counter</h1>

  {#if usable.length === 0}
    <div class="panel empty-all">
      <LaneIcon lane="mid" size={36} />
      <h2>Havuzun boş</h2>
      <p class="muted">
        Öneri alabilmek için önce Havuzum'dan en az bir lane'e oynadığın şampiyonları ekle.
      </p>
      <button type="button" class="btn btn-primary" onclick={ongopool}>Havuzum'a git</button>
    </div>
  {:else}
    <div class="panel matchbar">
      <div class="seg" role="radiogroup" aria-label="Lane" bind:this={segEl}>
        {#each LANES as l (l)}
          {@const enabled = poolOf(l).length > 0}
          <button
            type="button"
            role="radio"
            class="seg-btn"
            class:on={lane === l}
            data-lane={l}
            tabindex={lane === l ? 0 : -1}
            aria-checked={lane === l}
            aria-disabled={!enabled}
            use:tooltip={enabled ? null : "Önce Havuzum'a şampiyon ekle"}
            onclick={() => selectLane(l)}
            onkeydown={laneKey}
          >
            <LaneIcon lane={l} size={17} />
            <span>{LANE_NAMES[l]}</span>
          </button>
        {/each}
      </div>

      <div class="vs" aria-hidden="true">VS</div>

      {#if enemyChamp}
        <div class="enemy">
          <GameIcon src={enemyChamp.icon} name={enemyChamp.name} size={40} crop tip={false} />
          <div class="enemy-text">
            <span class="enemy-label">Rakip</span>
            <span class="enemy-name">{enemyChamp.name}</span>
          </div>
          <button
            type="button"
            class="btn btn-sm"
            bind:this={changeBtn}
            onclick={() => (changing = true)}
            disabled={changing}>Değiştir</button
          >
        </div>
      {:else}
        <div class="enemy placeholder">
          <span class="enemy-slot" aria-hidden="true">?</span>
          <span class="muted">Rakibini seç</span>
        </div>
      {/if}
    </div>

    {#if showPicker}
      <div class="panel picker-panel">
        <div class="picker-head">
          <h2 class="section-title">Karşındaki şampiyon</h2>
          {#if enemy !== null}
            <button type="button" class="btn btn-sm ghost" onclick={() => (changing = false)}
              >Vazgeç</button
            >
          {/if}
        </div>
        <ChampionPicker
          champions={enemyChoices}
          bind:query
          onpick={pickEnemy}
          placeholder="Karşındaki şampiyon"
          label="Karşındaki şampiyonu ara"
          large
          autofocus={active}
        >
          {#snippet toolbar()}
            {#if roles && lane}
              <LaneFilter {lane} bind:all={showAll} {searching} large />
            {/if}
          {/snippet}
        </ChampionPicker>
      </div>
    {:else}
      <div class="results">
        <div class="results-head">
          <h2 class="section-title">Öneriler</h2>
          {#if lane}
            <span class="muted small">
              {LANE_NAMES[lane]} havuzun · {poolOf(lane).length} şampiyon
            </span>
          {/if}
        </div>

        <div class="list" aria-busy={result?.status === 'loading'}>
          {#if result?.status === 'loading'}
            <span class="sr-only" role="status">Öneriler yükleniyor</span>
            {#each [0, 1, 2] as i (i)}
              <div class="skeleton" class:big={i === 0} aria-hidden="true">
                <span class="sk sk-icon"></span>
                <span class="sk-lines">
                  <span class="sk sk-line w60"></span>
                  <span class="sk sk-line w30"></span>
                </span>
                <span class="sk sk-pill"></span>
              </div>
            {/each}
          {:else if result?.status === 'error'}
            <div class="panel error" role="alert">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M12 3 2 20h20z" />
                <path d="M12 10v4.5M12 17.2v.3" />
              </svg>
              <p class="error-title">Veri alınamadı. İnternet bağlantını kontrol et.</p>
              <p class="error-detail">{result.detail}</p>
              <button type="button" class="btn btn-primary" onclick={retry}>Tekrar dene</button>
            </div>
          {:else if result?.status === 'ok'}
            {#key result.key}
              {#if result.recs.length === 0}
                <p class="none muted">Bu lane için öneri yok.</p>
              {:else}
                {#if result.recs.some((r) => r.error !== null)}
                  <div class="partial" role="status">
                    <span>Bazı şampiyonların verisi alınamadı; sıralama eksik olabilir.</span>
                    <button type="button" class="btn btn-sm" onclick={retry}>Tekrar dene</button>
                  </div>
                {/if}
                {#each result.recs as rec, i (rec.champion)}
                  <RecommendationCard
                    {rec}
                    {data}
                    champion={champByKey.get(rec.champion)}
                    featured={i === 0}
                  />
                {/each}
              {/if}
            {/key}
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</section>

<style>
  .counter {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100%;
    min-height: 0;
  }

  .matchbar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 12px;
    flex: none;
  }
  .seg {
    display: flex;
    padding: 3px;
    gap: 2px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: rgba(1, 10, 19, 0.7);
    min-width: 0;
  }
  .seg-btn {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 34px;
    padding: 0 12px;
    border-radius: 2px;
    color: var(--muted);
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    transition:
      color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease),
      box-shadow var(--t-fast) var(--ease);
  }
  .seg-btn:hover:not([aria-disabled='true']):not(.on) {
    color: var(--text);
    background: rgba(255, 255, 255, 0.04);
  }
  .seg-btn.on {
    color: var(--gold-light);
    background: linear-gradient(180deg, rgba(200, 170, 110, 0.2), rgba(200, 170, 110, 0.07));
    box-shadow: inset 0 0 0 1px var(--gold-mid);
  }
  .seg-btn.on :global(svg) {
    color: var(--gold);
  }
  .seg-btn[aria-disabled='true'] {
    color: #3d4148;
  }

  .vs {
    flex: none;
    color: var(--gold-mid);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.2em;
  }

  .enemy {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .enemy-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.2;
  }
  .enemy-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--hard);
    opacity: 0.85;
  }
  .enemy-name {
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .enemy.placeholder {
    gap: 12px;
  }
  .enemy-slot {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border: 1px dashed var(--gold-dark);
    border-radius: 3px;
    color: var(--gold-mid);
    font-weight: 700;
  }

  .picker-panel {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
  }
  .picker-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 28px;
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

  .results {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .results-head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 0 2px;
  }
  .small {
    font-size: 12px;
  }
  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 10px 6px 12px 0;
    margin-right: -6px;
  }

  /* loading skeleton: the shimmer only exists while loading */
  .skeleton {
    display: flex;
    align-items: center;
    gap: 14px;
    flex: none;
    height: 78px;
    padding: 0 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: rgba(10, 20, 40, 0.8);
  }
  .skeleton.big {
    height: 220px;
    align-items: flex-start;
    padding-top: 16px;
    border-color: rgba(120, 90, 40, 0.4);
  }
  .sk {
    display: block;
    border-radius: 3px;
    background: linear-gradient(90deg, #121c2c 0%, #1c2940 40%, #121c2c 80%);
    background-size: 300% 100%;
    animation: shimmer 1.3s linear infinite;
  }
  .sk-icon {
    width: 52px;
    height: 52px;
    flex: none;
  }
  .big .sk-icon {
    width: 64px;
    height: 64px;
  }
  .sk-lines {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 4px;
  }
  .sk-line {
    height: 12px;
  }
  .w60 {
    width: 38%;
  }
  .w30 {
    width: 18%;
  }
  .sk-pill {
    width: 96px;
    height: 26px;
    border-radius: 13px;
  }
  @keyframes shimmer {
    from {
      background-position: 100% 0;
    }
    to {
      background-position: -50% 0;
    }
  }

  .error {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 32px 24px;
    text-align: center;
  }
  .error svg {
    width: 34px;
    height: 34px;
    fill: none;
    stroke: var(--hard);
    stroke-width: 1.6;
    stroke-linejoin: round;
    stroke-linecap: round;
  }
  .error-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
  }
  .error-detail {
    max-width: 520px;
    color: var(--faint);
    font-size: 12px;
    word-break: break-word;
    margin-bottom: 6px;
  }

  .partial {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 10px 8px 14px;
    border: 1px solid color-mix(in srgb, var(--hard) 45%, transparent);
    border-left-width: 3px;
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--hard) 8%, var(--panel));
    color: var(--text);
    font-size: 13px;
  }
  .none {
    padding: 24px;
    text-align: center;
  }

  .empty-all {
    margin: auto;
    width: min(460px, 100%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 36px 32px;
    text-align: center;
    color: var(--gold-mid);
  }
  .empty-all h2 {
    font-size: 15px;
  }
  .empty-all p {
    margin-bottom: 8px;
  }
</style>
