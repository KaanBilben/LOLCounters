//! LoL istemcisi (LCU) bağlantısı: şampiyon seçimini okur. LCU ile ilgili bütün kod
//! yalnızca bu dosyadadır; biçim ve adresler: docs/data-sources.md "League client (LCU)".
//!
//! Boşta CPU harcamaz:
//! - İstemci kapalıyken tek bir iş parçacığı, kurulum klasörü için Windows'un değişiklik
//!   bildirimini süresiz bekler (`WaitForSingleObject(INFINITE)`); klasörde bir dosya
//!   değişmedikçe hiç uyanmaz.
//! - İstemci açıkken yalnızca WebSocket okunur; istemci bir olay göndermedikçe uyanılmaz.
//! - Lockfile göründükten sonra bağlantı en çok 4 kez denenir (0/1/2/4 sn), sonra yeniden
//!   klasör bildirimi beklenir. Sürekli yoklama yoktur.
//!
//! Riot kuralı: oturumdan yalnızca hücre numarası, şampiyon, seçim niyeti, atanmış rol ve
//! eylemler (seçim / ban) okunur; oyuncu adı, summonerId, puuid gibi alanlar hiç çözümlenmez.

use crate::settings::Lane;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};

/// Kurulum yolu (1. kaynak): Riot Client'ın ürün ayar dosyası.
const PRODUCT_SETTINGS: &str = "C:/ProgramData/Riot Games/Metadata/league_of_legends.live/league_of_legends.live.product_settings.yaml";
/// Kurulum yolu (2. kaynak): HKCU altındaki kaldırma kaydı, değer `InstallLocation`.
#[cfg(windows)]
const UNINSTALL_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Riot Game league_of_legends.live";
/// Kurulum yolu (3. kaynak): `associated_client` anahtarları.
const RIOT_CLIENT_INSTALLS: &str = "C:/ProgramData/Riot Games/RiotClientInstalls.json";
/// Kurulum yolu (son çare).
const DEFAULT_INSTALL: &str = "C:/Riot Games/League of Legends";

/// Riot'un LCU kök sertifikası (https://static.developer.riotgames.com/docs/lol/riotgames.pem).
const RIOT_PEM: &[u8] = include_bytes!("../certs/riotgames.pem");
/// Lockfile göründükten sonra bağlantı denemeleri arasındaki bekleme (sn).
const RETRY_DELAYS: [u64; 4] = [0, 1, 2, 4];
/// Aynı lockfile ile bağlantı koptuktan sonra en çok bu kadar yeniden bağlanılır.
const MAX_RECONNECTS: u8 = 2;
/// Rol payı bilinmeyen eşleştirmeler için taban değer (çarpım sıfır olmasın).
const SHARE_FLOOR: f64 = 0.001;

const SESSION_URI: &str = "/lol-champ-select/v1/session";
const PHASE_URI: &str = "/lol-gameflow/v1/gameflow-phase";
const SESSION_EVENT: &str = "OnJsonApiEvent_lol-champ-select_v1_session";
const PHASE_EVENT: &str = "OnJsonApiEvent_lol-gameflow_v1_gameflow-phase";
/// Arayüze gönderilen Tauri olayı.
pub const CHAMP_SELECT_EVENT: &str = "champ-select";

// --- Kurulum klasörü ve lockfile ---------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockfile {
    pub port: u16,
    pub password: String,
}

/// `LeagueClient:<pid>:<port>:<parola>:https`
pub fn parse_lockfile(text: &str) -> Option<Lockfile> {
    let mut parts = text.trim().split(':');
    let (_name, _pid) = (parts.next()?, parts.next()?);
    let port = parts.next()?.parse().ok()?;
    let password = parts.next()?;
    (!password.is_empty()).then(|| Lockfile { port, password: password.to_string() })
}

/// Ürün ayar dosyasındaki `product_install_full_path: "..."` satırı.
pub fn parse_product_settings(text: &str) -> Option<PathBuf> {
    text.lines().find_map(|line| {
        let v = line.trim().strip_prefix("product_install_full_path:")?;
        let v = v.trim().trim_matches(|c| c == '"' || c == '\'');
        (!v.is_empty()).then(|| PathBuf::from(v))
    })
}

