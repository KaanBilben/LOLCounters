//! Tauri komutları ve uygulama durumu.
//!
//! Ağ isteği yalnızca veri eksikse yapılır; sonuçlar yama başına diske yazılır.
//! Zamanlayıcı ya da sürekli arka plan işi yoktur.

pub mod cache;
pub mod compact;
pub mod ddragon;
pub mod launch;
pub mod lcu;
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use ugg::{Build, MatchupRow, UggPatch};

/// Aynı anda en çok bu kadar şampiyon verisi indirilir (öneri ve arka plan indirmesi).
const FETCH_CONCURRENCY: usize = 4;

/// Statik veri yeniden yüklendi ve sürüm ya da yama değişti; arayüz `load_static`'i yeniden ister.
pub const STATIC_UPDATED_EVENT: &str = "static-updated";

/// Havuz kaydedildi; küçük pencere önerisini yeniden ister (LC-009).
pub const POOLS_CHANGED_EVENT: &str = "pools-changed";

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
    /// Bellekteki statik veri kesin değil (`static_settled` yanlış): sonraki tetikte yeniden denenir.
    static_unsettled: AtomicBool,
    /// Süren bir yeniden deneme var (aynı anda yalnızca biri).
    rechecking: AtomicBool,
}

/// Dış isteklerin (Data Dragon, u.gg) kimliği. Bir sorun olursa u.gg bizi
/// engellemek yerine repo adresinden ulaşabilsin diye adres de yazılır.
pub const USER_AGENT: &str = "LOLCounters/0.1 (+https://github.com/KaanBilben/LOLCounters)";

impl AppState {
    fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .gzip(true)
            .user_agent(USER_AGENT)
            .build()
            .expect("HTTP istemcisi kurulamadı");
        Self {
            client,
            static_data: Mutex::new(None),
            patch: Mutex::new(None),
            champs: Mutex::new(HashMap::new()),
            partial: Mutex::new(HashMap::new()),
            inflight: Mutex::new(HashMap::new()),
            static_unsettled: AtomicBool::new(false),
            rechecking: AtomicBool::new(false),
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
            // u.gg bu yamayı henüz listelemiyorsa kaydetme; `recheck_static` yeniden sorar.
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

/// Yüklenen statik veri kesin mi: Data Dragon'ın en yeni sürümü (`latest`, ağdan öğrenildiyse)
/// yüklendi ve u.gg o sürümün kendi yamasını listeliyor (`patch_confirmed`: ağdan ya da disk kaydından).
/// Değilse (ağ yoktu, u.gg yamayı henüz eklemedi) veri yalnızca geçici tutulur: süreç tepside
/// günlerce yaşadığından ana pencere gösterilince ve şampiyon seçimi başlayınca yeniden denenir.
pub fn static_settled(latest: Option<&str>, patch: &UggPatch, patch_confirmed: bool) -> bool {
    patch_confirmed && latest.is_some_and(|v| patch.is_current_for(v))
}

/// Statik veri ve u.gg yaması (bellek kullanılmaz); üçüncü değer `static_settled`.
async fn resolve_static(
    app: &AppHandle,
    state: &AppState,
) -> Result<(StaticPayload, UggPatch, bool), String> {
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
    // Eski sürüm/yama klasörlerini temizle (yalnızca güncel olan doğrulandıysa).
    if let Some(v) = &fresh {
        cache::remove_other_dirs(&dir.join("ddragon"), v);
    }
    if patch_confirmed {
        cache::remove_other_dirs(&dir.join("ugg"), &patch.patch);
    }
    let settled = static_settled(fresh.as_deref(), &patch, patch_confirmed);
    Ok((StaticPayload { ugg_patch: patch.display(), data }, patch, settled))
}

fn store_static(state: &AppState, payload: StaticPayload, patch: UggPatch, settled: bool) -> Arc<StaticPayload> {
    let payload = Arc::new(payload);
    *lock(&state.patch) = Some(patch);
    *lock(&state.static_data) = Some(payload.clone());
    state.static_unsettled.store(!settled, Ordering::Relaxed);
    payload
}

async fn static_payload(app: &AppHandle, state: &AppState) -> Result<Arc<StaticPayload>, String> {
    let cached = lock(&state.static_data).clone();
    if let Some(s) = cached {
        return Ok(s);
    }
    let (payload, patch, settled) = resolve_static(app, state).await?;
    Ok(store_static(state, payload, patch, settled))
}

/// Olay güdümlü yeniden deneme (zamanlayıcı yok): ana pencere gösterilince ve şampiyon seçimi
/// başlayınca çağrılır. Boş gelmiş şampiyon / lane verisini bellekten atar (sonraki istekte
/// yeniden sorulur). Statik veri kesin değilse arka planda yeniden yükler; sürüm ya da yama
/// değiştiyse `static-updated` olayını gönderir. Veri kesinse ağ isteği yapılmaz.
pub(crate) fn recheck_static(app: &AppHandle) {
    forget_empty(app);
    let state = app.state::<AppState>();
    if !state.static_unsettled.load(Ordering::Relaxed) || state.rechecking.swap(true, Ordering::AcqRel) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        if let Ok((payload, patch, settled)) = resolve_static(&app, &state).await {
            let old = lock(&state.static_data).clone();
            let changed = old.is_none_or(|o| {
                o.data.version != payload.data.version || o.ugg_patch != payload.ugg_patch
            });
            store_static(&state, payload, patch, settled);
            if changed {
                let _ = app.emit(STATIC_UPDATED_EVENT, ());
            }
        }
        state.rechecking.store(false, Ordering::Release);
    });
}

