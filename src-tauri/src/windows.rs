//! The floating timer window and the settings window.

use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

pub const TIMER_WINDOW: &str = "timer";
pub const SETTINGS_WINDOW: &str = "settings";

pub fn timer_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(TIMER_WINDOW)
}

pub fn is_timer_window_visible(app: &AppHandle) -> bool {
    timer_window(app)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

/// Shows or hides the floating window. When shown, the window is moved to
/// the primary display if its saved position is on a disconnected display.
pub fn set_timer_window_visible(app: &AppHandle, visible: bool) -> tauri::Result<()> {
    let Some(window) = timer_window(app) else {
        return Ok(());
    };
    if visible {
        ensure_on_screen(&window)?;
        window.show()?;
    } else {
        window.hide()?;
    }
    Ok(())
}

/// Opens the settings window, or focuses it if already open.
pub fn open_settings_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW) {
        window.unminimize()?;
        window.show()?;
        return window.set_focus();
    }
    // The Angular app picks the view from the window label.
    let window =
        WebviewWindowBuilder::new(app, SETTINGS_WINDOW, WebviewUrl::App("index.html".into()))
            .title("Työaikalaskuri – Asetukset")
            .inner_size(480.0, 640.0)
            .min_inner_size(400.0, 480.0)
            .center()
            .build()?;
    window.set_focus()
}

/// Centers the window if it does not overlap any connected monitor.
fn ensure_on_screen(window: &WebviewWindow) -> tauri::Result<()> {
    let position = window.outer_position()?;
    let size = window.outer_size()?;
    let visible = window
        .available_monitors()?
        .iter()
        .any(|m| overlaps(position, size, *m.position(), *m.size()));
    if !visible {
        log::info!("timer window is off-screen; moving it to the primary display");
        window.center()?;
    }
    Ok(())
}

fn overlaps(
    a_pos: PhysicalPosition<i32>,
    a_size: PhysicalSize<u32>,
    b_pos: PhysicalPosition<i32>,
    b_size: PhysicalSize<u32>,
) -> bool {
    let right =
        |p: PhysicalPosition<i32>, s: PhysicalSize<u32>| i64::from(p.x) + i64::from(s.width);
    let bottom =
        |p: PhysicalPosition<i32>, s: PhysicalSize<u32>| i64::from(p.y) + i64::from(s.height);
    i64::from(a_pos.x) < right(b_pos, b_size)
        && i64::from(b_pos.x) < right(a_pos, a_size)
        && i64::from(a_pos.y) < bottom(b_pos, b_size)
        && i64::from(b_pos.y) < bottom(a_pos, a_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_overlap_with_monitor() {
        let monitor = (PhysicalPosition::new(0, 0), PhysicalSize::new(1920, 1080));
        let window = PhysicalSize::new(260, 120);
        assert!(overlaps(
            PhysicalPosition::new(100, 100),
            window,
            monitor.0,
            monitor.1
        ));
        assert!(!overlaps(
            PhysicalPosition::new(2500, 100),
            window,
            monitor.0,
            monitor.1
        ));
        assert!(overlaps(
            PhysicalPosition::new(-200, 100),
            window,
            monitor.0,
            monitor.1
        ));
    }
}
