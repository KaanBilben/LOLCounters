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

export interface Recommendation {
  champion: number;
  poolIndex: number;
  wins: number;
  matches: number;
  winrate: number | null;
  adjusted: number;
  score: number;
  verdict: Verdict;
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
  /** My lane opponent: the user's choice, else the guess; null if unknown. */
  opponent: number | null;
  /** True when `opponent` was chosen by the user (not guessed). */
  opponentChosen: boolean;
}
