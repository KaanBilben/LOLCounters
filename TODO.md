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
