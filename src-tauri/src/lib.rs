//! Tauri komutları ve uygulama durumu.
//!
//! Ağ isteği yalnızca veri eksikse yapılır; sonuçlar yama başına diske yazılır.
//! Zamanlayıcı ya da sürekli arka plan işi yoktur.

pub mod cache;
pub mod ddragon;
pub mod scoring;
pub mod settings;
pub mod ugg;

use ddragon::StaticData;
use futures::StreamExt;
use futures::future::{BoxFuture, FutureExt, Shared};
use scoring::{Candidate, Scored};
use serde::{Deserialize, Serialize};
use settings::{Lane, Settings};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use ugg::{Build, MatchupRow, UggPatch};

/// Aynı anda en çok bu kadar şampiyon verisi indirilir (öneri ve arka plan indirmesi).
const FETCH_CONCURRENCY: usize = 4;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticPayload {
    #[serde(flatten)]
    pub data: StaticData,
    /// Görüntü biçimi, ör. "16.23".
    pub ugg_patch: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    #[serde(flatten)]
    pub scored: Scored,
    pub build: Option<Build>,
    /// Bu şampiyonun verisi (ya da bir kısmı) indirilemediyse hata metni; yoksa `null`.
    /// Doluysa 0/0 ya da eksik build gerçek "az veri" değildir.
    pub error: Option<String>,
}

/// Bir şampiyonun yama verisi (diskte `ugg/{yama}/champ_{key}.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChampData {
    matchups: HashMap<Lane, Vec<MatchupRow>>,
    overview: HashMap<Lane, Build>,
}

/// Yüklenen şampiyon verisi; `error` iki yarıdan biri (eşleşme / build) alınamadıysa dolu.
#[derive(Debug, Clone)]
struct Champ {
    data: Arc<ChampData>,
    error: Option<String>,
}

/// Yarım kalan indirme: alınan yarı saklanır, sonraki istekte yalnızca eksik yarı indirilir.
#[derive(Debug, Clone, Default)]
struct Partial {
    matchups: Option<HashMap<Lane, Vec<MatchupRow>>>,
    overview: Option<HashMap<Lane, Build>>,
}

/// (yama, şampiyon key)
type ChampKey = (String, i64);
/// Süren bir indirme; aynı şampiyonu isteyen herkes bunu bekler.
type ChampFuture = Shared<BoxFuture<'static, Result<Champ, String>>>;

/// `ugg/patch.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchFile {
    ddragon_version: String,
    ugg_patch: UggPatch,
}

pub struct AppState {
    client: reqwest::Client,
    static_data: Mutex<Option<Arc<StaticPayload>>>,
    patch: Mutex<Option<UggPatch>>,
    /// Tam yüklenmiş veri.
    champs: Mutex<HashMap<ChampKey, Arc<ChampData>>>,
    /// Bir yarısı alınamamış veri.
    partial: Mutex<HashMap<ChampKey, Partial>>,
    /// Süren indirmeler.
    inflight: Mutex<HashMap<ChampKey, ChampFuture>>,
}

impl AppState {
    fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .gzip(true)
            .user_agent("LOLCounters/0.1")
            .build()
            .expect("HTTP istemcisi kurulamadı");
        Self {
            client,
            static_data: Mutex::new(None),
            patch: Mutex::new(None),
            champs: Mutex::new(HashMap::new()),
            partial: Mutex::new(HashMap::new()),
            inflight: Mutex::new(HashMap::new()),
        }
    }

    fn complete(&self, key: ChampKey, d: ChampData) -> Champ {
        let data = Arc::new(d);
        lock(&self.champs).insert(key, data.clone());
        Champ { data, error: None }
    }
}

/// Zehirlenmiş kilitte de veriyi kullan (önbellek verisi her zaman tutarlı).
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_cache_dir()
        .map_err(|e| format!("Önbellek klasörü bulunamadı: {e}"))
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|d| d.join("settings.json"))
        .map_err(|e| format!("Ayar klasörü bulunamadı: {e}"))
}

fn static_path(dir: &Path, version: &str) -> PathBuf {
    dir.join("ddragon").join(version).join("static.json")
}

fn version_order(v: &str) -> Vec<u32> {
    v.split('.').map(|x| x.parse().unwrap_or(0)).collect()
}

/// Ağ yokken: diskteki en yeni sürümün statik verisi.
fn newest_cached_static(dir: &Path) -> Option<StaticData> {
    let mut versions: Vec<String> = std::fs::read_dir(dir.join("ddragon"))
        .ok()?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .collect();
    versions.sort_by_key(|v| std::cmp::Reverse(version_order(v)));
    versions
        .iter()
        .find_map(|v| cache::read::<StaticData>(&static_path(dir, v)))
}

