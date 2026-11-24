//! Öneri puanlaması. Açıklama: docs/scoring.md (sabitler değişirse orası da güncellenir).

use crate::ugg::RunePage;
use serde::Serialize;

/// Düzeltilmiş oran için eklenen sanal maç sayısı (yarısı galibiyet).
pub const PRIOR_GAMES: f64 = 100.0;
/// İlk tercihe eklenen en büyük bonus (yüzde puanı).
pub const PREF_MAX_BONUS: f64 = 3.0;
/// Bu sayıdan az maç "Az veri" sayılır.
pub const LOW_DATA_MATCHES: u32 = 30;
/// Bu düzeltilmiş oran ve üstü "İyi eşleşme".
pub const GOOD: f64 = 0.52;
/// Bu düzeltilmiş oran ve altı "Zor eşleşme".
pub const HARD: f64 = 0.48;
/// Ban önerisinde gösterilen en kötü eşleşme sayısı.
pub const BAN_SUGGESTIONS: usize = 3;
/// Küçük pencerede gösterilen rün seçeneği sayısı (LC-013).
pub const RUNE_OPTIONS: usize = 3;
/// Eşleşmeye özel sayfa bundan az maçlıysa altın vurgu yapılmaz.
pub const MATCHUP_PAGE_MIN_GAMES: u32 = 100;
/// "Önerilen" rün kartı için gereken en az maç (LC-014).
pub const RECOMMENDED_PAGE_MIN_GAMES: u32 = 100;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub champion: i64,
    pub pool_index: usize,
    pub wins: u32,
    pub matches: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    Good,
    Even,
    Hard,
    LowData,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scored {
    pub champion: i64,
    pub pool_index: usize,
    pub wins: u32,
    pub matches: u32,
    /// Ham oran (galibiyet / maç); maç yoksa `None`.
    pub winrate: Option<f64>,
    /// Düzeltilmiş oran, 0..1.
    pub adjusted: f64,
    pub score: f64,
    pub verdict: Verdict,
}

/// Adayları puanlar ve büyükten küçüğe sıralar (eşitlikte tercih sırası).
pub fn rank(c: &[Candidate]) -> Vec<Scored> {
    let n = c.len().max(1) as f64;
    let mut out: Vec<Scored> = c
        .iter()
        .map(|cand| {
            let wins = cand.wins as f64;
            let matches = cand.matches as f64;
            let adjusted = (wins + PRIOR_GAMES / 2.0) / (matches + PRIOR_GAMES);
            let bonus = PREF_MAX_BONUS * (1.0 - cand.pool_index as f64 / n);
            let verdict = if cand.matches < LOW_DATA_MATCHES {
                Verdict::LowData
            } else if adjusted >= GOOD {
                Verdict::Good
            } else if adjusted <= HARD {
                Verdict::Hard
            } else {
                Verdict::Even
            };
            Scored {
                champion: cand.champion,
                pool_index: cand.pool_index,
                wins: cand.wins,
                matches: cand.matches,
                winrate: (cand.matches > 0).then(|| wins / matches),
                adjusted,
                score: adjusted * 100.0 + bonus,
                verdict,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then(a.pool_index.cmp(&b.pool_index))
    });
    out
}

/// Ban önerisi: benim şampiyonumun en kötü eşleşmeleri. `cands` satırlarında `champion` rakiptir,
/// oranlar benim şampiyonumun o rakibe karşı sonucudur. `exclude` (banlanmış, takımın şampiyonu)
/// ve `LOW_DATA_MATCHES`'tan az maçlı rakipler atılır; düzeltilmiş orana göre küçükten büyüğe
/// (eşitlikte çok maçlı önce) ilk `BAN_SUGGESTIONS` tanesi döner.
pub fn worst_matchups(cands: &[Candidate], exclude: &[i64]) -> Vec<Scored> {
    let kept: Vec<Candidate> = cands
        .iter()
        .filter(|c| c.matches >= LOW_DATA_MATCHES && !exclude.contains(&c.champion))
        .cloned()
        .collect();
    let mut out = rank(&kept);
    out.sort_by(|a, b| a.adjusted.total_cmp(&b.adjusted).then(b.matches.cmp(&a.matches)));
    out.truncate(BAN_SUGGESTIONS);
    out
}

/// Rün seçenekleri (LC-013): genel listeden (en çok oynanandan başlayarak) ilk `RUNE_OPTIONS`
/// sayfa. Eşleşmeye özel sayfa en az `MATCHUP_PAGE_MIN_GAMES` maçlıysa: aynı ana rün + ikincil
/// ağaçtaki seçeneğin yerine geçer, yoksa son seçeneğin yerine (liste kısaysa sona eklenir).
/// İkinci değer altın vurgulu seçeneğin sırası.
pub fn pick_rune_options(general: &[RunePage], matchup: Option<&RunePage>) -> (Vec<RunePage>, Option<usize>) {
    let mut opts: Vec<RunePage> = general.iter().take(RUNE_OPTIONS).cloned().collect();
    let Some(mu) = matchup.filter(|m| m.matches >= MATCHUP_PAGE_MIN_GAMES) else {
        return (opts, None);
    };
    let group = |p: &RunePage| (p.perks.first().copied(), p.secondary_tree);
    match opts.iter().position(|o| group(o) == group(mu)) {
        Some(i) => opts[i] = mu.clone(),
        None if opts.len() < RUNE_OPTIONS => opts.push(mu.clone()),
        None => opts[RUNE_OPTIONS - 1] = mu.clone(),
    }
    let i = opts.iter().position(|o| o == mu);
    (opts, i)
}

/// "Önerilen" kart (LC-014): en az `RECOMMENDED_PAGE_MIN_GAMES` maçlı seçenekler içinde kazanma
/// oranı en yüksek olan; oran eşitse maçı çok olan. Eşleşme kartı (`gold`) aday değildir: oranı
/// yalnızca bu rakibe karşı, genel sayfalarla aynı ölçekte değil.
pub fn best_rune_option(opts: &[RunePage], gold: Option<usize>) -> Option<usize> {
    opts.iter()
        .enumerate()
        .filter(|&(i, p)| Some(i) != gold && p.matches >= RECOMMENDED_PAGE_MIN_GAMES)
        .max_by(|(_, a), (_, b)| {
            // a.wins/a.matches ile b.wins/b.matches çapraz çarpımla (kayan nokta yok)
            let ra = u64::from(a.wins) * u64::from(b.matches);
            let rb = u64::from(b.wins) * u64::from(a.matches);
            ra.cmp(&rb).then(a.matches.cmp(&b.matches))
        })
        .map(|(i, _)| i)
}
