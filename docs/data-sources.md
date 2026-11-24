# Veri kaynakları

## Data Dragon (Riot, resmi)

Şampiyon, eşya, rün ve büyü adları ile ikonları. Dil: `tr_TR`.

| Veri | Adres |
|---|---|
| Sürümler | `https://ddragon.leagueoflegends.com/api/versions.json` (ilki en yenisi) |
| Şampiyonlar | `.../cdn/{sürüm}/data/tr_TR/champion.json` |
| Eşyalar | `.../cdn/{sürüm}/data/tr_TR/item.json` |
| Rünler | `.../cdn/{sürüm}/data/tr_TR/runesReforged.json` |
| Büyüler | `.../cdn/{sürüm}/data/tr_TR/summoner.json` |
| İkonlar | `.../cdn/{sürüm}/img/champion/{id}.png`, `img/item/{id}.png`, `img/spell/{dosya}`, rün: `.../cdn/img/{icon}` |

Rün parçacıkları (shard, ör. `5008`) Data Dragon'da yok; adları
`src-tauri/src/ddragon.rs` içindeki küçük tablodadır.

En yeni sürüme ulaşılamazsa (ör. Windows açılışında ağ henüz yok) ya da u.gg o
sürümün yamasını henüz listelemiyorsa diskteki eski kayıt yalnızca geçici
kullanılır: ana pencere gösterilince ve şampiyon seçimi başlayınca yeniden
denenir (zamanlayıcı yok; `lib.rs` `recheck_static`), sürüm değişirse arayüz
yeni veriyi yükler.

## u.gg (resmi olmayan)

