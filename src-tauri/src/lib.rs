mod api;
mod keychain;
mod tray_icon;

use std::sync::Mutex;

use tauri::{
    image::Image,
    tray::TrayIconBuilder,
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};

use api::UsageData;

const RETINA_ICON_SIZE: u32 = 44;
const POPUP_LABEL: &str = "popup";
const POPUP_WIDTH: f64 = 320.0;
const POPUP_HEIGHT: f64 = 400.0;
const POLL_INTERVAL_SECS: u64 = 60;

/// Shared state holding the latest usage data.
struct UsageState(Mutex<Option<UsageData>>);

#[tauri::command]
fn get_usage(state: tauri::State<'_, UsageState>) -> Result<Option<UsageData>, String> {
    let data = state.0.lock().map_err(|e| e.to_string())?;
    Ok(data.clone())
}

/// Perform a single poll: read keychain, fetch usage, emit events, update state.
async fn poll_usage(app_handle: &tauri::AppHandle) {
    let credentials = match keychain::read_credentials() {
        Ok(c) => c,
        Err(e) => {
            let _ = app_handle.emit("usage-error", e);
            return;
        }
    };

    match api::fetch_usage(&credentials.access_token).await {
        Ok(usage) => {
            // Update shared state
            if let Some(state) = app_handle.try_state::<UsageState>() {
                if let Ok(mut data) = state.0.lock() {
                    *data = Some(usage.clone());
                }
            }
            let _ = app_handle.emit("usage-update", &usage);
        }
        Err(e) => {
            let _ = app_handle.emit("usage-error", e.to_string());
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(UsageState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![get_usage])
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

            // Spawn background polling loop
            let poll_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Initial fetch immediately
                poll_usage(&poll_handle).await;

                let mut interval = tokio::time::interval(
                    tokio::time::Duration::from_secs(POLL_INTERVAL_SECS),
                );
                // First tick completes immediately, skip it since we already polled
                interval.tick().await;

                loop {
                    interval.tick().await;
                    poll_usage(&poll_handle).await;
                }
            });

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
