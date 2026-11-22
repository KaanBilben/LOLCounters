//! Tauri komutları ve uygulama durumu.

pub mod cache;
pub mod ddragon;
pub mod settings;
pub mod ugg;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
