<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import {
    banSuggestions,
    championMatchup,
    closeCompact,
    errorText,
    getChampSelectState,
    importLoadout,
    loadStatic,
    onChampSelect,
    onPoolsChanged,
    onStaticUpdated,
    openMainWindow,
    recommend,
    runeOptions,
    setMyLane,
    setOpponent
  } from '$lib/api';
  import GameIcon from '$lib/GameIcon.svelte';
  import LaneIcon from '$lib/LaneIcon.svelte';
  import { tooltip } from '$lib/tooltip';
  import type {
    BanSuggestion,
    ChampSelectState,
    Item,
    Lane,
    Recommendation,
    Rune,
    RuneOptions,
    RunePage,
    Scored,
    Spell,
    StaticPayload
  } from '$lib/types';
  import { LANES, LANE_NAMES, VERDICT_NAMES, formatCount, formatPercent } from '$lib/util';

  // Compact always-on-top window shown during champion select (src-tauri/src/compact.rs).
  // State comes from the League client (lcu.rs): pulled once on mount, then pushed
  // with the `champ-select` event. No timers, no animations.

  type Kind = 'pool' | 'mine' | 'ban';
  type Result =
    | { kind: 'none' }
    | { kind: 'loading'; for: Kind }
    | { kind: 'error'; for: Kind; message: string }
    | { kind: 'pool'; recs: Recommendation[] }
    | { kind: 'mine'; rec: Recommendation }
    | { kind: 'ban'; recs: BanSuggestion[] };

  let data = $state.raw<StaticPayload | null>(null);
  let staticError = $state<string | null>(null);
  let cs = $state.raw<ChampSelectState | null>(null);
  /** Lane chosen here when the client gives none (blind pick). */
  let pickedLane = $state<Lane | null>(null);
  let result = $state.raw<Result>({ kind: 'none' });
  let retries = $state(0);

  const champByKey = $derived(new Map((data?.champions ?? []).map((c) => [c.key, c])));
  const champName = (k: number) => champByKey.get(k)?.name ?? `Şampiyon #${k}`;

  const active = $derived(!!cs?.connected && !!cs?.active);
  const lane = $derived(cs?.myLane ?? pickedLane);
  const opponent = $derived(cs?.opponent ?? null);
  const mine = $derived(cs?.locked ? (cs.myChampion ?? null) : null);
  // LC-011: my ban is not done yet; the suggestions are for the champion I hover
  const banning = $derived(!!cs?.banPhase && mine === null);
  const intent = $derived(cs?.myIntent ?? null);

  // What the body needs: in the ban phase the counters of the hovered champion (bans and
  // allies skipped), after lock-in my champion vs the opponent (build even without one),
  // before lock-in my pool vs the opponent. A string so it only changes with its values.
  const requestKey = $derived.by(() => {
    if (!data || !active || !lane || !cs) return null;
    if (banning) {
      if (intent === null) return null;
      const skip = [...new Set([...cs.bans, ...cs.allies])].sort((x, y) => x - y);
      return `ban:${lane}:${intent}:${skip.join(',')}`;
    }
    if (mine !== null) return `mine:${lane}:${mine}:${opponent ?? 0}`;
    if (opponent !== null) return `pool:${lane}:${opponent}`;
    return null;
  });

  let seq = 0;
  $effect(() => {
    const key = requestKey;
    void retries;
    const my = ++seq;
    if (!key) {
      result = { kind: 'none' };
      return;
    }
    const [kind, l, a, b] = key.split(':') as [Kind, Lane, string, string];
    // keep the old rows visible while the same kind refreshes (no flicker)
    const prev = untrack(() => result);
    if (prev.kind !== kind) result = { kind: 'loading', for: kind };
    const job: Promise<Result> =
      kind === 'ban'
        ? banSuggestions(l, Number(a), b ? b.split(',').map(Number) : []).then((recs) => ({
            kind: 'ban',
            recs
          }))
        : kind === 'mine'
          ? championMatchup(l, Number(a), Number(b)).then((rec) => ({ kind: 'mine', rec }))
          : recommend(l, Number(a)).then((recs) => ({ kind: 'pool', recs }));
    job.then(
      (r) => {
        if (my === seq) result = r;
      },
      (e) => {
        if (my === seq) result = { kind: 'error', for: kind, message: errorText(e) };
      }
    );
  });

  // champ select ended: forget the lane picked by hand
  $effect(() => {
    if (!active) pickedLane = null;
  });

  // blind pick: the lane picked here also goes to the backend for the opponent guess (lcu.rs)
  async function pickLane(l: Lane) {
    pickedLane = l;
    try {
      cs = await setMyLane(l);
    } catch {
      /* the guess stays off; the user can still click an enemy */
    }
  }

  async function chooseOpponent(k: number) {
    if (!cs) return;
    const clear = cs.opponentChosen && cs.opponent === k;
    try {
      cs = await setOpponent(clear ? null : k);
    } catch {
      /* the next champ-select event brings the real state */
    }
  }

  function opponentTip(k: number): string {
    const n = champName(k);
    if (k !== opponent) return `${n} · rakibim olarak seç`;
    return cs?.opponentChosen ? `${n} · rakibin (senin seçimin, tahmine dönmek için tıkla)` : `${n} · rakibin (tahmin)`;
  }

  onMount(() => {
    const unlisten: (() => void)[] = [];
    let disposed = false;
    const keep = (u: () => void) => (disposed ? u() : unlisten.push(u));
    // listen first, then pull: an event sent while the window was opening is not lost
    onChampSelect((s) => (cs = s)).then(keep);
    // a pool saved in the main window (e.g. after "Havuzu düzenle"): ask again
    onPoolsChanged(() => retries++).then(keep);
    // the backend reloaded data that was not final (old version / patch)
    onStaticUpdated(() => {
      loadStatic().then(
        (d) => (data = d),
        () => {}
      );
    }).then(keep);
    getChampSelectState().then(
      (s) => (cs = s),
      () => (cs = null)
    );
    loadStatic().then(
      (d) => (data = d),
      (e) => (staticError = errorText(e))
    );
    return () => {
      disposed = true;
      for (const u of unlisten) u();
    };
  });

  // --- LC-014: rune options (after lock-in) ---
  // Fetched apart from the matchup score so a slow `builds` download does not hold the
  // score back. Opens on the gold (matchup) option, else on the most played one.
  type RuneState =
    | { kind: 'none' }
    | { kind: 'loading' }
    | { kind: 'error'; message: string }
    | { kind: 'ok'; data: RuneOptions };
  let runeState = $state.raw<RuneState>({ kind: 'none' });
  let runeIdx = $state(0);
  const runeKey = $derived(
    data && active && lane && mine !== null ? `${lane}:${mine}:${opponent ?? ''}` : null
  );
  let runeSeq = 0;
  $effect(() => {
    const key = runeKey;
    void retries;
    const my = ++runeSeq;
    if (!key) {
      runeState = { kind: 'none' };
      return;
    }
    const [l, c, o] = key.split(':');
    // a new opponent: keep the current page visible until the new options arrive
    if (untrack(() => runeState.kind) !== 'ok') runeState = { kind: 'loading' };
    runeOptions(l as Lane, Number(c), o ? Number(o) : null).then(
      (d) => {
        if (my !== runeSeq) return;
        runeState = { kind: 'ok', data: d };
        runeIdx = d.gold ?? 0;
      },
      (e) => {
        if (my === runeSeq) runeState = { kind: 'error', message: errorText(e) };
      }
    );
  });
  const runeList = $derived(runeState.kind === 'ok' ? runeState.data.options : []);
  const runeGold = $derived(runeState.kind === 'ok' ? runeState.data.gold : null);
  const page = $derived<RunePage | undefined>(runeList[runeIdx]);
  const treePerks = (p: RunePage, tree: number) =>
    p.perks
      .map((id) => data?.runes[String(id)])
      .filter((r): r is Rune => !!r && r.tree === tree)
      .sort((a, b) => a.slot - b.slot);
  const primaryPerks = $derived(page ? treePerks(page, page.primaryTree) : []);
  const secondaryPerks = $derived(page ? treePerks(page, page.secondaryTree) : []);
  const secondaryTree = $derived(page && data ? data.trees[String(page.secondaryTree)] : undefined);
  const shardTip = $derived(
    page && data
      ? `Parçalar: ${page.shards.map((id) => data!.shards[String(id)] ?? `#${id}`).join(' · ')}`
      : ''
  );
  const pageTip = (p: RunePage, i: number) => {
    const tree = data?.trees[String(p.primaryTree)]?.name ?? '';
    const tree2 = data?.trees[String(p.secondaryTree)]?.name ?? '';
    const ks = treePerks(p, p.primaryTree)[0]?.name ?? tree;
    const gold = i === runeGold && opponent !== null ? ` · ${champName(opponent)} için önerilen` : '';
    return `${i + 1}. seçenek: ${ks} · ${tree2}${gold}`;
  };

  // --- build pieces (after lock-in) ---
  const mineRec = $derived(result.kind === 'mine' ? result.rec : null);
  const build = $derived(mineRec?.build ?? null);
  // the matchup's spells when the gold option exists, else the lane's general spells
  const spellKeys = $derived(
    runeState.kind === 'ok' && runeState.data.spells.length ? runeState.data.spells : (build?.spells ?? [])
  );
  const spells = $derived(
    data ? spellKeys.map((k) => data!.spells[String(k)]).filter((s): s is Spell => !!s) : []
  );
  const core = $derived(
    build && data
      ? build.coreItems.map((id) => data!.items[String(id)]).filter((x): x is Item => !!x)
      : []
  );

  // --- LC-015: import the shown page and spells into the client (only on click) ---
  type ImportState =
    | { kind: 'idle' }
    | { kind: 'busy' }
    | { kind: 'confirm'; id: number; page: string }
    | { kind: 'done'; spells: boolean }
    | { kind: 'error'; message: string };
  let importState = $state.raw<ImportState>({ kind: 'idle' });
  let importSeq = 0;
  // another page or another matchup: the button starts over (a late answer is dropped)
  $effect(() => {
    void runeKey;
    void runeIdx;
    importSeq++;
    importState = { kind: 'idle' };
  });
  async function importPage(confirm: number | null) {
    if (!page || importState.kind === 'busy') return;
    const my = ++importSeq;
    importState = { kind: 'busy' };
    try {
      const r = await importLoadout(page, spellKeys, confirm);
      if (my !== importSeq) return;
      importState =
        r.kind === 'needsConfirm' ? { kind: 'confirm', id: r.id, page: r.page } : { kind: 'done', spells: r.spells };
    } catch (e) {
      if (my === importSeq) importState = { kind: 'error', message: errorText(e) };
    }
  }
  const importTip = $derived.by(() => {
    switch (importState.kind) {
      case 'done':
        return importState.spells
          ? "\"LOLCounters\" rün sayfası seçildi, büyüler ayarlandı"
          : "\"LOLCounters\" rün sayfası seçildi; büyüler ayarlanamadı";
      case 'error':
        return `Aktarılamadı: ${importState.message}`;
      default:
        return 'Bu rün sayfasını ve büyüleri LoL istemcisine aktar ("LOLCounters" sayfası)';
    }
  });

  // A Recommendation carries `error`; a ban suggestion (plain Scored) has none.
  type Row = Scored & { error?: string | null };
  const failed = (r: Row) => r.error != null && r.matches === 0;
  const vClass = (r: Row) => (failed(r) ? 'failed' : r.verdict);
  const vLabel = (r: Row) => (failed(r) ? 'Alınamadı' : VERDICT_NAMES[r.verdict]);
  const rate = (r: Row) => (r.winrate === null ? '—' : formatPercent(r.winrate));
  const games = (r: Row) =>
    r.matches > 0 ? `${formatCount(r.matches)} maç` : r.error != null ? 'Veri alınamadı' : 'Eşleşme verisi yok';
  const banTip = (r: BanSuggestion) =>
    intent === null ? '' : `${champName(intent)}, ${champName(r.champion)} karşısında ${rate(r)} kazanıyor`;
