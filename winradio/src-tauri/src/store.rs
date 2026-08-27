use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::State;
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
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            StoreData::default()
        };

        Ok(Self {
            path,
            data: Arc::new(Mutex::new(data)),
        })
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

    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        let data = self.data.lock();
        match key {
            "minimizeToTray" => Some(serde_json::Value::Bool(data.settings.minimize_to_tray)),
            "startMinimized" => Some(serde_json::Value::Bool(data.settings.start_minimized)),
            _ => None,
        }
    }
}