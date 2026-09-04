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
    #[serde(default)]
    pub favorite_order: i64,
    // spec-2-2: Location Tile reads these off the already-cached `Station`
    // (AD-5 "no redundant fetching") — no network round-trip for data
    // already in hand. `#[serde(default)]` so a `store.json` written before
    // this story still loads (back-compat, same shape as `favorite_order`
    // above).
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub geo_lat: Option<f64>,
    #[serde(default)]
    pub geo_long: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub minimize_to_tray: bool,
    pub sleep_timer_default_minutes: u32,
    pub theme: String,
    pub volume: f32,
    // Full station snapshot, not just an id: a last-played station that was
    // never favorited has no other persisted record once the app restarts
    // (search results are never written to store.json), so an id-only
    // reference would be unresolvable for that case (spec-1-5).
    #[serde(default)]
    pub last_station: Option<Station>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // On by default: the app's whole premise is staying tray-resident
            // while playing (FR-10, Story 1.6's "instead of quitting" framing;
            // matches the Settings modal mockup's canonical depicted state,
            // settings-modal.html:49, which shows this toggle already "on").
            minimize_to_tray: true,
            sleep_timer_default_minutes: 30,
            theme: "system".to_string(),
            volume: 0.7,
            last_station: None,
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
