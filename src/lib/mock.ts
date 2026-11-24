// DEV-ONLY mock backend, used by api.ts when the page runs outside Tauri
// (plain browser on `npm run dev`). Static data is real Data Dragon tr_TR data;
// recommendations are deterministic synthetic numbers. Never shipped: api.ts
// only imports this module when import.meta.env.DEV is true.
//
// Preview the error state with: localStorage.setItem('lolcounters.mockOffline', '1')
//
// Champion select (compact window, http://localhost:1420/compact): pick the state with
//   localStorage.setItem('lolcounters.mockChampSelect',
//     'none' | 'waiting' | 'banning' | 'banningNoIntent' | 'picking' | 'locked' | 'blind')
// and reload (default 'picking'). The pre-lock list ranks your Havuzum top-lane pool.
// 'banning' (LC-011) shows the ban suggestions for the hovered champion (first top-lane pool
// champion); 'banningNoIntent' is the same phase before a champion is hovered.

import type {
  AutostartState,
  BanSuggestion,
  Build,
  Champion,
  Item,
  Lane,
  Recommendation,
  Roles,
  Rune,
  RuneOptions,
  RunePage,
  Settings,
  Spell,
  StaticPayload,
  Verdict
} from './types';

const DD = 'https://ddragon.leagueoflegends.com';
const SETTINGS_KEY = 'lolcounters.mockSettings';
const OFFLINE_KEY = 'lolcounters.mockOffline';

const SHARDS: Record<string, string> = {
  '5008': 'Uyarlanabilir Güç',
  '5005': 'Saldırı Hızı',
  '5007': 'Yetenek Hızı',
  '5010': 'Hareket Hızı',
  '5001': 'Artan Can',
  '5011': 'Can',
  '5013': 'Direnç ve Yavaşlatma Direnci',
  '5002': 'Zırh',
  '5003': 'Büyü Direnci'
};

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

