use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use crate::commands::{Station, Settings};

#[derive(Serialize, Deserialize, Clone, Default)]
struct StoreData {
    stations: Vec<Station>,
    settings: Settings,
}

pub struct Store {
    path: std::path::PathBuf,
    data: Arc<Mutex<StoreData>>,
}

impl Store {
    pub fn new() -> Result<Self, String> {
        let app_dir = dirs::data_dir()
            .ok_or("Could not find data directory")?
            .join("WinRadio");

        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

        let path = app_dir.join("store.json");
        let data = if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            Self::parse_store_data(&content)
        } else {
            StoreData::default()
        };

        Ok(Self {
            path,
            data: Arc::new(Mutex::new(data)),
        })
    }

    /// Decodes `stations` and `settings` independently rather than as one
    /// `StoreData` unit: previously, any single field failing to parse (a
    /// future schema change, hand-edited JSON, etc.) fell back to
    /// `unwrap_or_default()` for the *entire* struct, silently wiping the
    /// user's saved stations along with it. Each half now falls back to its
    /// own default on its own, so a malformed `settings` (or `stations`)
    /// shape can never take the other down with it.
    fn parse_store_data(content: &str) -> StoreData {
        let raw: serde_json::Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(_) => return StoreData::default(),
        };

        let stations = raw
            .get("stations")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let settings = raw
            .get("settings")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        StoreData { stations, settings }
    }

    fn save(&self) -> Result<(), String> {
        let data = self.data.lock();
        let content = serde_json::to_string_pretty(&*data).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, content).map_err(|e| e.to_string())
    }

    pub fn get_stations(&self) -> Result<Vec<Station>, String> {
        Ok(self.data.lock().stations.clone())
    }

    pub fn save_stations(&self, stations: &[Station]) -> Result<(), String> {
        self.data.lock().stations = stations.to_vec();
        self.save()
    }

    pub fn get_settings(&self) -> Result<Settings, String> {
        Ok(self.data.lock().settings.clone())
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        self.data.lock().settings = settings.clone();
        self.save()
    }

    pub fn clear_all(&self) -> Result<(), String> {
        *self.data.lock() = StoreData::default();
        self.save()
    }

    pub fn minimize_to_tray(&self) -> bool {
        self.data.lock().settings.minimize_to_tray
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_settings_does_not_drop_stations() {
        // `settings` shape is from a hypothetical future/incompatible
        // version (wrong types throughout); `stations` is well-formed.
        let content = r#"{
            "stations": [
                {"id": "a", "name": "Station A", "url": "https://a.example/stream",
                 "faviconUrl": null, "homepage": null, "category": null,
                 "isFavorite": true, "addedAt": 123}
            ],
            "settings": {"minimizeToTray": "not-a-bool", "theme": 42}
        }"#;

        let data = Store::parse_store_data(content);

        assert_eq!(data.stations.len(), 1);
        assert_eq!(data.stations[0].id, "a");
        // Falls back to Settings::default() rather than failing entirely.
        assert_eq!(data.settings.theme, "system");
    }

    #[test]
    fn missing_new_settings_fields_fall_back_instead_of_failing_the_whole_struct() {
        // Simulates a store.json written before `sleepTimerDefaultMinutes`/
        // `volume` existed.
        let content = r#"{
            "stations": [],
            "settings": {"minimizeToTray": true, "theme": "dark"}
        }"#;

        let data = Store::parse_store_data(content);

        assert!(data.settings.minimize_to_tray);
        assert_eq!(data.settings.theme, "dark");
        assert_eq!(data.settings.sleep_timer_default_minutes, 30);
        assert_eq!(data.settings.volume, 0.7);
    }

    #[test]
    fn malformed_stations_does_not_drop_settings() {
        let content = r#"{
            "stations": "not-an-array",
            "settings": {"minimizeToTray": true, "theme": "dark",
                         "sleepTimerDefaultMinutes": 60, "volume": 0.5}
        }"#;

        let data = Store::parse_store_data(content);

        assert!(data.stations.is_empty());
        assert!(data.settings.minimize_to_tray);
        assert_eq!(data.settings.theme, "dark");
    }

    #[test]
    fn totally_invalid_json_falls_back_to_full_default() {
        let data = Store::parse_store_data("not json at all");

        assert!(data.stations.is_empty());
        assert_eq!(data.settings.theme, "system");
    }

    #[test]
    fn missing_favorite_order_defaults_instead_of_failing_the_whole_station() {
        // Simulates a store.json written before `favoriteOrder` existed
        // (spec-1-4) — `#[serde(default)]` must let it parse instead of
        // dropping the station (or the whole collection) entirely.
        let content = r#"{
            "stations": [
                {"id": "a", "name": "Station A", "url": "https://a.example/stream",
                 "faviconUrl": null, "homepage": null, "category": null,
                 "isFavorite": true, "addedAt": 123}
            ],
            "settings": {}
        }"#;

        let data = Store::parse_store_data(content);

        assert_eq!(data.stations.len(), 1);
        assert_eq!(data.stations[0].favorite_order, 0);
    }
}
