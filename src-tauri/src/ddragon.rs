//! Riot Data Dragon (resmi): şampiyon, eşya, rün ve büyü listeleri. Dil: tr_TR.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

const BASE: &str = "https://ddragon.leagueoflegends.com";
const LANG: &str = "tr_TR";
/// Küçük sürüm listesi isteğinin süre sınırı (büyük dosyalar istemcinin 20 sn'sini kullanır).
const VERSIONS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Champion {
    pub key: i64,
    pub id: String,
    pub name: String,
    pub icon: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rune {
    pub id: i64,
    pub name: String,
    pub icon: String,
    /// Rünün ağacı (ağacın kendisi için kendi id'si).
    pub tree: i64,
    /// Ağaçtaki sıra: 0 = ana rün (keystone), 1..3 = alt sıralar. Ağaçlar için 0.
    pub slot: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spell {
    pub key: i64,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticData {
    pub version: String,
    /// Ada göre sıralı.
    pub champions: Vec<Champion>,
    pub items: HashMap<i64, Item>,
    pub runes: HashMap<i64, Rune>,
    pub trees: HashMap<i64, Rune>,
    pub spells: HashMap<i64, Spell>,
    pub shards: HashMap<i64, String>,
}

const SHARD_IDS: [i64; 9] = [5008, 5005, 5007, 5010, 5001, 5011, 5013, 5002, 5003];

/// Rün parçacığı adları (Data Dragon'da yok).
pub fn shard_name(id: i64) -> &'static str {
    match id {
        5008 => "Uyarlanabilir Güç",
        5005 => "Saldırı Hızı",
        5007 => "Yetenek Hızı",
        5010 => "Hareket Hızı",
        5001 => "Artan Can",
        5011 => "Can",
        5013 => "Direnç ve Yavaşlatma Direnci",
        5002 => "Zırh",
        5003 => "Büyü Direnci",
        _ => "Parçacık",
    }
}

async fn get_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    send_json(client.get(url)).await
}

async fn send_json(req: reqwest::RequestBuilder) -> Result<Value, String> {
    let resp = req
        .send()
        .await
        .map_err(|e| format!("Data Dragon isteği başarısız: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Data Dragon yanıtı: {}", resp.status()));
    }
    resp.json::<Value>()
        .await
        .map_err(|e| format!("Data Dragon verisi okunamadı: {e}"))
}

/// En yeni oyun sürümü (listenin ilki), ör. "16.23.1".
/// Kısa süre sınırı: ağ takılırsa diskteki veriye hızlı düşülsün.
pub async fn latest_version(client: &reqwest::Client) -> Result<String, String> {
    let req = client
        .get(format!("{BASE}/api/versions.json"))
        .timeout(VERSIONS_TIMEOUT);
    let v = send_json(req).await?;
    v.get(0)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "Data Dragon sürüm listesi boş".to_string())
}

fn str_of(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn image_full(v: &Value) -> String {
    v.get("image")
        .and_then(|i| i.get("full"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// Türkçe alfabe sırası için sıralama anahtarı.
fn tr_sort_key(s: &str) -> Vec<u32> {
    const ALPHABET: &str = "abcçdefgğhıijklmnoöpqrsştuüvwxyz";
    s.chars()
        .map(|c| {
            let lower = match c {
                'I' => 'ı',
                'İ' => 'i',
                _ => c.to_lowercase().next().unwrap_or(c),
            };
            match ALPHABET.chars().position(|a| a == lower) {
                Some(p) => 1000 + p as u32,
                // Harf olmayanlar (boşluk, nokta, kesme işareti) önce, diğerleri sonra.
                None if !lower.is_alphanumeric() => (lower as u32).min(999),
                None => 2000 + lower as u32,
            }
        })
        .collect()
}

fn parse_champions(v: &Value, version: &str) -> Vec<Champion> {
    let mut out: Vec<Champion> = v
        .get("data")
        .and_then(Value::as_object)
        .map(|m| {
            m.values()
                .filter_map(|c| {
                    let key = c.get("key")?.as_str()?.parse().ok()?;
                    Some(Champion {
                        key,
                        id: str_of(c, "id"),
                        name: str_of(c, "name"),
                        icon: format!("{BASE}/cdn/{version}/img/champion/{}", image_full(c)),
                        tags: c
                            .get("tags")
                            .and_then(Value::as_array)
                            .map(|t| t.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort_by_cached_key(|c| tr_sort_key(&c.name));
    out
}

fn parse_items(v: &Value, version: &str) -> HashMap<i64, Item> {
    v.get("data")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(id, it)| {
                    let id: i64 = id.parse().ok()?;
                    Some((
                        id,
                        Item {
                            id,
                            name: str_of(it, "name"),
                            icon: format!("{BASE}/cdn/{version}/img/item/{id}.png"),
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_spells(v: &Value, version: &str) -> HashMap<i64, Spell> {
    v.get("data")
        .and_then(Value::as_object)
        .map(|m| {
            m.values()
                .filter_map(|s| {
                    let key: i64 = s.get("key")?.as_str()?.parse().ok()?;
                    Some((
                        key,
                        Spell {
                            key,
                            name: str_of(s, "name"),
                            icon: format!("{BASE}/cdn/{version}/img/spell/{}", image_full(s)),
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// runesReforged.json → (rünler, ağaçlar).
fn parse_runes(v: &Value) -> (HashMap<i64, Rune>, HashMap<i64, Rune>) {
    let mut runes = HashMap::new();
    let mut trees = HashMap::new();
    let icon = |x: &Value| format!("{BASE}/cdn/img/{}", str_of(x, "icon"));
    for t in v.as_array().into_iter().flatten() {
        let Some(tree_id) = t.get("id").and_then(Value::as_i64) else {
            continue;
        };
        trees.insert(
            tree_id,
            Rune { id: tree_id, name: str_of(t, "name"), icon: icon(t), tree: tree_id, slot: 0 },
        );
        let slots = t.get("slots").and_then(Value::as_array);
        for (slot, s) in slots.into_iter().flatten().enumerate() {
            for r in s.get("runes").and_then(Value::as_array).into_iter().flatten() {
                let Some(id) = r.get("id").and_then(Value::as_i64) else {
                    continue;
                };
                runes.insert(
                    id,
                    Rune { id, name: str_of(r, "name"), icon: icon(r), tree: tree_id, slot: slot as u8 },
                );
            }
        }
    }
    (runes, trees)
}

pub async fn load_static_data(client: &reqwest::Client, version: &str) -> Result<StaticData, String> {
    let url = |file: &str| format!("{BASE}/cdn/{version}/data/{LANG}/{file}");
    let urls = ["champion.json", "item.json", "runesReforged.json", "summoner.json"].map(url);
    let (champs, items, runes, spells) = futures::try_join!(
        get_json(client, &urls[0]),
        get_json(client, &urls[1]),
        get_json(client, &urls[2]),
        get_json(client, &urls[3]),
    )?;
    let champions = parse_champions(&champs, version);
    if champions.is_empty() {
        return Err("Data Dragon şampiyon listesi boş".to_string());
    }
    let (runes, trees) = parse_runes(&runes);
    Ok(StaticData {
        version: version.to_string(),
        champions,
        items: parse_items(&items, version),
        runes,
        trees,
        spells: parse_spells(&spells, version),
        shards: SHARD_IDS.iter().map(|&id| (id, shard_name(id).to_string())).collect(),
    })
}
