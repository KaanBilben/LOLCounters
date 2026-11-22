//! u.gg (resmi olmayan kaynak). u.gg ile ilgili her şey yalnızca bu dosyadadır.
//! Biçim: docs/data-sources.md.

use crate::settings::Lane;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

const VERSIONS_URL: &str =
    "https://static.bigbrain.gg/assets/lol/riot_patch_update/prod/ugg/ugg-api-versions.json";
const STATS_BASE: &str = "https://stats2.u.gg/lol/1.5";
const QUEUE: &str = "ranked_solo_5x5";
const DEFAULT_API_VERSION: &str = "1.5.0";
/// Bölge: 12 = Dünya.
const REGION: &str = "12";
/// Rank tercih sırası: Emerald+, Platinum+, Hepsi.
const RANKS: [&str; 3] = ["17", "10", "8"];
/// Lane filtresi: şampiyonun bir lane'deki maçı, beş lane'deki toplam maçının en az
/// bu payıysa şampiyon o lane'de listelenir (0.10 = %10). LoL istemcisinin rol
/// filtresine en yakın sonuç (ör. Darius Orman'da, Brand Üst'te çıkmaz); bkz. docs/data-sources.md.
pub const ROLE_MIN_SHARE: f64 = 0.10;

/// u.gg rol anahtarı.
fn role_key(lane: Lane) -> &'static str {
    match lane {
        Lane::Jungle => "1",
        Lane::Support => "2",
        Lane::Adc => "3",
        Lane::Top => "4",
        Lane::Mid => "5",
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchupRow {
    pub opponent: i64,
    pub wins: u32,
    pub matches: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunePage {
    pub primary_tree: i64,
    pub secondary_tree: i64,
    /// u.gg sayısal sıralı verir (ağaca göre değil); arayüz Rune.tree/slot ile gruplar.
    pub perks: Vec<i64>,
    pub shards: Vec<i64>,
    pub matches: u32,
    pub wins: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Build {
    pub runes: RunePage,
    pub spells: Vec<i64>,
    pub starting_items: Vec<i64>,
    pub core_items: Vec<i64>,
    pub skill_max_order: String,
    /// 4./5./6. eşya yuvaları; her yuvada en çok oynanan 2 seçenek.
    pub late_options: Vec<Vec<i64>>,
    pub matches: u32,
    pub wins: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UggPatch {
    /// ör. "16_23"
    pub patch: String,
    pub matchups_version: String,
    pub overview_version: String,
    /// Lane filtresi dosyasının sürümü; eski `patch.json` kayıtlarında yoksa varsayılan.
    #[serde(default = "default_api_version")]
    pub ranking_version: String,
}

fn default_api_version() -> String {
    DEFAULT_API_VERSION.to_string()
}

// ---------- JSON yardımcıları ----------

fn as_u32(v: &Value) -> Option<u32> {
    v.as_u64()
        .or_else(|| v.as_f64().filter(|f| *f >= 0.0).map(|f| f as u64))
        .map(|n| n.min(u32::MAX as u64) as u32)
}

fn as_id(v: &Value) -> Option<i64> {
    v.as_i64()
        .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
}

fn id_list(v: Option<&Value>) -> Vec<i64> {
    v.and_then(Value::as_array)
        .map(|a| a.iter().filter_map(as_id).collect())
        .unwrap_or_default()
}

/// `region → rank → role → [veri, zaman]` içinden, rank sırasıyla `parse`'ın kabul ettiği ilk veri.
fn per_lane<T>(v: &Value, parse: impl Fn(&Value) -> Option<T>) -> HashMap<Lane, T> {
    let mut out = HashMap::new();
    let Some(region) = v.get(REGION) else {
        return out;
    };
    for lane in Lane::ALL {
        let found = RANKS.iter().find_map(|rank| {
            let data = region.get(rank)?.get(role_key(lane))?.get(0)?;
            parse(data)
        });
        if let Some(t) = found {
            out.insert(lane, t);
        }
    }
    out
}

// ---------- Ayrıştırma ----------

fn parse_matchup_rows(data: &Value) -> Option<Vec<MatchupRow>> {
    let rows: Vec<MatchupRow> = data
        .as_array()?
        .iter()
        .filter_map(|row| {
            let row = row.as_array()?;
            let opponent = row.first()?.as_i64()?;
            let losses = as_u32(row.get(1)?)?;
            let matches = as_u32(row.get(2)?)?;
            Some(MatchupRow {
                opponent,
                wins: matches.saturating_sub(losses),
                matches,
            })
        })
        .collect();
    (!rows.is_empty()).then_some(rows)
}

/// Eşleşme satırı `[rakip, kayıp, maç, ...]` → galibiyet = maç − kayıp.
pub fn parse_matchups(v: &Value) -> HashMap<Lane, Vec<MatchupRow>> {
    per_lane(v, parse_matchup_rows)
}

/// `[[id, galibiyet, maç], ...]` → maça göre azalan, ilk 2 id.
fn top_options(slot: Option<&Value>) -> Vec<i64> {
    let mut opts: Vec<(i64, u32)> = slot
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|o| {
                    let o = o.as_array()?;
                    Some((as_id(o.first()?)?, as_u32(o.get(2)?)?))
                })
                .collect()
        })
        .unwrap_or_default();
    opts.sort_by_key(|a| std::cmp::Reverse(a.1));
    opts.into_iter().take(2).map(|(id, _)| id).collect()
}

fn parse_build(data: &Value) -> Option<Build> {
    let a = data.as_array()?;

    // 0: [maç, galibiyet, ana ağaç, ikincil ağaç, [rünler]]
    let r = a.first()?.as_array()?;
    let primary_tree = as_id(r.get(2)?)?;
    let secondary_tree = as_id(r.get(3)?)?;
    let perks = id_list(r.get(4));
    if perks.is_empty() {
        return None;
    }
    // 8: [maç, galibiyet, ["5005", ...]]
    let shards = id_list(a.get(8).and_then(|s| s.get(2)));
    let runes = RunePage {
        primary_tree,
        secondary_tree,
        perks,
        shards,
        matches: r.first().and_then(as_u32).unwrap_or(0),
        wins: r.get(1).and_then(as_u32).unwrap_or(0),
    };

    // 1/2/3: [maç, galibiyet, [id'ler]]
    let list_at = |i: usize| id_list(a.get(i).and_then(|x| x.get(2)));

    // 4: [maç, galibiyet, [sıra], "QEW"]
    let skill_max_order = a
        .get(4)
        .and_then(|x| x.get(3))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    // 5: yuva listeleri; ilk 3 yuva (4./5./6. eşya)
    let slots = a.get(5).and_then(Value::as_array);
    let late_options = (0..3)
        .map(|i| top_options(slots.and_then(|s| s.get(i))))
        .collect();

    // 6: [galibiyet, maç]
    let total = a.get(6);
    let wins = total.and_then(|t| t.get(0)).and_then(as_u32).unwrap_or(0);
    let matches = total.and_then(|t| t.get(1)).and_then(as_u32).unwrap_or(0);

    Some(Build {
        runes,
        spells: list_at(1),
        starting_items: list_at(2),
        core_items: list_at(3),
        skill_max_order,
        late_options,
        matches,
        wins,
    })
}

pub fn parse_overview(v: &Value) -> HashMap<Lane, Build> {
    per_lane(v, parse_build)
}

// ---------- Yama ----------

/// "16.23.1" → "16_23".
fn patch_from_version(ddragon_version: &str) -> Option<String> {
    let mut parts = ddragon_version.split('.');
    let major = parts.next()?.trim();
    let minor = parts.next()?.trim();
    if major.is_empty() || minor.is_empty() {
        return None;
    }
    Some(format!("{major}_{minor}"))
}

fn patch_order(p: &str) -> (u32, u32) {
    let mut it = p.split('_').map(|x| x.parse::<u32>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0))
}

