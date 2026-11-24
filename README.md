# LOLCounters

League of Legends için hafif bir masaüstü counter öneri uygulaması (Windows).

1. İlk açılışta her lane için oynadığın şampiyonları tercih sırasıyla seçersin.
2. Şampiyon seçimine girince ekranın sağ altında küçük bir pencere açılır:
   ban aşamasında oynamak için gösterdiğin şampiyonun en zor 3 rakibini
   banlaman için listeler; kilitlemeden önce havuzundaki şampiyonları rakibine karşı sıralar (tercih
   sıran + eşleşme kazanma oranı), kilitleyince rün, büyü ve eşya önerir.
   Oyun başlayınca kendiliğinden kapanır.
3. LoL istemcisi kapalıyken ana pencerede lane'ini ve rakibini elle de
   seçebilirsin.

Tauri 2 (Rust) + SvelteKit ile yazıldı; boştayken CPU harcamaz.

## İndir ve kur

1. [Releases](https://github.com/KaanBilben/LOLCounters/releases) sayfasından
   `LOLCounters_<sürüm>_x64-setup.exe` dosyasını indir ve çift tıkla.
2. Kurulum dosyası imzasız olduğu için Windows "Windows bilgisayarınızı
   korudu" diyebilir: **Ek bilgi → Yine de çalıştır**.
3. Kurulum yönetici izni istemez; yalnızca senin kullanıcına kurar.

Ayrıntılar (tepsi, Windows açılınca başlatma): aşağıda
[Kurulum ve günlük kullanım](#kurulum-ve-günlük-kullanım).

## Veri ve gizlilik

- **Şampiyon, eşya, rün ve büyü adları ve ikonları:** Riot'un resmi
  [Data Dragon](https://developer.riotgames.com/docs/lol#data-dragon)
  kaynağından, uygulama çalışırken indirilir.
- **Eşleşme, build ve lane istatistikleri:** u.gg'nin belgelenmemiş, resmi
  olmayan adreslerinden yama başına bir kez indirilir. LOLCounters'ın u.gg ile
  bir bağı yoktur; u.gg bu adresleri değiştirirse ya da kapatırsa öneriler
  çalışmayabilir. Ayrıntı: [docs/data-sources.md](docs/data-sources.md).
- **LoL istemcisi:** uygulama oyunun belleğini okumaz. Bilgisayarındaki LoL
  istemcisinin yerel arayüzünden yalnızca şampiyon seçimindeki şampiyonları ve
  senin lane'ini okur; oyuncu adlarını ya da başka oyuncuların istatistiklerini
  hiç okumaz ve göstermez.
- Hiçbir veri bir sunucuya gönderilmez; havuzun ve ayarların yalnızca
  bilgisayarında durur.

## Kurulum ve günlük kullanım

- **Kurulum dosyası üretmek:** `npm run tauri build` →
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

## Geliştirme

```powershell
npm install
npm run tauri dev
```

Gerekenler: Rust, Node.js, WebView2 (Windows 11'de hazır gelir).

## Belge haritası

| Belge | Konusu |
|---|---|
| [CLAUDE.md](CLAUDE.md) | Bu projede çalışırken uyulacak kurallar |
| [docs/data-sources.md](docs/data-sources.md) | Veri nereden ve hangi biçimde geliyor (Data Dragon, u.gg, LoL istemcisi) |
| [docs/scoring.md](docs/scoring.md) | Önerilerin nasıl sıralandığı |
| [CHANGELOG.md](CHANGELOG.md) | Yapılan işler (thread biçiminde) |
| [LICENSE](LICENSE) | Kodun lisansı (MIT) |

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
| `src/lib/SettingsMenu.svelte` | Üst çubuktaki Ayarlar paneli (Windows açılınca başlat, Hakkında) |
| `src/routes/+page.svelte` | Ana sayfa (Havuzum / Counter görünümleri) |
| `src/routes/compact/+page.svelte` | Küçük pencerenin sayfası (rakipler, havuz sıralaması, kilitten sonra build) |

## Durum ve sonraki adımlar

- Şampiyon ızgaraları (Havuzum ve rakip seçimi) varsayılan olarak yalnızca
  seçili lane'in şampiyonlarını gösterir ("Hepsi" ile hepsi); veri u.gg'den,
  kural [docs/data-sources.md](docs/data-sources.md#lane-filtresi-ugg-champion_ranking).
- Şampiyon seçimi LoL istemcisinden okunur (rol, kilit, görünen rakipler,
  rakip tahmini) ve küçük pencerede gösterilir; Normal Draft'ta canlı test
  edildi.
- Rün ve eşya önerileri şampiyonun o lane'deki genel build'idir; rakibe özel
  build ve matchup ipuçları henüz yok.

## Lisans ve yasal not

Kod [MIT lisansı](LICENSE) ile açıktır. Lisans yalnızca bu projenin kodunu
kapsar; Riot Games'in varlıkları (şampiyon adları, ikonlar) ve u.gg'nin
istatistikleri bu lisansın parçası değildir.

LOLCounters isn't endorsed by Riot Games and doesn't reflect the views or
opinions of Riot Games or anyone officially involved in producing or managing
Riot Games properties. Riot Games, and all associated properties are
trademarks or registered trademarks of Riot Games, Inc.

LOLCounters was created under Riot Games' "Legal Jibber Jabber" policy using
assets owned by Riot Games. Riot Games does not endorse or sponsor this
project.