/// Bu oturumda boş gelmiş (u.gg'de dosyası olmayan) şampiyon ve lane verisini bellekten atar.
fn forget_empty(app: &AppHandle) {
    lock(&app.state::<AppState>().champs).retain(|_, d| !d.matchups.is_empty() || !d.overview.is_empty());
    lock(&app.state::<RolesState>().games).retain(|_, g| !g.is_empty());
    let runes = app.state::<RunesState>();
    lock(&runes.options).retain(|_, d| !d.is_empty());
    lock(&runes.matchups).retain(|_, d| !d.is_empty());
    lock(&runes.duos).retain(|_, d| !d.is_empty());
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
            // `recheck_static` tetiklenene kadar (ana pencere / şampiyon seçimi) yeniden istenmez.
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
    let path = settings_path(&app)?;
    // Arayüz yalnızca havuzu gönderir; autostart gibi Rust alanları diskteki kayıttan korunur.
    let settings = settings::keep_backend_fields(settings, &settings::load(&path));
    settings::save(&path, &settings)
        .map_err(|e| format!("Ayarlar kaydedilemedi: {e}"))?;
    // Küçük pencere açıksa havuz listesini yeniler (olay güdümlü, yoklama yok).
    let _ = app.emit(POOLS_CHANGED_EVENT, ());
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

/// Lane verisi (diskte `ugg/{yama}/roles.json`, lane başına galibiyet ve maç).
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
            // Boşsa (u.gg'de dosya yok) yalnızca bellekte tutulur; `recheck_static` tetiklenince atılır.
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

// --- LC-008: LoL istemcisi (LCU) ---------------------------------------------

/// Rakibin lane tahmini için şampiyon başına rol payları (key → lane → 0..1).
/// LC-006'nın lane verisi: bellekte yoksa diskten, o da yoksa yama başına bir kez indirilir.
/// Veri alınamazsa hata döner; o zaman bu seçimde tahmin olmaz, kullanıcı rakibi elle seçer.
async fn lcu_role_shares(app: &AppHandle) -> Result<HashMap<i64, HashMap<Lane, f64>>, String> {
    role_shares_cached(app).await
}

/// Şampiyon seçimi başladı: küçük pencere açılır (LC-009). Kesin olmayan statik veri de
/// burada yeniden denenir (LC-007 `recheck_static`): uygulama tepside günlerce yaşar ve ana
/// pencere hiç açılmadan yalnızca seçimde kullanılabilir.
fn on_champ_select_start(app: &AppHandle) {
    recheck_static(app);
    compact::sync(app, true);
}

/// Şampiyon seçimi bitti: lobiye dönüş, kuyruk, oyun başlangıcı ya da bağlantı kopması.
/// Küçük pencere kapanır (LC-009).
fn on_champ_select_end(app: &AppHandle) {
    compact::sync(app, false);
}

#[tauri::command]
fn get_champ_select_state(state: State<'_, lcu::LcuState>) -> lcu::ChampSelectPayload {
    state.payload()
}

#[tauri::command]
async fn set_opponent(app: AppHandle, champion_key: Option<i64>) -> lcu::ChampSelectPayload {
    lcu::set_opponent(&app, champion_key).await
}

/// Atanmış rol yokken kullanıcının seçtiği lane (rakip tahmini için); `null`: geri al.
#[tauri::command]
async fn set_my_lane(app: AppHandle, lane: Option<Lane>) -> lcu::ChampSelectPayload {
    lcu::set_my_lane(&app, lane).await
}

// --- LC-009: küçük pencere -----------------------------------------------------

/// Tek şampiyonun bir rakibe karşı sonucu ve o lane'deki genel build'i (havuzda olmasa da).
/// Küçük pencere kilitten sonra bunu gösterir.
#[tauri::command]
async fn champion_matchup(
    app: AppHandle,
    state: State<'_, AppState>,
    lane: Lane,
    champion: i64,
    enemy: i64,
) -> Result<Recommendation, String> {
    let patch = current_patch(&app, &state).await?;
    let dir = cache_dir(&app)?;
    let c = champion_data(&app, &dir, &patch, champion).await?;
    let row = c
        .data
        .matchups
        .get(&lane)
        .and_then(|rows| rows.iter().find(|r| r.opponent == enemy).cloned());
    let candidate = Candidate {
        champion,
        pool_index: 0,
        wins: row.as_ref().map_or(0, |r| r.wins),
        matches: row.as_ref().map_or(0, |r| r.matches),
    };
    let scored = scoring::rank(&[candidate]).remove(0);
    Ok(Recommendation { scored, build: c.data.overview.get(&lane).cloned(), error: c.error.clone() })
}

// --- LC-011: ban önerisi -------------------------------------------------------

/// Ban aşamasında: gösterdiğim şampiyonun bu lane'deki en kötü eşleşmeleri (`scoring::worst_matchups`).
/// `exclude`: banlanmış ve takımın şampiyonları; şampiyonun kendisi de atılır.
/// Eşleşme verisi indirilemediyse hata döner; u.gg'de veri yoksa boş liste.
#[tauri::command]
async fn ban_suggestions(
    app: AppHandle,
    state: State<'_, AppState>,
    lane: Lane,
    champion: i64,
    exclude: Vec<i64>,
) -> Result<Vec<Scored>, String> {
    let patch = current_patch(&app, &state).await?;
    let dir = cache_dir(&app)?;
    let c = champion_data(&app, &dir, &patch, champion).await?;
    let Some(rows) = c.data.matchups.get(&lane) else {
        return match c.error {
            Some(e) if c.data.matchups.is_empty() => Err(e),
            _ => Ok(Vec::new()),
        };
    };
    let cands: Vec<Candidate> = rows
        .iter()
        .map(|r| Candidate { champion: r.opponent, pool_index: 0, wins: r.wins, matches: r.matches })
        .collect();
    let mut exclude = exclude;
    exclude.push(champion);
    Ok(scoring::worst_matchups(&cands, &exclude))
}

// --- LC-013: rün seçenekleri ---------------------------------------------------

/// Bellekteki lane başına veri: anahtar → lane → değer.
type LaneCache<K, V> = Mutex<HashMap<K, Arc<HashMap<Lane, V>>>>;

/// Rün seçenekleri ve eşleşme sayfaları (bellek; disk yerleşimi `cache.rs` başında).
#[derive(Default)]
struct RunesState {
    /// (yama, şampiyon) → lane → seçenekler
    options: LaneCache<ChampKey, Vec<ugg::RunePage>>,
    /// (yama, şampiyon, rakip) → lane → eşleşme sayfası
    matchups: LaneCache<(String, i64, i64), ugg::MatchupPage>,
    /// (yama, partner) → partnerin lane'i → birlikte oynadığı şampiyonlar (LC-016)
    duos: LaneCache<ChampKey, Vec<ugg::DuoRow>>,
    /// Dosya başına kapı: aynı dosyayı isteyen ikinci çağrı ilk indirmeyi bekler (iki kez indirilmez).
    gates: Mutex<HashMap<PathBuf, Arc<tokio::sync::Mutex<()>>>>,
}

/// Lane başına veri: bellek → disk → ağ (bir kez). Boş sonuç (u.gg'de dosya yok) diske yazılmaz,
/// yalnızca bellekte tutulur; `recheck_static` tetiklenince atılır (`forget_empty`).
async fn lane_data<K, V, Fut>(
    mem: &LaneCache<K, V>,
    gates: &Mutex<HashMap<PathBuf, Arc<tokio::sync::Mutex<()>>>>,
    key: K,
    path: PathBuf,
    fetch: impl FnOnce() -> Fut,
) -> Result<Arc<HashMap<Lane, V>>, String>
where
    K: Eq + std::hash::Hash,
    V: Serialize + serde::de::DeserializeOwned,
    Fut: std::future::Future<Output = Result<HashMap<Lane, V>, String>>,
{
    let cached = lock(mem).get(&key).cloned();
    if let Some(d) = cached {
        return Ok(d);
    }
    let gate = lock(gates).entry(path.clone()).or_default().clone();
    let _open = gate.lock().await;
    // kapıda beklerken önceki çağrı indirmiş olabilir
    let cached = lock(mem).get(&key).cloned();
    if let Some(d) = cached {
        return Ok(d);
    }
    let data = match cache::read::<HashMap<Lane, V>>(&path) {
        Some(d) => d,
        None => {
            let d = fetch().await?;
            if !d.is_empty() {
                let _ = cache::write(&path, &d);
            }
            d
        }
    };
    let data = Arc::new(data);
    lock(mem).insert(key, data.clone());
    Ok(data)
}

/// Küçük pencerenin rün görünümü (LC-013): seçenekler, altın vurgulu seçeneğin sırası ve büyüler.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuneOptions {
    pub options: Vec<ugg::RunePage>,
    /// Eşleşmeye özel sayfa olan seçenek (`scoring::pick_rune_options`); yoksa `None`.
    pub gold: Option<usize>,
    /// "Önerilen" seçenek (`scoring::best_rune_option`); yoksa `None`.
    pub best: Option<usize>,
    /// Altın seçenek varsa eşleşmenin büyüleri, yoksa lane'in genel büyüleri.
    pub spells: Vec<i64>,
    /// Bir kaynak alınamadı ve liste eksik olabilir (ör. tek sayfa, parçasız): arayüz yeniden
    /// denemeyi sunar. Hata önbelleğe yazılmadığı için yeniden deneme yeniden indirir.
    pub error: Option<String>,
}

