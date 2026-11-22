import type { Champion, Lane, Verdict } from './types';

export const LANES: Lane[] = ['top', 'jungle', 'mid', 'adc', 'support'];

export const LANE_NAMES: Record<Lane, string> = {
  top: 'Üst',
  jungle: 'Orman',
  mid: 'Orta',
  adc: 'Nişancı',
  support: 'Destek'
};

export const VERDICT_NAMES: Record<Verdict, string> = {
  good: 'İyi eşleşme',
  even: 'Dengeli',
  hard: 'Zor eşleşme',
  lowData: 'Az veri'
};

const TR_MAP: Record<string, string> = {
  ı: 'i',
  İ: 'i',
  ş: 's',
  Ş: 's',
  ğ: 'g',
  Ğ: 'g',
  ü: 'u',
  Ü: 'u',
  ö: 'o',
  Ö: 'o',
  ç: 'c',
  Ç: 'c'
};

/** Lowercase, Turkish-insensitive, diacritics and punctuation removed. */
export function normalize(s: string): string {
  return s
    .replace(/[ıİşŞğĞüÜöÖçÇ]/g, (ch) => TR_MAP[ch] ?? ch)
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '');
}

/** Filters champions by name or id; names starting with the query come first. */
export function searchChampions(champions: Champion[], query: string): Champion[] {
  const q = normalize(query);
  if (!q) return champions;
  const starts: Champion[] = [];
  const contains: Champion[] = [];
  for (const c of champions) {
    const n = normalize(c.name);
    const id = normalize(c.id);
    if (n.startsWith(q) || id.startsWith(q)) starts.push(c);
    else if (n.includes(q) || id.includes(q)) contains.push(c);
  }
  return starts.concat(contains);
}

const pct = new Intl.NumberFormat('tr-TR', {
  style: 'percent',
  minimumFractionDigits: 1,
  maximumFractionDigits: 1
});
const int = new Intl.NumberFormat('tr-TR');
const dec = new Intl.NumberFormat('tr-TR', { minimumFractionDigits: 1, maximumFractionDigits: 1 });

/** 0.537 -> "%53,7" */
export const formatPercent = (v: number) => pct.format(v);
/** 2131 -> "2.131" */
export const formatCount = (v: number) => int.format(v);
/** 54.81 -> "54,8" */
export const formatDecimal = (v: number) => dec.format(v);
