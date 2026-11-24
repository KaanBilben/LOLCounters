# TODO

Yalnızca açık işler. Biten iş buradan silinir; ne yapıldığı
[CHANGELOG.md](CHANGELOG.md)'de durur.

## Canlı test bekleyenler

- [ ] **LC-011 Ban önerisi** — gerçek LoL istemcisinde denenmedi (`main`'e
  canlı testsiz girdi). `npm run tauri dev` → Dereceli ya da Normal Seçim (Draft):
  - ban aşamasında şampiyon gösterince küçük pencerede "Ban önerisi · X için"
    ve 3 satır çıkıyor, rakip satırında "Banlar sürüyor…" yazıyor;
  - banlanmış şampiyon ve takım arkadaşının gösterdiği şampiyon listede yok;
  - kendi banını atınca normal havuz görünümüne dönüyor.

  Geçince: `src-tauri/tests/ban_onerisi.rs` silinir, CHANGELOG LC-011 durumu
  "canlı test edildi" yapılır.

- [ ] **LC-012 Küçük pencerede lane gecikmesi** — gerçek istemcide denenmedi (`main`'e
  canlı testsiz girdi).
  `npm run tauri dev` → sıraya gir, şampiyon seçimine geç:
  - küçük pencere açılır açılmaz başlıkta lane etiketi var, "Lane'in belli
    değil" ekranı hiç görünmüyor;
  - terminaldeki `[lcu … ms]` satırlarında `lol-champ-select/v1/session` mi
    `gameflow-phase` mı önce geliyor (nedeni doğrular).

  Geçince: `src-tauri/tests/lane_gecikmesi.rs` silinir, CHANGELOG LC-012 durumu
  "canlı test edildi" yapılır. Gecikme sürerse sıradaki adaylar: pencerenin her
  seçimde sıfırdan kurulması (WebView), yeni yamada ilk veri indirmesi.

- [ ] **LC-013/014/015 Rün seçenekleri ve içe aktarma** — gerçek istemcide denenmedi
  (`main`'e canlı testsiz girdi). `npm run tauri dev` → seçimde şampiyon kilitle:
  - küçük pencere biraz daha uzun (380×356) ve kaydırma çubuğu yok;
  - "Rünler"de 3 kart yan yana; kazanma oranı en yüksek kartta sarı "★ Önerilen",
    rakibe özel sayfada sarı "Eşleşmeye özel" (sağ üstte); pencere eşleşme kartı
    seçili açılıyor; karta tıklayınca mavi çerçeve o karta geçiyor;
  - "İçe aktar": istemcide "LOLCounters" adlı rün sayfası oluşup seçiliyor,
    büyüler D/F'ye geliyor (Flash olduğu tuşta kalıyor); ikinci aktarmada aynı
    sayfanın üstüne yazıyor, yeni sayfa açmıyor;
  - boş sayfa yeri yokken önce "… üstüne yazılsın mı?" soruyor; "Vazgeç" hiçbir
    şeyi değiştirmiyor.

  Geçince: `src-tauri/tests/run_verisi.rs`, `run_import.rs` ve `run_kartlari.rs` silinir,
  CHANGELOG LC-013/014/015 durumu "canlı test edildi" yapılır.
