# CHANGELOG

## LC-016 — Bot lane sinerjisi verisi
Başlangıç 2026-11-24 · Durum: uygulandı, canlı test edilmedi · İlgili: LC-006, LC-008, LC-017

> ### 2026-11-24 · Claude — u.gg `champion_duos`, partner ve sinerji puanı
> - **Ne:** `ugg.rs` partnerin `champion_duos` dosyasını okur (`[birlikte oynayan, galibiyet, maç, …]`); lane verisi artık galibiyeti de saklar (`LaneStat`, tek başına oran için). `lcu.rs` `duoPartner` / `duoPartnerLocked`: ADC ↔ support partnerinin gösterdiği ya da kilitlediği şampiyon. `scoring::duo_synergies` (ikilinin düzeltilmiş oranı − iki tek başına oranın ortalaması, ≥200 maç, iyi ≥ +1,5) ve `best_synergies` (artı sinerjili en iyi 5, banlı / alınmış yok, counter'lar önce). Yeni komut `duo_synergy` (bellek → disk `duos_{partner}.json` → ağ; rakibin eşleşme verisi alınamazsa `error` ile döner, counter işaretleri eksik olabilir); `types.ts`, `api.ts`, sahte veri
> - **Neden:** ADC ya da support oynarken partnerle iyi giden şampiyonları göstermek (arayüz: LC-017)
> - **Etkilenen:** LC-006 (`roles.json` biçimi değişti; eski kayıt bir kez yeniden indirilir)
> - **Dosyalar:** src-tauri/src/ugg.rs, src-tauri/src/lcu.rs, src-tauri/src/scoring.rs, src-tauri/src/lib.rs, src-tauri/src/cache.rs, src/lib/types.ts, src/lib/api.ts, src/lib/mock.ts, docs/data-sources.md, docs/scoring.md
> - **Commit:** —

## LC-015 — Rün ve büyü içe aktarma
Başlangıç 2026-11-24 · Durum: main'de (merge 36913fb), canlı test edilmedi — uç noktalar gerçek istemcide denenmedi (bkz. TODO.md) · İlgili: LC-008, LC-014

> ### 2026-11-24 · Claude — "İçe aktar" düğmesi
> - **Ne:** küçük pencerede gösterilen rün sayfası ve büyüler tek tıkla istemciye yazılır: "LOLCounters" sayfasının üstüne (yoksa yeni sayfa; yer yoksa pencerede onay, sonra seçili sayfa). Sayfa seçili yapılır; seçim sürüyorsa büyüler `my-selection` ile ayarlanır (bir tuştaki büyü o tuşta kalır). `ddragon::order_perks` 9'lu listeyi ağaç/sıraya göre dizer; `lcu.rs` bağlıyken REST bağlantısını saklar (`Conn`), `import_target`, `order_spells`, `import_loadout`; yeni komut `import_loadout`. Düğme durumları: İçe aktar / Aktarılıyor… / Aktarıldı / Tekrar dene; sayfa ya da rakip değişince sıfırlanır. Sahte veri: `mockImport`
> - **Neden:** önerilen rünleri ve büyüleri elle kurmadan kullanmak
> - **Etkilenen:** LC-008 (`lcu.rs` artık istek üzerine istemciye yazıyor; REST istemcisi bağlantıda bir kez kuruluyor)
> - **Dosyalar:** src-tauri/src/lcu.rs, src-tauri/src/ddragon.rs, src-tauri/src/lib.rs, src/routes/compact/+page.svelte, src/lib/types.ts, src/lib/api.ts, src/lib/mock.ts, docs/data-sources.md, README.md, CLAUDE.md
> - **Commit:** 9322a9b
>
> ### 2026-11-24 · Claude — Fix: inceleme bulguları (kullanıcı sayfası silinmez, tek içe aktarma)
> - **Ne:** onaylanan kullanıcı sayfasında PUT reddedilirse artık `DELETE` + `POST` yapılmaz, hata gösterilir (yedek yol yalnızca uygulamanın "LOLCounters" sayfası için); `import_loadout` tek seferde bir tane çalışır (üst üste tıklama ikinci sayfa açmaz). Arayüz: rakip ya da lane değişince yeni rünler gelene kadar eski sayfa soluk, altın etiket gizli ve içe aktarma kapalı (eski rakibin sayfası yeni rakibe "önerilen" diye yazılmaz); parçası olmayan sayfada düğme kapalı, nedeni ipucunda. Belge: takımın büyüleri yalnızca içe aktarmada çözümleniyor (yalnızca benimki kullanılıyor); TODO.md'ye canlı test adımları
> - **Neden:** 4 yönlü inceleme + şüpheci doğrulama; en ciddisi, geçersiz bir sayfada kullanıcının onayladığı sayfanın kalıcı silinebilmesiydi
> - **Dosyalar:** src-tauri/src/lcu.rs, src/routes/compact/+page.svelte, docs/data-sources.md, TODO.md
> - **Commit:** 9d3013d

## LC-014 — Rün seçenekleri arayüzü: yan yana 3 kart ve sarı etiketler
Başlangıç 2026-11-24 · Durum: main'de (merge 31a6037; kart düzeni 36913fb ile), canlı test edilmedi (bkz. TODO.md) · İlgili: LC-009, LC-013, LC-015

> ### 2026-11-24 · Claude — Küçük pencerede tam rün sayfası
> - **Ne:** kilitten sonra "Rünler" başlığında ‹ • • • › ile 3 seçenek arası geçiş; tam sayfa (ana rün + 3 ana, ikincil ağaç + 2 rün, sağda oran ve maç; parçalar ipucunda); u.gg eşleşme sayfası altın çerçeve, altın nokta ve "★ {rakip} için önerilen" etiketiyle (tıklayınca ona döner); pencere altın seçenekte, yoksa en çok oynananda açılır. Altında büyüler + çekirdek eşyalar tek satırda (genel build oranı ipucunda; "Genel build" satırı kalktı). Rünler maç verisinden ayrı istenir; yükleniyor / alınamadı (Tekrar dene) / veri yok durumları. Sahte veri: `mockRunesFail`, `mockRunesEmpty`
> - **Neden:** tek sayfa yerine seçenek görmek ve eşleşmeye göre en iyisini hemen tanımak
> - **Etkilenen:** LC-009 (kilit sonrası görünüm değişti)
> - **Dosyalar:** src/routes/compact/+page.svelte, src/lib/mock.ts, README.md, CLAUDE.md
> - **Commit:** 51ac2a9
>
> ### 2026-11-24 · Claude — → LC-013 ve LC-015 düzeltmeleriyle değiştirildi
>
> ### 2026-11-24 · Claude — Değişiklik: ‹ › geçiş yerine yan yana 3 dikey kart
> - **Ne:** LoL'ün önerilen sayfalarına benzer 3 dikey kart (üstte ana rün, altında 3 ana rün, çizgi, ikincil ağaç + 2 rün, en altta oran ve maç); sağ üstte sarı "★ Önerilen" (`best`) ve "Eşleşmeye özel" (`gold`) etiketleri; karta tıklayınca seçilir (mavi çerçeve), "İçe aktar" seçili kartı yazar; açılışta eşleşme kartı, yoksa "Önerilen" seçili. ‹ ›, noktalar ve "★ {rakip} için önerilen" etiketi kalktı. Kartlar bir radiogroup: tek sekme durağı, ok tuşlarıyla geçiş; kart etiketi etiketleri, rün adlarını ve oranı okur; içteki ikonlar odaklanmaz (`GameIcon` yeni `focusable` seçeneği). Küçük pencere 380×260 → 380×356 (`compact.rs` `HEIGHT`)
> - **Neden:** kullanıcı isteği: 3 seçeneği aynı anda görmek, LoL istemcisine benzer görünüm
> - **Etkilenen:** LC-013 (`RuneOptions.best`, `scoring::best_rune_option`), LC-009 (küçük pencere boyu)
> - **Dosyalar:** src/routes/compact/+page.svelte, src/lib/GameIcon.svelte, src-tauri/src/compact.rs, src-tauri/src/scoring.rs, src-tauri/src/lib.rs, src/lib/types.ts, src/lib/mock.ts, docs/scoring.md, README.md, TODO.md
> - **Commit:** 7c2418e

## LC-013 — Rün seçenekleri verisi: en çok oynanan 3 sayfa ve eşleşme sayfası
Başlangıç 2026-11-24 · Durum: main'de (merge b20ab2b), canlı test edilmedi (bkz. TODO.md) · İlgili: LC-002, LC-014, LC-015

> ### 2026-11-24 · Claude — u.gg `builds` ve eşleşmeye özel overview
> - **Ne:** `ugg.rs` `builds` dosyasından lane başına rün gruplarını (maça göre, her grubun en çok oynanan sayfası) ve `overview/.../matchups/{ben}_{rakip}` dosyasından eşleşme sayfasını + büyüleri okur (oradaki 6. eleman kayıp olduğu için okunmaz); `scoring::pick_rune_options` (ilk 3; ≥100 maçlı eşleşme sayfası aynı gruptaki seçeneğin, yoksa 3.'nün yerine geçer ve altın olur); yeni komut `rune_options` (bellek → disk `runes_{key}.json` / `mu_{key}_{rakip}.json` → ağ); `types.ts`, `api.ts`, sahte veri
> - **Neden:** küçük pencerede 3 rün seçeneği ve eşleşmeye göre en iyisinin vurgusu; u.gg overview yalnızca tek sayfa veriyor
> - **Dosyalar:** src-tauri/src/ugg.rs, src-tauri/src/scoring.rs, src-tauri/src/lib.rs, src-tauri/src/cache.rs, src/lib/types.ts, src/lib/api.ts, src/lib/mock.ts, docs/data-sources.md, docs/scoring.md
> - **Commit:** a4610f6
>
> ### 2026-11-24 · Claude — Fix: inceleme bulguları (tek indirme, eksik veride yeniden deneme)
> - **Ne:** `lane_data` dosya başına kapıyla aynı dosyayı bir kez indirir (rakip değişince ~3 MB `builds` ikinci kez inmez); `RuneOptions.error`: `builds` ya da overview alınamadıysa eldeki seçenekler hatayla döner ve pencerede "yeniden dene" simgesi çıkar (önceden tek sayfaya sessizce düşüyordu); overview yoksa parçalar eşleşme sayfasından. Sahte veri: `mockRunesPartial` (branch: `feature/run-import`, LC-015 düzeltmesiyle aynı commit)
> - **Neden:** inceleme: geçici bir hata seçim boyunca tek sayfa / parçasız sayfa bırakıyordu
> - **Etkilenen:** LC-014 (rün başlığında yeniden dene simgesi)
> - **Dosyalar:** src-tauri/src/lib.rs, src/lib/types.ts, src/lib/mock.ts, src/routes/compact/+page.svelte, docs/data-sources.md, CLAUDE.md
> - **Commit:** 9d3013d
>
> ### 2026-11-24 · Claude — → LC-014 kart düzeniyle değiştirildi (`RuneOptions.best`: "Önerilen" kart)

## LC-012 — Küçük pencere seçim başında lane'siz açılıyor
Başlangıç 2026-11-24 · Durum: main'de (merge ac30ea7), canlı test edilmedi (bkz. TODO.md) · İlgili: LC-008, LC-009

> ### 2026-11-24 · Claude — Fix: seçimden önce gelen oturum saklanıyor
> - **Ne:** istemci oturumu `ChampSelect` aşamasından önce gönderirse (`ReadyCheck` vb. sırasında) oturum artık atılmıyor, saklanıyor (`Inner.pending`) ve aşama gelince uygulanıyor; oyun başladıktan sonra gelen oturum eskisi gibi yok sayılıyor (`session_action`). Geliştirme sürümü gelen olayların sırasını terminale yazıyor
> - **Neden:** kullanıcı seçim başında lane'in geç geldiğini / takıldığını gördü; araştırma (3 bakış açısı + doğrulama) en olası nedeni bu buldu: pencere lane'siz açılıp "Lane'in belli değil" gösteriyor, lane ancak istemcinin sonraki güncellemesiyle geliyordu. Olay sırası gerçek istemcide henüz görülmedi
> - **Etkilenen:** LC-008 (`lcu.rs` `Inner::apply`)
> - **Dosyalar:** src-tauri/src/lcu.rs, docs/data-sources.md, TODO.md
> - **Commit:** 5b8dfc9

## LC-011 — Ban önerisi: gösterdiğin şampiyonun counter'ları
Başlangıç 2026-11-24 · Durum: main'de, canlı test edilmedi (bkz. TODO.md) · İlgili: LC-008, LC-009

> ### 2026-11-24 · Claude — Ban aşamasında en kötü 3 eşleşme
> - **Ne:** `lcu.rs` oturumdan ban aşamasını (benim tamamlanmamış `ban` eylemim, kilit yok), tamamlanmış banları ve takım arkadaşlarının şampiyonlarını okur; `scoring::worst_matchups` (30+ maç, düzeltilmiş orana göre en kötü 3); yeni komut `ban_suggestions`. Küçük pencere ban aşamasında "Ban önerisi · X için" başlığıyla 3 satır gösterir (ilk satır "En tehlikeli"); şampiyon gösterilmediyse yönlendirme mesajı, hata durumunda "Tekrar dene". Sahte veri: `banning`, `banningNoIntent`
> - **Neden:** ban ekranında oynayacağın şampiyonun counter'larını banlayabilmek
> - **Etkilenen:** LC-008 (`ChampSelectState`'e `banPhase`, `bans`, `allies`), LC-009 (küçük pencerenin ban görünümü)
> - **Dosyalar:** src-tauri/src/lcu.rs, src-tauri/src/scoring.rs, src-tauri/src/lib.rs, src/routes/compact/+page.svelte, src/lib/types.ts, src/lib/api.ts, src/lib/mock.ts, docs/scoring.md, docs/data-sources.md, README.md
> - **Commit:** 0f48857

## LC-010 — Yayın hazırlığı: lisans, Riot uyarısı, kullanıcı belgesi
Başlangıç 2026-11-23 · Durum: canlı test edildi · İlgili: LC-007

> ### 2026-11-23 · Claude — Herkese açık yayın için eksikler
> - **Ne:** MIT `LICENSE` ve `Cargo.toml` `license = "MIT"`; README'ye "İndir ve kur" (Releases, SmartScreen uyarısı), "Veri ve gizlilik" (u.gg resmi değil, LCU'dan yalnızca şampiyonlar ve kendi lane'in) ve "Lisans ve yasal not" (Riot'un istediği İngilizce metinler) bölümleri; Ayarlar paneline sürüm + Türkçe Riot uyarısı ile "Hakkında"; dış isteklerin User-Agent'ına repo adresi (`lib.rs` `USER_AGENT`)
> - **Neden:** repo public olacak ve v0.1.0 GitHub Releases'te yayınlanacak; yayın denetimi Riot uyarı metni ile lisansın eksik olduğunu buldu
> - **Dosyalar:** LICENSE, README.md, src-tauri/Cargo.toml, src-tauri/src/lib.rs, src/lib/SettingsMenu.svelte
> - **Commit:** 2d2771f

## LC-009 — Küçük pencere: şampiyon seçiminde her zaman üstte
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-008

> ### 2026-11-22 · Claude — Şampiyon seçimi penceresi
> - **Ne:** yeni `compact.rs`: seçim başlayınca birincil ekranın çalışma alanının sağ altında (12 px boşluk) 380×260, çerçevesiz, gölgeli, her zaman üstte, görev çubuğunda görünmeyen ve odak çalmayan pencere; seçim bitince ya da oyun başlayınca yok edilir. Sayfa `src/routes/compact/+page.svelte`: sürüklenebilir başlık (lane, ana pencereyi aç, ×), görünen rakipler (tahmin edilen altın halkalı, tıklayınca değişir), kilitten önce havuzun rakibe karşı ilk 3'ü (oran, etiket, ★ en iyi, "Seçimin"), kilitten sonra şampiyonum vs rakip + genel build (ana rün, ikincil ağaç, 2 büyü, çekirdek eşyalar); lane bilinmiyorsa küçük lane seçici; istemci yok / seçim bekleniyor / havuz boş / veri alınamadı durumları. Yeni komutlar `champion_matchup` (havuz dışı şampiyon için de), `open_main_window`, `close_compact`, `open_compact_preview` (yalnızca geliştirme; ana pencerede dev düğmesi). Ayrı yetki dosyası `capabilities/compact.json` (yalnızca başlıktan sürükleme izni eklendi). Sahte veri: `lolcounters.mockChampSelect`
> - **Neden:** seçim sırasında oyunu bırakmadan öneriyi görmek; oyun içinde kaplama olmasın (Riot kuralı)
> - **Etkilenen:** LC-008 — seçim başlangıç/bitiş kancaları artık pencereyi açıp kapatıyor
> - **Dosyalar:** src-tauri/src/compact.rs, src-tauri/src/lib.rs, src-tauri/capabilities/compact.json, src/routes/compact/+page.svelte, src/routes/+page.svelte, src/lib/api.ts, src/lib/mock.ts, README.md, CLAUDE.md
> - **Commit:** 0a79e9e
>
> ### 2026-11-22 · Claude — Entegrasyon: ana pencereyi açma tek yerde
> - **Ne:** `open_main_window` artık LC-007'nin `launch::show_main` fonksiyonunu çağırır (tepsi ve tek kopya da aynısını kullanır)
> - **Neden:** aynı kod iki yerde durmasın
> - **Etkilenen:** LC-007
> - **Dosyalar:** src-tauri/src/lib.rs
> - **Commit:** 0a79e9e
>
> ### 2026-11-22 · Claude — Fix: inceleme bulguları
> - **Ne:** havuz değişince küçük pencere listeyi yeniliyor: `save_settings` kayıttan sonra `pools-changed` olayını gönderir, küçük pencere bunda öneriyi yeniden ister ("Havuzu düzenle" ile doldurulan havuz artık "havuzun boş" olarak kalmıyor; eski satırlar yenisi gelene kadar görünür, zamanlayıcı yok). Entegrasyon: küçük penceredeki lane seçici LC-008'in `set_my_lane`'ini çağırır (atanmış rol yokken rakip tahmini); LC-007'nin `static-updated` olayında statik veri yeniden yüklenir. Sahte veri: `onPoolsChanged` başka önizleme penceresindeki kaydı `storage` olayıyla duyar, `setMyLane` seçilen lane'in rakibini tahmin eder
> - **Etkilenen:** LC-003 (`save_settings` olay gönderiyor), LC-007, LC-008
> - **Dosyalar:** src-tauri/src/lib.rs, src/routes/compact/+page.svelte, src/lib/api.ts, src/lib/mock.ts
> - **Commit:** 0a79e9e
>
> ### 2026-11-24 · Claude — → LC-011 tarafından değiştirildi
>
> ### 2026-11-24 · Claude — → LC-014 tarafından değiştirildi
>
> ### 2026-11-24 · Claude — → LC-014 kart düzeniyle değiştirildi (pencere 380×356)

## LC-008 — LoL istemcisi (LCU): şampiyon seçimi bağlantısı
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-006, LC-009

> ### 2026-11-22 · Claude — İstemciden şampiyon seçimini okuma
> - **Ne:** yeni `lcu.rs`: kurulum klasörü bulma (ürün ayarı → kayıt defteri → RiotClientInstalls.json → varsayılan), lockfile okuma, klasörü Windows değişiklik bildirimiyle izleme (yoklama yok), `wss://127.0.0.1` bağlantısı (önce yalnızca Riot kökü, reddedilirse yalnızca 127.0.0.1 için yedek doğrulayıcı), oturum ve aşama aboneliği + bağlanınca tek REST eşitlemesi; oturumdan rolüm, kilit, niyetim ve görünen rakipler; rakibimi rol paylarıyla en olası bire bir eşleştirmeyle tahmin; `champ-select` olayı (yalnızca değişince), `get_champ_select_state` ve `set_opponent` komutları; seçim başlangıç/bitiş kancaları (pencereyi LC-009 bağlar)
> - **Neden:** rakibi elle seçmek yerine şampiyon seçiminde otomatik öneri
> - **Etkilenen:** LC-006 — rol payları `lib.rs` `lcu_role_shares()` üzerinden alınır; şimdilik boş, entegrasyonda `role_shares_cached`'e bağlanacak (o zamana kadar rakip tahmini yok, elle seçilebilir)
> - **Dosyalar:** src-tauri/src/lcu.rs, src-tauri/src/lib.rs, src-tauri/Cargo.toml, src-tauri/certs/riotgames.pem, src/lib/types.ts, src/lib/api.ts, src/lib/mock.ts, docs/data-sources.md, README.md, CLAUDE.md
> - **Commit:** 64ed8ba
>
> ### 2026-11-22 · Claude — Entegrasyon: rakip tahmini lane verisine bağlandı
> - **Ne:** `lcu_role_shares()` artık LC-006'nın `role_shares_cached` verisini kullanır (bellek → disk `ugg/{yama}/roles.json` → yoksa bir kez indirme); veri alınamazsa boş döner ve rakip elle seçilir
> - **Neden:** stub boş harita döndürdüğü için rakip tahmini hiç çalışmıyordu
> - **Etkilenen:** LC-006 (`role_shares_cached` artık LCU de kullanıyor)
> - **Dosyalar:** src-tauri/src/lib.rs, src-tauri/tests/lc008.rs, docs/data-sources.md
> - **Commit:** 64ed8ba
>
> ### 2026-11-22 · Claude — Fix: inceleme bulguları
> - **Ne:** (1) olay döngüsü artık ağ beklemiyor: `update` değişikliği uygular, başlangıç/bitiş kancalarını çağırır ve hemen döner; rakip tahmini ayrı bir görevde (`guess_task`) hesaplanır ve yalnızca seçim ve girdi hâlâ aynıysa yazılır. Tahmin yalnızca görünen rakipler ya da lane değişince yeniden hesaplanır (`guess_input`; niyet/gezinme yeni istek başlatmaz); lane verisi alınamazsa o seçimde yeniden denenmez. Böylece yavaş u.gg oyun başlangıcı çerçevesini ve küçük pencerenin kapanmasını geciktirmez. (2) Yeni `set_my_lane` komutu: atanmış rol yokken (pozisyonsuz seçim) elle seçilen lane tahminde kullanılır; seçim bitince sıfırlanır. (3) LCU REST istemcisi `no_proxy()` ile kuruluyor: `HTTPS_PROXY`/Windows proxy ayarı parolayı bir proxy'ye göndermez. Ayrıca seçim başlangıcında LC-007'nin `recheck_static`'i çağrılır. Yeni testler: `tests/lc008.rs` `guess_input_ignores_my_hover`, `guess_input_uses_manual_lane_without_assigned_role`
> - **Etkilenen:** LC-007 (`recheck_static` artık seçim başlangıcında da çalışır), LC-009 (küçük penceredeki lane seçici `set_my_lane`'i çağıracak)
> - **Dosyalar:** src-tauri/src/lcu.rs, src-tauri/src/lib.rs, src-tauri/tests/lc008.rs, src/lib/api.ts, src/lib/mock.ts, docs/data-sources.md
> - **Commit:** 64ed8ba
>
> ### 2026-11-24 · Claude — → LC-011 tarafından değiştirildi
>
> ### 2026-11-24 · Claude — → LC-012 tarafından değiştirildi
>
> ### 2026-11-24 · Claude — → LC-015 tarafından değiştirildi

## LC-007 — Kolay açılış: simge, kurulum, tepsi, tek kopya, autostart
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-001, LC-003, LC-004

> ### 2026-11-22 · Claude — Simge, NSIS kurulumu, tepsi ve Windows açılınca başlatma
> - **Ne:** temaya uygun yeni uygulama simgesi (lacivert zemin + altın elmas; `icons/app-icon.svg`'den üretildi, favicon da); yalnızca NSIS kurulumu (kullanıcıya özel, yönetici izni yok, Türkçe); tepsi simgesi (`Ayarları aç` / `Çık`, sol tık pencereyi açar); kapatınca tepsiye gizleme; tek kopya (ikinci açılış açık pencereyi öne getirir); kurulan sürümde ilk açılışta Windows başlangıcına ekleme (`--minimized` ile tepside gizli başlar); üst çubukta dişli → `Ayarlar` paneli ve anahtar (`get_autostart` / `set_autostart`); `Settings.autostart` alanı
> - **Neden:** uygulama tek tıkla kurulsun, oyun açılmadan hazır beklesin ve kapatınca kaybolmasın
> - **Etkilenen:** LC-003 — eski ayar dosyası bozulmadan okunur; `save_settings` arayüzün göndermediği autostart tercihini diskten korur. LC-004 — üst çubuğa `SettingsMenu` eklendi
> - **Dosyalar:** src-tauri/src/launch.rs, src-tauri/src/lib.rs, src-tauri/src/settings.rs, src-tauri/Cargo.toml, src-tauri/tauri.conf.json, src-tauri/icons/, src/lib/SettingsMenu.svelte, src/lib/api.ts, src/lib/types.ts, src/lib/mock.ts, src/routes/+page.svelte, static/favicon.png, README.md, CLAUDE.md
> - **Commit:** f5ff872
>
> ### 2026-11-22 · Claude — Fix: entegrasyon, eski autostart değeri tercihi ezmesin
> - **Ne:** `keep_backend_fields` artık autostart'ı her zaman diskteki kayıttan alır (önce arayüzden gelen değer öncelikliydi)
> - **Neden:** LC-005 otomatik kaydı `{ ...saved, pools }` gönderiyor; içindeki açılıştaki autostart değeri, sonradan Ayarlar panelinden kapatılan tercihi geri açıyordu
> - **Etkilenen:** LC-005 (PoolEditor otomatik kaydı)
> - **Dosyalar:** src-tauri/src/settings.rs, src-tauri/tests/lc007.rs
> - **Commit:** f5ff872
>
> ### 2026-11-22 · Claude — Fix: inceleme bulguları
> - **Ne:** süreç tepside günlerce yaşadığı için kesin olmayan statik veri artık oturum boyunca kilitlenmiyor: `static_settled` (en yeni Data Dragon sürümü + u.gg'nin o sürümün kendi yaması) yanlışsa veri geçici sayılır ve ana pencere gösterilince (`show_main` → `recheck_static`) arka planda yeniden yüklenir; sürüm ya da yama değişirse `static-updated` olayı gider ve ana sayfa `load_static`'i yeniden ister. Aynı anda boş gelmiş şampiyon/lane verisi bellekten atılır (sonraki istekte yeniden sorulur). Zamanlayıcı yok; veri kesinse ağ isteği yok. Yeni test: `tests/lc007.rs` `only_latest_version_with_its_own_patch_is_settled`
> - **Etkilenen:** LC-002 (`static_payload` → `resolve_static` + `store_static`), LC-006 (boş lane verisi bellekten atılabiliyor)
> - **Dosyalar:** src-tauri/src/lib.rs, src-tauri/src/launch.rs, src-tauri/tests/lc007.rs, src/lib/api.ts, src/lib/mock.ts, src/routes/+page.svelte, docs/data-sources.md
> - **Commit:** f5ff872

## LC-006 — Lane filtresi
Başlangıç 2026-11-22 · Durum: canlı test edildi · İlgili: LC-002, LC-004

> ### 2026-11-22 · Claude — Lane'e göre şampiyon listesi
> - **Ne:** Havuzum'daki şampiyon ızgarası ve Counter'daki rakip seçimi varsayılan olarak yalnızca seçili lane'de oynanan şampiyonları gösterir; yanında "<Lane> şampiyonları | Hepsi" seçici var; arama kutusu doluyken bütün şampiyonlarda arar. Veri u.gg champion_ranking dosyasından (yama başına bir kez, `ugg/{yama}/roles.json`); lane payı %10 (`ROLE_MIN_SHARE`) ve üstü sayılır; verisi olmayan şampiyon her lane'de görünür; veri alınamazsa liste sessizce tam kalır. Yeni `get_roles` komutu ve diğer Rust kodu (LCU rakip tahmini) için `role_shares_cached`
> - **Neden:** LoL istemcisindeki rol filtresi gibi; 170+ şampiyon içinde aramadan seçim
> - **Etkilenen:** LC-004 (`ChampionPicker` arama satırına `toolbar` snippet'i eklendi); LC-002 (`UggPatch.rankingVersion` alanı; eski `patch.json` kayıtlarında varsayılan `1.5.0`)
> - **Dosyalar:** src-tauri/src/ugg.rs, src-tauri/src/lib.rs, src-tauri/src/cache.rs, src-tauri/tests/lc006_lane_filter.rs, src/lib/roles.ts, src/lib/LaneFilter.svelte, src/lib/ChampionPicker.svelte, src/lib/PoolEditor.svelte, src/lib/CounterView.svelte, src/lib/api.ts, src/lib/types.ts, src/lib/mock.ts, docs/data-sources.md, README.md, CLAUDE.md
> - **Commit:** 72754e6
>
> ### 2026-11-22 · Claude — Fix: inceleme bulguları
> - **Ne:** lane verisi bir kez alınamazsa (ör. Windows açılışında ağ henüz yokken) filtre artık oturum boyunca kaybolmuyor: `roles.ts` `watchRoles()` veri gelene kadar pencere gösterilince ya da odak alınca yeniden sorar (zamanlayıcı yok); Havuzum ve Counter bunu kullanır
> - **Dosyalar:** src/lib/roles.ts, src/lib/PoolEditor.svelte, src/lib/CounterView.svelte
> - **Commit:** 72754e6

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
