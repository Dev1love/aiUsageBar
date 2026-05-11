mod api;
mod db;
mod keychain;
mod settings;
mod swift_bridge;
mod system_monitor;
mod tray_icon;
mod tray_text;

use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::RwLock;

use tauri::{
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

extern "C" fn on_tray_click() {
    if let Some(handle) = APP_HANDLE.get() {
        toggle_popup(handle);
    }
}

use api::{AllUsage, UsageData};
use db::{BatterySnapshot, DailySnapshot, Database, NetworkDaily};
use system_monitor::SystemMetrics;

const TRAY_ID: &str = "main-tray";

const RETINA_ICON_SIZE: u32 = 44;
const POPUP_LABEL: &str = "popup";
const POPUP_WIDTH: f64 = 350.0;
const POPUP_HEIGHT: f64 = 600.0;
const POLL_INTERVAL_SECS: u64 = 300; // 5 minutes

/// Tracks which notification thresholds have fired per reset cycle.
#[derive(Default)]
struct NotificationState {
    /// resets_at value when last notified for five_hour 80%
    five_hour_80_reset: Option<String>,
    /// resets_at value when last notified for five_hour 95%
    five_hour_95_reset: Option<String>,
    /// resets_at value when last notified for seven_day 80%
    seven_day_80_reset: Option<String>,
    /// extra usage 80%
    extra_80_fired: bool,
    /// extra usage 100%
    extra_100_fired: bool,
    /// resets_at value when last notified for seven_day 95%
    seven_day_95_reset: Option<String>,
}

struct NotificationTracker(Mutex<NotificationState>);

/// Send a notification if the threshold is crossed and hasn't been notified for this reset cycle.
fn maybe_notify(
    _app_handle: &tauri::AppHandle,
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
            swift_bridge::notification::show("VibeUsageBar", body);
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

    // Extra usage notifications (no reset cycle — fire once per app session)
    if let Some(util) = usage.extra_usage.utilization {
        if util >= 100.0 && !state.extra_100_fired {
            swift_bridge::notification::show("VibeUsageBar", "Extra usage at 100% — monthly limit reached!");
            state.extra_100_fired = true;
        } else if util >= 80.0 && !state.extra_80_fired {
            swift_bridge::notification::show("VibeUsageBar", "Extra usage at 80% of monthly limit");
            state.extra_80_fired = true;
        }
    }
    let _ = app_handle;
}

/// Shared state holding the latest usage data.
struct UsageState(Mutex<Option<AllUsage>>);

struct SettingsState(RwLock<settings::UserSettings>);

struct SystemState(std::sync::RwLock<SystemMetrics>);

#[tauri::command]
fn get_settings(state: tauri::State<'_, SettingsState>) -> Result<settings::UserSettings, String> {
    let s = state.0.read().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

#[tauri::command]
fn save_settings_cmd(
    new_settings: settings::UserSettings,
    state: tauri::State<'_, SettingsState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    settings::save_settings(&app_data_dir, &new_settings)?;
    let mut s = state.0.write().map_err(|e| e.to_string())?;
    *s = new_settings;
    Ok(())
}

#[tauri::command]
fn get_system_metrics(state: tauri::State<'_, SystemState>) -> Result<SystemMetrics, String> {
    let m = state.0.read().map_err(|e| e.to_string())?;
    Ok(m.clone())
}

#[tauri::command]
fn get_usage(state: tauri::State<'_, UsageState>) -> Result<Option<AllUsage>, String> {
    let data = state.0.lock().map_err(|e| e.to_string())?;
    Ok(data.clone())
}

#[tauri::command]
fn get_history(db: tauri::State<'_, Database>, days: Option<i32>) -> Result<Vec<DailySnapshot>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(db::get_daily_snapshots(&conn, days.unwrap_or(7)))
}

#[tauri::command]
fn get_battery_history(db: tauri::State<'_, Database>, days: Option<i32>) -> Result<Vec<BatterySnapshot>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(db::get_battery_history(&conn, days.unwrap_or(30)))
}

#[tauri::command]
fn get_network_daily(db: tauri::State<'_, Database>, days: Option<i32>) -> Result<Vec<NetworkDaily>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(db::get_network_daily(&conn, days.unwrap_or(30)))
}

/// Update the tray icon to reflect current usage levels.
fn update_tray_icon(_app_handle: &tauri::AppHandle, usage: &UsageData) {
    let session_util = usage.five_hour.utilization / 100.0;
    let weekly_util = usage.seven_day.utilization / 100.0;
    let icon_rgba = tray_icon::render_tray_icon(session_util, weekly_util, None, None);
    swift_bridge::tray::set_icon_rgba(&icon_rgba, RETINA_ICON_SIZE, RETINA_ICON_SIZE);
}

/// Set the tray icon to the dimmed/gray error state.
fn set_tray_error_icon(_app_handle: &tauri::AppHandle) {
    let icon_rgba = tray_icon::render_error_icon();
    swift_bridge::tray::set_icon_rgba(&icon_rgba, RETINA_ICON_SIZE, RETINA_ICON_SIZE);
}

/// Handle successful usage fetch: update state, store snapshot, update tray, notify, emit event.
fn handle_all_usage(app_handle: &tauri::AppHandle, all: AllUsage) {
    // Update tray icon based on Claude data (primary) or Codex
    if let Some(ref claude) = all.claude {
        update_tray_icon(app_handle, claude);
        check_and_notify(app_handle, claude);
        if let Some(db) = app_handle.try_state::<Database>() {
            if let Ok(conn) = db.0.lock() {
                db::insert_snapshot(&conn, claude);
            }
        }
    }

    if let Some(state) = app_handle.try_state::<UsageState>() {
        if let Ok(mut data) = state.0.lock() {
            *data = Some(all.clone());
        }
    }
    let _ = app_handle.emit("usage-update", &all);
}

/// Perform a single poll for all providers.
async fn poll_usage(app_handle: &tauri::AppHandle) {
    // eprintln!("[aiUsageBar] Polling usage...");

    let mut all = AllUsage {
        claude: None,
        codex: None,
    };
    let mut any_success = false;

    // Poll Claude
    match keychain::read_credentials() {
        Ok(c) => {
            match api::fetch_usage(&c.access_token).await {
                Ok(usage) => {
                    // eprintln!("[aiUsageBar] Claude: 5h={:.1}%, 7d={:.1}%", usage.five_hour.utilization, usage.seven_day.utilization);
                    all.claude = Some(usage);
                    any_success = true;
                }
                Err(api::ApiError::TokenExpired) => {
                    if let Ok(new_token) = api::refresh_access_token(&c.refresh_token).await {
                        if let Ok(usage) = api::fetch_usage(&new_token).await {
                            all.claude = Some(usage);
                            any_success = true;
                        }
                    }
                }
                Err(_e) => {} // eprintln!("[aiUsageBar] Claude error: {e}"),
            }
        }
        Err(_e) => {} // eprintln!("[aiUsageBar] Claude keychain: {e}"),
    }

    // Poll Codex
    match api::fetch_codex_usage().await {
        Ok(codex) => {
            // eprintln!("[aiUsageBar] Codex: primary={:.1}%", codex.primary.utilization);
            all.codex = Some(codex);
            any_success = true;
        }
        Err(_e) => {} // eprintln!("[aiUsageBar] Codex error: {e}"),
    }

    if any_success {
        handle_all_usage(app_handle, all);
    } else {
        set_tray_error_icon(app_handle);
        let _ = app_handle.emit("usage-error", "No providers available".to_string());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(UsageState(Mutex::new(None)))
        .manage(NotificationTracker(Mutex::new(NotificationState::default())))
        .invoke_handler(tauri::generate_handler![get_usage, get_history, get_settings, save_settings_cmd, get_system_metrics, get_battery_history, get_network_daily])
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

            // Migrate data from old aiUsageBar app if exists
            let old_app_dir = app_data_dir
                .parent()
                .map(|p| p.join("com.aiusagebar.app"));
            if let Some(ref old_dir) = old_app_dir {
                let old_db = old_dir.join("claudebar.db");
                let new_db = app_data_dir.join("claudebar.db");
                if old_db.exists() && !new_db.exists() {
                    let _ = std::fs::create_dir_all(&app_data_dir);
                    if let Err(e) = std::fs::copy(&old_db, &new_db) {
                        eprintln!("Failed to migrate old database: {e}");
                    }
                }
            }

            let user_settings = settings::load_settings(&app_data_dir);
            app.manage(SettingsState(RwLock::new(user_settings)));

            let conn = db::open_database(app_data_dir)
                .map_err(|e| format!("Database init failed: {e}"))?;
            app.manage(Database(Mutex::new(conn)));

            // Native Swift NSStatusItem (macOS 26 doesn't render Tauri tray icons).
            let _ = APP_HANDLE.set(app.handle().clone());
            let tray_ok = swift_bridge::tray::init(on_tray_click);
            if tray_ok {
                let icon_rgba = tray_icon::render_default_icon();
                swift_bridge::tray::set_icon_rgba(&icon_rgba, RETINA_ICON_SIZE, RETINA_ICON_SIZE);
            } else {
                eprintln!("[VibeUsageBar] Native tray init failed");
            }

            // Register global shortcut Shift+Option+V
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                let shortcut_handle = app.handle().clone();
                app.global_shortcut().on_shortcut("shift+alt+d", move |_app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        toggle_popup(&shortcut_handle);
                    }
                })?;
            }

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

            app.manage(SystemState(std::sync::RwLock::new(SystemMetrics::default())));

            // Spawn system metrics polling loop
            let sys_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut sys = sysinfo::System::new();
                let mut networks = sysinfo::Networks::new_with_refreshed_list();
                let mut net_tracker = system_monitor::NetworkTracker::new();
                // Initial CPU refresh needs two calls (first returns 0)
                sys.refresh_cpu_usage();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let mut tick_count: u64 = 0;
                // Seconds elapsed since last battery DB snapshot (target: 1800s = 30min)
                let mut secs_since_battery_snapshot: u64 = u64::MAX; // force first snapshot

                loop {
                    // Read interval dynamically from settings each iteration
                    let sys_interval = sys_handle
                        .try_state::<SettingsState>()
                        .and_then(|s| s.0.read().ok().map(|st| st.polling.system_interval_sec))
                        .unwrap_or(3);

                    tokio::time::sleep(tokio::time::Duration::from_secs(sys_interval)).await;
                    tick_count += 1;

                    let (cpu, ram, disk, net) = system_monitor::refresh_sysinfo_metrics(
                        &mut sys, &mut networks, &mut net_tracker, sys_interval,
                    );

                    // Battery: refresh every 30 seconds (slow-changing)
                    let battery = if tick_count % (30 / sys_interval).max(1) == 0 || tick_count == 1 {
                        system_monitor::read_battery()
                    } else {
                        if let Some(state) = sys_handle.try_state::<SystemState>() {
                            state.0.read().ok().and_then(|m| m.battery.clone())
                        } else {
                            None
                        }
                    };

                    // Battery low notifications
                    if let Some(ref batt) = battery {
                        if !batt.charging {
                            let pct = batt.percent as u32;
                            if pct <= 10 && pct > 0 {
                                // Notify every ~5 minutes at critical level
                                if tick_count % (300 / sys_interval).max(1) == 0 {
                                    swift_bridge::notification::show(
                                        "🪫 Battery Critical",
                                        &format!("{}% — plug in now!", pct),
                                    );
                                }
                            } else if pct <= 20 {
                                // Notify once when crossing 20%
                                if tick_count % (600 / sys_interval).max(1) == 0 {
                                    swift_bridge::notification::show(
                                        "🔋 Battery Low",
                                        &format!("{}% remaining", pct),
                                    );
                                }
                            }
                        }
                    }

                    // Insert battery snapshot every ~30 minutes
                    secs_since_battery_snapshot += sys_interval;
                    if secs_since_battery_snapshot >= 1800 {
                        if let Some(ref batt) = battery {
                            if let Some(db_state) = sys_handle.try_state::<Database>() {
                                if let Ok(conn) = db_state.0.lock() {
                                    db::insert_battery_snapshot(
                                        &conn,
                                        batt.percent,
                                        batt.health_percent,
                                        batt.cycle_count,
                                        batt.charging,
                                    );
                                }
                            }
                        }
                        secs_since_battery_snapshot = 0;
                    }

                    // Accumulate network bytes daily
                    let download_bytes = net.download_speed * sys_interval;
                    let upload_bytes = net.upload_speed * sys_interval;
                    if download_bytes > 0 || upload_bytes > 0 {
                        if let Some(db_state) = sys_handle.try_state::<Database>() {
                            if let Ok(conn) = db_state.0.lock() {
                                db::insert_network_daily(&conn, download_bytes, upload_bytes);
                            }
                        }
                    }

                    if tick_count == 1 {
                        #[cfg(has_swift_dylib)]
                        eprintln!("[VibeUsageBar] Swift dylib available");
                        #[cfg(not(has_swift_dylib))]
                        eprintln!("[VibeUsageBar] Swift dylib NOT available — temps/fans/bluetooth will be empty");
                    }

                    let (temps, fans) = swift_bridge::get_smc_data();
                    if tick_count == 1 && temps.is_empty() && fans.is_empty() {
                        eprintln!("[VibeUsageBar] SMC returned no data - dylib may not be loaded or SMC access denied");
                    }

                    let bluetooth = if tick_count % (10 / sys_interval).max(1) == 0 || tick_count == 1 {
                        swift_bridge::get_bluetooth_devices()
                    } else {
                        if let Some(state) = sys_handle.try_state::<SystemState>() {
                            state.0.read().ok().map(|m| m.bluetooth.clone()).unwrap_or_default()
                        } else {
                            vec![]
                        }
                    };

                    if tick_count == 1 {
                        eprintln!("[VibeUsageBar] System metrics tick 1: {} temps, {} fans, {} bt devices",
                            temps.len(), fans.len(), bluetooth.len());
                    }

                    let metrics = SystemMetrics {
                        cpu,
                        ram,
                        disk,
                        network: net,
                        battery,
                        temps,
                        fans,
                        bluetooth,
                    };

                    if let Some(state) = sys_handle.try_state::<SystemState>() {
                        if let Ok(mut m) = state.0.write() {
                            *m = metrics.clone();
                        }
                    }
                    let _ = sys_handle.emit("system-update", &metrics);

                    // Update tray title
                    if let Some(settings_state) = sys_handle.try_state::<SettingsState>() {
                        if let Ok(settings) = settings_state.0.read() {
                            let ai = sys_handle.try_state::<UsageState>()
                                .and_then(|s| s.0.lock().ok().map(|d| d.clone()))
                                .flatten();
                            let title = tray_text::format_tray_title(&settings.tray, &metrics, &ai);
                            swift_bridge::tray::set_title(&title);
                        }
                    }
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
            .title("VibeUsageBar")
            .inner_size(POPUP_WIDTH, POPUP_HEIGHT)
            .decorations(false)
            .resizable(false)
            .always_on_top(true)
            .visible(true)
            .skip_taskbar(true)
            .transparent(true);

        match builder.build() {
            Ok(window) => {
                // Hide popup when it loses focus (standard menubar app behavior)
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = w.hide();
                    }
                });
            }
            Err(e) => eprintln!("Failed to create popup window: {e}"),
        }
    }
}
