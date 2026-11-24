//! Küçük pencere: şampiyon seçimi sürerken ekranın sağ altında, her zaman üstte.
//!
//! Seçim başlayınca oluşturulur, bitince (oyun başlangıcı dahil) yok edilir: oyun içinde
//! hiçbir zaman açık kalmaz (Riot kuralı: oyun içi kaplama yok). Her açılışta yeniden
//! oluşturulması odak çalmamayı garanti eder (`focused(false)` yalnızca ilk gösterimde
//! geçerlidir) ve kapalıyken WebView süreci hiç çalışmaz.

use tauri::webview::Color;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const LABEL: &str = "compact";
/// Mantıksal piksel.
pub const WIDTH: f64 = 380.0;
/// LC-014: rün kartları (yan yana 3 dikey kart) sığsın diye 260'tan uzatıldı.
pub const HEIGHT: f64 = 356.0;
/// Çalışma alanı kenarına uzaklık.
pub const MARGIN: f64 = 12.0;
/// `--bg` (#010a13): pencere WebView çizilmeden önce de koyu görünsün.
const BACKGROUND: Color = Color(1, 10, 19, 255);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactAction {
    Open,
    Close,
}

/// Seçim durumu → pencere kararı: seçim başlayınca aç, bitince kapat; açıksa dokunma.
pub fn decide(window_open: bool, champ_select_active: bool) -> Option<CompactAction> {
    match (window_open, champ_select_active) {
        (false, true) => Some(CompactAction::Open),
        (true, false) => Some(CompactAction::Close),
        _ => None,
    }
}

/// Çalışma alanının (fiziksel piksel: konum, boyut) sağ alt köşesine göre pencerenin
/// mantıksal sol üst köşesi.
pub fn bottom_right(
    area_pos: (i32, i32),
    area_size: (u32, u32),
    scale: f64,
    window: (f64, f64),
    margin: f64,
) -> (f64, f64) {
    let right = (f64::from(area_pos.0) + f64::from(area_size.0)) / scale;
    let bottom = (f64::from(area_pos.1) + f64::from(area_size.1)) / scale;
    (right - window.0 - margin, bottom - window.1 - margin)
}

/// Pencereyi açar (zaten açıksa bir şey yapmaz).
pub fn open(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(LABEL).is_some() {
        return Ok(());
    }
    let (x, y) = match app.primary_monitor()? {
        Some(m) => {
            let wa = m.work_area();
            bottom_right(
                (wa.position.x, wa.position.y),
                (wa.size.width, wa.size.height),
                m.scale_factor(),
                (WIDTH, HEIGHT),
                MARGIN,
            )
        }
        None => (MARGIN, MARGIN),
    };
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("compact".into()))
        .title("LOLCounters")
        .inner_size(WIDTH, HEIGHT)
        .position(x, y)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(true)
        .background_color(BACKGROUND)
        .build()?;
    Ok(())
}

/// Pencereyi yok eder (açık değilse bir şey yapmaz).
pub fn close(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.destroy();
    }
}

/// Seçim başladı/bitti kancalarından çağrılır.
pub fn sync(app: &AppHandle, champ_select_active: bool) {
    let open_now = app.get_webview_window(LABEL).is_some();
    match decide(open_now, champ_select_active) {
        Some(CompactAction::Open) => {
            if let Err(e) = open(app) {
                eprintln!("Küçük pencere açılamadı: {e}");
            }
        }
        Some(CompactAction::Close) => close(app),
        None => {}
    }
}
