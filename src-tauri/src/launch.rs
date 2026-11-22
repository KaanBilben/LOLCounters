//! Kolay açılış: tepsi simgesi, kapatınca tepsiye gizleme, `--minimized` ile gizli başlama
//! ve Windows açılınca başlatma (autostart).
//!
//! Hepsi olay güdümlüdür (tepsi tıklaması, pencere olayı, Run kaydı); zamanlayıcı yoktur.

use crate::settings;
use serde::Serialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Window, WindowEvent};
use tauri_plugin_autostart::ManagerExt;

/// Autostart kaydına eklenen argüman: uygulama pencere açmadan tepside başlar.
pub const MINIMIZED_ARG: &str = "--minimized";

/// Geliştirme (debug) derlemesinde autostart hiç kaydedilmez; yalnızca kurulan sürüm kaydedilir.
const AUTOSTART_AVAILABLE: bool = cfg!(not(debug_assertions));

/// Uygulama `--minimized` ile mi başlatıldı (Windows açılışında autostart böyle başlatır).
pub fn started_minimized<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|a| a.as_ref() == MINIMIZED_ARG)
}

/// Ana pencereyi gösterir, simge durumundan çıkarır ve öne getirir. Uygulama tepside günlerce
/// yaşayabildiğinden kesin olmayan statik veri bu anda yeniden denenir (`recheck_static`).
pub fn show_main(app: &AppHandle) {
    crate::recheck_static(app);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// `Builder::setup`: tepsi simgesi, ilk açılışta autostart, pencereyi gösterme.
pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    build_tray(app)?;
    if AUTOSTART_AVAILABLE {
        enable_on_first_run(app.handle());
    }
    if !started_minimized(std::env::args()) {
        show_main(app.handle());
    }
    Ok(())
}

/// `Builder::on_window_event`: ana pencere kapatılınca uygulama kapanmaz, tepsiye gizlenir.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event
        && window.label() == "main"
    {
        api.prevent_close();
        let _ = window.hide();
    }
}

fn build_tray(app: &App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Ayarları aç", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Çık", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("LOLCounters")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id().as_ref() {
            "open" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = e
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// İlk açılışta (tercih hiç kaydedilmemişse) autostart'ı açar ve `Some(true)` kaydeder.
/// Kayıt başarısızsa tercih boş kalır; sonraki açılışta yeniden denenir.
fn enable_on_first_run(app: &AppHandle) {
    let Ok(path) = crate::settings_path(app) else {
        return;
    };
    let mut s = settings::load(&path);
    if s.autostart.is_none() && app.autolaunch().enable().is_ok() {
        s.autostart = Some(true);
        let _ = settings::save(&path, &s);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutostartState {
    /// Windows açılınca başlatma kayıtlı mı (gerçek durum: Run kaydı).
    pub enabled: bool,
    /// `false`: geliştirme derlemesi; anahtar kapalı gösterilir.
    pub available: bool,
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> AutostartState {
    AutostartState {
        enabled: AUTOSTART_AVAILABLE && app.autolaunch().is_enabled().unwrap_or(false),
        available: AUTOSTART_AVAILABLE,
    }
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<AutostartState, String> {
    if !AUTOSTART_AVAILABLE {
        return Err("Geliştirme modunda kapalı".to_string());
    }
    let manager = app.autolaunch();
    let changed = if enabled { manager.enable() } else { manager.disable() };
    changed.map_err(|e| format!("Başlangıç ayarı değiştirilemedi: {e}"))?;
    let path = crate::settings_path(&app)?;
    let mut s = settings::load(&path);
    s.autostart = Some(enabled);
    settings::save(&path, &s).map_err(|e| format!("Ayarlar kaydedilemedi: {e}"))?;
    Ok(get_autostart(app))
}
