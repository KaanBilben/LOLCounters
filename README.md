# LOLCounters

League of Legends için hafif bir masaüstü counter öneri uygulaması.

1. İlk açılışta her lane için oynadığın şampiyonları tercih sırasıyla seçersin.
2. Maçta lane'ini ve karşındaki şampiyonu seçersin.
3. Uygulama havuzundaki şampiyonları o rakibe karşı sıralar (tercih sıran +
   eşleşme kazanma oranı) ve seçilen şampiyon için rün, büyü ve eşya önerir.

Tauri 2 (Rust) + SvelteKit ile yazıldı; boştayken CPU harcamaz.

## Çalıştırma

```powershell
npm install
npm run tauri dev
```

Gerekenler: Rust, Node.js, WebView2 (Windows 11'de hazır gelir).

Kurulum dosyası üretmek için: `npm run tauri build`.

## Belge haritası

| Belge | Konusu |
|---|---|
| [CLAUDE.md](CLAUDE.md) | Bu projede çalışırken uyulacak kurallar |
| [docs/data-sources.md](docs/data-sources.md) | Veri nereden ve hangi biçimde geliyor (Data Dragon, u.gg) |
| [docs/scoring.md](docs/scoring.md) | Önerilerin nasıl sıralandığı |
| [CHANGELOG.md](CHANGELOG.md) | Yapılan işler (thread biçiminde) |

## Kod yapısı

| Yer | Ne yapar |
|---|---|
| `src-tauri/src/lib.rs` | Tauri komutları ve uygulama durumu |
| `src-tauri/src/ddragon.rs` | Riot Data Dragon: şampiyon, eşya, rün, büyü listeleri |
| `src-tauri/src/ugg.rs` | u.gg: eşleşme, build ve lane (rol) verisi (resmi olmayan kaynak) |
| `src-tauri/src/cache.rs` | Disk önbelleği (yama başına) |
| `src-tauri/src/scoring.rs` | Öneri puanlaması |
| `src-tauri/src/settings.rs` | Şampiyon havuzunun kaydı |
| `src/lib/` | Arayüz bileşenleri ve Tauri çağrıları |
| `src/lib/roles.ts`, `src/lib/LaneFilter.svelte` | Lane filtresi: şampiyon ızgarasında yalnızca seçili lane'in şampiyonları |
| `src/routes/+page.svelte` | Ana sayfa (Havuzum / Counter görünümleri) |

## Durum ve sonraki adımlar

- Şampiyon ızgaraları (Havuzum ve rakip seçimi) varsayılan olarak yalnızca
  seçili lane'in şampiyonlarını gösterir ("Hepsi" ile hepsi); veri u.gg'den,
  kural [docs/data-sources.md](docs/data-sources.md#lane-filtresi-ugg-champion_ranking).

- Rakip şampiyon şimdilik elle seçiliyor. Sonraki sürüm: LoL istemcisinden
  otomatik algılama.
- Rün ve eşya önerileri şampiyonun o lane'deki genel build'idir; rakibe özel
  build henüz yok.
