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

        let settings_value = raw.get("settings").cloned().unwrap_or(serde_json::Value::Null);

        // `lastStation` is decoded independently from the rest of
        // `Settings` — same fix shape as the stations/settings split above.
        // Without this, a malformed nested `lastStation` (a future/
        // incompatible shape, hand-edited JSON) would fail the single
        // `Settings` deserialize as a whole, falling back to
        // `Settings::default()` and silently resetting theme/volume/
        // minimizeToTray/sleepTimerDefaultMinutes too — not just dropping
        // the one bad field.
        let last_station: Option<Station> = settings_value
            .get("lastStation")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let mut settings_without_last_station = settings_value;
        if let serde_json::Value::Object(ref mut map) = settings_without_last_station {
            map.remove("lastStation");
        }

        let mut settings: Settings =
            serde_json::from_value(settings_without_last_station).unwrap_or_default();
        settings.last_station = last_station;

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

    /// Test-only constructor that loads from (and later saves to) an
    /// arbitrary path instead of the real OS data directory, so tests can
    /// exercise a genuine save-then-reload round trip through the
    /// filesystem without touching the user's actual `store.json`.
    #[cfg(test)]
    fn new_at(path: std::path::PathBuf) -> Self {
        let data = if path.exists() {
            let content = std::fs::read_to_string(&path).expect("read test store file");
            Self::parse_store_data(&content)
        } else {
            StoreData::default()
        };

        Self {
            path,
            data: Arc::new(Mutex::new(data)),
        }
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
    fn default_settings_minimize_to_tray_is_on() {
        // Story 1.6: a fresh install must default to tray-resident behavior
        // ("instead of quitting"), not opt-in — locks in the deliberate
        // default flip so it can't silently regress back to `false`.
        assert!(Settings::default().minimize_to_tray);
    }

    #[test]
    fn missing_last_station_defaults_to_none_instead_of_failing_the_whole_struct() {
        // Simulates a store.json written before `lastStation` existed
        // (spec-1-5) — must parse instead of dropping the settings record.
        let content = r#"{
            "stations": [],
            "settings": {"minimizeToTray": true, "theme": "dark",
                         "sleepTimerDefaultMinutes": 60, "volume": 0.5}
        }"#;

        let data = Store::parse_store_data(content);

        assert!(data.settings.last_station.is_none());
        assert_eq!(data.settings.volume, 0.5);
    }

    #[test]
    fn last_station_deserializes_as_a_full_station_snapshot() {
        // spec-1-5: a never-favorited last-played station must persist as a
        // full snapshot (not just an id) since it has no other persisted
        // record once the app restarts. (Deserialize-only check — see
        // `last_station_survives_a_genuine_save_then_reload_round_trip`
        // below for an actual save-then-reload test through the
        // filesystem.)
        let content = r#"{
            "stations": [],
            "settings": {"minimizeToTray": false, "theme": "system",
                         "sleepTimerDefaultMinutes": 30, "volume": 0.7,
                         "lastStation": {"id": "x", "name": "Station X",
                             "url": "https://x.example/stream", "faviconUrl": null,
                             "homepage": null, "category": "Jazz",
                             "isFavorite": false, "addedAt": 0}}
        }"#;

        let data = Store::parse_store_data(content);

        let last = data.settings.last_station.expect("last_station should be present");
        assert_eq!(last.id, "x");
        assert_eq!(last.name, "Station X");
        assert!(!last.is_favorite);
    }

    #[test]
    fn last_station_survives_a_genuine_save_then_reload_round_trip() {
        // Actually exercises `Store::save_settings` writing to disk and a
        // fresh `Store` loading it back — not just a hand-written JSON
        // string fed to `parse_store_data` (code review finding #4).
        let path = std::env::temp_dir()
            .join(format!("winradio_test_store_{}_{}.json", std::process::id(), line!()));
        let _ = std::fs::remove_file(&path);

        let station = Station {
            id: "x".to_string(),
            name: "Station X".to_string(),
            url: "https://x.example/stream".to_string(),
            favicon_url: None,
            homepage: None,
            category: Some("Jazz".to_string()),
            is_favorite: false,
            added_at: 0,
            favorite_order: 0,
            country: None,
            geo_lat: None,
            geo_long: None,
        };
        let mut settings = Settings::default();
        settings.last_station = Some(station);

        let store = Store::new_at(path.clone());
        store.save_settings(&settings).expect("save_settings should write to disk");

        let reloaded = Store::new_at(path.clone());
        let got = reloaded.get_settings().expect("get_settings should succeed");

        let _ = std::fs::remove_file(&path);

        let last = got.last_station.expect("last_station should survive the reload");
        assert_eq!(last.id, "x");
        assert_eq!(last.name, "Station X");
    }

    #[test]
    fn malformed_last_station_does_not_reset_the_rest_of_settings() {
        // Same class of bug as `malformed_settings_does_not_drop_stations`:
        // a corrupt/future-incompatible `lastStation` shape must only drop
        // that one field, never fall back to `Settings::default()` for the
        // whole record (which would blank theme/volume/etc. too).
        let content = r#"{
            "stations": [],
            "settings": {"minimizeToTray": true, "theme": "dark",
                         "sleepTimerDefaultMinutes": 60, "volume": 0.5,
                         "lastStation": {"totally": "wrong-shape"}}
        }"#;

        let data = Store::parse_store_data(content);

        assert!(data.settings.last_station.is_none());
        assert!(data.settings.minimize_to_tray);
        assert_eq!(data.settings.theme, "dark");
        assert_eq!(data.settings.sleep_timer_default_minutes, 60);
        assert_eq!(data.settings.volume, 0.5);
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

    #[test]
    fn missing_location_fields_default_instead_of_failing_the_whole_station() {
        // Simulates a store.json written before `country`/`geoLat`/`geoLong`
        // existed (spec-2-2) — `#[serde(default)]` must let it parse instead
        // of dropping the station (or the whole collection) entirely. Same
        // back-compat shape as `missing_favorite_order_defaults_instead_of_
        // failing_the_whole_station` above.
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
        assert_eq!(data.stations[0].country, None);
        assert_eq!(data.stations[0].geo_lat, None);
        assert_eq!(data.stations[0].geo_long, None);
    }
}
