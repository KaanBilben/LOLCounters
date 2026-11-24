// LC-012 karar testleri (canlı testten sonra silinir).

use lolcounters_lib::lcu::{SessionAction, session_action};

/// Karar: seçimden hemen önce ("hazır mısın", sıra, lobi) gelen oturum atılmaz, saklanır;
/// "ChampSelect" gelince pencere bu oturumla (lane'iyle) açılır.
#[test]
fn session_before_champ_select_is_kept() {
    for phase in ["ReadyCheck", "Matchmaking", "Lobby"] {
        assert_eq!(session_action(Some(phase)), SessionAction::Hold, "{phase}");
    }
    assert_eq!(session_action(Some("ChampSelect")), SessionAction::Apply);
    assert_eq!(session_action(None), SessionAction::Apply);
}

/// Karar: oyun başladıktan sonra gelen geç oturum güncellemesi pencereyi yeniden açmaz.
#[test]
fn late_session_after_game_start_is_ignored() {
    for phase in ["GameStart", "InProgress", "EndOfGame"] {
        assert_eq!(session_action(Some(phase)), SessionAction::Ignore, "{phase}");
    }
}
