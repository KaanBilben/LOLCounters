//! Şampiyon havuzunun (lane başına, tercih sırasıyla) diske kaydı.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lane {
    Top,
    Jungle,
    Mid,
    Adc,
    Support,
}

impl Lane {
    pub const ALL: [Lane; 5] = [Lane::Top, Lane::Jungle, Lane::Mid, Lane::Adc, Lane::Support];
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Lane → şampiyon key'leri, tercih sırasıyla.
    #[serde(default)]
    pub pools: HashMap<Lane, Vec<i64>>,
}

/// Dosya yoksa ya da bozuksa boş ayar döner.
pub fn load(path: &Path) -> Settings {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Ayarları atomik yazar (geçici dosya + taşıma): yazma yarıda kesilirse eski
/// dosya bozulmadan kalır. Arayüz her değişiklikte otomatik kaydettiği için
/// sık çağrılır.
pub fn save(path: &Path, s: &Settings) -> std::io::Result<()> {
    crate::cache::write_atomic(path, &serde_json::to_vec_pretty(s)?)
}
