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
