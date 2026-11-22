// DEV-ONLY mock backend, used by api.ts when the page runs outside Tauri
// (plain browser on `npm run dev`). Static data is real Data Dragon tr_TR data;
// recommendations are deterministic synthetic numbers. Never shipped: api.ts
// only imports this module when import.meta.env.DEV is true.
//
// Preview the error state with: localStorage.setItem('lolcounters.mockOffline', '1')

import type {
  Build,
  Champion,
  Item,
  Lane,
  Recommendation,
  Roles,
  Rune,
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
