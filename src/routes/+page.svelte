<script lang="ts">
  import { onMount } from 'svelte';
  import { errorText, getSettings, loadStatic, saveSettings } from '$lib/api';
  import CounterView from '$lib/CounterView.svelte';
  import PoolEditor from '$lib/PoolEditor.svelte';
  import type { Settings, StaticPayload } from '$lib/types';
  import { LANES } from '$lib/util';

  type View = 'counter' | 'pool';

  let data = $state.raw<StaticPayload | null>(null);
  let settings = $state.raw<Settings>({ pools: {} });
  let loadError = $state<string | null>(null);
  let view = $state<View>('counter');
  let welcome = $state(false);
  let poolDirty = $state(false);

  const champByKey = $derived(new Map((data?.champions ?? []).map((c) => [c.key, c])));

  async function init() {
    loadError = null;
    try {
      const [d, s] = await Promise.all([loadStatic(), getSettings()]);
      settings = s;
      if (LANES.every((l) => (s.pools[l]?.length ?? 0) === 0)) {
        view = 'pool';
        welcome = true;
      }
      data = d;
    } catch (e) {
      loadError = errorText(e);
    }
  }

  async function save(s: Settings) {
    await saveSettings(s);
    settings = s;
    welcome = false;
  }

  onMount(init);

  const tabs: { id: View; label: string }[] = [
    { id: 'counter', label: 'Counter' },
    { id: 'pool', label: 'Havuzum' }
  ];
</script>

<div class="app">
  <header class="topbar">
    <div class="brand" aria-label="LOLCounters">
      <svg class="mark" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 2 21 12 12 22 3 12z" />
        <path d="M12 7.5 16.5 12 12 16.5 7.5 12z" />
      </svg>
      <span class="word">LOL<b>COUNTERS</b></span>
    </div>

    {#if data}
      <div class="tabs" role="tablist" aria-label="Görünüm">
        {#each tabs as t (t.id)}
          <button
            type="button"
            role="tab"
            id="tab-{t.id}"
            class="tab"
            class:on={view === t.id}
            aria-selected={view === t.id}
            aria-controls="view-{t.id}"
            onclick={() => (view = t.id)}
          >
            {t.label}
            {#if t.id === 'pool' && poolDirty}
              <span class="unsaved" aria-label="kaydedilmemiş değişiklik var"></span>
            {/if}
          </button>
        {/each}
      </div>
      <span class="patch" title="Data Dragon {data.version}">Yama {data.uggPatch}</span>
    {/if}
  </header>

  <main class="main">
    {#if data}
      <div class="view" id="view-counter" role="tabpanel" aria-labelledby="tab-counter" hidden={view !== 'counter'}>
        <CounterView
          {data}
          {champByKey}
          {settings}
          active={view === 'counter'}
          ongopool={() => (view = 'pool')}
        />
      </div>
      <div class="view" id="view-pool" role="tabpanel" aria-labelledby="tab-pool" hidden={view !== 'pool'}>
        <PoolEditor
          champions={data.champions}
          {champByKey}
          saved={settings}
          {welcome}
          active={view === 'pool'}
          onsave={save}
          bind:dirty={poolDirty}
        />
      </div>
    {:else if loadError !== null}
      <div class="center">
        <div class="panel boot-error" role="alert">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 3 2 20h20z" />
            <path d="M12 10v4.5M12 17.2v.3" />
          </svg>
          <p class="boot-title">Veri alınamadı. İnternet bağlantını kontrol et.</p>
          <p class="boot-detail">{loadError}</p>
          <button type="button" class="btn btn-primary" onclick={init}>Tekrar dene</button>
        </div>
      </div>
    {:else}
      <div class="center" role="status">
        <div class="boot">
          <span class="boot-text">Şampiyon verileri yükleniyor…</span>
          <span class="bar" aria-hidden="true"><span></span></span>
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    height: 100vh;
    min-width: 0;
  }

  .topbar {
    display: flex;
    align-items: center;
    gap: 28px;
    height: 54px;
    padding: 0 20px;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(1, 10, 19, 0.95), rgba(1, 10, 19, 0.7));
    box-shadow: 0 1px 0 rgba(120, 90, 40, 0.25);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
  }
  .mark {
    width: 20px;
    height: 20px;
    fill: none;
    stroke: var(--gold);
    stroke-width: 1.5;
    stroke-linejoin: round;
  }
  .mark path:last-child {
    fill: var(--gold);
    stroke: none;
    opacity: 0.85;
  }
  .word {
    font-size: 16px;
    font-weight: 600;
    letter-spacing: 0.2em;
    color: var(--gold-mid);
  }
  .word b {
    font-weight: 700;
    background: linear-gradient(180deg, #f0e6d2, #c8aa6e 60%, #a8884a);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }

  .tabs {
    display: flex;
    align-self: stretch;
    gap: 4px;
  }
  .tab {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 14px;
    color: var(--muted);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    transition: color var(--t-fast) var(--ease);
  }
  .tab::after {
    content: '';
    position: absolute;
    left: 10px;
    right: 10px;
    bottom: -1px;
    height: 2px;
    background: var(--gold);
    transform: scaleX(0);
    transition: transform var(--t) var(--ease);
  }
  .tab:hover {
    color: var(--text);
  }
  .tab.on {
    color: var(--gold-light);
  }
  .tab.on::after {
    transform: scaleX(1);
  }
  .tab:focus-visible {
    outline-offset: -6px;
  }
  .unsaved {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--gold);
    box-shadow: 0 0 6px rgba(200, 170, 110, 0.7);
  }

  .patch {
    margin-left: auto;
    padding: 3px 10px;
    border: 1px solid var(--gold-dark);
    border-radius: 11px;
    color: var(--gold);
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.06em;
    white-space: nowrap;
  }

  .main {
    min-height: 0;
    min-width: 0;
    padding: 16px 20px;
  }
  @media (max-height: 720px) {
    .main {
      padding: 10px 20px;
    }
  }
  .view {
    height: 100%;
    min-height: 0;
    max-width: 1200px;
    margin: 0 auto;
  }

  .center {
    height: 100%;
    display: grid;
    place-items: center;
  }
  .boot {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
  }
  .boot-text {
    color: var(--muted);
    letter-spacing: 0.04em;
  }
  .bar {
    position: relative;
    width: 220px;
    height: 2px;
    overflow: hidden;
    background: #1a2230;
    border-radius: 1px;
  }
  .bar span {
    position: absolute;
    inset: 0;
    width: 40%;
    background: linear-gradient(90deg, transparent, var(--gold), transparent);
    animation: slide 1.2s var(--ease) infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(250%);
    }
  }

  .boot-error {
    width: min(480px, 100%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 32px 28px;
    text-align: center;
  }
  .boot-error svg {
    width: 36px;
    height: 36px;
    fill: none;
    stroke: var(--hard);
    stroke-width: 1.6;
    stroke-linejoin: round;
    stroke-linecap: round;
  }
  .boot-title {
    font-size: 15px;
    font-weight: 600;
  }
  .boot-detail {
    color: var(--faint);
    font-size: 12px;
    word-break: break-word;
    margin-bottom: 6px;
  }
</style>
