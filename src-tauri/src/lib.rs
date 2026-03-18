mod api;
mod db;
mod keychain;
mod tray_icon;

use std::sync::Mutex;

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconId},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_notification::NotificationExt;

use api::UsageData;
use db::{DailySnapshot, Database};

const TRAY_ID: &str = "main-tray";

const RETINA_ICON_SIZE: u32 = 44;
const POPUP_LABEL: &str = "popup";
const POPUP_WIDTH: f64 = 300.0;
const POPUP_HEIGHT: f64 = 380.0;
const POLL_INTERVAL_SECS: u64 = 60;

/// Tracks which notification thresholds have fired per reset cycle.
#[derive(Default)]
struct NotificationState {
    /// resets_at value when last notified for five_hour 80%
    five_hour_80_reset: Option<String>,
    /// resets_at value when last notified for five_hour 95%
    five_hour_95_reset: Option<String>,
    /// resets_at value when last notified for seven_day 80%
    seven_day_80_reset: Option<String>,
    /// resets_at value when last notified for seven_day 95%
    seven_day_95_reset: Option<String>,
}

struct NotificationTracker(Mutex<NotificationState>);

/// Send a notification if the threshold is crossed and hasn't been notified for this reset cycle.
fn maybe_notify(
    app_handle: &tauri::AppHandle,
    utilization: f64,
    resets_at: &str,
    threshold: u8,
    body: &str,
    last_reset: &mut Option<String>,
) {
    // utilization comes from API as 0-100 percentage
    let pct = utilization as u8;
    if pct >= threshold {
        if last_reset.as_deref() != Some(resets_at) {
            let _ = app_handle
                .notification()
                .builder()
                .title("aiUsageBar")
                .body(body)
                .show();
            *last_reset = Some(resets_at.to_string());
        }
    }
}

/// Check usage thresholds and send notifications (only once per reset cycle).
fn check_and_notify(app_handle: &tauri::AppHandle, usage: &UsageData) {
    let Some(tracker) = app_handle.try_state::<NotificationTracker>() else {
        return;
    };
    let Ok(mut state) = tracker.0.lock() else {
        return;
    };

    // Check higher threshold first so both 80% and 95% can fire independently
    maybe_notify(app_handle, usage.five_hour.utilization, &usage.five_hour.resets_at, 95,
        "Session usage at 95% — limit approaching!", &mut state.five_hour_95_reset);
    maybe_notify(app_handle, usage.five_hour.utilization, &usage.five_hour.resets_at, 80,
        "Session usage at 80%", &mut state.five_hour_80_reset);
    maybe_notify(app_handle, usage.seven_day.utilization, &usage.seven_day.resets_at, 95,
        "Weekly usage at 95% — limit approaching!", &mut state.seven_day_95_reset);
    maybe_notify(app_handle, usage.seven_day.utilization, &usage.seven_day.resets_at, 80,
        "Weekly usage at 80%", &mut state.seven_day_80_reset);
}

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
    // API returns utilization as 0-100 percentage, normalize to 0-1
    let session_util = usage.five_hour.utilization / 100.0;
    let weekly_util = usage.seven_day.utilization / 100.0;
    let icon_rgba = tray_icon::render_tray_icon(
        session_util,
        weekly_util,
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

/// Handle a successful usage fetch: update state, store snapshot, update tray, notify, emit event.
fn handle_usage_success(app_handle: &tauri::AppHandle, usage: UsageData) {
    if let Some(state) = app_handle.try_state::<UsageState>() {
        if let Ok(mut data) = state.0.lock() {
            *data = Some(usage.clone());
        }
    }
    if let Some(db) = app_handle.try_state::<Database>() {
        if let Ok(conn) = db.0.lock() {
            db::insert_snapshot(&conn, &usage);
        }
    }
    update_tray_icon(app_handle, &usage);
    check_and_notify(app_handle, &usage);
    let _ = app_handle.emit("usage-update", &usage);
}

/// Perform a single poll: read keychain, fetch usage, emit events, update state & tray icon.
/// On 401, attempts token refresh once before giving up.
async fn poll_usage(app_handle: &tauri::AppHandle) {
    eprintln!("[aiUsageBar] Polling usage...");
    let credentials = match keychain::read_credentials() {
        Ok(c) => {
            eprintln!("[aiUsageBar] Keychain OK, token starts with: {}...", &c.access_token[..20]);
            c
        }
        Err(e) => {
            eprintln!("[aiUsageBar] Keychain error: {e}");
            set_tray_error_icon(app_handle);
            let _ = app_handle.emit("usage-error", e);
            return;
        }
    };

    match api::fetch_usage(&credentials.access_token).await {
        Ok(usage) => {
            eprintln!("[aiUsageBar] Usage fetched: 5h={:.1}%, 7d={:.1}%", usage.five_hour.utilization, usage.seven_day.utilization);
            handle_usage_success(app_handle, usage);
        }
        Err(api::ApiError::TokenExpired) => {
            // Attempt token refresh once
            match api::refresh_access_token(&credentials.refresh_token).await {
                Ok(new_token) => {
                    // Retry with refreshed token
                    match api::fetch_usage(&new_token).await {
                        Ok(usage) => {
                            handle_usage_success(app_handle, usage);
                        }
                        Err(e) => {
                            set_tray_error_icon(app_handle);
                            let _ = app_handle.emit("usage-error", e.to_string());
                        }
                    }
                }
                Err(_) => {
                    set_tray_error_icon(app_handle);
                    let _ = app_handle.emit(
                        "usage-error",
                        "Token expired, run 'claude login' to re-authenticate.".to_string(),
                    );
                }
            }
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
        .plugin(tauri_plugin_notification::init())
        .manage(UsageState(Mutex::new(None)))
        .manage(NotificationTracker(Mutex::new(NotificationState::default())))
        .invoke_handler(tauri::generate_handler![get_usage, get_history])
        .setup(|app| {
            // Hide dock icon — menubar-only app
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                app.set_activation_policy(ActivationPolicy::Accessory);
            }
            // Initialize SQLite database
            let app_data_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
            let conn = db::open_database(app_data_dir)
                .map_err(|e| format!("Database init failed: {e}"))?;
            app.manage(Database(Mutex::new(conn)));

            // Render default tray icon (green bars at 0%)
            let icon_rgba = tray_icon::render_default_icon();
            eprintln!("[aiUsageBar] Icon RGBA data length: {} (expected {})", icon_rgba.len(), RETINA_ICON_SIZE * RETINA_ICON_SIZE * 4);
            let icon = Image::new_owned(icon_rgba, RETINA_ICON_SIZE, RETINA_ICON_SIZE);

            let app_handle = app.handle().clone();

            // Build a simple context menu for the tray
            let quit = MenuItemBuilder::with_id("quit", "Quit aiUsageBar").build(app)?;
            let menu = MenuBuilder::new(app).item(&quit).build()?;

            eprintln!("[aiUsageBar] Building tray icon...");
            TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .icon_as_template(false)
                .tooltip("aiUsageBar")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        toggle_popup(&app_handle);
                    }
                })
                .build(app)?;

            eprintln!("[aiUsageBar] Tray icon created successfully!");

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
            .title("aiUsageBar")
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