</script>

<svelte:head>
  <title>LOLCounters · Şampiyon seçimi</title>
</svelte:head>

{#snippet lanePicker()}
  <div class="lanes" role="radiogroup" aria-label="Lane'ini seç">
    {#each LANES as l (l)}
      <button
        type="button"
        class="lane-btn"
        class:on={lane === l}
        role="radio"
        aria-checked={lane === l}
        aria-label={LANE_NAMES[l]}
        use:tooltip={LANE_NAMES[l]}
        onclick={() => pickLane(l)}
      >
        <LaneIcon lane={l} size={14} />
      </button>
    {/each}
  </div>
{/snippet}

{#snippet verdict(r: Row)}
  <span class="verdict v-{vClass(r)}" use:tooltip={games(r)}>
    <span class="vdot" aria-hidden="true"></span>{vLabel(r)}
  </span>
{/snippet}

{#snippet message(title: string, detail: string | null, tone: 'calm' | 'error' = 'calm')}
  <div class="msg" class:error={tone === 'error'} role={tone === 'error' ? 'alert' : 'status'}>
    <p class="msg-title">{title}</p>
    {#if detail}<p class="msg-detail">{detail}</p>{/if}
  </div>
{/snippet}

<div class="compact">
  <header class="bar" data-tauri-drag-region>
    <span class="brand">
      <svg class="mark" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 2 21 12 12 22 3 12z" />
        <path d="M12 7.5 16.5 12 12 16.5 7.5 12z" />
      </svg>
      <span class="word">LOL<b>COUNTERS</b></span>
      {#if active && lane}
        <span class="lane-chip">
          <LaneIcon {lane} size={13} />
          {LANE_NAMES[lane]}
        </span>
      {/if}
    </span>
    <button
      type="button"
      class="hbtn"
      aria-label="Ana pencereyi aç"
      use:tooltip={'Ana pencereyi aç'}
      onclick={() => openMainWindow()}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <rect x="3.5" y="5" width="17" height="14" rx="1.5" />
        <path d="M3.5 9h17" />
      </svg>
    </button>
    <button
      type="button"
      class="hbtn close"
      aria-label="Kapat"
      use:tooltip={'Kapat · sonraki seçimde yeniden açılır'}
      onclick={() => closeCompact()}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6.5 6.5 11 11M17.5 6.5l-11 11" /></svg>
    </button>
  </header>

  {#if active && cs}
    <section class="enemies" aria-label="Rakipler">
      <span class="label">Rakip</span>
      {#if cs.enemies.length > 0}
        <div class="enemy-list">
          {#each cs.enemies as k (k)}
            {@const c = champByKey.get(k)}
            <button
              type="button"
              class="enemy"
              class:on={k === opponent}
              aria-pressed={k === opponent}
              aria-label={opponentTip(k)}
              use:tooltip={opponentTip(k)}
              onclick={() => chooseOpponent(k)}
            >
              {#if c}
                <GameIcon src={c.icon} name={c.name} size={28} crop tip={false} />
              {:else}
                <span class="ph" aria-hidden="true">?</span>
              {/if}
            </button>
          {/each}
        </div>
        <span class="who-tag" class:chosen={cs.opponentChosen}>
          {opponent === null ? 'Rakibini seç' : cs.opponentChosen ? 'Senin seçimin' : 'Tahmin'}
        </span>
      {:else}
        <span class="waiting">{banning ? 'Banlar sürüyor…' : 'Rakip bekleniyor…'}</span>
      {/if}
    </section>
  {/if}

  <main class="body">
    {#if staticError !== null}
      {@render message('Veri alınamadı', staticError, 'error')}
    {:else if !data || cs === null}
      {@render message('Yükleniyor…', null)}
    {:else if !cs.connected}
      {@render message('LoL istemcisi bağlı değil', 'Şampiyon seçimi başlayınca öneriler burada görünür.')}
    {:else if !cs.active}
      {@render message('Şampiyon seçimi bekleniyor', 'Seçim başlayınca havuzun rakibe göre sıralanır.')}
    {:else if !lane}
      <div class="pick-lane">
        <p class="msg-title">Lane'in belli değil</p>
        <p class="msg-detail">Önerileri görmek için lane'ini seç.</p>
        {@render lanePicker()}
      </div>
    {:else if banning}
      {@const me = intent !== null ? champByKey.get(intent) : undefined}
      <div class="title-row">
        <h2 class="section-title">Ban önerisi</h2>
        {#if intent !== null}
          <span class="vs for">
            {#if me}<GameIcon src={me.icon} name={me.name} size={16} crop tip={false} />{/if}
            <b>{champName(intent)}</b> için
          </span>
        {/if}
      </div>

      {#if intent === null}
        {@render message(
          'Oynayacağın şampiyonu göster',
          'Ona karşı en zor rakipleri burada sıralarım; birini banlayabilirsin.'
        )}
      {:else if result.kind === 'error'}
        <div class="inline-error" role="alert">
          <span>Veri alınamadı</span>
          <button type="button" class="btn btn-sm" use:tooltip={result.message} onclick={() => retries++}>
            Tekrar dene
          </button>
        </div>
      {:else if result.kind === 'ban' && result.recs.length === 0}
        {@render message(`${LANE_NAMES[lane]} için eşleşme verisi yok`, null)}
      {:else if result.kind === 'ban'}
        <ol class="rows">
          {#each result.recs as r, i (r.champion)}
            {@const c = champByKey.get(r.champion)}
            <li class="rec ban" class:worst={i === 0}>
              {#if c}<GameIcon src={c.icon} name={c.name} size={32} crop tip={false} />{/if}
              <div class="who">
                <span class="cname">{champName(r.champion)}</span>
                <span class="sub">
                  {#if i === 0}<span class="danger">En tehlikeli</span>{:else}Counter #{i + 1}{/if}
                </span>
              </div>
              <span class="rate v-{vClass(r)}" use:tooltip={banTip(r)}>{rate(r)}</span>
              {@render verdict(r)}
            </li>
          {/each}
        </ol>
      {:else}
        <ol class="rows" aria-busy="true" aria-label="Ban önerisi yükleniyor">
          {#each [0, 1, 2] as i (i)}<li class="rec skeleton"><span class="sk-icon"></span><span class="sk-line"></span></li>{/each}
        </ol>
      {/if}
    {:else if mine !== null}
      {@const me = champByKey.get(mine)}
      {@const opp = opponent !== null ? champByKey.get(opponent) : undefined}
      <div class="duel">
        {#if me}<GameIcon src={me.icon} name={me.name} size={36} crop tip={false} />{/if}
        <div class="who">
          <span class="cname">{champName(mine)}</span>
          <span class="sub">
            {#if opponent !== null}
              vs
              {#if opp}<GameIcon src={opp.icon} name={opp.name} size={16} crop tip={false} />{/if}
              <b>{champName(opponent)}</b>
            {:else}
              Rakip bekleniyor…
            {/if}
          </span>
        </div>
        {#if cs.myLane === null}{@render lanePicker()}{/if}
        {#if mineRec && opponent !== null}
          <div class="score">
            <span class="rate big v-{vClass(mineRec)}">{rate(mineRec)}</span>
            {@render verdict(mineRec)}
          </div>
        {/if}
      </div>

      {#if result.kind === 'error'}
        <div class="inline-error" role="alert">
          <span>Veri alınamadı</span>
          <button type="button" class="btn btn-sm" use:tooltip={result.message} onclick={() => retries++}>
            Tekrar dene
          </button>
        </div>
      {:else if result.kind === 'loading' || !mineRec}
        <div class="build skeleton" aria-busy="true" aria-label="Build yükleniyor">
          {#each [0, 1, 2] as i (i)}<span class="sk-group"></span>{/each}
        </div>
      {:else}
        <section class="runes" aria-label="Rün seçenekleri">
          <div class="runes-head">
            <h4 class="section-title">Rünler</h4>
            {#if runeList.length > 1}
              <div class="pager">
                <button
                  type="button"
                  class="pg"
                  aria-label="Önceki rün sayfası"
                  disabled={runeIdx === 0}
                  onclick={() => (runeIdx = Math.max(0, runeIdx - 1))}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m14.5 6-6 6 6 6" /></svg>
                </button>
                {#each runeList as p, i (i)}
                  <button
                    type="button"
                    class="dot"
                    class:on={i === runeIdx}
                    class:gold={i === runeGold}
                    aria-label={pageTip(p, i)}
                    aria-current={i === runeIdx}
                    use:tooltip={pageTip(p, i)}
                    onclick={() => (runeIdx = i)}
                  ></button>
                {/each}
                <button
                  type="button"
                  class="pg"
                  aria-label="Sonraki rün sayfası"
                  disabled={runeIdx >= runeList.length - 1}
                  onclick={() => (runeIdx = Math.min(runeList.length - 1, runeIdx + 1))}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9.5 6 6 6-6 6" /></svg>
                </button>
              </div>
            {/if}
            {#if runeGold !== null && opponent !== null}
              <button
                type="button"
                class="gold-tag"
                class:dim={runeIdx !== runeGold}
                use:tooltip={'u.gg bu eşleşmede bu sayfayı öneriyor'}
                onclick={() => (runeIdx = runeGold ?? 0)}
              >
                ★ {champName(opponent)} için önerilen
              </button>
            {/if}
          </div>

          {#if runeState.kind === 'error'}
            <div class="page empty" role="alert">
              <span>Rünler alınamadı</span>
              <button type="button" class="btn btn-sm" use:tooltip={runeState.message} onclick={() => retries++}>
                Tekrar dene
              </button>
            </div>
          {:else if runeState.kind !== 'ok'}
            <div class="page skeleton" aria-busy="true" aria-label="Rünler yükleniyor"></div>
          {:else if !page}
            <div class="page empty"><span>Bu lane için rün verisi yok</span></div>
          {:else}
            <div class="page" class:gold={runeIdx === runeGold}>
              <div class="tree" aria-label="Ana ağaç">
                {#each primaryPerks as r, i (r.id)}
                  <GameIcon src={r.icon} name={r.name} size={i === 0 ? 30 : 20} shape="round" />
                {/each}
              </div>
              <span class="tsep" aria-hidden="true"></span>
              <div class="tree" aria-label="İkincil ağaç">
                {#if secondaryTree}
                  <span class="tree2">
                    <GameIcon src={secondaryTree.icon} name={secondaryTree.name} size={16} shape="round" />
                  </span>
                {/if}
                {#each secondaryPerks as r (r.id)}
                  <GameIcon src={r.icon} name={r.name} size={20} shape="round" />
                {/each}
              </div>
              <span class="pstats" use:tooltip={shardTip}>
                {#if page.matches > 0}
                  <b>{formatPercent(page.wins / page.matches)}</b>
                  <span>{formatCount(page.matches)} maç</span>
                {/if}
              </span>
            </div>
          {/if}
        </section>

        <div class="kit">
          {#if importState.kind === 'confirm'}
            {@const target = importState}
            <div class="confirm" role="alertdialog" aria-label="Boş rün sayfası yok">
              <span class="ctext">Boş sayfa yok · <b>{target.page}</b> üstüne yazılsın mı?</span>
              <button type="button" class="btn btn-sm btn-primary" onclick={() => importPage(target.id)}>Yaz</button>
              <button type="button" class="cancel" onclick={() => (importState = { kind: 'idle' })}>Vazgeç</button>
            </div>
          {:else}
          {#if spells.length}
            <div class="row" aria-label="Büyüler">
              {#each spells as s (s.key)}<GameIcon src={s.icon} name={s.name} size={26} />{/each}
            </div>
          {/if}
          {#if core.length && build}
            <span class="ksep" aria-hidden="true"></span>
            <div
              class="row"
              aria-label="Çekirdek eşyalar"
              use:tooltip={build.matches > 0
                ? `Genel build · ${formatPercent(build.wins / build.matches)} kazanma · ${formatCount(build.matches)} maç`
                : 'Genel build'}
            >
              {#each core as it, i (i)}
                {#if i > 0}<span class="sep" aria-hidden="true">›</span>{/if}
                <GameIcon src={it.icon} name={it.name} size={26} />
              {/each}
            </div>
          {/if}
          {#if page}
            <button
              type="button"
              class="btn btn-sm btn-primary import"
              class:done={importState.kind === 'done'}
              class:failed={importState.kind === 'error'}
              disabled={importState.kind === 'busy'}
              aria-label={importTip}
              use:tooltip={importTip}
              onclick={() => importPage(null)}
            >
              {#if importState.kind === 'done'}
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12.5 4.5 4.5L19 7.5" /></svg>Aktarıldı
              {:else if importState.kind === 'busy'}
                Aktarılıyor…
              {:else if importState.kind === 'error'}
                Tekrar dene
              {:else}
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 4v11M7.5 10.5 12 15l4.5-4.5M5 19.5h14" /></svg>İçe aktar
              {/if}
            </button>
          {/if}
          {/if}
        </div>
      {/if}
    {:else}
      <div class="title-row">
        <h2 class="section-title">Havuzun</h2>
        {#if cs.myLane === null}{@render lanePicker()}{/if}
        {#if opponent !== null}
          <span class="vs">vs <b>{champName(opponent)}</b></span>
        {/if}
      </div>

      {#if opponent === null}
        {@render message(
          'Rakibin belli olunca havuzun sıralanır',
          cs.enemies.length > 0 ? 'Rakibini yukarıdan seçebilirsin.' : null
        )}
      {:else if result.kind === 'error'}
        <div class="inline-error" role="alert">
          <span>Veri alınamadı</span>
          <button type="button" class="btn btn-sm" use:tooltip={result.message} onclick={() => retries++}>
            Tekrar dene
          </button>
        </div>
      {:else if result.kind === 'pool' && result.recs.length === 0}
        <div class="msg" role="status">
          <p class="msg-title">{LANE_NAMES[lane]} havuzun boş</p>
          <button type="button" class="btn btn-sm" onclick={() => openMainWindow()}>Havuzu düzenle</button>
        </div>
      {:else if result.kind === 'pool'}
        <ol class="rows">
          {#each result.recs.slice(0, 3) as r, i (r.champion)}
            {@const c = champByKey.get(r.champion)}
            {@const isIntent = r.champion === intent}
            <li class="rec" class:best={i === 0} class:intent={isIntent}>
              {#if c}<GameIcon src={c.icon} name={c.name} size={32} crop tip={false} />{/if}
              <div class="who">
                <span class="cname">{champName(r.champion)}</span>
                <span class="sub">
                  {#if i === 0}<span class="star">★ En iyi</span>{:else}Tercih #{r.poolIndex + 1}{/if}
                  {#if isIntent}<span class="tag">Seçimin</span>{/if}
                </span>
              </div>
              <span class="rate v-{vClass(r)}">{rate(r)}</span>
              {@render verdict(r)}
            </li>
          {/each}
        </ol>
      {:else}
        <ol class="rows" aria-busy="true" aria-label="Öneriler yükleniyor">
          {#each [0, 1, 2] as i (i)}<li class="rec skeleton"><span class="sk-icon"></span><span class="sk-line"></span></li>{/each}
        </ol>
      {/if}
    {/if}
  </main>
</div>

<style>
  .compact {
    display: grid;
    grid-template-rows: auto auto minmax(0, 1fr);
    height: 100vh;
    overflow: hidden;
    border-top: 1px solid rgba(200, 170, 110, 0.35);
  }

  /* ---- header (drag region) ---- */
  .bar {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 30px;
    padding: 0 4px 0 10px;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(1, 10, 19, 0.98), rgba(1, 10, 19, 0.75));
    box-shadow: 0 1px 0 rgba(120, 90, 40, 0.22);
    user-select: none;
  }
  .brand {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    /* clicks fall through to the header, so the whole bar drags the window */
    pointer-events: none;
  }
  .mark {
    width: 14px;
    height: 14px;
    flex: none;
    fill: none;
    stroke: var(--gold);
    stroke-width: 1.6;
    stroke-linejoin: round;
  }
  .mark path:last-child {
    fill: var(--gold);
    stroke: none;
    opacity: 0.85;
  }
  .word {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.2em;
    color: var(--gold-mid);
    white-space: nowrap;
  }
  .word b {
    font-weight: 700;
    background: linear-gradient(180deg, #f0e6d2, #c8aa6e 60%, #a8884a);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .lane-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 18px;
    padding: 0 8px 0 6px;
    border: 1px solid var(--gold-dark);
    border-radius: 9px;
    color: var(--gold);
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
  }
  .hbtn {
    display: grid;
    place-items: center;
    width: 26px;
    height: 22px;
    flex: none;
    border-radius: 2px;
    color: var(--muted);
    transition:
      color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .hbtn svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .hbtn:hover {
    color: var(--gold-light);
    background: rgba(200, 170, 110, 0.1);
  }
  .hbtn.close:hover {
    color: #fff;
    background: rgba(232, 64, 87, 0.75);
  }
  .hbtn:focus-visible {
    outline-offset: -2px;
  }

  /* ---- enemies ---- */
  .enemies {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 10px;
    border-bottom: 1px solid #141b24;
    background: rgba(1, 10, 19, 0.35);
  }
  .label {
    flex: none;
    width: 38px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--gold);
  }
  .enemy-list {
    display: flex;
    gap: 6px;
    min-width: 0;
  }
  .enemy {
    position: relative;
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    flex: none;
    border-radius: 3px;
    opacity: 0.72;
    transition:
      opacity var(--t-fast) var(--ease),
      box-shadow var(--t-fast) var(--ease),
      transform var(--t-fast) var(--ease);
  }
  .enemy:hover {
    opacity: 1;
    transform: translateY(-1px);
  }
  .enemy.on {
    opacity: 1;
    box-shadow:
      0 0 0 2px var(--gold),
      0 0 10px rgba(200, 170, 110, 0.45);
  }
  .enemy :global(.icon) {
    border-color: #2a2f36;
  }
  .enemy.on :global(.icon) {
    border-color: transparent;
  }
  .ph {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 1px solid var(--gold-dark);
    border-radius: 3px;
    background: #0b111a;
    color: var(--muted);
    font-size: 12px;
  }
  .who-tag {
    margin-left: auto;
    flex: none;
    font-size: 11px;
    color: var(--faint);
    white-space: nowrap;
  }
  .who-tag.chosen {
    color: var(--gold);
  }
  .waiting {
    color: var(--muted);
    font-size: 12px;
  }

  /* ---- body ---- */
  .body {
    grid-row: 3;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px 10px;
  }
  .title-row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 22px;
  }
  .title-row .section-title {
    font-size: 10.5px;
  }
  .vs {
    margin-left: auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--muted);
    font-size: 12px;
  }
  .vs b {
    color: var(--gold-light);
    font-weight: 600;
  }

  .lanes {
    display: flex;
    gap: 3px;
  }
  .lane-btn {
    display: grid;
    place-items: center;
    width: 24px;
    height: 22px;
    border: 1px solid #262d36;
    border-radius: 2px;
    color: var(--muted);
    transition:
      color var(--t-fast) var(--ease),
      border-color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .lane-btn:hover {
    color: var(--gold-light);
    border-color: var(--gold-mid);
  }
  .lane-btn.on {
    color: var(--gold-light);
    border-color: var(--gold);
    background: rgba(200, 170, 110, 0.12);
  }

  /* pool rows */
  .rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .rec {
    --vc: var(--even);
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 8px 0 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: linear-gradient(180deg, rgba(15, 26, 43, 0.95), rgba(10, 20, 40, 0.95));
  }
  .rec.best {
    border-color: var(--gold-mid);
    background:
      radial-gradient(ellipse 60% 140% at 0% 0%, rgba(200, 170, 110, 0.1), transparent 60%),
      linear-gradient(180deg, rgba(19, 33, 58, 0.97), rgba(10, 20, 40, 0.97));
  }
  /* ban suggestions (LC-011): the worst matchup is marked in the "hard" colour */
  .rec.ban.worst {
    border-color: color-mix(in srgb, var(--hard) 50%, var(--border));
    background:
      radial-gradient(ellipse 60% 140% at 0% 0%, color-mix(in srgb, var(--hard) 12%, transparent), transparent 60%),
      linear-gradient(180deg, rgba(19, 33, 58, 0.97), rgba(10, 20, 40, 0.97));
  }
  .danger {
    color: var(--hard);
    font-weight: 600;
  }
  .vs.for {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .rec.intent::before {
    content: '';
    position: absolute;
    left: -1px;
    top: 6px;
    bottom: 6px;
    width: 2px;
    border-radius: 1px;
    background: var(--teal);
    box-shadow: 0 0 6px rgba(10, 200, 185, 0.6);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.25;
  }
  .cname {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
  }
  .rec.best .cname {
    color: var(--gold-light);
  }
  .sub {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    color: var(--muted);
    font-size: 11px;
  }
  .sub b {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--gold-light);
    font-weight: 600;
  }
  .star {
    color: var(--gold);
    font-weight: 600;
  }
  .tag {
    padding: 0 6px;
    border: 1px solid rgba(10, 200, 185, 0.45);
    border-radius: 8px;
    background: var(--teal-soft);
    color: #7fe9df;
    font-size: 10px;
    font-weight: 600;
    line-height: 14px;
  }

  .rate {
    flex: none;
    min-width: 48px;
    text-align: right;
    color: var(--vc);
    font-size: 15px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .rate.big {
    min-width: 0;
    font-size: 20px;
    line-height: 1.1;
  }
  .v-good {
    --vc: var(--good);
  }
  .v-even {
    --vc: var(--even);
  }
  .v-hard,
  .v-failed {
    --vc: var(--hard);
  }
  .v-lowData {
    --vc: var(--low);
  }
  .verdict {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    padding: 0 9px;
    flex: none;
    border: 1px solid color-mix(in srgb, var(--vc) 45%, transparent);
    border-radius: 11px;
    background: color-mix(in srgb, var(--vc) 10%, transparent);
    color: var(--vc);
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    cursor: default;
  }
  .vdot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--vc);
  }

  /* after lock-in */
  .duel {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 2px 0 8px;
    border-bottom: 1px dashed #1f2630;
  }
  .duel .cname {
    font-size: 15px;
    color: var(--gold-light);
  }
  .duel .sub {
    font-size: 12px;
  }
  .score {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 3px;
    flex: none;
  }
  .build {
    display: flex;
    align-items: flex-start;
    gap: 18px;
    padding-top: 4px;
  }

  /* rune options (LC-014) */
  .runes {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .runes-head {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 20px;
  }
  .runes-head .section-title {
    font-size: 9.5px;
  }
  .pager {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .pg {
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border-radius: 2px;
    color: var(--gold);
    transition:
      color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .pg svg {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .pg:hover:not(:disabled) {
    color: var(--gold-light);
    background: rgba(200, 170, 110, 0.12);
  }
  .pg:disabled {
    color: #3a4048;
    cursor: default;
  }
  .dot {
    display: grid;
    place-items: center;
    width: 14px;
    height: 18px;
  }
  .dot::before {
    content: '';
    width: 6px;
    height: 6px;
    border: 1px solid var(--gold-dark);
    border-radius: 50%;
    background: transparent;
    transition:
      background var(--t-fast) var(--ease),
      border-color var(--t-fast) var(--ease),
      transform var(--t-fast) var(--ease);
  }
  .dot:hover::before {
    border-color: var(--gold-light);
  }
  .dot.on::before {
    border-color: var(--gold-light);
    background: var(--gold-light);
    transform: scale(1.15);
  }
  .dot.gold::before {
    border-color: #f0c050;
    box-shadow: 0 0 5px rgba(240, 192, 80, 0.6);
  }
  .dot.gold.on::before {
    background: #f0c050;
  }
  .gold-tag {
    margin-left: auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    height: 18px;
    padding: 0 8px;
    border: 1px solid rgba(240, 192, 80, 0.55);
    border-radius: 9px;
    background: rgba(240, 192, 80, 0.1);
    color: #f0c050;
    font-size: 10.5px;
    font-weight: 600;
    transition:
      opacity var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .gold-tag.dim {
    opacity: 0.55;
  }
  .gold-tag:hover {
    opacity: 1;
    background: rgba(240, 192, 80, 0.18);
  }
  .page {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    padding: 0 10px 0 7px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: linear-gradient(180deg, rgba(15, 26, 43, 0.95), rgba(10, 20, 40, 0.95));
    transition:
      border-color var(--t-fast) var(--ease),
      box-shadow var(--t-fast) var(--ease);
  }
  .page.gold {
    border-color: #f0c050;
    background:
      radial-gradient(ellipse 60% 140% at 0% 0%, rgba(240, 192, 80, 0.12), transparent 60%),
      linear-gradient(180deg, rgba(19, 33, 58, 0.97), rgba(10, 20, 40, 0.97));
    box-shadow:
      0 0 0 1px rgba(240, 192, 80, 0.25),
      0 0 12px rgba(240, 192, 80, 0.3);
  }
  .page.empty {
    justify-content: center;
    gap: 12px;
    color: var(--muted);
    font-size: 12px;
  }
  .page.empty[role='alert'] {
    color: var(--hard);
    font-weight: 600;
  }
  .page.skeleton {
    border-color: #161d26;
  }
  .tree {
    display: flex;
    align-items: center;
    gap: 3px;
  }
  .tree :global(.icon) {
    border-color: transparent;
    background: #0b111a;
  }
  .tsep {
    width: 1px;
    height: 24px;
    background: #222a35;
  }
  .tree2 {
    margin-right: 1px;
    opacity: 0.85;
  }
  .tree2 :global(.icon) {
    background: transparent;
  }
  .pstats {
    margin-left: auto;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    line-height: 1.2;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    cursor: default;
  }
  .pstats b {
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
  }
  .page.gold .pstats b {
    color: #f0c050;
  }
  .pstats span {
    color: var(--faint);
    font-size: 10.5px;
  }
  .kit {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  /* import (LC-015) */
  .import {
    margin-left: auto;
    gap: 5px;
    height: 26px;
    padding: 0 10px;
    font-size: 10.5px;
  }
  .import svg {
    width: 13px;
    height: 13px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .import.done {
    border-color: color-mix(in srgb, var(--good) 60%, transparent);
    background: color-mix(in srgb, var(--good) 12%, transparent);
    color: var(--good);
  }
  .import.failed {
    border-color: color-mix(in srgb, var(--hard) 60%, transparent);
    background: color-mix(in srgb, var(--hard) 10%, transparent);
    color: var(--hard);
  }
  .confirm {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 28px;
  }
  .ctext {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--muted);
    font-size: 11.5px;
  }
  .ctext b {
    color: var(--gold-light);
    font-weight: 600;
  }
  .confirm .btn {
    height: 24px;
    padding: 0 10px;
  }
  .cancel {
    flex: none;
    color: var(--muted);
    font-size: 11.5px;
  }
  .cancel:hover {
    color: var(--gold-light);
  }
  .ksep {
    width: 1px;
    height: 18px;
    background: #222a35;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 28px;
  }
  .sep {
    color: var(--gold-mid);
    font-size: 14px;
    line-height: 1;
  }

  /* static placeholders while loading (no animation: idle CPU stays at zero) */
  .skeleton {
    opacity: 0.55;
  }
  .rec.skeleton {
    border-color: #161d26;
  }
  .sk-icon {
    width: 32px;
    height: 32px;
    border-radius: 3px;
    background: #111a27;
  }
  .sk-line {
    width: 45%;
    height: 10px;
    border-radius: 5px;
    background: #111a27;
  }
  .build.skeleton {
    padding-top: 10px;
  }
  .sk-group {
    width: 72px;
    height: 44px;
    border-radius: 3px;
    background: #0e1622;
  }

  /* messages */
  .msg,
  .pick-lane {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    text-align: center;
  }
  .msg-title {
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
  }
  .msg-detail {
    max-width: 300px;
    color: var(--muted);
    font-size: 12px;
    overflow-wrap: anywhere;
  }
  .msg.error .msg-title {
    color: var(--hard);
  }
  .msg.error .msg-detail {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    color: var(--faint);
  }
  .pick-lane .lanes {
    margin-top: 4px;
    gap: 6px;
  }
  .pick-lane .lane-btn {
    width: 34px;
    height: 30px;
  }
  .inline-error {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--hard);
    font-size: 12px;
    font-weight: 600;
  }
</style>
