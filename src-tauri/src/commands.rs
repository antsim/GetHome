//! Tauri commands invoked from the Angular UI. Kept thin: the logic lives in
//! [`AppService`].

use tauri::{AppHandle, State};

use crate::engine::{RefreshOutcome, TimerViewModel};
use crate::error::AppError;
use crate::service::{AppService, SettingsView};
use crate::settings::{AccountInfo, Settings};
use crate::windows;

type CommandResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn get_state(service: State<'_, AppService>) -> TimerViewModel {
    service.view_model()
}

#[tauri::command]
pub async fn refresh_now(service: State<'_, AppService>) -> CommandResult<RefreshOutcome> {
    service.refresh().await
}

#[tauri::command]
pub async fn stop_timer(service: State<'_, AppService>) -> CommandResult<()> {
    service.stop_timer().await
}

#[tauri::command]
pub async fn resume_timer(service: State<'_, AppService>) -> CommandResult<()> {
    service.resume_timer().await
}

#[tauri::command]
pub fn get_settings(service: State<'_, AppService>) -> SettingsView {
    service.settings_view()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    service: State<'_, AppService>,
    settings: Settings,
) -> CommandResult<Settings> {
    // Window visibility is controlled by the tray and the window itself; a
    // settings form opened earlier may hold a stale value.
    let settings = Settings {
        timer_window_visible: service.settings().timer_window_visible,
        ..settings
    };
    service.save_settings(&app, settings)
}

#[tauri::command]
pub async fn set_api_token(
    service: State<'_, AppService>,
    token: String,
) -> CommandResult<AccountInfo> {
    service.set_api_token(token).await
}

#[tauri::command]
pub fn clear_api_token(service: State<'_, AppService>) -> CommandResult<()> {
    service.clear_api_token()
}

#[tauri::command]
pub async fn test_connection(service: State<'_, AppService>) -> CommandResult<AccountInfo> {
    service.test_connection().await
}

#[tauri::command]
pub fn toggle_window(app: AppHandle, service: State<'_, AppService>) -> CommandResult<()> {
    service.toggle_timer_window(&app)
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> CommandResult<()> {
    Ok(windows::open_settings_window(&app)?)
}

#[tauri::command]
pub fn list_timezones() -> Vec<&'static str> {
    chrono_tz::TZ_VARIANTS.iter().map(|tz| tz.name()).collect()
}