/// `RiotClientInstalls.json` içindeki `associated_client` anahtarları (kurulum klasörleri).
pub fn parse_riot_client_installs(text: &str) -> Vec<PathBuf> {
    #[derive(Deserialize)]
    struct File {
        #[serde(default)]
        associated_client: HashMap<String, IgnoredAny>,
    }
    serde_json::from_str::<File>(text)
        .map(|f| f.associated_client.into_keys().map(PathBuf::from).collect())
        .unwrap_or_default()
}

#[cfg(windows)]
fn registry_install_dir() -> Option<PathBuf> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(UNINSTALL_KEY).ok()?;
    let v: String = key.get_value("InstallLocation").ok()?;
    let v = v.trim();
    (!v.is_empty()).then(|| PathBuf::from(v))
}

#[cfg(not(windows))]
fn registry_install_dir() -> Option<PathBuf> {
    None
}

/// LoL kurulum klasörü: ürün ayarı → kayıt defteri → RiotClientInstalls.json → varsayılan.
/// `LeagueClient.exe` bulunan ilk aday seçilir; hiçbirinde yoksa var olan ilk klasör.
pub fn find_install_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(t) = std::fs::read_to_string(PRODUCT_SETTINGS) {
        candidates.extend(parse_product_settings(&t));
    }
    candidates.extend(registry_install_dir());
    if let Ok(t) = std::fs::read_to_string(RIOT_CLIENT_INSTALLS) {
        candidates.extend(parse_riot_client_installs(&t));
    }
    candidates.push(PathBuf::from(DEFAULT_INSTALL));
    candidates
        .iter()
        .find(|d| d.join("LeagueClient.exe").is_file())
        .or_else(|| candidates.iter().find(|d| d.is_dir()))
        .cloned()
}

fn read_lockfile(dir: &Path) -> Option<Lockfile> {
    parse_lockfile(&std::fs::read_to_string(dir.join("lockfile")).ok()?)
}

// --- Şampiyon seçimi oturumu -------------------------------------------------

/// Arayüzün ihtiyacı olan sade şampiyon seçimi durumu.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectState {
    /// Atanmış rolüm (sıralı/seçmeli); kör seçimde `None`.
    pub my_lane: Option<Lane>,
    /// Kilitlediğim şampiyon.
    pub my_champion: Option<i64>,
    /// Kilitlemeden önce üzerinde durduğum / niyet ettiğim şampiyon.
    pub my_intent: Option<i64>,
    pub locked: bool,
    /// Görünen rakip şampiyonlar (0 olmayanlar), sırayla.
    pub enemies: Vec<i64>,
    /// Benim ban eylemim var ve henüz tamamlanmadı (planlama ya da ban aşaması), kilitlemedim.
    /// Kör seçim / ARAM gibi bansız modlarda hep `false`.
    pub ban_phase: bool,
    /// Tamamlanmış banlardaki şampiyonlar (iki takım).
    pub bans: Vec<i64>,
    /// Takım arkadaşlarımın seçtiği ya da gösterdiği şampiyonlar (yalnızca şampiyon numarası).
    pub allies: Vec<i64>,
}

/// `null` gelirse varsayılan değer.
fn null_default<'de, D: Deserializer<'de>, T: Default + Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    Option::<T>::deserialize(d).map(Option::unwrap_or_default)
}

