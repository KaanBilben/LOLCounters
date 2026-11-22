<script lang="ts">
  import { untrack } from 'svelte';
  import { slide } from 'svelte/transition';
  import GameIcon from './GameIcon.svelte';
  import { tooltip } from './tooltip';
  import type { Champion, Item, Recommendation, Rune, StaticPayload } from './types';
  import { VERDICT_NAMES, formatCount, formatDecimal, formatPercent } from './util';

  let {
    rec,
    champion,
    data,
    featured = false
  }: {
    rec: Recommendation;
    champion: Champion | undefined;
    data: StaticPayload;
    featured?: boolean;
  } = $props();

  // first card starts open (cards are re-created for every new result list)
  let expanded = $state(untrack(() => featured));

  const uid = $props.id();
  const name = $derived(champion?.name ?? `Şampiyon #${rec.champion}`);
  const build = $derived(rec.build);

  const runeOf = (id: number): Rune | undefined => data.runes[String(id)];
  const items = (ids: number[]): Item[] =>
    ids.map((id) => data.items[String(id)]).filter((x): x is Item => !!x);

  const perksFor = (tree: number) =>
    (build?.runes.perks ?? [])
      .map(runeOf)
      .filter((r): r is Rune => !!r && r.tree === tree)
      .sort((a, b) => a.slot - b.slot);

  const primaryTree = $derived(build ? data.trees[String(build.runes.primaryTree)] : undefined);
  const secondaryTree = $derived(build ? data.trees[String(build.runes.secondaryTree)] : undefined);
  const primaryPerks = $derived(build ? perksFor(build.runes.primaryTree) : []);
  const secondaryPerks = $derived(build ? perksFor(build.runes.secondaryTree) : []);
  const shards = $derived(
    (build?.runes.shards ?? []).map((id) => data.shards[String(id)]).filter((s): s is string => !!s)
  );
  const spells = $derived(
    (build?.spells ?? []).map((k) => data.spells[String(k)]).filter((s) => !!s)
  );
  const starting = $derived(items(build?.startingItems ?? []));
  const core = $derived(items(build?.coreItems ?? []));
  const late = $derived(
    (build?.lateOptions ?? [])
      .slice(0, 3)
      .map((opts, i) => ({ slot: i + 4, items: items(opts) }))
      .filter((g) => g.items.length > 0)
  );
  const skills = $derived((build?.skillMaxOrder ?? '').split('').filter((ch) => /[QWER]/i.test(ch)));

  // a fetch error is shown as its own state, not as real "low data"
  const failed = $derived(rec.error !== null);
  const vClass = $derived(failed ? 'failed' : rec.verdict);
  const vLabel = $derived(failed ? 'Alınamadı' : VERDICT_NAMES[rec.verdict]);
  const verdictTip = $derived(
    failed
      ? `Veri alınamadı: ${rec.error}`
      : `Düzeltilmiş oran ${formatPercent(rec.adjusted)} · Puan ${formatDecimal(rec.score)}`
  );
</script>

