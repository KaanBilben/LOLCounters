// LC-010 karar testleri (hafif TDD). Canlı testten sonra silinecek.
use std::fs;

fn repo_file(name: &str) -> String {
    let path = format!("{}/../{name}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path} okunamadı"))
}

// Karar: kod MIT lisanslı; LICENSE dosyası ve Cargo.toml aynı lisansı söyler.
#[test]
fn license_is_mit_everywhere() {
    assert_eq!(env!("CARGO_PKG_LICENSE"), "MIT");
    let license = repo_file("LICENSE");
    assert!(license.starts_with("MIT License"));
    assert!(license.contains("KaanBilben"));
}

// Karar: README Riot'un istediği uyarı metnini içerir.
#[test]
fn readme_has_riot_disclaimer() {
    let readme = repo_file("README.md");
    assert!(readme.contains("isn't endorsed by Riot Games"));
    assert!(readme.contains("Legal Jibber Jabber"));
}

// Karar: u.gg'ye giden istekler uygulamanın adını ve repo adresini söyler.
#[test]
fn user_agent_names_app_and_repo() {
    let ua = lolcounters_lib::USER_AGENT;
    assert!(ua.starts_with("LOLCounters/0.1"));
    assert!(ua.contains("github.com/KaanBilben/LOLCounters"));
}
