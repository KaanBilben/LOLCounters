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

- [ ] **LC-013/014/015 Rün seçenekleri ve içe aktarma** — `feature/run-import`
  (LC-013 ve LC-014'ün üstünde). `npm run tauri dev` → seçimde şampiyon kilitle:
  - "Rünler"de ‹ › ve noktalarla 3 sayfa geziliyor; rakibe özel sayfa varsa altın
    çerçeve + "★ {rakip} için önerilen" ve pencere o sayfada açılıyor;
  - "İçe aktar": istemcide "LOLCounters" adlı rün sayfası oluşup seçiliyor,
    büyüler D/F'ye geliyor (Flash olduğu tuşta kalıyor); ikinci aktarmada aynı
    sayfanın üstüne yazıyor, yeni sayfa açmıyor;
  - boş sayfa yeri yokken önce "… üstüne yazılsın mı?" soruyor; "Vazgeç" hiçbir
    şeyi değiştirmiyor.

  Geçince: `src-tauri/tests/run_verisi.rs` ve `run_import.rs` silinir, CHANGELOG
  LC-013/014/015 durumu "canlı test edildi" yapılır, üç branch sırayla `main`'e
  `--no-ff` ile girer.