/// Kilitledikten sonra: şampiyonumun bu lane'deki rün seçenekleri (`builds`) ve rakip biliniyorsa
/// eşleşmeye özel sayfası. Parçalar ve büyüler overview'dan gelir (`builds`'te yok).
/// Eşleşme sayfası alınamazsa vurgusuz devam edilir; seçenek hiç yoksa hata döner.
#[tauri::command]
async fn rune_options(
    app: AppHandle,
    state: State<'_, AppState>,
    lane: Lane,
    champion: i64,
    enemy: Option<i64>,
) -> Result<RuneOptions, String> {
    let patch = current_patch(&app, &state).await?;
    let dir = cache_dir(&app)?;
    let patch_dir = dir.join("ugg").join(&patch.patch);
    let runes = app.state::<RunesState>();
    let client = &state.client;
    let (general, champ, mu) = futures::join!(
        lane_data(
            &runes.options,
            &runes.gates,
            (patch.patch.clone(), champion),
            patch_dir.join(format!("runes_{champion}.json")),
            || ugg::fetch_rune_options(client, &patch, champion),
        ),
        champion_data(&app, &dir, &patch, champion),
        async {
            let Some(e) = enemy else { return Ok(None) };
            lane_data(
                &runes.matchups,
                &runes.gates,
                (patch.patch.clone(), champion, e),
                patch_dir.join(format!("mu_{champion}_{e}.json")),
                || ugg::fetch_matchup_pages(client, &patch, champion, e),
            )
            .await
            .map(Some)
        },
    );
    // Liste eksik kalabilecek hata: `builds` ya da (parçaları veren) overview alınamadı.
    let (overview, champ_error) = match champ {
        Ok(c) => (c.data.overview.get(&lane).cloned(), c.error),
        Err(e) => (None, Some(e)),
    };
    // eşleşme yarısının hatası rünleri etkilemez; yalnızca overview eksikse sayılır
    let champ_error = champ_error.filter(|_| overview.is_none());
    let mu_page = mu.ok().flatten().and_then(|m| m.get(&lane).cloned());
    // Parçalar overview'dan; o yoksa eşleşme sayfasından (ikisi de aynı biçim).
    let shards = overview
        .as_ref()
        .map(|b| b.runes.shards.clone())
        .filter(|s| !s.is_empty())
        .or_else(|| mu_page.as_ref().map(|m| m.runes.shards.clone()))
        .unwrap_or_default();
    let (mut list, general_error) = match general {
        Ok(g) => (g.get(&lane).cloned().unwrap_or_default(), None),
        Err(e) => (Vec::new(), Some(e)),
    };
    for p in &mut list {
        p.shards.clone_from(&shards);
    }
    // `builds` yoksa ya da alınamadıysa overview'un tek sayfası.
    if list.is_empty()
        && let Some(b) = &overview
    {
        list.push(b.runes.clone());
    }
    let error = general_error.or(champ_error);
    if list.is_empty() {
        return match error {
            Some(e) => Err(e),
            None => Ok(RuneOptions { options: Vec::new(), gold: None, best: None, spells: Vec::new(), error: None }),
        };
    }
    let (options, gold) = scoring::pick_rune_options(&list, mu_page.as_ref().map(|m| &m.runes));
    let spells = match (gold, &mu_page) {
        (Some(_), Some(m)) if m.spells.len() == 2 => m.spells.clone(),
        _ => overview.map(|b| b.spells).unwrap_or_default(),
    };
    let best = scoring::best_rune_option(&options, gold);
    Ok(RuneOptions { options, gold, best, spells, error })
}