/// Oturumun yalnızca gereken alanları (başka hiçbir alan çözümlenmez).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Session {
    local_player_cell_id: Option<i64>,
    #[serde(deserialize_with = "null_default")]
    my_team: Vec<Member>,
    #[serde(deserialize_with = "null_default")]
    their_team: Vec<Member>,
    #[serde(deserialize_with = "null_default")]
    actions: Vec<Vec<Action>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Member {
    #[serde(deserialize_with = "null_default")]
    cell_id: i64,
    #[serde(deserialize_with = "null_default")]
    champion_id: i64,
    #[serde(deserialize_with = "null_default")]
    champion_pick_intent: i64,
    #[serde(deserialize_with = "null_default")]
    assigned_position: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Action {
    #[serde(deserialize_with = "null_default")]
    actor_cell_id: i64,
    #[serde(deserialize_with = "null_default")]
    champion_id: i64,
    #[serde(deserialize_with = "null_default")]
    completed: bool,
    #[serde(rename = "type", deserialize_with = "null_default")]
    kind: String,
}

fn lane_from_position(p: &str) -> Option<Lane> {
    match p.to_ascii_lowercase().as_str() {
        "top" => Some(Lane::Top),
        "jungle" => Some(Lane::Jungle),
        "middle" | "mid" => Some(Lane::Mid),
        "bottom" => Some(Lane::Adc),
        "utility" | "support" => Some(Lane::Support),
        _ => None,
    }
}

impl Session {
    fn reduce(&self) -> ChampSelectState {
        let nz = |v: i64| (v != 0).then_some(v);
        let cell = self.local_player_cell_id;
        let me = cell.and_then(|c| self.my_team.iter().find(|m| m.cell_id == c));
        // Kilit: benim tamamlanmış "pick" eylemim; şampiyonu myTeam ile çapraz kontrol.
        let my_champion = cell
            .and_then(|c| {
                self.actions
                    .iter()
                    .flatten()
                    .find(|a| a.actor_cell_id == c && a.kind == "pick" && a.completed)
            })
            .and_then(|a| nz(a.champion_id).or_else(|| me.and_then(|m| nz(m.champion_id))));
        let locked = my_champion.is_some();
        let my_intent = if locked {
            None
        } else {
            me.and_then(|m| nz(m.champion_pick_intent).or_else(|| nz(m.champion_id)))
        };
        let actions = || self.actions.iter().flatten();
        let ban_phase = !locked
            && cell.is_some_and(|c| {
                actions().any(|a| a.actor_cell_id == c && a.kind == "ban" && !a.completed)
            });
        let mut bans: Vec<i64> = Vec::new();
        for a in actions().filter(|a| a.kind == "ban" && a.completed) {
            if let Some(k) = nz(a.champion_id)
                && !bans.contains(&k)
            {
                bans.push(k);
            }
        }
        let allies = self
            .my_team
            .iter()
            .filter(|m| Some(m.cell_id) != cell)
            .filter_map(|m| nz(m.champion_id).or_else(|| nz(m.champion_pick_intent)))
            .collect();
        ChampSelectState {
            my_lane: me.and_then(|m| lane_from_position(&m.assigned_position)),
            my_champion,
            my_intent,
            locked,
            enemies: self.their_team.iter().filter_map(|m| nz(m.champion_id)).collect(),
            ban_phase,
            bans,
            allies,
        }
    }
}

/// `/lol-champ-select/v1/session` verisini sade duruma çevirir.
pub fn parse_session(v: &serde_json::Value) -> ChampSelectState {
    Session::deserialize(v).unwrap_or_default().reduce()
}

/// Rakip tahmininin girdisi. Tahmin yalnızca bu değişince yeniden hesaplanır (niyetim, kilidim
/// gibi diğer alanlar değişince değil).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuessInput {
    /// Görünen rakipler, sırayla.
    pub enemies: Vec<i64>,
    /// Lane'im: atanmış rol, yoksa elle seçilen lane.
    pub lane: Lane,
}

/// Atanmış rol yoksa (kör seçim, pozisyonsuz turnuva seçimi) elle seçilen lane kullanılır.
/// Rakip görünmüyorsa ya da lane bilinmiyorsa `None` (tahmin yok).
pub fn guess_input(state: &ChampSelectState, manual_lane: Option<Lane>) -> Option<GuessInput> {
    let lane = state.my_lane.or(manual_lane)?;
    (!state.enemies.is_empty()).then(|| GuessInput { enemies: state.enemies.clone(), lane })
}

