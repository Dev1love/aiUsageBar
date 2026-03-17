mod api;
mod keychain;
mod tray_icon;

use tauri::{
    image::Image,
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};

const RETINA_ICON_SIZE: u32 = 44;
const POPUP_LABEL: &str = "popup";
const POPUP_WIDTH: f64 = 320.0;
const POPUP_HEIGHT: f64 = 400.0;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            // Render default tray icon (green bars at 0%)
            let icon_rgba = tray_icon::render_default_icon();
            let icon = Image::new_owned(icon_rgba, RETINA_ICON_SIZE, RETINA_ICON_SIZE);

            let app_handle = app.handle().clone();

            TrayIconBuilder::new()
                .icon(icon)
                .icon_as_template(false)
                .tooltip("ClaudeBar")
                .on_tray_icon_event(move |_tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        toggle_popup(&app_handle);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_popup(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(POPUP_LABEL) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        // Create the popup window
        let builder = WebviewWindowBuilder::new(app, POPUP_LABEL, WebviewUrl::App("index.html".into()))
            .title("ClaudeBar")
            .inner_size(POPUP_WIDTH, POPUP_HEIGHT)
            .decorations(false)
            .resizable(false)
            .always_on_top(true)
            .visible(true)
            .skip_taskbar(true);

        match builder.build() {
            Ok(_) => {}
            Err(e) => eprintln!("Failed to create popup window: {e}"),
        }
    }
}
