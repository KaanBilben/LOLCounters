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
