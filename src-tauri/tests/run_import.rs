// LC-015 karar testleri (canlı testten sonra silinir).

use lolcounters_lib::ddragon::{Rune, order_perks};
use lolcounters_lib::lcu::{IMPORT_PAGE_NAME, ImportTarget, PerkPageInfo, import_target, order_spells};
use std::collections::HashMap;

fn info(id: i64, name: &str, editable: bool, current: bool) -> PerkPageInfo {
    PerkPageInfo { id, name: name.into(), is_editable: editable, current }
}

#[test]
fn existing_lolcounters_page_is_overwritten_without_asking() {
    let pages = [info(1, "Benim", true, true), info(7, IMPORT_PAGE_NAME, true, false)];
    assert_eq!(import_target(&pages, false, None), ImportTarget::Update(7));
    // yoksa ve yer varsa yeni sayfa
    assert_eq!(import_target(&pages[..1], true, None), ImportTarget::Create);
}

#[test]
fn full_inventory_asks_then_overwrites_the_selected_editable_page() {
    let pages = [info(1, "Varsayılan", false, false), info(2, "A", true, false), info(3, "B", true, true)];
    assert_eq!(import_target(&pages, false, None), ImportTarget::Ask(3, "B".into()));
    assert_eq!(import_target(&pages, false, Some(3)), ImportTarget::Overwrite(3));
    // onaydan sonra seçili sayfa değiştiyse yeniden sorulur
    assert_eq!(import_target(&pages, false, Some(2)), ImportTarget::Ask(3, "B".into()));
    // seçili sayfa düzenlenemiyorsa ilk düzenlenebilir sayfa
    let pages = [info(1, "Varsayılan", false, true), info(2, "A", true, false)];
    assert_eq!(import_target(&pages, false, None), ImportTarget::Ask(2, "A".into()));
    assert_eq!(import_target(&pages[..1], false, Some(1)), ImportTarget::NoPage);
}

fn rune(id: i64, tree: i64, slot: u8) -> (i64, Rune) {
    (id, Rune { id, name: String::new(), icon: String::new(), tree, slot })
}

#[test]
fn perks_are_ordered_by_tree_and_slot_for_the_client() {
    // Fatih sayfası: Hassasiyet (8000) ana, Kararlılık (8400) ikincil
    let runes: HashMap<i64, Rune> = [
        rune(8010, 8000, 0),
        rune(9111, 8000, 1),
        rune(9104, 8000, 2),
        rune(8299, 8000, 3),
        rune(8444, 8400, 1),
        rune(8242, 8400, 2),
    ]
    .into_iter()
    .collect();
    // u.gg sırası: ana rün başta, gerisi sayıca artan
    let perks = [8010, 8242, 8299, 8444, 9104, 9111];
    assert_eq!(
        order_perks(&runes, 8000, 8400, &perks, &[5005, 5008, 5001]),
        Some([8010, 9111, 9104, 8299, 8444, 8242, 5005, 5008, 5001])
    );
    // eksik parça ya da eksik rün: içe aktarılmaz
    assert_eq!(order_perks(&runes, 8000, 8400, &perks, &[]), None);
    assert_eq!(order_perks(&runes, 8000, 8400, &perks[..5], &[5005, 5008, 5001]), None);
}

#[test]
fn spells_keep_the_key_they_are_already_on() {
    // Flash F'de (spell2) ise orada kalır
    assert_eq!(order_spells((14, 4), [4, 12]), (12, 4));
    // Flash D'de ise orada kalır
    assert_eq!(order_spells((4, 14), [4, 12]), (4, 12));
    // ortak büyü yoksa u.gg sırası
    assert_eq!(order_spells((11, 3), [4, 12]), (4, 12));
}