/// LC-015: küçük penceredeki "İçe aktar". Sayfayı istemcinin sırasına dizer (`order_perks`) ve
/// istemciye yazar; yer yoksa önce `needsConfirm` döner, onaylanan sayfa `confirm` ile gelir.
#[tauri::command]
async fn import_loadout(
    app: AppHandle,
    state: State<'_, AppState>,
    page: ugg::RunePage,
    spells: Vec<i64>,
    confirm: Option<i64>,
) -> Result<lcu::ImportOutcome, String> {
    let data = static_payload(&app, &state).await?;
    let perks = ddragon::order_perks(&data.data.runes, page.primary_tree, page.secondary_tree, &page.perks, &page.shards)
        .ok_or("Rün sayfası eksik; içe aktarılamadı")?;
    let spells = <[i64; 2]>::try_from(spells).ok();
    lcu::import_loadout(&app, page.primary_tree, page.secondary_tree, perks, spells, confirm).await
}

// --- LC-016: bot lane sinerjisi ------------------------------------------------

/// Küçük pencerenin sinerji verisi: partnerle bütün ikililer ve sinerji sekmesinin listesi.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuoSynergy {
    /// `scoring::duo_synergies`: en az `DUO_MIN_GAMES` maçlı bütün ikililer, sinerji sırasıyla.
    pub all: Vec<scoring::Synergy>,
    /// `scoring::best_synergies`: sinerji sekmesinde gösterilenler.
    pub best: Vec<scoring::Synergy>,
    /// Rakibin eşleşme verisi alınamadı: "counter" işaretleri eksik olabilir; arayüz yeniden
    /// denemeyi sunar (hata önbelleğe yazılmaz).
    pub error: Option<String>,
}