/// Data Dragon sürümünden tahmin edilen yama, varsayılan API sürümleriyle
/// (ağ yokken ya da u.gg listesi okunamadığında).
pub fn fallback_patch(ddragon_version: &str) -> UggPatch {
    UggPatch {
        patch: patch_from_version(ddragon_version)
            .unwrap_or_else(|| ddragon_version.replace('.', "_")),
        matchups_version: DEFAULT_API_VERSION.to_string(),
        overview_version: DEFAULT_API_VERSION.to_string(),
        ranking_version: DEFAULT_API_VERSION.to_string(),
    }
}

impl UggPatch {
    /// Görüntü biçimi: "16_23" → "16.23".
    pub fn display(&self) -> String {
        self.patch.replace('_', ".")
    }

    /// Bu yama, verilen Data Dragon sürümünün kendi yaması mı (u.gg onu listeliyor mu)?
    /// Değilse u.gg listedeki eski bir yamaya düşülmüştür.
    pub fn is_current_for(&self, ddragon_version: &str) -> bool {
        self.patch == fallback_patch(ddragon_version).patch
    }
}

fn choose_patch(versions: &Value, ddragon_version: &str) -> Result<UggPatch, String> {
    let map = versions
        .as_object()
        .ok_or("u.gg yama listesi okunamadı")?;
    let wanted = patch_from_version(ddragon_version);
    let patch = match wanted {
        Some(p) if map.contains_key(&p) => p,
        _ => map
            .keys()
            .max_by_key(|k| patch_order(k))
            .cloned()
            .ok_or("u.gg yama listesi boş")?,
    };
    let entry = map.get(&patch);
    let ver = |name: &str| {
        entry
            .and_then(|e| e.get(name))
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_API_VERSION)
            .to_string()
    };
    Ok(UggPatch {
        matchups_version: ver("matchups"),
        overview_version: ver("overview"),
        ranking_version: ver("champion_ranking"),
        patch,
    })
}