/// Rakiplerin rollerini, rol paylarının çarpımını en büyük yapan bire bir eşleştirmeyle
/// tahmin eder (en çok 5! = 120 deneme) ve benim lane'imdeki rakibi döner.
/// Bu rakibin o lane için verisi yoksa tahmin yapılmaz (`None`).
pub fn guess_opponent(
    enemies: &[i64],
    my_lane: Lane,
    shares: &HashMap<i64, HashMap<Lane, f64>>,
) -> Option<i64> {
    let mut list: Vec<i64> = Vec::new();
    for &e in enemies {
        if !list.contains(&e) && list.len() < Lane::ALL.len() {
            list.push(e);
        }
    }
    let share = |e: i64, l: Lane| shares.get(&e).and_then(|m| m.get(&l)).copied().unwrap_or(0.0);
    let cost: Vec<[f64; 5]> = list
        .iter()
        .map(|&e| Lane::ALL.map(|l| share(e, l).max(SHARE_FLOOR).ln()))
        .collect();
    let mut best = (f64::NEG_INFINITY, Vec::new());
    best_assignment(&cost, 0, &mut Vec::new(), &mut best);
    let lane_idx = Lane::ALL.iter().position(|&l| l == my_lane)?;
    let i = best.1.iter().position(|&l| l == lane_idx)?;
    (share(list[i], my_lane) > SHARE_FLOOR).then_some(list[i])
}

/// `cost[i][l]`: i. rakibin l. lane'de olmasının log payı; en iyi atama `best`'e yazılır.
fn best_assignment(
    cost: &[[f64; 5]],
    used: u8,
    cur: &mut Vec<usize>,
    best: &mut (f64, Vec<usize>),
) {
    let i = cur.len();
    if i == cost.len() {
        let total: f64 = cur.iter().enumerate().map(|(e, &l)| cost[e][l]).sum();
        if total > best.0 {
            *best = (total, cur.clone());
        }
        return;
    }
    for l in 0..5 {
        if used & (1 << l) == 0 {
            cur.push(l);
            best_assignment(cost, used | (1 << l), cur, best);
            cur.pop();
        }
    }
}

// --- Durum ve arayüz olayı ---------------------------------------------------

/// Arayüze giden durum (komut `get_champ_select_state` ve olay `champ-select`).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectPayload {
    /// LoL istemcisine bağlı mı.
    pub connected: bool,
    /// Şampiyon seçimi sürüyor mu.
    pub active: bool,
    #[serde(flatten)]
    pub state: ChampSelectState,
    /// Rakibim: kullanıcının seçtiği, yoksa tahmin; bilinmiyorsa `None`.
    pub opponent: Option<i64>,
    /// `opponent` kullanıcının seçimi mi (değilse tahmin).
    pub opponent_chosen: bool,
}

#[derive(Debug, Default)]
struct Inner {
    connected: bool,
    /// Son bilinen oyun akışı aşaması (ör. "ChampSelect"); bağlantıdan beri gelmediyse `None`.
    phase: Option<String>,
    active: bool,
    state: ChampSelectState,
    chosen: Option<i64>,
    guess: Option<i64>,
    last_sent: Option<ChampSelectPayload>,
    /// Elle seçilen lane (atanmış rol yokken kullanılır).
    manual_lane: Option<Lane>,
    /// Şampiyon seçimi sayacı: arka plandaki tahmin bitmiş bir seçime yazmasın.
    round: u64,
    /// Son tahmini istenen girdi; aynı girdiyle yeniden hesaplanmaz.
    guess_for: Option<GuessInput>,
    /// Bu seçimde lane verisi alınamadı: her değişiklikte yeniden indirilmez.
    roles_failed: bool,
}

impl Inner {
    fn payload(&self) -> ChampSelectPayload {
        ChampSelectPayload {
            connected: self.connected,
            active: self.active,
            state: self.state.clone(),
            opponent: self.chosen.or(self.guess),
            opponent_chosen: self.chosen.is_some(),
        }
    }