/// Ben ADC'ysem partnerim support'tur, support'sam ADC; başka lane'de bot ikilisi yok.
fn partner_lane(lane: Lane) -> Option<Lane> {
    match lane {
        Lane::Adc => Some(Lane::Support),
        Lane::Support => Some(Lane::Adc),
        _ => None,
    }
}

/// Partnerimle (bot lane) hangi şampiyonların iyi gittiği. Sinerji = ikilinin düzeltilmiş oranı −
/// iki şampiyonun bu lane'lerdeki tek başına oranlarının ortalaması (`scoring::duo_synergies`).
/// Rakip biliniyorsa rakibe karşı iyi eşleşenler "counter" sayılır (rakibin eşleşme verisinden).
/// `exclude`: banlanmış ve alınmış şampiyonlar (yalnızca `best` listesinden atılır).
#[tauri::command]
async fn duo_synergy(
    app: AppHandle,
    state: State<'_, AppState>,
    lane: Lane,
    partner: i64,
    opponent: Option<i64>,
    exclude: Vec<i64>,
) -> Result<DuoSynergy, String> {
    let Some(p_lane) = partner_lane(lane) else {
        return Ok(DuoSynergy { all: Vec::new(), best: Vec::new(), error: None });
    };
    let patch = current_patch(&app, &state).await?;
    let dir = cache_dir(&app)?;
    let runes = app.state::<RunesState>();
    let client = &state.client;
    let (duos, roles, opp) = futures::join!(
        lane_data(
            &runes.duos,
            &runes.gates,
            (patch.patch.clone(), partner),
            dir.join("ugg").join(&patch.patch).join(format!("duos_{partner}.json")),
            || ugg::fetch_champion_duos(client, &patch, partner),
        ),
        role_games(&app),
        async {
            match opponent {
                Some(o) => Some(champion_data(&app, &dir, &patch, o).await),
                None => None,
            }
        },
    );
    let (duos, roles) = (duos?, roles?);
    let Some(rows) = duos.get(&p_lane) else {
        return Ok(DuoSynergy { all: Vec::new(), best: Vec::new(), error: None });
    };
    let solo = |champ: i64, l: Lane| {
        roles
            .get(&champ)
            .and_then(|m| m.get(&l))
            .filter(|s| s.matches > 0)
            .map(|s| f64::from(s.wins) / f64::from(s.matches))
    };
    // Lane oranı yoksa sinerji hesaplanamaz (yanlış artı göstermemek için boş döner).
    let Some(partner_wr) = solo(partner, p_lane) else {
        return Ok(DuoSynergy { all: Vec::new(), best: Vec::new(), error: None });
    };
    // Rakibin satırı rakibin sonucudur; adayın sonucu = maç − rakibin galibiyeti.
    let (opp_rows, error) = match &opp {
        None => (None, None),
        Some(Err(e)) => (None, Some(e.clone())),
        Some(Ok(c)) => match c.data.matchups.get(&lane) {
            Some(rows) => (Some(rows), None),
            // eşleşme yarısı alınamadıysa hata; u.gg'de veri yoksa yalnızca counter yok
            None => (None, c.error.clone().filter(|_| c.data.matchups.is_empty())),
        },
    };
    let counters = |champ: i64| {
        opp_rows
            .and_then(|rows| rows.iter().find(|r| r.opponent == champ))
            .is_some_and(|r| {
                let cand = Candidate {
                    champion: champ,
                    pool_index: 0,
                    wins: r.matches.saturating_sub(r.wins),
                    matches: r.matches,
                };
                scoring::rank(&[cand])[0].verdict == scoring::Verdict::Good
            })
    };
    let cands: Vec<scoring::DuoCandidate> = rows
        .iter()
        .filter_map(|r| {
            Some(scoring::DuoCandidate {
                champion: r.partner,
                wins: r.wins,
                matches: r.matches,
                own_winrate: solo(r.partner, lane)?,
                counter: counters(r.partner),
            })
        })
        .collect();
    let all = scoring::duo_synergies(&cands, partner_wr);
    let mut exclude = exclude;
    exclude.push(partner);
    exclude.extend(opponent);
    let best = scoring::best_synergies(&all, &exclude);
    Ok(DuoSynergy { all, best, error })
}

