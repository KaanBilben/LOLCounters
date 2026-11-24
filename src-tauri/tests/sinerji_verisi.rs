// LC-016 karar testleri (canlı testten sonra silinir).

use lolcounters_lib::lcu::parse_session;
use lolcounters_lib::scoring::{DuoCandidate, best_synergies, duo_synergies};
use lolcounters_lib::settings::Lane;
use lolcounters_lib::ugg::parse_champion_duos;
use serde_json::json;

fn cand(champion: i64, wins: u32, matches: u32, own: f64, counter: bool) -> DuoCandidate {
    DuoCandidate { champion, wins, matches, own_winrate: own, counter }
}

/// Karar: sinerji = ikilinin düzeltilmiş oranı − iki şampiyonun tek başına oranlarının
/// ortalaması (yüzde puanı); 200 maçtan az ikili sayılmaz; sinerjiye göre sıralı.
#[test]
fn synergy_is_duo_rate_minus_the_two_solo_rates_and_needs_200_games() {
    // (5450 + 50) / (10000 + 100) = %54,455; ortalama (%51 + %50) / 2 = %50,5 → +3,955
    let all = duo_synergies(
        &[cand(1, 5450, 10000, 0.51, false), cand(2, 150, 199, 0.50, false), cand(3, 1000, 2000, 0.50, false)],
        0.50,
    );
    let ids: Vec<i64> = all.iter().map(|s| s.champion).collect();
    assert_eq!(ids, vec![1, 3]);
    assert!((all[0].synergy - 3.955).abs() < 0.01, "{}", all[0].synergy);
    assert!(all[0].good);
    assert!(!all[1].good); // %50 − %50 = 0
}

/// Karar: sinerji listesi = en iyi 5 artı sinerjili şampiyon; rakibe de iyi olanlar ("+ counter")
/// başa geçer; banlanmış / alınmış şampiyonlar çıkmaz.
#[test]
fn synergy_list_puts_counters_first_and_skips_taken_champions() {
    let all = duo_synergies(
        &[
            cand(1, 5600, 10000, 0.50, false), // +5,5
            cand(2, 5500, 10000, 0.50, true),  // +4,5, counter
            cand(3, 5400, 10000, 0.50, false), // +3,5 ama banlı
            cand(4, 5300, 10000, 0.50, false),
            cand(5, 5200, 10000, 0.50, false),
            cand(6, 5150, 10000, 0.50, false),
            cand(7, 5120, 10000, 0.50, true), // 6. sırada: listeye girmez
            cand(8, 4800, 10000, 0.50, false), // eksi sinerji: hiç girmez
        ],
        0.50,
    );
    let best: Vec<i64> = best_synergies(&all, &[3]).iter().map(|s| s.champion).collect();
    assert_eq!(best, vec![2, 1, 4, 5, 6]);
}

/// Karar: partner = takımda benim lane'imi tamamlayan oyuncu (ADC ↔ support); kilitlemediyse
/// gösterdiği şampiyon, kilitleyip kilitlemediği ayrıca bilinir.
#[test]
fn duo_partner_is_the_ally_on_the_other_bot_lane_role() {
    let session = |me: &str, partner_done: bool| {
        json!({
            "localPlayerCellId": 1,
            "myTeam": [
                {"cellId": 0, "championId": 222, "championPickIntent": 0, "assignedPosition": "bottom"},
                {"cellId": 1, "championId": 0, "championPickIntent": 412, "assignedPosition": me},
                {"cellId": 2, "championId": 64, "championPickIntent": 0, "assignedPosition": "jungle"}
            ],
            "theirTeam": [],
            "actions": [[
                {"actorCellId": 0, "championId": 222, "completed": partner_done, "type": "pick"}
            ]]
        })
    };
    let hover = parse_session(&session("utility", false));
    assert_eq!(hover.duo_partner, Some(222));
    assert!(!hover.duo_partner_locked);
    assert!(parse_session(&session("utility", true)).duo_partner_locked);
    // orman oyuncusunun partneri yok
    assert_eq!(parse_session(&session("jungle", true)).duo_partner, None);
}

/// Karar: u.gg `champion_duos` satırı `[partner, galibiyet, maç, …, partner rolü]`; dünya
/// verisi, rank sırasıyla ilk dolu olan.
#[test]
fn champion_duos_rows_are_read_per_lane() {
    let v = json!({"12": {"17": {"3": [[
        [201, 2097, 3867, 0, 0, 0, 0, 0, 0, 0, 2],
        [412, 6298, 11990, 0, 0, 0, 0, 0, 0, 0, 2]
    ], "2026-11-24"]}}});
    let got = &parse_champion_duos(&v)[&Lane::Adc];
    assert_eq!(got.len(), 2);
    assert_eq!((got[0].partner, got[0].wins, got[0].matches), (201, 2097, 3867));
}
