<script lang="ts">
  import { errorText, getAutostart, setAutostart } from './api';
  import { tooltip } from './tooltip';
  import type { AutostartState } from './types';

  // Header gear button + small "Ayarlar" panel. Autostart state is read from Rust
  // each time the panel opens (the user can also change it in Task Manager).

  let open = $state(false);
  let autostart = $state<AutostartState | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let root = $state<HTMLDivElement>();
  let gear = $state<HTMLButtonElement>();

  async function refresh() {
    try {
      autostart = await getAutostart();
    } catch (e) {
      error = errorText(e);
    }
  }

  function toggle() {
    open = !open;
    if (open) {
      error = null;
      refresh();
    }
  }

  function close(returnFocus: boolean) {
    open = false;
    if (returnFocus) gear?.focus();
  }

  async function flip() {
    if (!autostart?.available || busy) return;
    busy = true;
    error = null;
    try {
      autostart = await setAutostart(!autostart.enabled);
    } catch (e) {
      error = errorText(e);
    } finally {
      busy = false;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (open && e.key === 'Escape') {
      e.preventDefault();
      close(true);
    }
  }

  function onpointerdown(e: PointerEvent) {
    if (open && root && !root.contains(e.target as Node)) close(false);
  }

  function onfocusout(e: FocusEvent) {
    const next = e.relatedTarget as Node | null;
    if (open && next && root && !root.contains(next)) close(false);
  }

  const enabled = $derived(autostart?.enabled ?? false);
  const available = $derived(autostart?.available ?? false);
</script>

<svelte:window {onkeydown} {onpointerdown} />

<div class="settings" bind:this={root} {onfocusout}>
  <button
    type="button"
    class="gear"
    class:on={open}
    bind:this={gear}
    aria-label="Ayarlar"
    aria-expanded={open}
    aria-controls="settings-panel"
    use:tooltip={open ? null : 'Ayarlar'}
    onclick={toggle}
  >
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
      />
      <circle cx="12" cy="12" r="3" />
    </svg>
  </button>

  {#if open}
    <div class="panel pop" id="settings-panel" role="dialog" aria-labelledby="settings-title">
      <h2 class="section-title" id="settings-title">Ayarlar</h2>

      <div class="row" class:disabled={autostart !== null && !available}>
        <div class="text">
          <span class="label" id="autostart-label">
            Windows açılınca başlat <span class="muted">(tepside gizli)</span>
          </span>
          <span class="hint" id="autostart-hint">
            {#if autostart === null}
              Durum okunuyor…
            {:else if !available}
              Geliştirme modunda kapalı
            {:else}
              Pencereyi kapatınca tepside çalışır; tamamen kapatmak için tepsi simgesi → Çık.
            {/if}
          </span>
        </div>
        <button
          type="button"
          role="switch"
          class="switch"
          class:checked={enabled}
          aria-checked={enabled}
          aria-labelledby="autostart-label"
          aria-describedby="autostart-hint"
          disabled={!available}
          aria-busy={busy}
          onclick={flip}
        >
          <span class="knob"></span>
        </button>
      </div>

      {#if error}
        <p class="err" role="alert">{error}</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .settings {
    position: relative;
    flex: none;
    margin-left: auto;
  }
  /* Next to the patch badge (which already pushes right) keep a small gap. */
  :global(.patch) + .settings {
    margin-left: -14px;
  }

  .gear {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border: 1px solid var(--gold-dark);
    border-radius: 2px;
    color: var(--gold);
    background: linear-gradient(180deg, rgba(30, 35, 40, 0.6), rgba(20, 26, 34, 0.6));
    transition:
      border-color var(--t-fast) var(--ease),
      color var(--t-fast) var(--ease),
      box-shadow var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .gear:hover {
    border-color: var(--gold);
    color: var(--gold-light);
    box-shadow: 0 0 12px rgba(200, 170, 110, 0.18);
  }
  .gear.on {
    border-color: var(--gold);
    color: var(--gold-light);
    background: var(--panel-3);
  }
  .gear svg {
    width: 17px;
    height: 17px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linejoin: round;
    transition: transform var(--t) var(--ease);
  }
  .gear.on svg {
    transform: rotate(60deg);
  }

  .pop {
    position: absolute;
    top: calc(100% + 11px);
    right: 0;
    z-index: 50;
    width: 320px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 16px 16px;
    border-color: var(--gold-dark);
    box-shadow:
      inset 0 1px 0 rgba(200, 170, 110, 0.08),
      0 14px 32px rgba(0, 0, 0, 0.6);
    animation: pop var(--t) var(--ease);
  }
  /* small gold notch pointing at the gear */
  .pop::before {
    content: '';
    position: absolute;
    top: -6px;
    right: 11px;
    width: 10px;
    height: 10px;
    background: var(--panel-2);
    border-left: 1px solid var(--gold-dark);
    border-top: 1px solid var(--gold-dark);
    transform: rotate(45deg);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }

  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }
  .label .muted {
    font-weight: 400;
  }
  .hint {
    font-size: 11.5px;
    line-height: 1.4;
    color: var(--muted);
  }
  .row.disabled .label {
    color: var(--muted);
  }
  .row.disabled .hint {
    color: var(--gold);
  }

  .switch {
    position: relative;
    flex: none;
    margin-left: auto;
    width: 40px;
    height: 22px;
    border: 1px solid var(--gold-mid);
    border-radius: 11px;
    background: #0b1320;
    transition:
      border-color var(--t-fast) var(--ease),
      background var(--t-fast) var(--ease),
      box-shadow var(--t-fast) var(--ease);
  }
  .switch:hover:not(:disabled) {
    border-color: var(--gold);
    box-shadow: 0 0 10px rgba(200, 170, 110, 0.18);
  }
  .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--gold-mid);
    transition:
      transform var(--t) var(--ease),
      background var(--t-fast) var(--ease);
  }
  .switch.checked {
    border-color: rgba(10, 200, 185, 0.7);
    background: var(--teal-soft);
  }
  .switch.checked:hover:not(:disabled) {
    border-color: var(--teal);
    box-shadow: 0 0 10px rgba(10, 200, 185, 0.28);
  }
  .switch.checked .knob {
    transform: translateX(18px);
    background: var(--teal);
  }
  .switch:disabled {
    border-color: #2e3238;
    background: #12161c;
  }
  .switch:disabled .knob {
    background: var(--faint);
  }

  .err {
    color: var(--hard);
    font-size: 12px;
    word-break: break-word;
  }
</style>