// ---------- Ağ ----------

/// Yama listesi isteğinin süre sınırı (veri dosyaları istemcinin 20 sn'sini kullanır).
const VERSIONS_TIMEOUT: Duration = Duration::from_secs(5);

/// İndirilen dosyadan yalnızca dünya bölgesi ve kullanılan rank'lar okunur;
/// geri kalanı (~%85) ağaç kurulmadan atlanır. Anahtarlar `REGION` / `RANKS` ile aynıdır.
#[derive(Deserialize)]
struct WorldOnly {
    #[serde(rename = "12")]
    world: Option<WorldRanks>,
}

#[derive(Deserialize)]
struct WorldRanks {
    #[serde(rename = "17")]
    emerald_plus: Option<Value>,
    #[serde(rename = "10")]
    platinum_plus: Option<Value>,
    #[serde(rename = "8")]
    all: Option<Value>,
}

impl WorldOnly {
    /// `parse_*` fonksiyonlarının beklediği `bölge → rank → ...` biçimine geri çevirir.
    fn into_value(self) -> Value {
        let mut ranks = serde_json::Map::new();
        if let Some(w) = self.world {
            for (rank, v) in RANKS.iter().zip([w.emerald_plus, w.platinum_plus, w.all]) {
                if let Some(v) = v {
                    ranks.insert((*rank).to_string(), v);
                }
            }
        }
        let mut root = serde_json::Map::new();
        root.insert(REGION.to_string(), Value::Object(ranks));
        Value::Object(root)
    }
}

/// 4xx (408/429 hariç): u.gg'de bu dosya yok (ör. yamada verisi olmayan şampiyon).
fn is_missing(status: reqwest::StatusCode) -> bool {
    status.is_client_error()
        && status != reqwest::StatusCode::REQUEST_TIMEOUT
        && status != reqwest::StatusCode::TOO_MANY_REQUESTS
}

/// `Ok(None)`: dosya u.gg'de yok. Ağ hatası ve 5xx `Err`.
async fn get_typed<T: DeserializeOwned>(req: reqwest::RequestBuilder) -> Result<Option<T>, String> {
    let resp = req
        .send()
        .await
        .map_err(|e| format!("u.gg isteği başarısız: {e}"))?;
    let status = resp.status();
    if is_missing(status) {
        return Ok(None);
    }
    if !status.is_success() {
        return Err(format!("u.gg yanıtı: {status}"));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("u.gg verisi indirilemedi: {e}"))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| format!("u.gg verisi okunamadı: {e}"))
}

/// Şampiyon dosyası; u.gg'de yoksa boş kök (bütün lane'ler boş çıkar).
async fn get_world(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    Ok(get_typed::<WorldOnly>(client.get(url))
        .await?
        .map(WorldOnly::into_value)
        .unwrap_or_default())
}

pub async fn resolve_patch(
    client: &reqwest::Client,
    ddragon_version: &str,
) -> Result<UggPatch, String> {
    let versions = get_typed::<Value>(client.get(VERSIONS_URL).timeout(VERSIONS_TIMEOUT))
        .await?
        .ok_or("u.gg yama listesi bulunamadı")?;
    choose_patch(&versions, ddragon_version)
}

