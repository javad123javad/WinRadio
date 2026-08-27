#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod audio;
mod commands;
mod store;
mod tray;
mod timer;

use std::sync::Arc;
use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem, WindowEvent, GlobalShortcutManager};

#[tokio::main]
async fn main() {
  let store = Arc::new(store::Store::new().expect("Failed to initialize store"));
  let audio_player = Arc::new(audio::RadioPlayer::new());
  let sleep_timer = timer::SleepTimer::new(audio_player.clone());

  let store_for_setup = store.clone();
  let audio_player_for_setup = audio_player.clone();

  tauri::Builder::default()
    .setup(move |app| {
      let handle = app.handle();

      // System tray
      let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Show"))
        .add_item(CustomMenuItem::new("play_pause".to_string(), "Play/Pause"))
        .add_item(CustomMenuItem::new("next".to_string(), "Next Station"))
        .add_item(CustomMenuItem::new("prev".to_string(), "Previous Station"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

      let _system_tray = SystemTray::new().with_menu(tray_menu);

      // Global shortcuts
      #[cfg(target_os = "windows")]
      {
        let mut shortcut_manager = app.global_shortcut_manager();
        let player_for_shortcut = audio_player_for_setup.clone();
        let _ = shortcut_manager.register("MediaPlayPause", move || {
          let player = player_for_shortcut.clone();
          tauri::async_runtime::spawn(async move {
            if player.is_playing() {
              let _ = player.stop().await;
            } else if let Some(url) = player.current_url().await {
              let _ = player.play(url).await;
            }
          });
        });
        let _ = shortcut_manager.register("MediaNext", move || {
          // Next station handled via event
        });
        let _ = shortcut_manager.register("MediaPrev", move || {
          // Previous station handled via event
        });
      }

      // Window event handling
      let _app_handle = handle.clone();
      let _minimize_to_tray = store_for_setup.get("minimizeToTray").and_then(|v| v.as_bool()).unwrap_or(false);
      let player_for_sleep = audio_player_for_setup.clone();
      app.listen_global("sleep-timeout", move |_| {
        let player = player_for_sleep.clone();
        tauri::async_runtime::spawn(async move {
          let _ = player.stop().await;
        });
      });

      Ok(())
    })
    .on_system_tray_event(|app, event| {
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
            "play_pause" => {
              // Handled via global shortcut
            }
            "next" => {
              app.emit_all("next-station", ()).ok();
            }
            "prev" => {
              app.emit_all("prev-station", ()).ok();
            }
            "quit" => {
              std::process::exit(0);
            }
            _ => {}
          }
        }
        _ => {}
      }
    })
    .on_window_event(|event| {
      if let WindowEvent::CloseRequested { api, .. } = event.event() {
        let app_handle = event.window().app_handle();
        let store_state = app_handle.state::<Arc<store::Store>>();
        let store = store_state.inner();
        let minimize_to_tray = store.get("minimizeToTray").and_then(|v| v.as_bool()).unwrap_or(false);
        if minimize_to_tray {
          event.window().hide().ok();
          api.prevent_close();
        }
      }
    })
    .manage(store)
    .manage(audio_player)
    .manage(sleep_timer)
    .invoke_handler(tauri::generate_handler![
      commands::play,
      commands::stop,
      commands::set_volume,
      commands::get_volume,
      commands::set_eq_band,
      commands::get_eq,
      commands::reset_eq,
      commands::start_recording,
      commands::stop_recording,
      commands::set_sleep_timer,
      commands::list_stations,
      commands::save_stations,
      commands::list_audio_devices,
      commands::load_settings,
      commands::save_settings,
      commands::export_stations,
      commands::clear_all_data,
      commands::get_metadata,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}