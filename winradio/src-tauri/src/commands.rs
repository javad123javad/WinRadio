use std::sync::Arc;
use tauri::State;
use crate::audio::RadioPlayer;
use crate::store::Store;
use crate::timer::SleepTimer;
use serde::{Deserialize, Serialize};
use crate::audio::Metadata;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub minimize_to_tray: bool,
    pub sleep_timer_default_minutes: u32,
    pub theme: String,
    pub volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            minimize_to_tray: false,
            sleep_timer_default_minutes: 30,
            theme: "system".to_string(),
            volume: 0.7,
        }
    }
}

#[tauri::command]
pub async fn play(station: Station, player: State<'_, Arc<RadioPlayer>>) -> Result<(), String> {
    Arc::clone(&player).play(station).await
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
