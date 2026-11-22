# LOLCounters — proje kuralları

Giriş sayfası ve belge haritası: [README.md](README.md). Genel çalışma
tercihleri kullanıcının global `CLAUDE.md`'sindedir; burada yalnızca bu
projeye özel kurallar var.

## Ürün kuralları

- **Performans:** Uygulama boştayken CPU harcamaz. Zamanlayıcı, sürekli yoklama
  (polling) ya da gereksiz arka plan işi eklenmez; ağ isteği yalnızca veri
  eksikse yapılır ve sonuç diske kaydedilir (şampiyon verisi yama başına bir
  kez indirilir; yama içinde yenilenmez).
- **Resmi olmayan veri tek yerde:** u.gg ile ilgili her şey (adresler, JSON
  biçimi) yalnızca `src-tauri/src/ugg.rs` içindedir.
- **Riot kuralı:** Uygulama başka oyuncuların adını ya da istatistiğini hiçbir
  yerde göstermez; yalnızca şampiyon verisi kullanılır.
- **Arayüz dili Türkçe;** şampiyon, eşya, rün ve büyü adları Data Dragon
  `tr_TR` verisinden gelir, elle yazılmaz.

## Bir şeyi değiştirince bunları da güncelle

| Değişen | Güncellenecek |
|---|---|
| u.gg adresi ya da JSON biçimi (`ugg.rs`) | [docs/data-sources.md](docs/data-sources.md) |
| Puanlama sabitleri (yalnızca `src-tauri/src/scoring.rs` başında) | [docs/scoring.md](docs/scoring.md) ve sahte veri `src/lib/mock.ts` |
| Tauri komutu ya da dönen veri (`lib.rs`) | `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/mock.ts` |
| Çalıştırma / kod yapısı | [README.md](README.md) |

## Kontrol ve test

- **Karar testleri** (hafif TDD) `src-tauri/tests/` altına yazılır, canlı
  testten sonra silinir.
- **Commit öncesi:** `npm run check`, `npm run build`, `cargo test` ve
  `cargo clippy` (son ikisi `src-tauri/` içinde) temiz olmalı.
- **Arayüzü tarayıcıda kontrol:** `npm run dev` (port 1420). Tauri dışında
  `src/lib/mock.ts` sahte veriyi kullanır; hata durumları için localStorage'a
  `lolcounters.mockOffline = '1'` ya da `lolcounters.mockPartial = '1'` yaz
  (otomatik kayıt hatası için `lolcounters.mockSaveFail = '1'`).
- **Canlı test (kullanıcı):** `npm run tauri dev`; dağıtım için
  `npm run tauri build`.

## CHANGELOG

Thread biçimi, kimlik öneki `LC-`.
