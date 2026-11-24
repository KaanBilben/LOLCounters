//! Öneri puanlaması. Açıklama: docs/scoring.md (sabitler değişirse orası da güncellenir).

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
