// LC-011 karar testleri (canlı testten sonra silinir).

use lolcounters_lib::lcu::parse_session;
use lolcounters_lib::scoring::{Candidate, worst_matchups};
use serde_json::json;

fn session(my_ban_done: bool) -> serde_json::Value {
    json!({
        "localPlayerCellId": 2,
        "myTeam": [
            {"cellId": 0, "championId": 0, "championPickIntent": 157, "assignedPosition": "jungle"},
            {"cellId": 2, "championId": 0, "championPickIntent": 122, "assignedPosition": "top"}
        ],
        "theirTeam": [],
        "actions": [[
            {"actorCellId": 0, "championId": 238, "completed": true, "type": "ban"},
            {"actorCellId": 2, "championId": 0, "completed": my_ban_done, "type": "ban"}
        ]]
    })
}

/// Karar: ban önerisi yalnızca benim ban sıram tamamlanmadan görünür.
#[test]
fn ban_phase_ends_when_my_ban_is_done() {
    let open = parse_session(&session(false));
    assert!(open.ban_phase);
    assert_eq!(open.my_intent, Some(122));
    assert_eq!(open.bans, vec![238]);
    assert_eq!(open.allies, vec![157]);
    assert!(!parse_session(&session(true)).ban_phase);
}

fn cand(champion: i64, wins: u32, matches: u32) -> Candidate {
    Candidate { champion, pool_index: 0, wins, matches }
}

/// Karar: banlanmış ve takımın gösterdiği şampiyonlar önerilmez.
#[test]
fn excluded_champions_are_skipped() {
    let rows = [cand(1, 300, 1000), cand(2, 350, 1000), cand(3, 400, 1000), cand(4, 450, 1000)];
    let got: Vec<i64> = worst_matchups(&rows, &[1, 3]).iter().map(|s| s.champion).collect();
    assert_eq!(got, vec![2, 4]);
}

/// Karar: 30 maçtan azı alınmaz; en kötü 3 eşleşme en kötüden başlayarak verilir.
#[test]
fn worst_three_with_enough_games() {
    let rows = [
        cand(1, 520, 1000),
        cand(2, 2, 20), // az maç
        cand(3, 400, 1000),
        cand(4, 450, 1000),
        cand(5, 480, 1000),
    ];
    let got: Vec<i64> = worst_matchups(&rows, &[]).iter().map(|s| s.champion).collect();
    assert_eq!(got, vec![3, 4, 5]);
}