    fn apply(&mut self, change: Change) {
        match change {
            Change::Connected(c) => {
                self.connected = c;
                self.phase = None;
                self.active = false;
            }
            Change::Phase(p) => {
                self.active = p == "ChampSelect";
                self.phase = Some(p);
            }
            Change::Session(Some(s)) => {
                // Aşama bilinmiyorsa oturumun kendisi seçimin sürdüğünü gösterir; oyun
                // başladıktan sonra gelen gecikmiş bir güncelleme pencereyi yeniden açmasın.
                if self.phase.as_deref().is_none_or(|p| p == "ChampSelect") {
                    self.active = true;
                    self.state = s;
                }
            }
            Change::Session(None) => self.active = false,
            Change::Opponent(o) => self.chosen = o,
            Change::Lane(l) => self.manual_lane = l,
        }
        if !self.active {
            self.state = ChampSelectState::default();
            self.chosen = None;
            self.guess = None;
            self.manual_lane = None;
            self.guess_for = None;
            self.roles_failed = false;
        }
    }

    /// Tahmin girdisi değiştiyse arka planda hesaplanacak işi döner. Lane değişince ya da girdi
    /// kalmayınca eski tahmin hemen silinir; yalnızca rakipler değiştiyse yenisi gelene kadar kalır.
    fn guess_job(&mut self) -> Option<(u64, GuessInput)> {
        if !self.active {
            return None;
        }
        let input = guess_input(&self.state, self.manual_lane);
        if input == self.guess_for {
            return None;
        }
        if input.as_ref().map(|i| i.lane) != self.guess_for.as_ref().map(|i| i.lane) {
            self.guess = None;
        }
        self.guess_for = input.clone();
        if self.roles_failed {
            return None;
        }
        input.map(|i| (self.round, i))
    }
}

/// LCU durumunu tutar (Tauri `manage` ile).
#[derive(Debug, Default)]
pub struct LcuState(Mutex<Inner>);

impl LcuState {
    pub fn payload(&self) -> ChampSelectPayload {
        crate::lock(&self.0).payload()
    }
}

enum Change {
    Connected(bool),
    Phase(String),
    Session(Option<ChampSelectState>),
    Opponent(Option<i64>),
    /// Elle seçilen lane (`None`: atanmış role dön).
    Lane(Option<Lane>),
}

/// Durumu günceller; seçim başlayınca/bitince kancaları çağırır; sade durum değiştiyse
/// `champ-select` olayını gönderir. Hiçbir ağ isteğini beklemez: olay döngüsü (`serve`) bir
/// sonraki çerçeveye (ör. oyun başlangıcı) hemen geçer. Rakip tahmininin girdisi değiştiyse
/// tahmin ayrı bir görevde hesaplanır (`guess_task`).
async fn update(app: &AppHandle, change: Change) -> ChampSelectPayload {
    let st = app.state::<LcuState>();
    let (edge, job) = {
        let mut g = crate::lock(&st.0);
        let was_active = g.active;
        g.apply(change);
        let edge = (was_active != g.active).then_some(g.active);
        if edge == Some(true) {
            g.round += 1;
        }
        (edge, g.guess_job())
    };
    match edge {
        Some(true) => crate::on_champ_select_start(app),
        Some(false) => crate::on_champ_select_end(app),
        None => {}
    }
    if let Some((round, input)) = job {
        tauri::async_runtime::spawn(guess_task(app.clone(), round, input));
    }
    emit_if_changed(app)
}

/// Rakip tahmini: rol payları (bellek → disk → bir kez ağ) gelince, seçim ve girdi hâlâ
/// aynıysa yazılır. Veri alınamazsa bu seçimde yeniden denenmez; rakip elle seçilir.
async fn guess_task(app: AppHandle, round: u64, input: GuessInput) {
    let shares = crate::lcu_role_shares(&app).await;
    {
        let st = app.state::<LcuState>();
        let mut g = crate::lock(&st.0);
        if !g.active || g.round != round {
            return;
        }
        match shares {
            Ok(shares) if g.guess_for.as_ref() == Some(&input) => {
                g.guess = guess_opponent(&input.enemies, input.lane, &shares);
            }
            Ok(_) => return,
            Err(_) => g.roles_failed = true,
        }
    }
    emit_if_changed(&app);
}

