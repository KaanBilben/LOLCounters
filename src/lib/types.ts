// Mirror of the Rust structs sent over Tauri IPC (serde camelCase).
// Keep in sync with src-tauri/src/{settings,ddragon,ugg,scoring,lib}.rs.

export type Lane = 'top' | 'jungle' | 'mid' | 'adc' | 'support';

export interface Champion {
  key: number;
  id: string;
  name: string;
  icon: string;
  tags: string[];
}

export interface Rune {
  id: number;
  name: string;
  icon: string;
  tree: number;
  slot: number;
}

export interface Item {
  id: number;
  name: string;
  icon: string;
}

export interface Spell {
  key: number;
  name: string;
  icon: string;
}

export interface StaticPayload {
  version: string;
  uggPatch: string;
  champions: Champion[];
  items: Record<string, Item>;
  runes: Record<string, Rune>;
  trees: Record<string, Rune>;
  spells: Record<string, Spell>;
  shards: Record<string, string>;
}

export interface Settings {
  pools: Partial<Record<Lane, number[]>>;
  /** Windows açılınca başlatma tercihi; yalnızca Rust yönetir (`null` = hiç sorulmadı). */
  autostart?: boolean | null;
}

export interface RunePage {
  primaryTree: number;
  secondaryTree: number;
  perks: number[];
  shards: number[];
  matches: number;
  wins: number;
}

export interface Build {
  runes: RunePage;
  spells: number[];
  startingItems: number[];
  coreItems: number[];
  skillMaxOrder: string;
  lateOptions: number[][];
  matches: number;
  wins: number;
}

export type Verdict = 'good' | 'even' | 'hard' | 'lowData';

/** src-tauri/src/scoring.rs `Scored`. */
export interface Scored {
  champion: number;
  poolIndex: number;
  wins: number;
  matches: number;
  winrate: number | null;
  adjusted: number;
  score: number;
  verdict: Verdict;
}

/**
 * LC-011 ban_suggestions: `champion` is the counter (the enemy), the numbers are my
 * champion's result against it; worst first.
 */
export type BanSuggestion = Scored;

export interface Recommendation extends Scored {
  build: Build | null;
  /** Set when this champion's data could not be fetched (network error), so 0/0 is not real "low data". */
  error: string | null;
}

/**
 * get_roles: champion key -> lanes it is played in (lane share >= ROLE_MIN_SHARE in ugg.rs).
 * Every Data Dragon champion is listed; one without u.gg data has all five lanes.
 */
export type Roles = Partial<Record<number, Lane[]>>;

/** Windows açılınca başlatma durumu (src-tauri/src/launch.rs `AutostartState`). */
export interface AutostartState {
  enabled: boolean;
  /** false: geliştirme derlemesi, anahtar kapalı gösterilir. */
  available: boolean;
}

/** LC-008: champion select read from the League client (src-tauri/src/lcu.rs ChampSelectPayload). */
export interface ChampSelectState {
  /** Connected to the League client. */
  connected: boolean;
  /** Champion select is running. */
  active: boolean;
  /** Assigned lane (draft/ranked); null in blind pick. */
  myLane: Lane | null;
  /** Locked-in champion. */
  myChampion: number | null;
  /** Hovered / intended champion before lock-in. */
  myIntent: number | null;
  locked: boolean;
  /** Revealed enemy champions, in order. */
  enemies: number[];
  /** LC-011: my ban action exists and is not done (planning / ban phase), not locked. */
  banPhase: boolean;
  /** Champions in completed bans (both teams). */
  bans: number[];
  /** Champions my teammates picked or hovered. */
  allies: number[];
  /** LC-016: my bot lane partner's champion (support if I am ADC, ADC if I am support), hovered
   * or locked; null on other lanes and in blind pick. */
  duoPartner: number | null;
  /** The partner locked the champion in. */
  duoPartnerLocked: boolean;
  /** My lane opponent: the user's choice, else the guess; null if unknown. */
  opponent: number | null;
  /** True when `opponent` was chosen by the user (not guessed). */
  opponentChosen: boolean;
}

/**
 * LC-013 rune_options (src-tauri/src/lib.rs RuneOptions): the rune pages offered after
 * lock-in, most played first; `gold` is the option equal to u.gg's page for this matchup.
 */
export interface RuneOptions {
  options: RunePage[];
  gold: number | null;
  /** LC-014 "Önerilen": highest win rate of the non-matchup options with enough games. */
  best: number | null;
  /** The matchup's spells when `gold` is set, else the lane's general spells. */
  spells: number[];
  /** A source failed and the list may be incomplete (one page, no shards): offer a retry. */
  error: string | null;
}

/**
 * LC-015 import_loadout (src-tauri/src/lcu.rs ImportOutcome): `done` — the page was written to
 * the client's "LOLCounters" page and selected (`spells`: the spells were set too);
 * `needsConfirm` — no free page slot, ask before overwriting page `page` (send `id` back).
 */
export type ImportOutcome = { kind: 'done'; spells: boolean } | { kind: 'needsConfirm'; id: number; page: string };

/** LC-016 (src-tauri/src/scoring.rs Synergy): one champion played with my bot lane partner. */
export interface Synergy {
  champion: number;
  /** Games and wins together with the partner. */
  wins: number;
  matches: number;
  /** Percentage points: the pair's adjusted win rate minus the mean of both solo win rates. */
  synergy: number;
  /** synergy >= GOOD_SYNERGY (1.5). */
  good: boolean;
  /** Also a good matchup against my lane opponent. */
  counter: boolean;
}

/** LC-016 duo_synergy (src-tauri/src/lib.rs DuoSynergy): `all` every pair with 200+ games,
 * best first; `best` the Sinerji tab (top 5 positive, bans / taken skipped, counters first). */
export interface DuoSynergy {
  all: Synergy[];
  best: Synergy[];
  /** The opponent's matchups could not be loaded: counter marks may be missing; offer a retry. */
  error: string | null;
}