/// Ana pencereyi öne getirir (küçük penceredeki düğme).
#[tauri::command]
fn open_main_window(app: AppHandle) {
    launch::show_main(&app);
}

/// Küçük pencereyi kapatır (× düğmesi); sonraki şampiyon seçiminde yeniden açılır.
#[tauri::command]
async fn close_compact(app: AppHandle) {
    compact::close(&app);
}

/// Yalnızca geliştirmede: istemci olmadan küçük pencereyi açar. Dağıtım sürümünde hata döner.
#[tauri::command]
async fn open_compact_preview(app: AppHandle) -> Result<(), String> {
    if !cfg!(debug_assertions) {
        return Err("Yalnızca geliştirme sürümünde".into());
    }
    compact::open(&app).map_err(|e| format!("Küçük pencere açılamadı: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // İlk eklenti olmalı: ikinci açılış yeni süreç başlatmaz, mevcut pencereyi öne getirir.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| launch::show_main(app)))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![launch::MINIMIZED_ARG]),
        ))
        .manage(AppState::new())
        .manage(RolesState::default())
        .manage(RunesState::default())
        .manage(lcu::LcuState::default())
        // Tek setup: tepsi + pencere (LC-007), sonra LCU izleyicisi (LC-008).
        .setup(|app| {
            launch::setup(app)?;
            lcu::start(app.handle().clone());
            Ok(())
        })
        .on_window_event(launch::on_window_event)
        .invoke_handler(tauri::generate_handler![
            load_static,
            get_settings,
            save_settings,
            recommend,
            get_roles,
            launch::get_autostart,
            launch::set_autostart,
            get_champ_select_state,
            set_opponent,
            set_my_lane,
            champion_matchup,
            ban_suggestions,
            rune_options,
            import_loadout,
            duo_synergy,
            open_main_window,
            close_compact,
            open_compact_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
