import { getRoles } from './api';
import type { Champion, Lane, Roles } from './types';

// Lane data is loaded once per session and shared by every view.
let pending: Promise<Roles | null> | null = null;

/** null when the data could not be loaded: then every champion is shown (asked again on next call). */
export function loadRoles(): Promise<Roles | null> {
  pending ??= getRoles().catch(() => {
    pending = null;
    return null;
  });
  return pending;
}

/**
 * Calls `set` once the lane data is loaded. If loading fails, it is asked again when the
 * window is shown or gets focus (the main window lives hidden in the tray for days; no timer).
 * Returns the cleanup (for `$effect`).
 */
export function watchRoles(set: (r: Roles) => void): () => void {
  let stopped = false;
  const retry = () => {
    if (document.visibilityState !== 'hidden') void attempt();
  };
  const stop = () => {
    stopped = true;
    window.removeEventListener('focus', retry);
    document.removeEventListener('visibilitychange', retry);
  };
  async function attempt() {
    const r = await loadRoles();
    if (r && !stopped) {
      stop();
      set(r);
    }
  }
  window.addEventListener('focus', retry);
  document.addEventListener('visibilitychange', retry);
  void attempt();
  return stop;
}

/** Champions played in `lane`; one missing from the data counts as playable everywhere. */
export function inLane(champions: Champion[], roles: Roles, lane: Lane): Champion[] {
  return champions.filter((c) => roles[c.key]?.includes(lane) ?? true);
}