<article class="card v-{vClass}" class:featured aria-label="{name}, {vLabel}">
  {#if featured}
    <span class="ribbon">Önerilen</span>
  {/if}

  <header class="head">
    {#if champion}
      <GameIcon src={champion.icon} {name} size={featured ? 64 : 52} crop tip={false} />
    {/if}

    <div class="who">
      <h3 class="cname">{name}</h3>
      <span class="pref">Tercih #{rec.poolIndex + 1}</span>
    </div>

    <div class="stats">
      <span class="rate">{rec.winrate === null ? '—' : formatPercent(rec.winrate)}</span>
      <span class="games" class:failed>
        {rec.matches > 0
          ? `${formatCount(rec.matches)} maç`
          : rec.error !== null
            ? 'Veri alınamadı'
            : 'Eşleşme verisi yok'}
      </span>
    </div>

    <span class="verdict" use:tooltip={verdictTip}>
      <span class="vdot" aria-hidden="true"></span>
      {vLabel}
    </span>

    <button
      type="button"
      class="toggle"
      class:open={expanded}
      aria-expanded={expanded}
      aria-controls="{uid}-build"
      onclick={() => (expanded = !expanded)}
    >
      <span lang={expanded ? undefined : 'en'}>{expanded ? 'Gizle' : 'Build'}</span>
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
    </button>
  </header>

  {#if expanded}
    <div class="build" id="{uid}-build" transition:slide={{ duration: 180 }}>
      {#if !build}
        <p class="nobuild">{rec.error !== null ? 'Build verisi alınamadı' : 'Build verisi yok'}</p>
      {:else}
        <div class="grid">
          <section class="block runes" aria-label="Rünler">
            <h4 class="section-title">Rünler</h4>
            <div class="tree-row">
              {#if primaryTree}
                <GameIcon src={primaryTree.icon} name={primaryTree.name} size={26} shape="round" />
              {/if}
              {#each primaryPerks as r (r.id)}
                <GameIcon src={r.icon} name={r.name} size={r.slot === 0 ? 46 : 32} shape="round" />
              {/each}
            </div>
            <div class="tree-row">
              {#if secondaryTree}
                <GameIcon src={secondaryTree.icon} name={secondaryTree.name} size={26} shape="round" />
              {/if}
              {#each secondaryPerks as r (r.id)}
                <GameIcon src={r.icon} name={r.name} size={30} shape="round" />
              {/each}
            </div>
            {#if shards.length}
              <ul class="shards" aria-label="Parçacıklar">
                {#each shards as s, i (i)}
                  <li class="shard">{s}</li>
                {/each}
              </ul>
            {/if}
          </section>

          <div class="side">
            {#if spells.length}
              <section class="block" aria-label="Büyüler">
                <h4 class="section-title">Büyüler</h4>
                <div class="row">
                  {#each spells as s (s.key)}
                    <GameIcon src={s.icon} name={s.name} size={36} />
                  {/each}
                </div>
              </section>
            {/if}
            {#if skills.length}
              <section class="block" aria-label="Yetenek sırası">
                <h4 class="section-title">Yetenek sırası</h4>
                <div class="row skills">
                  {#each skills as k, i (i)}
                    {#if i > 0}<span class="sep" aria-hidden="true">›</span>{/if}
                    <kbd class="key">{k.toUpperCase()}</kbd>
                  {/each}
                </div>
              </section>
            {/if}
          </div>

          <div class="items">
            {#if starting.length}
              <section class="block" aria-label="Başlangıç eşyaları">
                <h4 class="section-title">Başlangıç</h4>
                <div class="row">
                  {#each starting as it, i (i)}
                    <GameIcon src={it.icon} name={it.name} size={36} />
                  {/each}
                </div>
              </section>
            {/if}
            {#if core.length}
              <section class="block" aria-label="Çekirdek eşyalar">
                <h4 class="section-title">Çekirdek</h4>
                <div class="row">
                  {#each core as it, i (i)}
                    {#if i > 0}<span class="sep" aria-hidden="true">›</span>{/if}
                    <GameIcon src={it.icon} name={it.name} size={36} />
                  {/each}
                </div>
              </section>
            {/if}
            {#each late as g (g.slot)}
              <section class="block" aria-label="{g.slot}. eşya seçenekleri">
                <h4 class="section-title">{g.slot}. eşya</h4>
                <div class="row opts">
                  {#each g.items as it, i (i)}
                    <GameIcon src={it.icon} name={it.name} size={30} />
                  {/each}
                </div>
              </section>
            {/each}
          </div>
        </div>

        {#if build.matches > 0}
          <p class="build-meta">
            Bu build: {formatPercent(build.wins / build.matches)} kazanma · {formatCount(build.matches)}
            maç
          </p>
        {/if}
      {/if}
    </div>
  {/if}
</article>

<style>
  .card {
    --vc: var(--even);
    position: relative;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: linear-gradient(180deg, rgba(15, 26, 43, 0.95), rgba(10, 20, 40, 0.95));
    transition: border-color var(--t-fast) var(--ease);
  }
  .card:hover {
    border-color: #2c3440;
  }
  .v-good {
    --vc: var(--good);
  }
  .v-even {
    --vc: var(--even);
  }
  .v-hard {
    --vc: var(--hard);
  }
  .v-lowData {
    --vc: var(--low);
  }
  .v-failed {
    --vc: var(--hard);
  }

  .card.featured {
    border-color: var(--gold-mid);
    background:
      radial-gradient(ellipse 60% 120% at 0% 0%, rgba(200, 170, 110, 0.1), transparent 60%),
      linear-gradient(180deg, rgba(19, 33, 58, 0.97), rgba(10, 20, 40, 0.97));
    box-shadow:
      0 0 0 1px rgba(200, 170, 110, 0.15),
      0 10px 30px rgba(0, 0, 0, 0.4),
      inset 0 1px 0 rgba(240, 230, 210, 0.08);
  }
  .card.featured:hover {
    border-color: var(--gold);
  }
  .ribbon {
    position: absolute;
    top: -9px;
    left: 16px;
    padding: 1px 10px;
    border: 1px solid var(--gold-mid);
    border-radius: 2px;
    background: linear-gradient(180deg, #2a2213, #17130b);
    color: var(--gold);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.18em;
    text-transform: uppercase;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 14px;
  }
  .featured .head {
    padding: 16px 16px 14px;
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .cname {
    color: var(--text);
    font-size: 16px;
    letter-spacing: 0.01em;
    text-transform: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .featured .cname {
    font-size: 19px;
    color: var(--gold-light);
  }
  .pref {
    font-size: 12px;
    color: var(--muted);
  }

  .stats {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    min-width: 92px;
  }
  .rate {
    font-size: 22px;
    font-weight: 600;
    line-height: 1.1;
    color: var(--vc);
    font-variant-numeric: tabular-nums;
  }
  .featured .rate {
    font-size: 26px;
  }
  .games {
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
  }
  .games.failed {
    color: var(--hard);
  }

  .verdict {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 26px;
    padding: 0 11px;
    flex: none;
    border: 1px solid color-mix(in srgb, var(--vc) 45%, transparent);
    border-radius: 13px;
    background: color-mix(in srgb, var(--vc) 10%, transparent);
    color: var(--vc);
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    cursor: default;
  }
  .vdot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--vc);
  }

  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 10px 0 12px;
    flex: none;
    border: 1px solid var(--border);
    border-radius: 2px;
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    transition:
      color var(--t-fast) var(--ease),
      border-color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .toggle span {
    min-width: 40px;
  }
  .toggle:hover {
    color: var(--gold-light);
    border-color: var(--gold-mid);
    background: rgba(200, 170, 110, 0.06);
  }
  .toggle svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
    transition: transform var(--t) var(--ease);
  }
  .toggle.open svg {
    transform: rotate(180deg);
  }

  .build {
    border-top: 1px solid var(--border);
  }
  .featured .build {
    border-top-color: rgba(120, 90, 40, 0.5);
  }
  .nobuild {
    padding: 16px;
    color: var(--muted);
    text-align: center;
  }

  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1.35fr) minmax(0, 1fr);
    gap: 16px 24px;
    padding: 14px 16px 6px;
  }
  .block {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .runes {
    gap: 10px;
  }
  .side {
    display: flex;
    flex-wrap: wrap;
    gap: 16px 28px;
    align-content: flex-start;
  }
  .items {
    grid-column: 1 / -1;
    display: flex;
    flex-wrap: wrap;
    gap: 14px 28px;
    padding-top: 12px;
    border-top: 1px dashed #1f2630;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .opts {
    gap: 4px;
  }
  .tree-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .tree-row :global(.icon:first-child) {
    margin-right: 4px;
    border-color: transparent;
    background: transparent;
  }
  .sep {
    color: var(--gold-mid);
    font-size: 16px;
    line-height: 1;
  }
  .shards {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .shard {
    padding: 2px 9px;
    border: 1px solid #262d36;
    border-radius: 11px;
    background: rgba(1, 10, 19, 0.6);
    color: var(--muted);
    font-size: 11.5px;
  }
  .key {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border: 1px solid var(--gold-mid);
    border-bottom-width: 3px;
    border-radius: 4px;
    background: linear-gradient(180deg, #1c232c, #10151b);
    color: var(--gold-light);
    font: inherit;
    font-size: 14px;
    font-weight: 700;
  }
  .skills {
    gap: 8px;
  }
  .build-meta {
    padding: 4px 16px 12px;
    font-size: 11.5px;
    color: var(--faint);
    text-align: right;
  }
</style>