/// Arayüze giden durum son gönderilenden farklıysa `champ-select` olayını gönderir.
fn emit_if_changed(app: &AppHandle) -> ChampSelectPayload {
    let (payload, send) = {
        let st = app.state::<LcuState>();
        let mut g = crate::lock(&st.0);
        let p = g.payload();
        let send = g.last_sent.as_ref() != Some(&p);
        if send {
            g.last_sent = Some(p.clone());
        }
        (p, send)
    };
    if send {
        let _ = app.emit(CHAMP_SELECT_EVENT, &payload);
    }
    payload
}

/// Kullanıcı rakibi elle seçti (`None`: tahmine dön). Seçim bitince sıfırlanır.
pub async fn set_opponent(app: &AppHandle, champion: Option<i64>) -> ChampSelectPayload {
    update(app, Change::Opponent(champion)).await
}

/// Kullanıcı lane'ini elle seçti: atanmış rol yokken tahmin bununla yapılır (`None`: geri al).
/// Seçim bitince sıfırlanır.
pub async fn set_my_lane(app: &AppHandle, lane: Option<Lane>) -> ChampSelectPayload {
    update(app, Change::Lane(lane)).await
}

// --- TLS ---------------------------------------------------------------------

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// Yalnızca Riot kök sertifikasına güvenen yapılandırma (kurulamazsa `None`).
fn strict_tls() -> Option<Arc<ClientConfig>> {
    static CFG: OnceLock<Option<Arc<ClientConfig>>> = OnceLock::new();
    CFG.get_or_init(|| {
        let mut roots = RootCertStore::empty();
        roots.add(CertificateDer::from_pem_slice(RIOT_PEM).ok()?).ok()?;
        let cfg = ClientConfig::builder_with_provider(provider())
            .with_safe_default_protocol_versions()
            .ok()?
            .with_root_certificates(roots)
            .with_no_client_auth();
        Some(Arc::new(cfg))
    })
    .clone()
}

/// Yedek doğrulayıcı: sertifikayı yalnızca hedef 127.0.0.1 (loopback) olduğu için kabul eder.
///
/// Neden: istemcinin sertifikası Riot köküyle rustls/webpki'nin kabul ettiği biçimde
/// doğrulanamayabilir (eski X.509 v1 / SHA-1 sertifikalar). Bağlantı makineden hiç çıkmaz;
/// port ve parola yalnızca kullanıcının okuyabildiği lockfile'dan gelir. O portu dinleyebilen
/// bir saldırgan zaten aynı makinede kod çalıştırıyor ve lockfile'ı da okuyabiliyor demektir;
/// bu durumda sertifika denetimi ek bir koruma sağlamaz. Başka hiçbir adres kabul edilmez.
#[derive(Debug)]
struct LoopbackOnly(Arc<rustls::crypto::CryptoProvider>);

