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
