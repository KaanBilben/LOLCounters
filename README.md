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

## Kurulum ve günlük kullanım

- **Kurulum dosyası:** `npm run tauri build` →
  `src-tauri/target/release/bundle/nsis/LOLCounters_0.1.0_x64-setup.exe`.
  Yalnızca bu kullanıcıya kurar (yönetici izni istemez,
  `%LOCALAPPDATA%\LOLCounters`). Başlat menüsü kısayolu her zaman eklenir;
  masaüstü kısayolu son sayfadaki kutuyla (varsayılan işaretli) eklenir.
  WebView2 yoksa kurulum onu internetten indirir.
- **Tepsi:** pencereyi kapatınca uygulama kapanmaz, saatin yanındaki tepsiye
  gizlenir. Tepsi simgesine sol tık ya da sağ tık → *Ayarları aç* pencereyi
  geri getirir; tamamen kapatmak için sağ tık → *Çık*.
- **Tek kopya:** uygulama ikinci kez açılırsa yeni kopya başlamaz, açık olan
  pencere öne gelir.
- **Windows açılınca başlat:** kurulan sürüm ilk açılışta kendini Windows
  başlangıcına ekler ve oradan tepside gizli (`--minimized`) başlar. Üst
  çubuktaki dişli → *Ayarlar* panelinden kapatılabilir. Geliştirme modunda
  (`npm run tauri dev`) bu ayar kapalıdır; geliştirme exe'si kaydedilmez.
- **Simge:** kaynak `src-tauri/icons/app-icon.svg`. Değişirse
  `npx tauri icon src-tauri/icons/app-icon.svg` ile bütün simgeler yeniden
  üretilir; oluşan `android/`, `ios/` klasörleri ve `64x64.png` silinir.

## Belge haritası

| Belge | Konusu |
|---|---|
| [CLAUDE.md](CLAUDE.md) | Bu projede çalışırken uyulacak kurallar |
| [docs/data-sources.md](docs/data-sources.md) | Veri nereden ve hangi biçimde geliyor (Data Dragon, u.gg, LoL istemcisi) |
| [docs/scoring.md](docs/scoring.md) | Önerilerin nasıl sıralandığı |
| [CHANGELOG.md](CHANGELOG.md) | Yapılan işler (thread biçiminde) |

## Kod yapısı

| Yer | Ne yapar |
|---|---|
| `src-tauri/src/lib.rs` | Tauri komutları ve uygulama durumu |
| `src-tauri/src/ddragon.rs` | Riot Data Dragon: şampiyon, eşya, rün, büyü listeleri |
| `src-tauri/src/ugg.rs` | u.gg: eşleşme, build ve lane (rol) verisi (resmi olmayan kaynak) |
| `src-tauri/src/lcu.rs` | LoL istemcisi (LCU): şampiyon seçimini okur; LCU kodunun tek yeri |
| `src-tauri/src/compact.rs` | Küçük pencere: şampiyon seçiminde sağ altta, her zaman üstte |
| `src-tauri/src/cache.rs` | Disk önbelleği (yama başına) |
| `src-tauri/src/scoring.rs` | Öneri puanlaması |
| `src-tauri/src/settings.rs` | Şampiyon havuzunun ve autostart tercihinin kaydı |
| `src-tauri/src/launch.rs` | Tepsi simgesi, kapatınca tepsiye gizleme, `--minimized` ile gizli başlama, Windows açılınca başlatma |
| `src/lib/` | Arayüz bileşenleri ve Tauri çağrıları |
| `src/lib/roles.ts`, `src/lib/LaneFilter.svelte` | Lane filtresi: şampiyon ızgarasında yalnızca seçili lane'in şampiyonları |
| `src/routes/+page.svelte` | Ana sayfa (Havuzum / Counter görünümleri) |
| `src/routes/compact/+page.svelte` | Küçük pencerenin sayfası (rakipler, havuz sıralaması, kilitten sonra build) |

## Durum ve sonraki adımlar

- Şampiyon ızgaraları (Havuzum ve rakip seçimi) varsayılan olarak yalnızca
  seçili lane'in şampiyonlarını gösterir ("Hepsi" ile hepsi); veri u.gg'den,
  kural [docs/data-sources.md](docs/data-sources.md#lane-filtresi-ugg-champion_ranking).
- Şampiyon seçimi LoL istemcisinden okunur (rol, kilit, görünen rakipler,
  rakip tahmini). İstemci açıkken henüz canlı test edilmedi.
- Şampiyon seçimi başlayınca sağ altta küçük bir pencere açılır (kilitten önce
  havuzun rakibe göre sıralaması, sonra şampiyonun build'i); oyun başlayınca
  kapanır, oyun içinde hiçbir şey göstermez. Henüz canlı test edilmedi.
- Rün ve eşya önerileri şampiyonun o lane'deki genel build'idir; rakibe özel
  build henüz yok.
