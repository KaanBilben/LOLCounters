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