/// u.gg yaması: disk (aynı Data Dragon sürümü için) → ağ → eski disk kaydı → tahmin.
/// İkinci değer: yama doğrulandı mı (disk kaydı ya da ağ); tahmin/eski kayıtta `false`.
async fn resolve_ugg_patch(state: &AppState, dir: &Path, version: &str) -> (UggPatch, bool) {
    let path = dir.join("ugg").join("patch.json");
    let disk: Option<PatchFile> = cache::read(&path);
    if let Some(f) = &disk
        && f.ddragon_version == version
    {
        return (f.ugg_patch.clone(), true);
    }
    match ugg::resolve_patch(&state.client, version).await {
        Ok(p) => {
            // u.gg bu yamayı henüz listelemiyorsa kaydetme; sonraki açılışta yeniden sorulur.
            if p.is_current_for(version) {
                let file = PatchFile { ddragon_version: version.to_string(), ugg_patch: p.clone() };
                let _ = cache::write(&path, &file);
            }
            (p, true)
        }
        Err(_) => (
            disk.map(|f| f.ugg_patch)
                .unwrap_or_else(|| ugg::fallback_patch(version)),
            false,
        ),
    }
}

async fn static_payload(app: &AppHandle, state: &AppState) -> Result<Arc<StaticPayload>, String> {
    let cached = lock(&state.static_data).clone();
    if let Some(s) = cached {
        return Ok(s);
    }
    let dir = cache_dir(app)?;
    // `fresh`: en yeni sürüm yüklendi (ağdan ya da o sürümün disk kaydından).
    let (data, fresh) = match ddragon::latest_version(&state.client).await {
        Ok(v) => match cache::read::<StaticData>(&static_path(&dir, &v)) {
            Some(d) => (d, Some(v)),
            None => match ddragon::load_static_data(&state.client, &v).await {
                Ok(d) => {
                    let _ = cache::write(&static_path(&dir, &v), &d);
                    (d, Some(v))
                }
                Err(e) => (newest_cached_static(&dir).ok_or(e)?, None),
            },
        },
        Err(e) => (
            newest_cached_static(&dir).ok_or_else(|| {
                format!("İnternet bağlantısı yok ve kayıtlı veri bulunamadı. ({e})")
            })?,
            None,
        ),
    };
    let (patch, patch_confirmed) = resolve_ugg_patch(state, &dir, &data.version).await;
    // Eski sürüm/yama klasörlerini açılışta bir kez temizle (yalnızca güncel olan doğrulandıysa).
    if let Some(v) = &fresh {
        cache::remove_other_dirs(&dir.join("ddragon"), v);
    }
    if patch_confirmed {
        cache::remove_other_dirs(&dir.join("ugg"), &patch.patch);
    }
    let payload = Arc::new(StaticPayload { ugg_patch: patch.display(), data });
    *lock(&state.patch) = Some(patch);
    *lock(&state.static_data) = Some(payload.clone());
    Ok(payload)
}

async fn current_patch(app: &AppHandle, state: &AppState) -> Result<UggPatch, String> {
    let cached = lock(&state.patch).clone();
    if let Some(p) = cached {
        return Ok(p);
    }
    static_payload(app, state).await?;
    lock(&state.patch)
        .clone()
        .ok_or_else(|| "u.gg yaması belirlenemedi".to_string())
}

/// Şampiyon verisi: bellek → (süren indirme varsa onu bekle) → disk → ağ.
async fn champion_data(
    app: &AppHandle,
    dir: &Path,
    patch: &UggPatch,
    key: i64,
) -> Result<Champ, String> {
    let state = app.state::<AppState>();
    let mem_key = (patch.patch.clone(), key);
    let cached = lock(&state.champs).get(&mem_key).cloned();
    if let Some(data) = cached {
        return Ok(Champ { data, error: None });
    }
    let fut = lock(&state.inflight)
        .entry(mem_key.clone())
        .or_insert_with(|| {
            load_champ(app.clone(), dir.to_path_buf(), patch.clone(), key)
                .boxed()
                .shared()
        })
        .clone();
    let result = fut.clone().await;
    let mut inflight = lock(&state.inflight);
    if inflight.get(&mem_key).is_some_and(|f| f.ptr_eq(&fut)) {
        inflight.remove(&mem_key);
    }
    result
}