/// u.gg'de bu şampiyonun dosyası yoksa (4xx) boş harita döner.
pub async fn fetch_matchups(
    client: &reqwest::Client,
    p: &UggPatch,
    champion_key: i64,
) -> Result<HashMap<Lane, Vec<MatchupRow>>, String> {
    let url = format!(
        "{STATS_BASE}/matchups/{}/{QUEUE}/{champion_key}/{}.json",
        p.patch, p.matchups_version
    );
    Ok(parse_matchups(&get_world(client, &url).await?))
}

/// u.gg'de bu şampiyonun dosyası yoksa (4xx) boş harita döner.
pub async fn fetch_overview(
    client: &reqwest::Client,
    p: &UggPatch,
    champion_key: i64,
) -> Result<HashMap<Lane, Build>, String> {
    let url = format!(
        "{STATS_BASE}/overview/{}/{QUEUE}/{champion_key}/{}.json",
        p.patch, p.overview_version
    );
    Ok(parse_overview(&get_world(client, &url).await?))
}

// ---------- Lane filtresi (champion_ranking) ----------

/// champion_ranking dosyasının tier'ları; sıra ve anlam `RANKS` ile aynı
/// (emerald_plus = 17, platinum_plus = 10, overall = 8).
const RANKING_TIERS: [&str; 3] = ["emerald_plus", "platinum_plus", "overall"];

/// champion_ranking dosyasındaki rol adı.
fn ranking_role_key(lane: Lane) -> &'static str {
    match lane {
        Lane::Top => "top",
        Lane::Jungle => "jungle",
        Lane::Mid => "mid",
        Lane::Adc => "adc",
        Lane::Support => "supp",
    }
}

/// Şampiyon key → lane → maç sayısı.
pub type RoleGames = HashMap<i64, HashMap<Lane, u32>>;

/// `v[0][rol]` = `[[key(metin), [rakipler], galibiyet, maç, ...], ...]`; yalnızca key ve maç okunur.
/// Maçı 0 olan satır alınmaz.
pub fn parse_roles(v: &Value) -> RoleGames {
    let mut out: RoleGames = HashMap::new();
    let Some(roles) = v.get(0) else {
        return out;
    };
    for lane in Lane::ALL {
        let Some(rows) = roles.get(ranking_role_key(lane)).and_then(Value::as_array) else {
            continue;
        };
        for row in rows {
            let Some(row) = row.as_array() else { continue };
            let (Some(key), Some(games)) = (row.first().and_then(as_id), row.get(3).and_then(as_u32))
            else {
                continue;
            };
            if games > 0 {
                out.entry(key).or_default().insert(lane, games);
            }
        }
    }
    out
}

/// Lane payı = o lane'deki maç / beş lane'deki toplam maç. Hiç maçı olmayan şampiyon alınmaz.
pub fn role_shares(games: &RoleGames) -> HashMap<i64, HashMap<Lane, f64>> {
    games
        .iter()
        .filter_map(|(&key, lanes)| {
            let total: u64 = lanes.values().map(|&g| u64::from(g)).sum();
            (total > 0).then(|| {
                let shares = lanes.iter().map(|(&l, &g)| (l, f64::from(g) / total as f64)).collect();
                (key, shares)
            })
        })
        .collect()
}

/// Payı en az `min` olan lane'ler, `Lane::ALL` sırasıyla.
pub fn lanes_over(shares: &HashMap<i64, HashMap<Lane, f64>>, min: f64) -> HashMap<i64, Vec<Lane>> {
    shares
        .iter()
        .map(|(&key, s)| {
            let lanes = Lane::ALL
                .into_iter()
                .filter(|l| s.get(l).is_some_and(|&x| x >= min))
                .collect();
            (key, lanes)
        })
        .collect()
}

/// Bütün şampiyonların lane başına maç sayısı (tier sırasıyla ilk dolu dosya).
/// Dosya u.gg'de yoksa (4xx, ör. yayınlanmamış yama) boş harita döner.
pub async fn fetch_roles(client: &reqwest::Client, p: &UggPatch) -> Result<RoleGames, String> {
    for tier in RANKING_TIERS {
        let url = format!(
            "{STATS_BASE}/champion_ranking/world/{}/{QUEUE}/{tier}/{}.json",
            p.patch, p.ranking_version
        );
        if let Some(v) = get_typed::<Value>(client.get(&url)).await? {
            let games = parse_roles(&v);
            if !games.is_empty() {
                return Ok(games);
            }
        }
    }
    Ok(HashMap::new())
}
