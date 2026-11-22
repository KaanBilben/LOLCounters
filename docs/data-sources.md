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
