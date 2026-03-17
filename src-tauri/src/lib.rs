mod api;
mod db;
mod keychain;
mod tray_icon;

use std::sync::Mutex;

use tauri::{
    image::Image,
    tray::{TrayIconBuilder, TrayIconId},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};

use api::UsageData;
use db::{DailySnapshot, Database};

const TRAY_ID: &str = "main-tray";

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

#[tauri::command]
fn get_history(db: tauri::State<'_, Database>, days: Option<i32>) -> Result<Vec<DailySnapshot>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(db::get_daily_snapshots(&conn, days.unwrap_or(7)))
}

/// Update the tray icon to reflect current usage levels.
fn update_tray_icon(app_handle: &tauri::AppHandle, usage: &UsageData) {
    let icon_rgba = tray_icon::render_tray_icon(
        usage.five_hour.utilization,
        usage.seven_day.utilization,
        None,
        None,
    );
    let icon = Image::new_owned(icon_rgba, RETINA_ICON_SIZE, RETINA_ICON_SIZE);
    if let Some(tray) = app_handle.tray_by_id(&TrayIconId::new(TRAY_ID)) {
        let _ = tray.set_icon(Some(icon));
    }
}

/// Set the tray icon to the dimmed/gray error state.
fn set_tray_error_icon(app_handle: &tauri::AppHandle) {
    let icon_rgba = tray_icon::render_error_icon();
    let icon = Image::new_owned(icon_rgba, RETINA_ICON_SIZE, RETINA_ICON_SIZE);
    if let Some(tray) = app_handle.tray_by_id(&TrayIconId::new(TRAY_ID)) {
        let _ = tray.set_icon(Some(icon));
    }
}

/// Perform a single poll: read keychain, fetch usage, emit events, update state & tray icon.
async fn poll_usage(app_handle: &tauri::AppHandle) {
    let credentials = match keychain::read_credentials() {
        Ok(c) => c,
        Err(e) => {
            set_tray_error_icon(app_handle);
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
            // Store snapshot in SQLite
            if let Some(db) = app_handle.try_state::<Database>() {
                if let Ok(conn) = db.0.lock() {
                    db::insert_snapshot(&conn, &usage);
                }
            }
            update_tray_icon(app_handle, &usage);
            let _ = app_handle.emit("usage-update", &usage);
        }
        Err(e) => {
            set_tray_error_icon(app_handle);
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
        .invoke_handler(tauri::generate_handler![get_usage, get_history])
        .setup(|app| {
            // Initialize SQLite database
            let app_data_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
            let conn = db::open_database(app_data_dir)
                .map_err(|e| format!("Database init failed: {e}"))?;
            app.manage(Database(Mutex::new(conn)));

            // Render default tray icon (green bars at 0%)
            let icon_rgba = tray_icon::render_default_icon();
            let icon = Image::new_owned(icon_rgba, RETINA_ICON_SIZE, RETINA_ICON_SIZE);

            let app_handle = app.handle().clone();

            TrayIconBuilder::with_id(TRAY_ID)
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