async function getJson<T>(url: string): Promise<T> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${url}`);
  return (await res.json()) as T;
}

interface DdImage {
  full: string;
}
interface DdChampion {
  key: string;
  id: string;
  name: string;
  tags: string[];
  image: DdImage;
}
interface DdItem {
  name: string;
  image: DdImage;
}
interface DdSpell {
  key: string;
  name: string;
  image: DdImage;
}
interface DdRune {
  id: number;
  name: string;
  icon: string;
}
interface DdTree extends DdRune {
  slots: { runes: DdRune[] }[];
}

let staticCache: StaticPayload | null = null;

export async function loadStatic(): Promise<StaticPayload> {
  if (staticCache) return staticCache;
  await delay(500);
  const versions = await getJson<string[]>(`${DD}/api/versions.json`);
  const version = versions[0];
  const base = `${DD}/cdn/${version}`;
  const [champJson, itemJson, runeJson, spellJson] = await Promise.all([
    getJson<{ data: Record<string, DdChampion> }>(`${base}/data/tr_TR/champion.json`),
    getJson<{ data: Record<string, DdItem> }>(`${base}/data/tr_TR/item.json`),
    getJson<DdTree[]>(`${base}/data/tr_TR/runesReforged.json`),
    getJson<{ data: Record<string, DdSpell> }>(`${base}/data/tr_TR/summoner.json`)
  ]);

  const champions: Champion[] = Object.values(champJson.data)
    .map((c) => ({
      key: Number(c.key),
      id: c.id,
      name: c.name,
      icon: `${base}/img/champion/${c.image.full}`,
      tags: c.tags
    }))
    .sort((a, b) => a.name.localeCompare(b.name, 'tr'));

  const items: Record<string, Item> = {};
  for (const [id, it] of Object.entries(itemJson.data)) {
    items[id] = { id: Number(id), name: it.name, icon: `${base}/img/item/${it.image.full}` };
  }

  const runes: Record<string, Rune> = {};
  const trees: Record<string, Rune> = {};
  for (const tree of runeJson) {
    trees[tree.id] = {
      id: tree.id,
      name: tree.name,
      icon: `${DD}/cdn/img/${tree.icon}`,
      tree: tree.id,
      slot: 0
    };
    tree.slots.forEach((slot, i) => {
      for (const r of slot.runes) {
        runes[r.id] = { id: r.id, name: r.name, icon: `${DD}/cdn/img/${r.icon}`, tree: tree.id, slot: i };
      }
    });
  }

  const spells: Record<string, Spell> = {};
  for (const s of Object.values(spellJson.data)) {
    spells[s.key] = { key: Number(s.key), name: s.name, icon: `${base}/img/spell/${s.image.full}` };
  }

  staticCache = {
    version,
    uggPatch: version.split('.').slice(0, 2).join('.'),
    champions,
    items,
    runes,
    trees,
    spells,
    shards: { ...SHARDS }
  };
  return staticCache;
}

/** The mock always loads the latest version, so it never sends `static-updated`. */
export async function onStaticUpdated(_cb: () => void): Promise<() => void> {
  return () => {};
}

export async function getSettings(): Promise<Settings> {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Settings;
      if (parsed && typeof parsed.pools === 'object') return parsed;
    }
  } catch {
    /* fall through to default */
  }
  return { pools: {} };
}

export async function saveSettings(settings: Settings): Promise<void> {
  await delay(250);
  // Preview the autosave error with: localStorage.setItem('lolcounters.mockSaveFail', '1')
  let fail = false;
  try {
    fail = localStorage.getItem('lolcounters.mockSaveFail') === '1';
  } catch {
    /* preview only */
  }
  if (fail) throw 'Erişim engellendi (sahte hata)';
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    /* preview only */
  }
}

// --- synthetic recommendations (mirrors src-tauri/src/scoring.rs) ----------

const PRIOR_GAMES = 100;
const PREF_MAX_BONUS = 3;
const LOW_DATA_MATCHES = 30;
const GOOD = 0.52;
const HARD = 0.48;

function hash(n: number): number {
  let x = (n ^ 0x5bd1e995) >>> 0;
  x = Math.imul(x ^ (x >>> 15), 0x2c1b3c6d) >>> 0;
  x = Math.imul(x ^ (x >>> 12), 0x297a2d39) >>> 0;
  return (x ^ (x >>> 15)) >>> 0;
}

const MOCK_BUILD: Build = {
  runes: {
    primaryTree: 8000,
    secondaryTree: 8200,
    // u.gg sorts perks numerically, not by tree
    perks: [8010, 8234, 8237, 8299, 9104, 9111],
    shards: [5005, 5008, 5011],
    matches: 18234,
    wins: 9540
  },
  spells: [4, 14],
  startingItems: [1055, 2003],
  coreItems: [3142, 3047, 6333],
  skillMaxOrder: 'QEW',
  lateOptions: [[3053, 3742], [3742, 3053], [3053]],
  matches: 9120,
  wins: 4870
};

export async function recommend(lane: Lane, enemy: number): Promise<Recommendation[]> {
  await delay(500);
  let offline = false;
  try {
    offline = localStorage.getItem(OFFLINE_KEY) === '1';
  } catch {
    /* storage blocked: treat as online */
  }
  if (offline) throw 'mock: network unreachable';
  // Preview a partial fetch failure with: localStorage.setItem('lolcounters.mockPartial', '1')
  let partial = false;
  try {
    partial = localStorage.getItem('lolcounters.mockPartial') === '1';
  } catch {
    /* storage blocked */
  }
  const pool = (await getSettings()).pools[lane] ?? [];
  const n = pool.length;
  const recs: Recommendation[] = pool.map((champion, i) => {
    if (partial && i === 1) {
      return {
        champion, poolIndex: i, wins: 0, matches: 0, winrate: null, adjusted: 0.5,
        score: 50 + PREF_MAX_BONUS * (1 - i / n), verdict: 'lowData' as Verdict, build: null,
        error: 'mock: zaman aşımı'
      };
    }
    const h = hash(champion * 7919 + enemy * 31 + lane.length);
    // the last champion of a pool with 3+ entries gets very few games
    const matches = n >= 3 && i === n - 1 ? 12 + (h % 15) : 250 + (h % 3200);
    const rate = 0.43 + ((h >>> 8) % 1400) / 10000; // 43% .. 57%
    const wins = Math.round(matches * rate);
    const adjusted = (wins + PRIOR_GAMES / 2) / (matches + PRIOR_GAMES);
    const score = adjusted * 100 + PREF_MAX_BONUS * (1 - i / n);
    let verdict: Verdict;
    if (matches < LOW_DATA_MATCHES) verdict = 'lowData';
    else if (adjusted >= GOOD) verdict = 'good';
    else if (adjusted <= HARD) verdict = 'hard';
    else verdict = 'even';
    return {
      champion,
      poolIndex: i,
      wins,
      matches,
      winrate: matches > 0 ? wins / matches : null,
      adjusted,
      score,
      verdict,
      build: verdict === 'lowData' ? null : MOCK_BUILD,
      error: null
    };
  });
  return recs.sort((a, b) => b.score - a.score || a.poolIndex - b.poolIndex);
}

// --- synthetic lane data (get_roles) ----------------------------------------
// Derived from Data Dragon class tags, close enough for a preview. The last
// champion gets all lanes, like a brand-new release without u.gg data.
// Preview a failed load with: localStorage.setItem('lolcounters.mockNoRoles', '1')

const ALL_LANES: Lane[] = ['top', 'jungle', 'mid', 'adc', 'support'];
const TAG_LANES: Record<string, Lane[]> = {
  Marksman: ['adc'],
  Support: ['support'],
  Mage: ['mid'],
  Assassin: ['mid'],
  Fighter: ['top'],
  Tank: ['top', 'support']
};

export async function getRoles(): Promise<Roles> {
  const { champions } = await loadStatic();
  await delay(300);
  let failed = false;
  try {
    failed = localStorage.getItem('lolcounters.mockNoRoles') === '1';
  } catch {
    /* storage blocked */
  }
  if (failed) throw 'mock: lane verisi alınamadı';
  const roles: Roles = {};
  champions.forEach((c, i) => {
    const lanes = new Set<Lane>(c.tags.slice(0, 2).flatMap((t) => TAG_LANES[t] ?? []));
    if (['Fighter', 'Assassin', 'Tank'].includes(c.tags[0]) && hash(c.key) % 3 === 0) lanes.add('jungle');
    const isNew = i === champions.length - 1;
    roles[c.key] = isNew || lanes.size === 0 ? [...ALL_LANES] : [...lanes];
  });
  return roles;
}

// --- autostart (src-tauri/src/launch.rs) ------------------------------------
// Preview the debug-build (disabled) state with: localStorage.setItem('lolcounters.mockDevBuild', '1')

const AUTOSTART_KEY = 'lolcounters.mockAutostart';

function mockFlag(key: string): boolean {
  try {
    return localStorage.getItem(key) === '1';
  } catch {
    return false;
  }
}

export async function getAutostart(): Promise<AutostartState> {
  const available = !mockFlag('lolcounters.mockDevBuild');
  return { enabled: available && mockFlag(AUTOSTART_KEY), available };
}

export async function setAutostart(enabled: boolean): Promise<AutostartState> {
  await delay(250);
  if (mockFlag('lolcounters.mockDevBuild')) throw 'Geliştirme modunda kapalı';
  try {
    localStorage.setItem(AUTOSTART_KEY, enabled ? '1' : '0');
  } catch {
    /* preview only */
  }
  return getAutostart();
}

// --- LC-008: League client champion select ------------------------------------
// No real client: a fake state picked with localStorage 'lolcounters.mockChampSelect'
// (see the top of this file).

import type { ChampSelectState } from './types';

const MOCK_MODES = ['none', 'waiting', 'banning', 'banningNoIntent', 'picking', 'locked', 'blind'] as const;
type MockChampSelect = (typeof MOCK_MODES)[number];
const CHAMP_SELECT_KEY = 'lolcounters.mockChampSelect';
// Zed and Jax banned; a teammate hovers Yasuo
const MOCK_BANS = [238, 24];
const MOCK_ALLIES = [157];
// Riven, Lee Sin, Ahri, Jinx, Thresh
const MOCK_ENEMIES = [92, 64, 103, 222, 412];

/** undefined: no manual choice (use the guess). */
let mockOpponent: number | undefined;
/** Lane picked by hand (blind pick has no assigned role). */
let mockLane: Lane | null = null;
/** The mock guess: the enemy in MOCK_ENEMIES order top, jungle, mid, adc, support. */
const MOCK_LANE_ORDER: Lane[] = ['top', 'jungle', 'mid', 'adc', 'support'];
const champSelectListeners = new Set<(s: ChampSelectState) => void>();

function mockChampSelectMode(): MockChampSelect {
  try {
    const v = localStorage.getItem(CHAMP_SELECT_KEY);
    const mode = MOCK_MODES.find((m) => m === v);
    if (mode) return mode;
  } catch {
    /* storage blocked */
  }
  return 'picking';
}

async function mockChampSelectState(): Promise<ChampSelectState> {
  const mode = mockChampSelectMode();
  const base: ChampSelectState = {
    connected: mode !== 'none',
    active: mode !== 'none',
    myLane: mode === 'none' || mode === 'blind' ? null : 'top',
    myChampion: null,
    myIntent: null,
    locked: false,
    enemies: [],
    banPhase: false,
    bans: [],
    allies: [],
    opponent: null,
    opponentChosen: false
  };
  if (mode === 'none' || mode === 'waiting') return base;
  const pool = (await getSettings()).pools.top ?? [];
  if (mode === 'banning' || mode === 'banningNoIntent') {
    return {
      ...base,
      myIntent: mode === 'banning' ? (pool[0] ?? 86) : null,
      banPhase: true,
      bans: MOCK_BANS,
      allies: MOCK_ALLIES
    };
  }
  const locked = mode === 'locked';
  const enemies = locked ? MOCK_ENEMIES : MOCK_ENEMIES.slice(0, 3);
  // the guess needs my lane; in blind pick only the hand-picked one (setMyLane)
  const lane = base.myLane ?? mockLane;
  const laneEnemy = lane ? MOCK_ENEMIES[MOCK_LANE_ORDER.indexOf(lane)] : undefined;
  const guess = laneEnemy !== undefined && enemies.includes(laneEnemy) ? laneEnemy : null;
  const opponent = mockOpponent !== undefined && enemies.includes(mockOpponent) ? mockOpponent : guess;
  return {
    ...base,
    myChampion: locked ? 122 : null,
    myIntent: locked ? null : (pool[1] ?? pool[0] ?? 86),
    locked,
    enemies,
    bans: MOCK_BANS,
    allies: MOCK_ALLIES,
    opponent,
    opponentChosen: opponent !== null && opponent === mockOpponent
  };
}

export async function getChampSelectState(): Promise<ChampSelectState> {
  return mockChampSelectState();
}

export async function setOpponent(championKey: number | null): Promise<ChampSelectState> {
  mockOpponent = championKey ?? undefined;
  const s = await mockChampSelectState();
  for (const cb of champSelectListeners) cb(s);
  return s;
}

export async function setMyLane(lane: Lane | null): Promise<ChampSelectState> {
  mockLane = lane;
  const s = await mockChampSelectState();
  for (const cb of champSelectListeners) cb(s);
  return s;
}

export function onChampSelect(cb: (s: ChampSelectState) => void): () => void {
  champSelectListeners.add(cb);
  return () => champSelectListeners.delete(cb);
}

// --- LC-009: compact window ---------------------------------------------------

/** One champion vs one enemy with the general build (same synthetic numbers as recommend). */
export async function championMatchup(
  lane: Lane,
  champion: number,
  enemy: number
): Promise<Recommendation> {
  await delay(400);
  let offline = false;
  try {
    offline = localStorage.getItem(OFFLINE_KEY) === '1';
  } catch {
    /* storage blocked: treat as online */
  }
  if (offline) throw 'mock: network unreachable';
  const h = hash(champion * 7919 + enemy * 31 + lane.length);
  const matches = enemy === 0 ? 0 : 250 + (h % 3200);
  const rate = 0.43 + ((h >>> 8) % 1400) / 10000;
  const wins = Math.round(matches * rate);
  const adjusted = (wins + PRIOR_GAMES / 2) / (matches + PRIOR_GAMES);
  let verdict: Verdict;
  if (matches < LOW_DATA_MATCHES) verdict = 'lowData';
  else if (adjusted >= GOOD) verdict = 'good';
  else if (adjusted <= HARD) verdict = 'hard';
  else verdict = 'even';
  return {
    champion,
    poolIndex: 0,
    wins,
    matches,
    winrate: matches > 0 ? wins / matches : null,
    adjusted,
    score: adjusted * 100 + PREF_MAX_BONUS,
    verdict,
    build: MOCK_BUILD,
    error: null
  };
}

// --- LC-011: ban suggestions (mirrors scoring::worst_matchups) ---------------

/** Synthetic matchups of `champion` against fighters/tanks; the worst 3 with 30+ games. */
export async function banSuggestions(
  lane: Lane,
  champion: number,
  exclude: number[]
): Promise<BanSuggestion[]> {
  await delay(400);
  if (mockFlag(OFFLINE_KEY)) throw 'mock: network unreachable';
  const { champions } = await loadStatic();
  const skip = new Set([...exclude, champion]);
  return champions
    .filter((c) => ['Fighter', 'Tank'].includes(c.tags[0]) && !skip.has(c.key))
    .map((c): BanSuggestion => {
      const h = hash(champion * 7919 + c.key * 31 + lane.length);
      const matches = 20 + (h % 2400);
      const wins = Math.round(matches * (0.42 + ((h >>> 8) % 1600) / 10000)); // 42% .. 58%
      const adjusted = (wins + PRIOR_GAMES / 2) / (matches + PRIOR_GAMES);
      let verdict: Verdict;
      if (adjusted >= GOOD) verdict = 'good';
      else if (adjusted <= HARD) verdict = 'hard';
      else verdict = 'even';
      return {
        champion: c.key,
        poolIndex: 0,
        wins,
        matches,
        winrate: wins / matches,
        adjusted,
        score: adjusted * 100 + PREF_MAX_BONUS,
        verdict
      };
    })
    .filter((s) => s.matches >= LOW_DATA_MATCHES)
    .sort((a, b) => a.adjusted - b.adjusted || b.matches - a.matches)
    .slice(0, 3);
}

/** The preview windows share localStorage: a save in another window fires `storage` here. */
export async function onPoolsChanged(cb: () => void): Promise<() => void> {
  const onStorage = (e: StorageEvent) => {
    if (e.key === SETTINGS_KEY) cb();
  };
  window.addEventListener('storage', onStorage);
  return () => window.removeEventListener('storage', onStorage);
}

export async function openMainWindow(): Promise<void> {
  /* the browser preview has no main window to raise */
}

export async function closeCompact(): Promise<void> {
  window.close(); // works for the preview popup opened by openCompactPreview
}

export async function openCompactPreview(): Promise<void> {
  window.open('/compact', 'lolcounters-compact', 'popup,width=380,height=260');
}

// --- LC-013: rune options (mirrors scoring::pick_rune_options) ---------------
// Precision/Resolve, Resolve/Precision, Precision/Inspiration; vs an enemy the second
// option is the matchup page (gold), except Ahri (103): no highlight.

const MOCK_RUNE_OPTIONS: RunePage[] = [
  { primaryTree: 8000, secondaryTree: 8400, perks: [8010, 8242, 8299, 8473, 9104, 9111], shards: [5005, 5008, 5001], matches: 21095, wins: 10741 },
  { primaryTree: 8400, secondaryTree: 8000, perks: [8437, 8444, 8446, 8451, 9104, 9111], shards: [5005, 5008, 5001], matches: 16408, wins: 8513 },
  { primaryTree: 8000, secondaryTree: 8300, perks: [8010, 8299, 8345, 8410, 9104, 9111], shards: [5005, 5008, 5001], matches: 9472, wins: 4717 }
];

export async function runeOptions(_lane: Lane, _champion: number, enemy: number | null): Promise<RuneOptions> {
  await delay(400);
  if (mockFlag(OFFLINE_KEY)) throw 'mock: network unreachable';
  const gold = enemy !== null && enemy !== 103 ? 1 : null;
  return { options: MOCK_RUNE_OPTIONS, gold, spells: gold === null ? [4, 14] : [4, 12] };
}
