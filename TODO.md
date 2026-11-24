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

- [ ] **LC-012 Küçük pencerede lane gecikmesi** — gerçek istemcide denenmedi.
  `npm run tauri dev` → sıraya gir, şampiyon seçimine geç:
  - küçük pencere açılır açılmaz başlıkta lane etiketi var, "Lane'in belli
    değil" ekranı hiç görünmüyor;
  - terminaldeki `[lcu … ms]` satırlarında `lol-champ-select/v1/session` mi
    `gameflow-phase` mı önce geliyor (nedeni doğrular).

  Geçince: `src-tauri/tests/lane_gecikmesi.rs` silinir, CHANGELOG LC-012 durumu
  "canlı test edildi" yapılır. Gecikme sürerse sıradaki adaylar: pencerenin her
  seçimde sıfırdan kurulması (WebView), yeni yamada ilk veri indirmesi.