Belgesi yok; biçim açık kaynak [uggo](https://github.com/kade-robertson/uggo)
projesinden alındı ve 2026-11-22'de canlı veriyle doğrulandı. Kod:
`src-tauri/src/ugg.rs`.

| Veri | Adres |
|---|---|
| Yama → API sürümü | `https://static.bigbrain.gg/assets/lol/riot_patch_update/prod/ugg/ugg-api-versions.json` |
| Eşleşmeler | `https://stats2.u.gg/lol/1.5/matchups/{yama}/ranked_solo_5x5/{şampiyon key}/{sürüm}.json` |
| Build (genel bakış) | `https://stats2.u.gg/lol/1.5/overview/{yama}/ranked_solo_5x5/{şampiyon key}/{sürüm}.json` |
| Lane filtresi (şampiyon sıralaması) | `https://stats2.u.gg/lol/1.5/champion_ranking/world/{yama}/ranked_solo_5x5/{tier}/{sürüm}.json` |
| Rün seçenekleri (LC-013) | `https://stats2.u.gg/lol/1.5/builds/{yama}/ranked_solo_5x5/{şampiyon key}/{sürüm}.json` |
| Eşleşmeye özel sayfa (LC-013) | `https://stats2.u.gg/lol/1.5/overview/{yama}/ranked_solo_5x5/matchups/{benim key}_{rakip key}/{sürüm}.json` |

- `{yama}`: Data Dragon sürümünün ilk iki parçası, `_` ile: `16.23.1` → `16_23`.
  u.gg o yamayı henüz listelemiyorsa listedeki en yeni yama kullanılır.
- `{şampiyon key}`: Data Dragon'daki sayısal `key` (ör. Darius = `122`).
- `{sürüm}`: yama haritasındaki `matchups` / `overview` değeri, yoksa `1.5.0`.

JSON yapısı: `bölge → rank → rol → [veri, zaman damgası]`.

| Anahtar | Kullanılan değer |
|---|---|
| Bölge | `12` = Dünya |
| Rank | `17` = Emerald+, yoksa `10` = Platinum+, yoksa `8` = Hepsi |
| Rol | Jungle `1`, Support `2`, ADC `3`, Top `4`, Mid `5` |

Eşleşme satırı: `[rakip key, kayıp, maç, ...]` → galibiyet = maç − kayıp
(satırlar o lane'deki bütün rakipleri içerir).

- Dosyanın yalnızca bölge `12` ve kullanılan rank'ları okunur; gerisi bellekte
  tutulmadan atlanır (eşleşme dosyası ~2,4 MB, bunun ~%14'ü dünya verisi).
- u.gg 4xx dönerse (408/429 hariç) "bu yamada bu şampiyon için veri yok"
  sayılır; ağ hatası ve 5xx hata olarak arayüze iletilir ve kullanıcı tekrar
  istediğinde yeniden denenir.

Build dizisi (sıra numarasıyla):

| # | İçerik |
|---|---|
| 0 | Rünler: `[maç, galibiyet, ana ağaç, ikincil ağaç, [rün id'leri]]` |
| 1 | Büyüler: `[maç, galibiyet, [büyü key'leri]]` |
| 2 | Başlangıç eşyaları: `[maç, galibiyet, [eşya id'leri]]` |
| 3 | Çekirdek eşyalar: `[maç, galibiyet, [eşya id'leri]]` |
| 4 | Yetenek sırası: `[maç, galibiyet, [sıra], "QEW"]` |
| 5 | 4./5./6. eşya seçenekleri: her biri `[id, galibiyet, maç]` listesi |
| 6 | Toplam: `[galibiyet, maç]` |
| 8 | Parçacıklar: `[maç, galibiyet, ["5005", ...]]` |

Rün id listesi slot sırasında değildir: önce ana rün, sonra diğer 5 rün sayıya
göre sıralı (ana ve ikincil ağaç karışık). Arayüz ve istemciye aktarma, Data
Dragon'daki ağaç ve slot bilgisiyle yeniden dizer.

### Rün seçenekleri ve eşleşme sayfası (LC-013)

2026-11-24'te canlı veriyle doğrulandı (16.23; Darius, Ahri).

- **`builds`** (`{sürüm}` = yama haritasındaki `builds`, yoksa `1.5.0`; ham dosya
  ~3,3 MB, yalnızca dünya verisi okunur): `bölge → rank → rol → [7 eleman]`
  (zaman damgası sarmalı yok). 0. eleman rün grupları:
  `[ana rün, ikincil ağaç, galibiyet, maç, ana ağaç, [sayfa, ...]]`, sayfa
  `[ana ağaç, ikincil ağaç, [6 rün], galibiyet, maç]`. Gruplar ve sayfalar
  sırasız gelir. Kullanılan: maça göre en büyük gruplar, her grubun en çok
  oynanan sayfası. Bu dosyada parça yoktur; parçalar ve büyüler overview'dan
  (8. ve 1. eleman) gelir. Diskte `ugg/{yama}/runes_{key}.json`.
- **Eşleşmeye özel overview** (`matchups/{ben}_{rakip}`, `{sürüm}` = `overview`):
  normal overview ile aynı 13 elemanlı biçim, tek rün sayfası. Dikkat: 6.
  eleman burada `[benim kayıplarım, maç]`dır, `[galibiyet, maç]` değil; bu
  yüzden yalnızca 0 (rün sayfası), 1 (büyüler) ve 8 (parçalar) okunur. Eşleşme
  u.gg'de yoksa 403 döner (veri yok sayılır). Yalnızca rakip belli olunca
  indirilir; diskte `ugg/{yama}/mu_{key}_{rakip}.json`.
- Seçim ve vurgu kuralı: [scoring.md](scoring.md#rün-seçenekleri).

## Lane filtresi (u.gg champion_ranking)

Havuzum ve rakip seçimindeki şampiyon ızgarası, LoL istemcisindeki gibi
yalnızca seçili lane'de oynanan şampiyonları gösterir. Veri bütün şampiyonlar
için tek dosyadır (~145 KB, gzip ile ~57 KB); yama başına bir kez indirilir ve
yalnızca lane başına maç sayısı `ugg/{yama}/roles.json` olarak saklanır (~12 KB).
Kod: `fetch_roles` / `parse_roles` (`src-tauri/src/ugg.rs`), `get_roles` komutu (`src-tauri/src/lib.rs`).

- `{sürüm}`: yama haritasındaki `champion_ranking` değeri, yoksa `1.5.0`.
- `{tier}` metindir (overview'daki sayı değil); sırasıyla denenir:

| Tier | Overview'daki karşılığı (doğrulandı) |
|---|---|
| `emerald_plus` | `17` = Emerald+ |
| `platinum_plus` | `10` = Platinum+ |
| `overall` | `8` = Hepsi |

JSON: 4 elemanlı dizi; yalnızca `[0]` okunur:
`{ "top" | "jungle" | "mid" | "adc" | "supp": [satır, ...] }`.
Satır: `["şampiyon key (metin)", [10 kötü eşleşme], galibiyet, maç, ...]` →
yalnızca `[0]` (key) ve `[3]` (maç) kullanılır. Üst dizinin `[1]`–`[3]` elemanları kullanılmaz.

Kural: lane payı = o lane'deki maç / beş lane'deki toplam maç. Pay
`ROLE_MIN_SHARE` (`ugg.rs` başında, `0.10` = %10) ya da üstündeyse şampiyon o
lane'de listelenir. 16.23 Emerald+ ile: Darius Üst + Orman, Ashe Nişancı + Destek,
Brand Orta + Nişancı + Destek (Üst %7 → yok). Dosyada olmayan şampiyon (yeni
çıkmış) her lane'de görünür; veri alınamazsa arayüz sessizce bütün şampiyonları
gösterir. Yayınlanmamış yama 403 döner (= veri yok).

## League client (LCU, yerel)

LoL istemcisinin kendi bilgisayardaki API'si; şampiyon seçimini okumak için.
Riot bunu üçüncü taraf uygulamalar için resmi olarak desteklemez. Kod yalnızca
`src-tauri/src/lcu.rs` içindedir. İstemci açık olmadan canlı doğrulanamadı
(2026-11-22): aşağıdaki biçim belgelerden ve çalışan açık kaynak
kütüphanelerden (irelia, league-connect) alındı.

**Kurulum klasörü** (sırayla; `LeagueClient.exe` olan ilk aday):

1. `C:/ProgramData/Riot Games/Metadata/league_of_legends.live/league_of_legends.live.product_settings.yaml`,
   satır `product_install_full_path: "..."`
2. Kayıt defteri `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Riot Game league_of_legends.live`,
   değer `InstallLocation`
3. `C:/ProgramData/Riot Games/RiotClientInstalls.json`, `associated_client` anahtarları
4. `C:/Riot Games/League of Legends`

**Lockfile:** `<kurulum>/lockfile`, yalnızca istemci açıkken var:
`LeagueClient:<pid>:<port>:<parola>:https`. Klasör, Windows'un değişiklik
bildirimiyle (`FindFirstChangeNotificationW`, süresiz bekleme) izlenir;
yoklama yoktur. Lockfile görününce bağlantı en çok 4 kez denenir (0/1/2/4 sn).

**Bağlantı:** `wss://127.0.0.1:<port>/`, başlık
`Authorization: Basic base64("riot:<parola>")`.

- Sertifika: önce yalnızca Riot kökü
  ([riotgames.pem](https://static.developer.riotgames.com/docs/lol/riotgames.pem),
  `src-tauri/certs/riotgames.pem`). El sıkışma reddedilirse sertifikayı yalnızca
  hedef 127.0.0.1 olduğu için kabul eden yedek kullanılır (neden: `lcu.rs`
  içindeki `LoopbackOnly` açıklaması).
- Proxy kullanılmaz: REST istemcisi `no_proxy()` ile kurulur (WebSocket zaten
  proxy kullanmaz); `HTTPS_PROXY` ya da Windows proxy ayarı parolayı başka bir
  yere göndermez.
- Abonelikler (WAMP, 5 = abone ol, 8 = olay):
  `[5,"OnJsonApiEvent_lol-champ-select_v1_session"]`,
  `[5,"OnJsonApiEvent_lol-gameflow_v1_gameflow-phase"]`.
- Gelen olay: `[8, "<olay>", {"uri", "eventType": "Create"|"Update"|"Delete", "data"}]`.
- Bağlanınca bir kez REST: `GET /lol-gameflow/v1/gameflow-phase`; sonuç
  `"ChampSelect"` ise `GET /lol-champ-select/v1/session`.

**Okunan alanlar** (başka hiçbir alan çözümlenmez; oyuncu adı, `summonerId`,
`puuid` okunmaz):

| Alan | Kullanım |
|---|---|
| `localPlayerCellId` | Benim hücrem |
| `myTeam[].cellId`, `championId`, `championPickIntent`, `assignedPosition` | Benim rolüm (`top/jungle/middle/bottom/utility`) ve niyetim; takım arkadaşlarının seçtiği ya da gösterdiği şampiyon (ban önerisinden çıkarılır) |
| `theirTeam[].championId` | Görünen rakipler (0 olmayanlar) |
| `actions[][].actorCellId`, `championId`, `completed`, `type` | Kilit: benim tamamlanmış `pick` eylemim. Ban aşaması: benim tamamlanmamış `ban` eylemim varsa (kilit yokken) ban önerisi gösterilir; tamamlanmış `ban` eylemlerindeki şampiyonlar öneriden çıkarılır |
| aşama (`gameflow-phase`) | `ChampSelect` başlangıç; başka her aşama ya da oturum `Delete` bitiş |

**Oturum ne zaman kabul edilir** (`lcu.rs` `session_action`): aşama `ChampSelect`
ya da henüz bilinmiyorsa uygulanır. `Lobby`, `Matchmaking`, `ReadyCheck`
sırasında gelen oturum saklanır ve `ChampSelect` gelince uygulanır; istemci
oturumu aşamadan önce gönderirse küçük pencere yine lane'iyle açılır. Oyun
başladıktan sonra (`GameStart`, `InProgress` vb.) gelen oturum yok sayılır.
Geliştirme sürümü (`npm run tauri dev`) gelen her olayı sırasıyla terminale
yazar (`[lcu … ms] <eventType> <uri>`).

Rakibimin hangi lane'de olduğu istemcide yok; şampiyonların rol paylarıyla
([lane filtresinin verisi](#lane-filtresi-ugg-champion_ranking), aynı önbellek)
tahmin edilir (en olası bire bir eşleştirme) ve kullanıcı tıklayarak düzeltebilir.
Atanmış rol yoksa (pozisyonsuz seçim) kullanıcının seçtiği lane kullanılır
(`set_my_lane`). Tahmin yalnızca görünen rakipler ya da lane değişince, olay
döngüsünü bekletmeden ayrı bir görevde hesaplanır. Lane verisi alınamazsa o
seçimde yeniden denenmez ve tahmin yapılmaz; rakip elle seçilir.
