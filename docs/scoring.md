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
