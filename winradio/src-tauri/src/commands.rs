use std::sync::Arc;
use tauri::{State, Window};
use crate::audio::RadioPlayer;
use crate::store::Store;
use crate::timer::SleepTimer;
use serde::{Deserialize, Serialize};
use crate::audio::Metadata;
use chrono;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub url: String,
    pub favicon_url: Option<String>,
    pub homepage: Option<String>,
    pub category: Option<String>,
    pub is_favorite: bool,
    pub added_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Settings {
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub show_notifications: bool,
    pub output_device: String,
    pub buffer_size: u32,
    pub eq_enabled: bool,
    pub theme: String,
    pub compact_player: bool,
    pub recording_format: String,
    pub recording_bitrate: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub async fn play(url: String, player: State<'_, Arc<RadioPlayer>>) -> Result<(), String> {
    Arc::clone(&player).play(url).await
}

#[tauri::command]
pub async fn stop(player: State<'_, Arc<RadioPlayer>>) -> Result<(), String> {
    player.stop().await
}

#[tauri::command]
pub async fn set_volume(volume: f32, player: State<'_, Arc<RadioPlayer>>) -> Result<(), String> {
    player.set_volume(volume).await
}

#[tauri::command]
pub async fn get_volume(player: State<'_, Arc<RadioPlayer>>) -> Result<f32, String> {
    Ok(player.get_volume())
}

#[tauri::command]
pub async fn set_eq_band(band: usize, gain_db: f32, player: State<'_, Arc<RadioPlayer>>) -> Result<(), String> {
    player.set_eq_band(band, gain_db).await
}

#[tauri::command]
pub async fn get_eq(player: State<'_, Arc<RadioPlayer>>) -> Result<Vec<f32>, String> {
    Ok(player.get_eq().to_vec())
}

#[tauri::command]
pub async fn reset_eq(player: State<'_, Arc<RadioPlayer>>) -> Result<(), String> {
    player.reset_eq().await
}

#[tauri::command]
pub async fn start_recording(filename: String, player: State<'_, Arc<RadioPlayer>>) -> Result<String, String> {
    player.start_recording(filename).await
}

#[tauri::command]
pub async fn stop_recording(player: State<'_, Arc<RadioPlayer>>, _window: Window) -> Result<(), String> {
    let path = player.stop_recording().await?;

    use tauri::api::dialog::FileDialogBuilder;
    FileDialogBuilder::new()
        .set_title("Save Recording")
        .add_filter("WAV Audio", &["wav"])
        .set_file_name(&format!("recording-{}.wav", chrono::Local::now().format("%Y%m%d-%H%M%S")))
        .save_file(move |save_path| {
            if let Some(save_path) = save_path {
                std::fs::copy(&path, save_path).ok();
            }
        });

    Ok(())
}

#[tauri::command]
pub async fn set_sleep_timer(minutes: u32, timer: State<'_, SleepTimer>) -> Result<(), String> {
    timer.set(minutes).await;
    Ok(())
}

#[tauri::command]
pub async fn list_stations(store: State<'_, Arc<Store>>) -> Result<Vec<Station>, String> {
    store.get_stations()
}

#[tauri::command]
pub async fn save_stations(stations: Vec<Station>, store: State<'_, Arc<Store>>) -> Result<(), String> {
    store.save_stations(&stations)
}

#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    use cpal::traits::{HostTrait, DeviceTrait};
    let host = cpal::default_host();
    let mut devices = Vec::new();

    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(name) = device.name() {
                devices.push(AudioDevice {
                    id: name.clone(),
                    name,
                });
            }
        }
    }

    Ok(devices)
}

#[tauri::command]
pub async fn load_settings(store: State<'_, Arc<Store>>) -> Result<Settings, String> {
    store.get_settings()
}

#[tauri::command]
pub async fn save_settings(settings: Settings, store: State<'_, Arc<Store>>) -> Result<(), String> {
    store.save_settings(&settings)
}

#[tauri::command]
pub async fn export_stations(store: State<'_, Arc<Store>>) -> Result<String, String> {
    let stations = store.get_stations()?;
    serde_json::to_string_pretty(&stations).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_all_data(store: State<'_, Arc<Store>>) -> Result<(), String> {
    store.clear_all()
}

#[tauri::command]
pub fn get_metadata(player: State<'_, Arc<RadioPlayer>>) -> Result<Option<Metadata>, String> {
    Ok(player.get_metadata())
}