/// Asıl yükleme (aynı şampiyon için aynı anda yalnızca bir tane çalışır).
/// İki yarı ayrı indirilir; biri başarısız olursa alınan yarı saklanır.
async fn load_champ(app: AppHandle, dir: PathBuf, patch: UggPatch, key: i64) -> Result<Champ, String> {
    let state = app.state::<AppState>();
    let mem_key = (patch.patch.clone(), key);
    let cached = lock(&state.champs).get(&mem_key).cloned();
    if let Some(data) = cached {
        return Ok(Champ { data, error: None });
    }
    let path = dir.join("ugg").join(&patch.patch).join(format!("champ_{key}.json"));
    if let Some(d) = cache::read::<ChampData>(&path) {
        return Ok(state.complete(mem_key, d));
    }

    let prev = lock(&state.partial).get(&mem_key).cloned().unwrap_or_default();
    let (client, p) = (&state.client, &patch);
    let (matchups, overview) = futures::join!(
        async move {
            match prev.matchups {
                Some(m) => Ok(m),
                None => ugg::fetch_matchups(client, p, key).await,
            }
        },
        async move {
            match prev.overview {
                Some(o) => Ok(o),
                None => ugg::fetch_overview(client, p, key).await,
            }
        },
    );
    match (matchups, overview) {
        (Ok(matchups), Ok(overview)) => {
            lock(&state.partial).remove(&mem_key);
            let d = ChampData { matchups, overview };
            // İkisi de boşsa (u.gg'de dosya yok) yalnızca bellekte tutulur:
            // bu oturumda yeniden istenmez, sonraki açılışta yeniden sorulur.
            if !d.matchups.is_empty() || !d.overview.is_empty() {
                let _ = cache::write(&path, &d);
            }
            Ok(state.complete(mem_key, d))
        }
        (Err(e), Err(_)) => Err(e),
        (m, o) => {
            let error = match (&m, &o) {
                (Err(e), _) => format!("Eşleşme verisi alınamadı ({e})"),
                (_, Err(e)) => format!("Build verisi alınamadı ({e})"),
                _ => unreachable!("iki yarıdan biri hatalı"),
            };
            let part = Partial { matchups: m.ok(), overview: o.ok() };
            lock(&state.partial).insert(mem_key, part.clone());
            let data = ChampData {
                matchups: part.matchups.unwrap_or_default(),
                overview: part.overview.unwrap_or_default(),
            };
            Ok(Champ { data: Arc::new(data), error: Some(error) })
        }
    }
}

/// Havuzdaki bütün şampiyonların verisini bir kez indirir (en çok 4 eşzamanlı).
async fn prefetch(app: AppHandle, keys: Vec<i64>) {
    let state = app.state::<AppState>();
    let (Ok(patch), Ok(dir)) = (current_patch(&app, &state).await, cache_dir(&app)) else {
        return;
    };
    let (app, patch, dir) = (&app, &patch, &dir);
    futures::stream::iter(keys)
        .for_each_concurrent(FETCH_CONCURRENCY, |k| async move {
            let _ = champion_data(app, dir, patch, k).await;
        })
        .await;
}

#[tauri::command]
async fn load_static(app: AppHandle, state: State<'_, AppState>) -> Result<StaticPayload, String> {
    Ok((*static_payload(&app, &state).await?).clone())
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    settings_path(&app).map(|p| settings::load(&p)).unwrap_or_default()
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings::save(&settings_path(&app)?, &settings)
        .map_err(|e| format!("Ayarlar kaydedilemedi: {e}"))?;
    let mut seen = HashSet::new();
    let keys: Vec<i64> = Lane::ALL
        .iter()
        .filter_map(|l| settings.pools.get(l))
        .flatten()
        .copied()
        .filter(|k| seen.insert(*k))
        .collect();
    if !keys.is_empty() {
        tauri::async_runtime::spawn(prefetch(app, keys));
    }
    Ok(())
}

#[tauri::command]
async fn recommend(
    app: AppHandle,
    state: State<'_, AppState>,
    lane: Lane,
    enemy: i64,
) -> Result<Vec<Recommendation>, String> {
    let pool = get_settings(app.clone()).pools.remove(&lane).unwrap_or_default();
    if pool.is_empty() {
        return Ok(Vec::new());
    }
    let patch = current_patch(&app, &state).await?;
    let dir = cache_dir(&app)?;
    // `buffered` sırayı korur: results[i] = pool[i].
    let results: Vec<Result<Champ, String>> = futures::stream::iter(pool.iter().copied())
        .map(|k| champion_data(&app, &dir, &patch, k))
        .buffered(FETCH_CONCURRENCY)
        .collect()
        .await;
    if let Some(Err(e)) = results.iter().find(|r| r.is_err())
        && results.iter().all(Result::is_err)
    {
        return Err(e.clone());
    }

    let candidates: Vec<Candidate> = pool
        .iter()
        .zip(&results)
        .enumerate()
        .map(|(i, (&champion, result))| {
            let row = result.as_ref().ok().and_then(|c| {
                c.data.matchups.get(&lane)?.iter().find(|r| r.opponent == enemy).cloned()
            });
            Candidate {
                champion,
                pool_index: i,
                wins: row.as_ref().map_or(0, |r| r.wins),
                matches: row.as_ref().map_or(0, |r| r.matches),
            }
        })
        .collect();

    Ok(scoring::rank(&candidates)
        .into_iter()
        .map(|scored| {
            let (build, error) = match &results[scored.pool_index] {
                Ok(c) => (c.data.overview.get(&lane).cloned(), c.error.clone()),
                Err(e) => (None, Some(e.clone())),
            };
            Recommendation { scored, build, error }
        })
        .collect())
}

