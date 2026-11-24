import { invoke } from '@tauri-apps/api/core';
import type {
  AutostartState,
  Lane,
  Recommendation,
  Roles,
  Settings,
  StaticPayload
} from './types';

// Outside Tauri (plain browser during `npm run dev`) a mock backend is used so
// the UI can be previewed. The check on import.meta.env.DEV lets the bundler
// drop the mock from production builds.
const useMock =
  import.meta.env.DEV && typeof window !== 'undefined' && !('__TAURI_INTERNALS__' in window);

const mock = () => import('./mock');

export async function loadStatic(): Promise<StaticPayload> {
  if (useMock) return (await mock()).loadStatic();
  return invoke<StaticPayload>('load_static');
}

/** Listens to `static-updated`: the backend reloaded data that was not final (old version /
 * patch, e.g. no network at Windows startup) and the version or patch changed. Returns unlisten. */
export async function onStaticUpdated(cb: () => void): Promise<() => void> {
  if (useMock) return (await mock()).onStaticUpdated(cb);
  const { listen } = await import('@tauri-apps/api/event');
  return listen('static-updated', () => cb());
}

export async function getSettings(): Promise<Settings> {
  if (useMock) return (await mock()).getSettings();
  return invoke<Settings>('get_settings');
}

export async function saveSettings(settings: Settings): Promise<void> {
  if (useMock) return (await mock()).saveSettings(settings);
  return invoke<void>('save_settings', { settings });
}

export async function recommend(lane: Lane, enemy: number): Promise<Recommendation[]> {
  if (useMock) return (await mock()).recommend(lane, enemy);
  return invoke<Recommendation[]>('recommend', { lane, enemy });
}

/** Turns an unknown error (Tauri returns plain strings) into text. */
export function errorText(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

export async function getRoles(): Promise<Roles> {
  if (useMock) return (await mock()).getRoles();
  return invoke<Roles>('get_roles');
}

export async function getAutostart(): Promise<AutostartState> {
  if (useMock) return (await mock()).getAutostart();
  return invoke<AutostartState>('get_autostart');
}

export async function setAutostart(enabled: boolean): Promise<AutostartState> {
  if (useMock) return (await mock()).setAutostart(enabled);
  return invoke<AutostartState>('set_autostart', { enabled });
}

// --- LC-008: League client champion select ------------------------------------

import type { ChampSelectState } from './types';

export async function getChampSelectState(): Promise<ChampSelectState> {
  if (useMock) return (await mock()).getChampSelectState();
  return invoke<ChampSelectState>('get_champ_select_state');
}

/** Sets my lane opponent by hand (null: back to the guess). Cleared when champ select ends. */
export async function setOpponent(championKey: number | null): Promise<ChampSelectState> {
  if (useMock) return (await mock()).setOpponent(championKey);
  return invoke<ChampSelectState>('set_opponent', { championKey });
}

/** Sets my lane by hand when no role is assigned (used for the opponent guess); null: undo.
 * Cleared when champ select ends. */
export async function setMyLane(lane: Lane | null): Promise<ChampSelectState> {
  if (useMock) return (await mock()).setMyLane(lane);
  return invoke<ChampSelectState>('set_my_lane', { lane });
}

/** Listens to the `champ-select` event (sent only when the state changes). Returns unlisten. */
export async function onChampSelect(cb: (s: ChampSelectState) => void): Promise<() => void> {
  if (useMock) return (await mock()).onChampSelect(cb);
  const { listen } = await import('@tauri-apps/api/event');
  return listen<ChampSelectState>('champ-select', (e) => cb(e.payload));
}

// --- LC-009: compact window ---------------------------------------------------

/** One champion (even outside the pool) vs one enemy, with its general build for the lane. */
export async function championMatchup(lane: Lane, champion: number, enemy: number): Promise<Recommendation> {
  if (useMock) return (await mock()).championMatchup(lane, champion, enemy);
  return invoke<Recommendation>('champion_matchup', { lane, champion, enemy });
}

// --- LC-011: ban suggestions --------------------------------------------------

import type { BanSuggestion } from './types';

/** The worst matchups of `champion` in `lane` (worst first), skipping `exclude` (bans, allies). */
export async function banSuggestions(
  lane: Lane,
  champion: number,
  exclude: number[]
): Promise<BanSuggestion[]> {
  if (useMock) return (await mock()).banSuggestions(lane, champion, exclude);
  return invoke<BanSuggestion[]>('ban_suggestions', { lane, champion, exclude });
}

/** Brings the main window to the front. */
export async function openMainWindow(): Promise<void> {
  if (useMock) return (await mock()).openMainWindow();
  return invoke<void>('open_main_window');
}

/** Listens to `pools-changed` (a pool was saved, e.g. from the main window). Returns unlisten. */
export async function onPoolsChanged(cb: () => void): Promise<() => void> {
  if (useMock) return (await mock()).onPoolsChanged(cb);
  const { listen } = await import('@tauri-apps/api/event');
  return listen('pools-changed', () => cb());
}

/** Closes the compact window (it opens again at the next champion select). */
export async function closeCompact(): Promise<void> {
  if (useMock) return (await mock()).closeCompact();
  return invoke<void>('close_compact');
}

/** Dev only: opens the compact window without the League client. */
export async function openCompactPreview(): Promise<void> {
  if (useMock) return (await mock()).openCompactPreview();
  return invoke<void>('open_compact_preview');
}

// --- LC-013: rune options -----------------------------------------------------

import type { RuneOptions } from './types';

/** Rune page options of `champion` in `lane` (most played first), with the matchup page vs `enemy`. */
export async function runeOptions(lane: Lane, champion: number, enemy: number | null): Promise<RuneOptions> {
  if (useMock) return (await mock()).runeOptions(lane, champion, enemy);
  return invoke<RuneOptions>('rune_options', { lane, champion, enemy });
}

import type { ImportOutcome, RunePage } from './types';

/** Writes `page` (and `spells`, during champ select) to the client; only on the user's click. */
export async function importLoadout(page: RunePage, spells: number[], confirm: number | null): Promise<ImportOutcome> {
  if (useMock) return (await mock()).importLoadout(page, spells, confirm);
  return invoke<ImportOutcome>('import_loadout', { page, spells, confirm });
}

// --- LC-016: bot lane synergy ---------------------------------------------------

import type { DuoSynergy } from './types';

/** Champions for my `lane` that go well with bot lane `partner`; `opponent` marks counters,
 * `exclude` (bans, taken) is left out of `best`. Other lanes get empty lists. */
export async function duoSynergy(
  lane: Lane,
  partner: number,
  opponent: number | null,
  exclude: number[]
): Promise<DuoSynergy> {
  if (useMock) return (await mock()).duoSynergy(lane, partner, opponent, exclude);
  return invoke<DuoSynergy>('duo_synergy', { lane, partner, opponent, exclude });
}
