# CHANGELOG

## LC-006 — Lane filtresi
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-002, LC-004

> ### 2026-11-22 · Claude — Lane'e göre şampiyon listesi
> - **Ne:** Havuzum'daki şampiyon ızgarası ve Counter'daki rakip seçimi varsayılan olarak yalnızca seçili lane'de oynanan şampiyonları gösterir; yanında "<Lane> şampiyonları | Hepsi" seçici var; arama kutusu doluyken bütün şampiyonlarda arar. Veri u.gg champion_ranking dosyasından (yama başına bir kez, `ugg/{yama}/roles.json`); lane payı %10 (`ROLE_MIN_SHARE`) ve üstü sayılır; verisi olmayan şampiyon her lane'de görünür; veri alınamazsa liste sessizce tam kalır. Yeni `get_roles` komutu ve diğer Rust kodu (LCU rakip tahmini) için `role_shares_cached`
> - **Neden:** LoL istemcisindeki rol filtresi gibi; 170+ şampiyon içinde aramadan seçim
> - **Etkilenen:** LC-004 (`ChampionPicker` arama satırına `toolbar` snippet'i eklendi); LC-002 (`UggPatch.rankingVersion` alanı; eski `patch.json` kayıtlarında varsayılan `1.5.0`)
> - **Dosyalar:** src-tauri/src/ugg.rs, src-tauri/src/lib.rs, src-tauri/src/cache.rs, src-tauri/tests/lc006_lane_filter.rs, src/lib/roles.ts, src/lib/LaneFilter.svelte, src/lib/ChampionPicker.svelte, src/lib/PoolEditor.svelte, src/lib/CounterView.svelte, src/lib/api.ts, src/lib/types.ts, src/lib/mock.ts, docs/data-sources.md, README.md, CLAUDE.md
> - **Commit:** —
>
> ### 2026-11-22 · Claude — Fix: inceleme bulguları
> - **Ne:** lane verisi bir kez alınamazsa (ör. Windows açılışında ağ henüz yokken) filtre artık oturum boyunca kaybolmuyor: `roles.ts` `watchRoles()` veri gelene kadar pencere gösterilince ya da odak alınca yeniden sorar (zamanlayıcı yok); Havuzum ve Counter bunu kullanır
> - **Dosyalar:** src/lib/roles.ts, src/lib/PoolEditor.svelte, src/lib/CounterView.svelte
> - **Commit:** —

## LC-005 — Otomatik kayıt
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-003, LC-004

> ### 2026-11-22 · Claude — Havuz her değişiklikte kendiliğinden kaydediliyor
> - **Ne:** Havuzum'da ekleme, çıkarma ve sıralama (sürükleme ya da oklar) son değişiklikten ~500 ms sonra kendiliğinden kaydediliyor; görünümden çıkınca, pencere gizlenince/kapanınca hemen yazılıyor. "Kaydet" / "Vazgeç" düğmeleri ve sekmedeki kaydedilmemiş noktası kalktı; kayıt çubuğunda "Kaydediliyor…" / "Kaydedildi ✓" (solarak kaybolur) / "Kaydedilemedi: …" + "Tekrar dene". Ctrl+S hemen kaydeder. Hoş geldin notu, havuzla Havuzum'dan çıkılınca kapanıyor. `settings.json` geçici dosya + taşıma ile atomik yazılıyor (`cache::write_atomic`), yarıda kalan yazma dosyayı bozmuyor
> - **Neden:** kullanıcı tercihleri hep kayıtlı olsun; kapatınca değişiklik kaybolmasın
> - **Etkilenen:** LC-004 (PoolEditor, +page.svelte) — kaydet düğmesi ve "dirty" kavramı kalktı; LC-003 (settings.rs) — kayıt atomik
> - **Dosyalar:** src/lib/PoolEditor.svelte, src/routes/+page.svelte, src/lib/mock.ts, src-tauri/src/settings.rs, src-tauri/src/cache.rs, src-tauri/tests/lc005_autosave.rs, CLAUDE.md
> - **Commit:** f7e2f2a

## LC-004 — Arayüz: Havuzum ve Counter ekranları
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-002, LC-003

> ### 2026-11-22 · Claude — İlk arayüz
> - **Ne:** Havuzum (lane sekmeleri, aramalı şampiyon ızgarası, sürükleyerek sıralama, kaydet) ve Counter (lane + rakip seçimi, öneri kartları, rün/büyü/eşya) ekranları; LoL temalı koyu tasarım
> - **Neden:** counter aramadan, tek tıkla öneri
> - **Dosyalar:** src/routes/+page.svelte, src/lib/
> - **Commit:** 7c86ab9
>
> ### 2026-11-22 · Claude — Fix: inceleme bulguları
> - **Ne:** küçük pencerede (900×620) şampiyon ızgarası sığıyor; boş aramada Enter seçim yapmıyor; klavye odağı (kaldırma, ikonlar, lane seçici); verisi alınamayan şampiyon "Alınamadı" olarak gösteriliyor ve sonuç önbelleğe alınmıyor; sürükle-bırak için `dragDropEnabled: false`
> - **Dosyalar:** src/lib/*.svelte, src-tauri/tauri.conf.json
> - **Commit:** 7c86ab9
>
> ### 2026-11-22 · Claude — Fix: havuz satırı yarım çip göstermiyor
> - **Ne:** havuz çip listesinin yüksekliği tam 2 satıra ayarlandı (92 px); 900×620 pencerede 5. şampiyon yarım kesik görünüyordu
> - **Dosyalar:** src/lib/PoolEditor.svelte
> - **Commit:** ee77315

## LC-003 — Puanlama ve havuz kaydı
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-002

> ### 2026-11-22 · Claude — Öneri puanı ve ayar dosyası
> - **Ne:** düzeltilmiş kazanma oranı + tercih bonusu ile sıralama ve etiketler; lane başına şampiyon havuzunun diske kaydı; Tauri komutları (`recommend`, `get_settings`, `save_settings`)
> - **Neden:** kendi şampiyonların öncelikli olsun ama kötü eşleşmede daha iyisi öne geçsin
> - **Dosyalar:** src-tauri/src/scoring.rs, src-tauri/src/settings.rs, src-tauri/src/lib.rs, docs/scoring.md
> - **Commit:** 9803cd1

## LC-002 — Veri katmanı: Data Dragon, u.gg, önbellek
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-001

> ### 2026-11-22 · Claude — Veri çekme ve önbellek
> - **Ne:** Data Dragon'dan Türkçe şampiyon/eşya/rün/büyü listesi; u.gg'den eşleşme ve build verisi; yama başına disk önbelleği, internet yoksa kayıtlı veri
> - **Neden:** Riot'un resmi API'sinde eşleşme oranı yok
> - **Dosyalar:** src-tauri/src/ddragon.rs, src-tauri/src/ugg.rs, src-tauri/src/cache.rs, docs/data-sources.md
> - **Commit:** b6d89b8
>
> ### 2026-11-22 · Claude — Fix: inceleme bulguları
> - **Ne:** aynı şampiyon aynı anda iki kez indirilmiyor; u.gg dosyasının yalnızca dünya verisi okunuyor (bellek ~25 MB → küçük); en çok 4 eşzamanlı indirme; yarım başarılı indirme saklanıyor; eski yama klasörleri açılışta siliniyor; sürüm isteği 5 sn sınırlı; hata `Recommendation.error` ile arayüze iletiliyor
> - **Etkilenen:** LC-004 (src/lib/types.ts) — `error` alanı eklendi
> - **Dosyalar:** src-tauri/src/lib.rs, src-tauri/src/ugg.rs, src-tauri/src/cache.rs, src-tauri/src/ddragon.rs
> - **Commit:** b6d89b8

## LC-001 — Proje iskeleti ve belgeler
Başlangıç 2026-11-22 · Durum: tamamlandı

> ### 2026-11-22 · Claude — Tauri 2 + SvelteKit iskeleti
> - **Ne:** Tauri 2 + SvelteKit şablonu, pencere ve güvenlik ayarları, CLAUDE.md, README (giriş sayfası + belge haritası), CHANGELOG
> - **Dosyalar:** package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json, CLAUDE.md, README.md
> - **Commit:** 367cf75
