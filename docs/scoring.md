# Öneri puanlaması

Kod ve sabitler: `src-tauri/src/scoring.rs`.

Havuzdaki her şampiyon için, seçilen rakibe karşı o lane'deki u.gg eşleşme
verisi kullanılır.

1. **Düzeltilmiş kazanma oranı:** az maçlı eşleşmeler %50'ye çekilir:
   `(galibiyet + 50) / (maç + 100)`. Veri yoksa %50 sayılır.
2. **Tercih bonusu:** havuzda `n` şampiyon varsa, `i`. sıradakine (0'dan)
   `3 × (1 − i / n)` puan eklenir. İlk sıradaki +3, sonrakiler giderek azalır.
3. **Puan** = düzeltilmiş oran (yüzde) + tercih bonusu. Büyükten küçüğe sıralanır.

Etiketler (düzeltilmiş orana göre):

| Etiket | Koşul |
|---|---|
| Az veri | 30 maçtan az |
| İyi eşleşme | ≥ %52 |
| Zor eşleşme | ≤ %48 |
| Dengeli | arası |

Örnek: ilk tercihin %45, ikinci tercihin %53 ise (2 şampiyonluk havuz):
45 + 3 = 48 < 53 + 1,5 = 54,5 → ikinci tercih önerilir. Oranlar yakınsa
tercih sırası kazanır.

## Ban önerisi

Ban aşamasında, gösterdiğin şampiyonun atanmış lane'indeki u.gg eşleşme
verisinden en kötü 3 eşleşme gösterilir (`worst_matchups`,
`BAN_SUGGESTIONS = 3`).

- Oran, senin şampiyonunun o rakibe karşı sonucudur; etiketler yukarıdaki
  tabloyla aynıdır ("Zor eşleşme" = iyi bir ban adayı).
- 30 maçtan az olan rakipler alınmaz. Banlanmış şampiyonlar, takım
  arkadaşlarının seçtiği ya da gösterdiği şampiyonlar ve şampiyonun kendisi
  de listeye girmez.
- Düzeltilmiş orana göre küçükten büyüğe sıralanır (eşitlikte çok maçlı önce).
  Tercih bonusu burada kullanılmaz.

## Rün seçenekleri

Kilitledikten sonra küçük pencerede şampiyonunun o lane'deki rün sayfaları
gösterilir (`pick_rune_options`, `RUNE_OPTIONS = 3`). Veri:
[data-sources.md](data-sources.md#rün-seçenekleri-ve-eşleşme-sayfası-lc-013).

- **Seçenekler:** ana rün + ikincil ağaç grupları maç sayısına göre; her grubun
  en çok oynanan tam sayfası. Oran ve maç sayısı o sayfanındır.
- **"Eşleşmeye özel" kartı:** u.gg her rakip için tek bir önerilen sayfa verir. Bu sayfa
  en az `MATCHUP_PAGE_MIN_GAMES = 100` maçlıysa sarı "Eşleşmeye özel" etiketi alır: aynı ana rün +
  ikincil ağaçtaki seçeneğin yerine geçer; böyle bir seçenek yoksa 3.
  seçeneğin yerine. 100 maçtan azsa etiket yoktur, seçenekler değişmez.
- **"Önerilen" kartı** (`best_rune_option`): en az `RECOMMENDED_PAGE_MIN_GAMES = 100`
  maçlı kartlar içinde kazanma oranı en yüksek olan; oran eşitse maçı çok olan.
  Eşleşme kartı aday değildir (oranı yalnızca bu rakibe karşı, genel kartlarla aynı
  ölçekte değil), bu yüzden iki etiket aynı karta düşmez.
- **Açılışta seçili kart:** eşleşme kartı, yoksa "Önerilen", yoksa ilk kart.
- **Büyüler:** eşleşme kartı varsa eşleşmenin büyüleri, yoksa lane'in genel
  büyüleri.

## Bot lane sinerjisi

ADC ya da support oynarken, partnerimin (gösterdiği ya da kilitlediği)
şampiyonuyla hangi şampiyonların iyi gittiği (`duo_synergies`,
`best_synergies`). Veri:
[data-sources.md](data-sources.md#bot-lane-ikilileri-lc-016).

- **Sinerji** (yüzde puanı) = ikilinin düzeltilmiş oranı
  `(galibiyet + 50) / (maç + 100)` − iki şampiyonun o lane'lerdeki tek başına
  oranlarının ortalaması. Tek başına oranlar lane filtresinin verisinden.
  İki güçlü şampiyon birlikte zaten çok kazanır; bu yüzden ham ikili oranı
  değil, beklenenin üstündeki kısım ölçülür.
- `DUO_MIN_GAMES = 200` maçtan az ikili hesaba katılmaz.
- **İyi sinerji:** `GOOD_SYNERGY = 1,5` puan ve üstü.
- **Sinerji listesi:** sinerjisi artı olan en iyi `SYNERGY_LIST = 5` şampiyon
  (havuzla sınırlı değil). Banlanmış ve alınmış şampiyonlar, partner ve rakip
  çıkarılır. Rakibime karşı da iyi eşleşenler ("+ counter": yukarıdaki
  "İyi eşleşme" kuralı, rakibin eşleşme verisinden) başa geçer; iki grup kendi
  içinde sinerji sırasını korur.
- Kör seçimde ve başka lane'lerde sinerji yoktur.
