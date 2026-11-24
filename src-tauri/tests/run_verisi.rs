// LC-013 karar testleri (canlı testten sonra silinir).

use lolcounters_lib::scoring::pick_rune_options;
use lolcounters_lib::settings::Lane;
use lolcounters_lib::ugg::{RunePage, parse_rune_options};
use serde_json::json;

fn page(primary: i64, secondary: i64, perks: &[i64], wins: u32, matches: u32) -> RunePage {
    RunePage {
        primary_tree: primary,
        secondary_tree: secondary,
        perks: perks.to_vec(),
        shards: vec![],
        matches,
        wins,
    }
}

/// Karar: 3 seçenek = maça göre en çok oynanan ana rün + ikincil ağaç grupları; her grubun
/// en çok oynanan tam sayfası (u.gg grupları sırasız verir).
#[test]
fn options_are_most_played_groups_with_their_most_played_page() {
    let v = json!({"12": {"17": {"4": [[
        [8010, 8300, 251, 484, 8000, [[8000, 8300, [8010, 1, 2, 3, 4, 5], 5, 7]]],
        [8010, 8200, 14320, 28040, 8000, [
            [8000, 8200, [8010, 11, 12, 13, 14, 15], 93, 168],
            [8000, 8200, [8010, 21, 22, 23, 24, 25], 10741, 21095]
        ]],
        [8230, 8000, 13425, 26351, 8200, [[8200, 8000, [8230, 31, 32, 33, 34, 35], 11401, 22517]]]
    ]]}}});
    let got = &parse_rune_options(&v)[&Lane::Top];
    let keystones: Vec<i64> = got.iter().map(|p| p.perks[0]).collect();
    assert_eq!(keystones, vec![8010, 8230, 8010]);
    assert_eq!(got[0].perks, vec![8010, 21, 22, 23, 24, 25]);
    assert_eq!((got[0].wins, got[0].matches), (10741, 21095));
    assert_eq!(got[2].secondary_tree, 8300);
}

fn general() -> Vec<RunePage> {
    vec![
        page(8000, 8400, &[8010, 1, 2, 3, 4, 5], 10, 300),
        page(8200, 8000, &[8230, 1, 2, 3, 4, 5], 10, 200),
        page(8000, 8300, &[8010, 1, 2, 3, 4, 5], 10, 100),
    ]
}

/// Karar: eşleşme sayfası aynı ana rün + ikincil ağaçtaki seçeneğin yerine geçer ve altın olur.
#[test]
fn matchup_page_takes_the_place_of_its_group() {
    let mu = page(8200, 8000, &[8230, 9, 9, 9, 9, 9], 60, 120);
    let (opts, gold) = pick_rune_options(&general(), Some(&mu));
    assert_eq!(gold, Some(1));
    assert_eq!(opts.len(), 3);
    assert_eq!(opts[1], mu);
}

/// Karar: eşleşme sayfası ilk 3'te yoksa 3. seçeneğin yerine geçer.
#[test]
fn matchup_page_outside_top_three_replaces_the_third() {
    let mu = page(8100, 8200, &[8112, 9, 9, 9, 9, 9], 60, 120);
    let (opts, gold) = pick_rune_options(&general(), Some(&mu));
    assert_eq!(gold, Some(2));
    assert_eq!(opts[2], mu);
    assert_eq!(opts[0], general()[0]);
}

/// Karar: 100 maçtan az eşleşme sayfası vurgulanmaz, seçenekler değişmez.
#[test]
fn small_matchup_sample_is_not_highlighted() {
    let mu = page(8100, 8200, &[8112, 9, 9, 9, 9, 9], 30, 60);
    let (opts, gold) = pick_rune_options(&general(), Some(&mu));
    assert_eq!(gold, None);
    assert_eq!(opts, general());
}
