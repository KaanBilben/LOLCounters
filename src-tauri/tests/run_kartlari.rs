// LC-014 kart düzeni karar testleri (canlı testten sonra silinir).

use lolcounters_lib::scoring::best_rune_option;
use lolcounters_lib::ugg::RunePage;

fn page(keystone: i64, wins: u32, matches: u32) -> RunePage {
    RunePage {
        primary_tree: 8000,
        secondary_tree: 8300,
        perks: vec![keystone, 1, 2, 3, 4, 5],
        shards: vec![],
        matches,
        wins,
    }
}

/// Karar: "Önerilen" = kazanma oranı en yüksek kart; oran eşitse maçı çok olan.
#[test]
fn recommended_is_the_highest_win_rate_then_more_games() {
    let opts = [page(1, 5000, 10000), page(2, 530, 1000), page(3, 1060, 2000)];
    assert_eq!(best_rune_option(&opts, None), Some(2));
}

/// Karar: 100 maçtan az kart "Önerilen" olamaz; hiçbiri yetmezse etiket yok.
#[test]
fn pages_under_100_games_are_never_recommended() {
    let opts = [page(1, 500, 1000), page(2, 90, 99)];
    assert_eq!(best_rune_option(&opts, None), Some(0));
    assert_eq!(best_rune_option(&[page(1, 50, 60)], None), None);
}

/// Karar: eşleşme kartının oranı yalnızca bu rakibe karşı; "Önerilen" adayı sayılmaz.
#[test]
fn matchup_card_is_not_a_candidate() {
    let opts = [page(1, 500, 1000), page(2, 700, 1000)];
    assert_eq!(best_rune_option(&opts, Some(1)), Some(0));
}
