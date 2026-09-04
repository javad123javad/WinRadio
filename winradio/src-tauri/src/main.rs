#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod audio;
mod commands;
mod directory;
mod store;
mod tray;
mod timer;

use std::sync::Arc;
use tauri::{Manager, WindowEvent, GlobalShortcutManager};

#[tokio::main]
async fn main() {
  let store = Arc::new(store::Store::new().expect("Failed to initialize store"));
  let initial_volume = store.get_settings().map(|s| s.volume).unwrap_or(0.7);
  let audio_player = Arc::new(audio::RadioPlayer::new(initial_volume));
  let sleep_timer = timer::SleepTimer::new(audio_player.clone());

  let audio_player_for_setup = audio_player.clone();
  let sleep_timer_for_setup = sleep_timer.clone();

  tauri::Builder::default()
    .setup(move |app| {
      let handle = app.handle();
      audio_player_for_setup.set_app_handle(handle.clone());
      sleep_timer_for_setup.set_app_handle(handle.clone());

      // Global media-key shortcuts
      #[cfg(target_os = "windows")]
      {
        let mut shortcut_manager = app.global_shortcut_manager();
        let player_for_shortcut = audio_player_for_setup.clone();
        let _ = shortcut_manager.register("MediaPlayPause", move || {
          player_for_shortcut.toggle_play_pause();
        });
      }

      Ok(())
    })
    .system_tray(tray::create_system_tray())
    .on_system_tray_event(tray::handle_system_tray_event)
    .on_window_event(|event| {
      if let WindowEvent::CloseRequested { api, .. } = event.event() {
        let app_handle = event.window().app_handle();
        let store_state = app_handle.state::<Arc<store::Store>>();
        if store_state.inner().minimize_to_tray() {
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
      commands::set_sleep_timer,
      commands::list_stations,
      commands::save_stations,
      commands::load_settings,
      commands::save_settings,
      commands::export_stations,
      commands::clear_all_data,
      commands::get_metadata,
      directory::search_stations,
      directory::get_filter_options,
      directory::get_location_tile,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
