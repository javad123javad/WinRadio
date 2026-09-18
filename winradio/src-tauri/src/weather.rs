//! Open-Meteo weather client backing Story 2.3's Weather Tile. Mirrors
//! `directory.rs`'s "pure parse split from the network call" shape: the
//! network call (`fetch_weather`) is a thin wrapper around a pure,
//! unit-testable `parse_weather` so the I/O matrix's malformed-body branch
//! can be exercised without a live network dependency.
//!
//! Unlike Location (spec-2-2), the whole payload here is small enough to
//! ride inside the `weather-updated` event itself — no follow-up
//! command/image-fetch step is needed.

use serde::{Deserialize, Serialize};
use std::time::Duration;

const BASE_URL: &str = "https://api.open-meteo.com";
// Module-local: unlike `directory.rs`'s Radio-Browser `USER_AGENT`, there's
// no shared-UA requirement across modules for Open-Meteo, so this stays
// scoped to this file rather than becoming a crate-wide constant.
const USER_AGENT: &str = "WinRadio/0.1";

/// Current-conditions + today's high/low, small enough to ride directly
/// inside `weather-updated`'s `data` field — no separate tile-image-style
/// follow-up fetch (per Approach).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherInfo {
    pub temperature_c: f64,
    pub condition: String,
    pub forecast_high_c: f64,
    pub forecast_low_c: f64,
}

// Mirrors Open-Meteo's `current_weather=true&daily=temperature_2m_max,
// temperature_2m_min` response shape.
#[derive(Debug, Clone, Deserialize)]
struct RawCurrentWeather {
    temperature: f64,
    weathercode: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDaily {
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawForecastResponse {
    current_weather: RawCurrentWeather,
    daily: RawDaily,
}

pub fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())
}

/// WMO weather code -> short human-readable description. Only enough
/// branches to cover the common cases (clear/cloudy/rain/snow/thunderstorm)
/// plus an `_ => "Unknown"` fallback — same spirit as `directory.rs`'s
/// `tile_url` well-documented-formula note (Design Notes); no need to
/// enumerate all ~30 WMO codes here.
pub fn condition_from_weather_code(code: u32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 | 2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 | 53 | 55 | 56 | 57 => "Drizzle",
        61 | 63 | 65 | 66 | 67 | 80 | 81 | 82 => "Rain",
        71 | 73 | 75 | 77 | 85 | 86 => "Snow",
        95 | 96 | 99 => "Thunderstorm",
        _ => "Unknown",
    }
}

/// Pure JSON->DTO mapping, split out from the network call so the
/// malformed-body branch of the I/O matrix is unit-testable without a live
/// network dependency (mirrors `directory.rs`'s `parse_stations`).
pub fn parse_weather(body: &str) -> Result<WeatherInfo, String> {
    let raw: RawForecastResponse =
        serde_json::from_str(body).map_err(|e| format!("Failed to parse weather response: {e}"))?;

    let forecast_high_c = raw
        .daily
        .temperature_2m_max
        .first()
        .copied()
        .ok_or_else(|| "Missing today's forecast high".to_string())?;
    let forecast_low_c = raw
        .daily
        .temperature_2m_min
        .first()
        .copied()
        .ok_or_else(|| "Missing today's forecast low".to_string())?;

    Ok(WeatherInfo {
        temperature_c: raw.current_weather.temperature,
        condition: condition_from_weather_code(raw.current_weather.weathercode).to_string(),
        forecast_high_c,
        forecast_low_c,
    })
}

async fn fetch_weather_at(client: &reqwest::Client, base_url: &str, lat: f64, long: f64) -> Result<WeatherInfo, String> {
    let url = format!(
        "{base_url}/v1/forecast?latitude={lat}&longitude={long}&current_weather=true&daily=temperature_2m_max,temperature_2m_min&timezone=auto"
    );

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Weather fetch failed with status {}", response.status()));
    }

    let body = response.text().await.map_err(|e| e.to_string())?;
    parse_weather(&body)
}

/// Fetches current weather + today's high/low for `(lat, long)` from
/// Open-Meteo. Any failure (network, non-2xx, unparseable body) is surfaced
/// as `Err` — the caller (`run_playback`) resolves this to the shared
/// "Weather unavailable" placeholder, same scope isolation as
/// Location/Stream Info (never a `playback-error`).
pub async fn fetch_weather(client: &reqwest::Client, lat: f64, long: f64) -> Result<WeatherInfo, String> {
    fetch_weather_at(client, BASE_URL, lat, long).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_BODY: &str = r#"{
        "current_weather": {"temperature": 18.3, "windspeed": 5.0, "weathercode": 3, "time": "2026-09-18T12:00"},
        "daily": {
            "time": ["2026-09-18"],
            "temperature_2m_max": [21.5],
            "temperature_2m_min": [12.1]
        }
    }"#;

    #[test]
    fn parse_weather_maps_a_valid_response() {
        let info = parse_weather(VALID_BODY).expect("valid response should parse");
        assert_eq!(info.temperature_c, 18.3);
        assert_eq!(info.condition, "Overcast");
        assert_eq!(info.forecast_high_c, 21.5);
        assert_eq!(info.forecast_low_c, 12.1);
    }

    #[test]
    fn parse_weather_rejects_malformed_json() {
        let result = parse_weather("not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_weather_rejects_a_body_missing_required_fields() {
        let body = r#"{"current_weather": {"temperature": 18.3, "weathercode": 3}}"#;
        let result = parse_weather(body);
        assert!(result.is_err());
    }

    #[test]
    fn condition_from_weather_code_maps_known_codes() {
        assert_eq!(condition_from_weather_code(0), "Clear sky");
        assert_eq!(condition_from_weather_code(61), "Rain");
        assert_eq!(condition_from_weather_code(95), "Thunderstorm");
    }

    #[test]
    fn condition_from_weather_code_falls_back_to_unknown_for_an_unrecognized_code() {
        assert_eq!(condition_from_weather_code(9999), "Unknown");
    }

    #[tokio::test]
    async fn fetch_weather_fails_when_the_api_is_unreachable() {
        let client = build_client().expect("client should build");
        let result = fetch_weather_at(&client, "http://127.0.0.1:65535", 50.85, 4.35).await;
        assert!(result.is_err());
    }
}