// ---------- Lane filtresi ----------

/// Süren bir lane verisi indirmesi; aynı yamayı isteyen herkes bunu bekler.
type RolesFuture = Shared<BoxFuture<'static, Result<Arc<ugg::RoleGames>, String>>>;

/// Lane verisi (diskte `ugg/{yama}/roles.json`, lane başına maç sayısı).
#[derive(Default)]
struct RolesState {
    /// yama → yüklenmiş veri
    games: Mutex<HashMap<String, Arc<ugg::RoleGames>>>,
    /// yama → süren indirme
    inflight: Mutex<HashMap<String, RolesFuture>>,
}

/// Lane başına maç sayıları: bellek → (süren indirme varsa onu bekle) → disk → ağ.
async fn role_games(app: &AppHandle) -> Result<Arc<ugg::RoleGames>, String> {
    let patch = current_patch(app, &app.state::<AppState>()).await?;
    let roles = app.state::<RolesState>();
    let cached = lock(&roles.games).get(&patch.patch).cloned();
    if let Some(g) = cached {
        return Ok(g);
    }
    let fut = lock(&roles.inflight)
        .entry(patch.patch.clone())
        .or_insert_with(|| load_roles(app.clone(), patch.clone()).boxed().shared())
        .clone();
    let result = fut.clone().await;
    let mut inflight = lock(&roles.inflight);
    if inflight.get(&patch.patch).is_some_and(|f| f.ptr_eq(&fut)) {
        inflight.remove(&patch.patch);
    }
    result
}

/// Asıl yükleme (aynı yama için aynı anda yalnızca bir tane çalışır).
async fn load_roles(app: AppHandle, patch: UggPatch) -> Result<Arc<ugg::RoleGames>, String> {
    let roles = app.state::<RolesState>();
    let cached = lock(&roles.games).get(&patch.patch).cloned();
    if let Some(g) = cached {
        return Ok(g);
    }
    let path = cache_dir(&app)?.join("ugg").join(&patch.patch).join("roles.json");
    let games = match cache::read::<ugg::RoleGames>(&path) {
        Some(g) => g,
        None => {
            let g = ugg::fetch_roles(&app.state::<AppState>().client, &patch).await?;
            // Boşsa (u.gg'de dosya yok) yalnızca bellekte tutulur; sonraki açılışta yeniden sorulur.
            if !g.is_empty() {
                let _ = cache::write(&path, &g);
            }
            g
        }
    };
    let games = Arc::new(games);
    lock(&roles.games).insert(patch.patch, games.clone());
    Ok(games)
}

/// Diğer Rust kodu için (ör. rakibin lane'ini tahmin etmek): şampiyon key → lane → pay
/// (lane maçı / toplam maç). Veri eksikse bir kez indirir; sonra bellekten döner.
pub(crate) async fn role_shares_cached(app: &AppHandle) -> Result<HashMap<i64, HashMap<Lane, f64>>, String> {
    let games = role_games(app).await?;
    Ok(ugg::role_shares(&games))
}

/// `lanes` içinde olmayan şampiyonlar (ör. yeni çıkmış) bütün lane'lerde oynanabilir sayılır.
pub fn lanes_with_fallback(
    mut lanes: HashMap<i64, Vec<Lane>>,
    champion_keys: &[i64],
) -> HashMap<i64, Vec<Lane>> {
    for &key in champion_keys {
        lanes.entry(key).or_insert_with(|| Lane::ALL.to_vec());
    }
    lanes
}

/// Şampiyon key → oynandığı lane'ler (payı en az `ugg::ROLE_MIN_SHARE` olanlar).
/// Data Dragon'daki her şampiyon listededir; verisi olmayan her lane'de görünür.
#[tauri::command]
async fn get_roles(app: AppHandle, state: State<'_, AppState>) -> Result<HashMap<i64, Vec<Lane>>, String> {
    let shares = role_shares_cached(&app).await?;
    let keys: Vec<i64> = static_payload(&app, &state).await?.data.champions.iter().map(|c| c.key).collect();
    Ok(lanes_with_fallback(ugg::lanes_over(&shares, ugg::ROLE_MIN_SHARE), &keys))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .manage(RolesState::default())
        .invoke_handler(tauri::generate_handler![
            load_static,
            get_settings,
            save_settings,
            recommend,
            get_roles
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
