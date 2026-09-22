use std::sync::Arc;
use tauri::{Manager, SystemTray, SystemTrayMenu, CustomMenuItem, SystemTrayEvent};
use crate::audio::RadioPlayer;

/// Shared with `audio::player::RadioPlayer::emit`, which sets this item's
/// title live as playback state changes — starts at "Play" since nothing is
/// playing at fresh launch.
pub const PLAY_PAUSE_ITEM_ID: &str = "play_pause";

pub fn create_system_tray() -> SystemTray {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Show WinRadio"))
        .add_item(CustomMenuItem::new(PLAY_PAUSE_ITEM_ID.to_string(), "Play"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

    SystemTray::new().with_menu(tray_menu)
}

pub fn handle_system_tray_event(app: &tauri::AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            if let Some(window) = app.get_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                PLAY_PAUSE_ITEM_ID => {
                    let player = app.state::<Arc<RadioPlayer>>().inner().clone();
                    player.toggle_play_pause();
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}