impl ServerCertVerifier for LoopbackOnly {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        match server_name {
            ServerName::IpAddress(ip) if std::net::IpAddr::from(*ip).is_loopback() => {
                Ok(ServerCertVerified::assertion())
            }
            _ => Err(rustls::Error::General("LCU yalnızca 127.0.0.1".into())),
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

fn loopback_tls() -> Arc<ClientConfig> {
    static CFG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    CFG.get_or_init(|| {
        let p = provider();
        let cfg = ClientConfig::builder_with_provider(p.clone())
            .with_safe_default_protocol_versions()
            .expect("ring TLS sürümleri")
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(LoopbackOnly(p)))
            .with_no_client_auth();
        Arc::new(cfg)
    })
    .clone()
}

/// Riot kökü ile el sıkışma bir kez reddedildiyse bu oturumda doğrudan yedek kullanılır.
static USE_LOOPBACK_TLS: AtomicBool = AtomicBool::new(false);

// --- Bağlantı ----------------------------------------------------------------

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

fn basic_auth(password: &str) -> String {
    let token = base64::engine::general_purpose::STANDARD.encode(format!("riot:{password}"));
    format!("Basic {token}")
}

async fn ws_connect(lf: &Lockfile, tls: Arc<ClientConfig>) -> Result<Ws, tungstenite::Error> {
    let mut req = format!("wss://127.0.0.1:{}/", lf.port).into_client_request()?;
    let auth = HeaderValue::from_str(&basic_auth(&lf.password))
        .map_err(|e| tungstenite::Error::HttpFormat(e.into()))?;
    req.headers_mut().insert("Authorization", auth);
    let (ws, _) =
        tokio_tungstenite::connect_async_tls_with_config(req, None, true, Some(Connector::Rustls(tls)))
            .await?;
    Ok(ws)
}

/// Önce yalnızca Riot köküyle; TLS el sıkışması reddedilirse loopback yedeğiyle bağlanır.
async fn open_socket(lf: &Lockfile) -> Result<(Ws, Arc<ClientConfig>), tungstenite::Error> {
    if !USE_LOOPBACK_TLS.load(Ordering::Relaxed)
        && let Some(cfg) = strict_tls()
    {
        match ws_connect(lf, cfg.clone()).await {
            Ok(ws) => return Ok((ws, cfg)),
            // tokio-rustls, rustls hatalarını InvalidData olarak döner (sertifika reddi).
            Err(tungstenite::Error::Io(e)) if e.kind() == std::io::ErrorKind::InvalidData => {
                USE_LOOPBACK_TLS.store(true, Ordering::Relaxed);
            }
            Err(e) => return Err(e),
        }
    }
    let cfg = loopback_tls();
    Ok((ws_connect(lf, cfg.clone()).await?, cfg))
}

async fn connect_with_retry(lf: &Lockfile) -> Option<(Ws, Arc<ClientConfig>)> {
    for wait in RETRY_DELAYS {
        if wait > 0 {
            tokio::time::sleep(Duration::from_secs(wait)).await;
        }
        if let Ok(x) = open_socket(lf).await {
            return Some(x);
        }
    }
    None
}

/// Bağlanınca bir kez: güncel aşama, seçimdeysek oturum (WebSocket yalnızca değişiklik gönderir).
async fn rest_sync(app: &AppHandle, lf: &Lockfile, tls: &ClientConfig) {
    // `no_proxy`: HTTPS_PROXY ya da Windows proxy ayarı LCU isteğini (parola başlığıyla) bir
    // proxy'ye göndermesin; yedek doğrulayıcı yalnızca hedef adı denetlediği için bu şarttır.
    let Ok(client) = reqwest::Client::builder()
        .no_proxy()
        .use_preconfigured_tls(tls.clone())
        .timeout(Duration::from_secs(5))
        .build()
    else {
        return;
    };
    let base = format!("https://127.0.0.1:{}", lf.port);
    let get = |uri: &str| client.get(format!("{base}{uri}")).basic_auth("riot", Some(&lf.password));
    let Ok(res) = get(PHASE_URI).send().await else { return };
    let Ok(phase) = res.json::<String>().await else { return };
    let in_select = phase == "ChampSelect";
    update(app, Change::Phase(phase)).await;
    if in_select
        && let Ok(res) = get(SESSION_URI).send().await
        && res.status().is_success()
        && let Ok(s) = res.json::<Session>().await
    {
        update(app, Change::Session(Some(s.reduce()))).await;
    }
}

/// `[8, "<olay>", {"uri": ..., "eventType": ..., "data": ...}]` çerçevesini çözer.
fn parse_frame(text: &str) -> Option<Change> {
    #[derive(Deserialize)]
    struct Head {
        uri: String,
        #[serde(rename = "eventType", default)]
        event_type: String,
    }
    #[derive(Deserialize)]
    struct Body<T> {
        data: Option<T>,
    }
    let (op, _, head): (u8, IgnoredAny, Head) = serde_json::from_str(text).ok()?;
    if op != 8 {
        return None;
    }
    match head.uri.as_str() {
        PHASE_URI => {
            let (_, _, b): (IgnoredAny, IgnoredAny, Body<String>) = serde_json::from_str(text).ok()?;
            Some(Change::Phase(b.data?))
        }
        SESSION_URI if head.event_type == "Delete" => Some(Change::Session(None)),
        SESSION_URI => {
            let (_, _, b): (IgnoredAny, IgnoredAny, Body<Session>) = serde_json::from_str(text).ok()?;
            Some(Change::Session(Some(b.data?.reduce())))
        }
        _ => None,
    }
}

/// Bağlantı açıkken: abone ol, bir kez eşitle, kapanana kadar olayları işle.
async fn serve(app: &AppHandle, lf: &Lockfile, mut ws: Ws, tls: Arc<ClientConfig>) {
    for event in [SESSION_EVENT, PHASE_EVENT] {
        let frame = format!(r#"[5,"{event}"]"#);
        if ws.send(Message::Text(frame.into())).await.is_err() {
            return;
        }
    }
    update(app, Change::Connected(true)).await;
    rest_sync(app, lf, &tls).await;
    while let Some(msg) = ws.next().await {
        match msg {
            Ok(Message::Text(t)) => {
                if let Some(change) = parse_frame(t.as_str()) {
                    update(app, change).await;
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => {}
        }
    }
    update(app, Change::Connected(false)).await;
}

/// Klasör bildirimi geldikçe lockfile'a bakar ve bağlanır.
async fn run(app: AppHandle, dir: PathBuf, mut signals: mpsc::Receiver<()>) {
    let mut current: Option<Lockfile> = None;
    let mut failed = false;
    let mut reconnects = 0u8;
    let mut recheck = false;
    loop {
        if !recheck && signals.recv().await.is_none() {
            return;
        }
        recheck = false;
        let Some(lf) = read_lockfile(&dir) else { continue };
        if current.as_ref() != Some(&lf) {
            current = Some(lf.clone());
            failed = false;
            reconnects = 0;
        }
        if failed {
            continue;
        }
        match connect_with_retry(&lf).await {
            Some((ws, tls)) => {
                serve(&app, &lf, ws, tls).await;
                // İstemci kapanıyorsa lockfile silinir; açık kaldıysa birkaç kez yeniden bağlan.
                reconnects += 1;
                if reconnects <= MAX_RECONNECTS {
                    recheck = true;
                } else {
                    failed = true;
                }
            }
            None => failed = true,
        }
    }
}

/// Kurulum klasöründe dosya adı ya da yazma değişikliği oldukça `signals`'a haber verir.
/// `WaitForSingleObject(INFINITE)`: değişiklik yokken iş parçacığı hiç uyanmaz.
#[cfg(windows)]
fn watch_dir(dir: &Path, signals: &mpsc::Sender<()>) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{INVALID_HANDLE_VALUE, WAIT_OBJECT_0};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FindCloseChangeNotification,
        FindFirstChangeNotificationW, FindNextChangeNotification,
    };
    use windows_sys::Win32::System::Threading::{INFINITE, WaitForSingleObject};

    let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: `wide` sıfırla biten geçerli bir UTF-16 yol ve çağrı süresince yaşıyor;
    // tutamaç yalnızca bu iş parçacığında kullanılır ve sonunda kapatılır.
    unsafe {
        let handle = FindFirstChangeNotificationW(
            wide.as_ptr(),
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE,
        );
        if handle == INVALID_HANDLE_VALUE {
            return;
        }
        while WaitForSingleObject(handle, INFINITE) == WAIT_OBJECT_0 {
            // Kapasite 1: üst üste gelen bildirimler tek kontrole indirgenir.
            if let Err(mpsc::error::TrySendError::Closed(_)) = signals.try_send(()) {
                break;
            }
            if FindNextChangeNotification(handle) == 0 {
                break;
            }
        }
        FindCloseChangeNotification(handle);
    }
}

#[cfg(not(windows))]
fn watch_dir(_dir: &Path, _signals: &mpsc::Sender<()>) {}

/// Uygulama açılışında bir kez çağrılır. Kurulum klasörü yoksa hiçbir şey yapmaz.
pub fn start(app: AppHandle) {
    let _ = std::thread::Builder::new()
        .name("lcu-watch".into())
        .spawn(move || {
            let Some(dir) = find_install_dir() else { return };
            let (tx, rx) = mpsc::channel(1);
            let _ = tx.try_send(()); // açılışta bir kontrol
            tauri::async_runtime::spawn(run(app, dir.clone(), rx));
            watch_dir(&dir, &tx);
        });
}
