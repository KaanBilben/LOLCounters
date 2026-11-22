//! Basit JSON disk önbelleği.
//!
//! Yerleşim (app_cache_dir altında):
//! - `ddragon/{sürüm}/static.json`
//! - `ugg/patch.json` (`{ ddragonVersion, uggPatch }`)
//! - `ugg/{yama}/champ_{key}.json` (eşleşmeler + build birlikte)

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Geçici dosya adlarını benzersiz yapar (aynı dosyaya eşzamanlı iki yazma çakışmasın).
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Dosya yoksa ya da okunamıyorsa `None`.
pub fn read<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Üst klasörleri oluşturur; önce geçici dosyaya yazıp sonra yerine taşır
/// (yarım kalan yazma bozuk önbellek bırakmasın diye).
pub fn write<T: Serialize>(path: &Path, v: &T) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec(v)?;
    let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".{}.{n}.tmp", std::process::id()));
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// `parent` altındaki, adı `keep` olmayan bütün alt klasörleri siler (eski
/// sürüm/yama önbelleği). Dosyalara dokunmaz; hatalar yok sayılır.
pub fn remove_other_dirs(parent: &Path, keep: &str) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    for e in entries.flatten() {
        let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
        if is_dir && e.file_name() != keep {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}
