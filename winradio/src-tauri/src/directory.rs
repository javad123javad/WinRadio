//! Radio-Browser directory client: Tauri commands backing Story 1.2's
//! Search & Browse feature. All network access to `all.api.radio-browser.info`
//! goes through this module — the frontend never calls Radio-Browser
//! directly (AD-1).
//!
//! Error contract (AD-1/NFR4, never conflated): a network/connect failure
//! returns `Err(CONNECT_ERROR)`; a successful call that simply matched
//! nothing returns `Ok(vec![])`.

use serde::{Deserialize, Serialize};
use std::time::Duration;

const BASE_URL: &str = "https://all.api.radio-browser.info";
const USER_AGENT: &str = "WinRadio/0.1";

/// Copy owned by EXPERIENCE.md's State Patterns table. This is the *only*
/// copy of this string in the app: `search.ts`'s `catch` block displays
/// whatever message `invoke()` rejects with verbatim rather than keeping a
/// second hardcoded copy on the frontend, so the two can never drift apart
/// (a prior version of this comment claimed that without it actually being
/// true — code review finding).
pub const CONNECT_ERROR: &str = "Can't reach the station directory — check your connection";

/// A search-result station from Radio-Browser — deliberately distinct from
/// the persisted `crate::commands::Station` (favorites) shape; the two are
/// never conflated (per Intent).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryStation {
    pub id: String,
    pub name: String,
    pub url: String,
    pub favicon: Option<String>,
    pub tags: Option<String>,
    pub country: Option<String>,
    pub language: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
}

/// Real genre/country/language values fetched lazily from Radio-Browser's
/// own discovery endpoints — never a hardcoded static list (per Never).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptions {
    pub genres: Vec<String>,
    pub countries: Vec<String>,
    pub languages: Vec<String>,
}

// Mirrors Radio-Browser's `/json/stations/search` response shape. Field
// names already match the API's JSON keys exactly (no camelCase rename
// needed). `#[serde(default)]` means a station missing an optional field
// entirely (rather than sending it as `""`/`null`) still deserializes
// instead of erroring the whole batch out.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawStation {
    stationuuid: String,
    name: String,
    url: String,
    favicon: String,
    tags: String,
    country: String,
    language: String,
    codec: String,
    bitrate: Option<u32>,
    geo_lat: Option<f64>,
    geo_long: Option<f64>,
}

impl From<RawStation> for DirectoryStation {
    fn from(raw: RawStation) -> Self {
        // Radio-Browser signals "unknown" with an empty string, not a JSON
        // null — normalize that into `None` so the frontend can use plain
        // truthiness checks instead of also special-casing `""`.
        let non_empty = |s: String| if s.trim().is_empty() { None } else { Some(s) };
        DirectoryStation {
            id: raw.stationuuid,
            name: raw.name,
            url: raw.url,
            favicon: non_empty(raw.favicon),
            tags: non_empty(raw.tags),
            country: non_empty(raw.country),
            language: non_empty(raw.language),
            codec: non_empty(raw.codec),
            bitrate: raw.bitrate.filter(|b| *b > 0),
            geo_lat: raw.geo_lat,
            geo_long: raw.geo_long,
        }
    }
}

// Mirrors the shape shared by Radio-Browser's `/json/tags`, `/json/countries`,
// and `/json/languages` endpoints — each is a list of `{name, stationcount, ...}`
// entries; only `name` is needed here.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawNamedEntry {
    name: String,
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| CONNECT_ERROR.to_string())
}

/// Pure JSON->DTO mapping, split out from the network call so the
/// "genuine zero match" branch (`Ok(vec![])`) can be unit-tested without a
/// live network dependency.
///
/// Parses the top level as loosely-typed `Value`s first and converts each
/// entry to a `RawStation` individually, skipping (never failing the whole
/// batch on) any single entry that doesn't convert — a community-maintained
/// directory occasionally has one station with a malformed field (e.g. a
/// `bitrate` sent as a string), and that must not turn a 99-good/1-bad
/// response into a false "Can't reach the station directory" (code review
/// finding).
fn parse_stations(body: &str) -> Result<Vec<DirectoryStation>, String> {
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(body).map_err(|_| CONNECT_ERROR.to_string())?;
    Ok(entries
        .into_iter()
        .filter_map(|entry| serde_json::from_value::<RawStation>(entry).ok())
        .map(DirectoryStation::from)
        .collect())
}

fn parse_names(body: &str) -> Result<Vec<String>, String> {
    let raw: Vec<RawNamedEntry> = serde_json::from_str(body).map_err(|_| CONNECT_ERROR.to_string())?;
    let mut seen = std::collections::HashSet::new();
    Ok(raw
        .into_iter()
        .map(|entry| entry.name)
        .filter(|name| !name.trim().is_empty())
        // Dedupe defensively (order-preserving: keeps the first, highest
        // -stationcount-ranked occurrence) in case a discovery endpoint
        // ever repeats a name — a Vue `:key` in `SearchPanel.vue`'s filter
        // `<select>` must stay unique (code review finding).
        .filter(|name| seen.insert(name.clone()))
        .collect())
}

async fn fetch_stations(
    client: &reqwest::Client,
    base_url: &str,
    params: &[(&str, String)],
) -> Result<Vec<DirectoryStation>, String> {
    let response = client
        .get(format!("{base_url}/json/stations/search"))
        .query(params)
        .send()
        .await
        .map_err(|_| CONNECT_ERROR.to_string())?;

    if !response.status().is_success() {
        return Err(CONNECT_ERROR.to_string());
    }

    let body = response.text().await.map_err(|_| CONNECT_ERROR.to_string())?;
    parse_stations(&body)
}

async fn fetch_names(client: &reqwest::Client, base_url: &str, path: &str) -> Result<Vec<String>, String> {
    let response = client
        .get(format!("{base_url}{path}"))
        .query(&[("order", "stationcount"), ("reverse", "true"), ("limit", "100")])
        .send()
        .await
        .map_err(|_| CONNECT_ERROR.to_string())?;

    if !response.status().is_success() {
        return Err(CONNECT_ERROR.to_string());
    }

    let body = response.text().await.map_err(|_| CONNECT_ERROR.to_string())?;
    parse_names(&body)
}

// `base_url`-parameterized so tests can point at an address guaranteed to
// refuse the connection, exercising the real offline error path without
// depending on the test machine's actual internet access.
async fn search_stations_at(
    base_url: &str,
    name: Option<String>,
    genre: Option<String>,
    country: Option<String>,
    language: Option<String>,
) -> Result<Vec<DirectoryStation>, String> {
    let client = build_client()?;

    // hidebroken=true / limit=100 per Code Map — always sent regardless of
    // which optional filters are active.
    let mut params: Vec<(&str, String)> = vec![
        ("hidebroken", "true".to_string()),
        ("limit", "100".to_string()),
    ];

    let mut push_if_present = |key: &'static str, value: Option<String>| {
        if let Some(trimmed) = value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
            params.push((key, trimmed));
        }
    };
    push_if_present("name", name);
    push_if_present("tag", genre);
    push_if_present("country", country);
    push_if_present("language", language);

    fetch_stations(&client, base_url, &params).await
}

/// Searches Radio-Browser for stations matching any combination of a free
/// text `name` and genre/country/language filters (all optional, AND'd
/// together per Boundaries & Constraints). Filters may be used standalone
/// with no text query at all.
#[tauri::command]
pub async fn search_stations(
    name: Option<String>,
    genre: Option<String>,
    country: Option<String>,
    language: Option<String>,
) -> Result<Vec<DirectoryStation>, String> {
    search_stations_at(BASE_URL, name, genre, country, language).await
}

async fn get_filter_options_at(base_url: &str) -> Result<FilterOptions, String> {
    let client = build_client()?;
    let (genres, countries, languages) = tokio::join!(
        fetch_names(&client, base_url, "/json/tags"),
        fetch_names(&client, base_url, "/json/countries"),
        fetch_names(&client, base_url, "/json/languages"),
    );

    // Each of the three endpoints is fetched independently, so one having a
    // transient hiccup must not blank out the other two, already-succeeded,
    // dropdowns — fall back to an empty list per endpoint instead of
    // propagating the first failure through the whole function (code review
    // finding). A genuinely offline directory still yields all three empty,
    // same net effect as before.
    Ok(FilterOptions {
        genres: genres.unwrap_or_default(),
        countries: countries.unwrap_or_default(),
        languages: languages.unwrap_or_default(),
    })
}

/// Lazily fetches real genre/country/language values from Radio-Browser's
/// own discovery endpoints (never a hardcoded static list, per Never).
#[tauri::command]
pub async fn get_filter_options() -> Result<FilterOptions, String> {
    get_filter_options_at(BASE_URL).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stations_returns_ok_empty_vec_for_a_genuine_zero_match_response() {
        // I/O matrix: "Genuine zero match" -> `Ok(vec![])`, never an error.
        assert_eq!(parse_stations("[]"), Ok(vec![]));
    }

    #[test]
    fn parse_stations_maps_fields_and_normalizes_empty_strings_to_none() {
        let body = r#"[{
            "stationuuid": "abc-123",
            "name": "Jazz FM",
            "url": "http://example.com/stream",
            "favicon": "",
            "tags": "jazz,smooth",
            "country": "Germany",
            "language": "",
            "codec": "MP3",
            "bitrate": 128,
            "geo_lat": 52.5,
            "geo_long": 13.4
        }]"#;

        let result = parse_stations(body).expect("valid response should parse");
        assert_eq!(result.len(), 1);

        let station = &result[0];
        assert_eq!(station.id, "abc-123");
        assert_eq!(station.name, "Jazz FM");
        assert_eq!(station.url, "http://example.com/stream");
        assert_eq!(station.favicon, None);
        assert_eq!(station.tags.as_deref(), Some("jazz,smooth"));
        assert_eq!(station.country.as_deref(), Some("Germany"));
        assert_eq!(station.language, None);
        assert_eq!(station.bitrate, Some(128));
    }

    #[test]
    fn parse_stations_rejects_malformed_json_as_the_connect_error() {
        let result = parse_stations("not json");
        assert_eq!(result, Err(CONNECT_ERROR.to_string()));
    }

    #[test]
    fn parse_stations_skips_a_single_malformed_entry_instead_of_failing_the_whole_batch() {
        // `bitrate` sent as a string is a type mismatch, not a missing
        // field — `#[serde(default)]` doesn't cover it, so this entry must
        // be dropped individually rather than erroring out the two good
        // entries around it (code review finding).
        let body = r#"[
            {"stationuuid": "good-1", "name": "Good One", "url": "http://a.example/stream"},
            {"stationuuid": "bad", "name": "Bad One", "url": "http://b.example/stream", "bitrate": "not-a-number"},
            {"stationuuid": "good-2", "name": "Good Two", "url": "http://c.example/stream"}
        ]"#;

        let result = parse_stations(body).expect("valid entries should still parse");
        let ids: Vec<&str> = result.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["good-1", "good-2"]);
    }

    #[test]
    fn parse_names_extracts_and_filters_blank_names() {
        let body = r#"[{"name":"jazz","stationcount":10},{"name":""},{"name":"rock","stationcount":5}]"#;
        assert_eq!(parse_names(body).unwrap(), vec!["jazz".to_string(), "rock".to_string()]);
    }

    #[test]
    fn parse_names_deduplicates_while_preserving_order() {
        let body = r#"[{"name":"jazz"},{"name":"rock"},{"name":"jazz"},{"name":"pop"}]"#;
        assert_eq!(
            parse_names(body).unwrap(),
            vec!["jazz".to_string(), "rock".to_string(), "pop".to_string()]
        );
    }

    #[tokio::test]
    async fn search_stations_returns_the_connect_error_when_the_directory_is_unreachable() {
        // Port 65535 on loopback refuses the connection immediately (nothing
        // listens there) — this deterministically exercises the "can't
        // reach the directory" path, distinct from a genuine zero-match
        // `Ok(vec![])`, without depending on the test machine's real
        // internet access.
        let result = search_stations_at(
            "http://127.0.0.1:65535",
            Some("jazz".to_string()),
            None,
            None,
            None,
        )
        .await;

        assert_eq!(result, Err(CONNECT_ERROR.to_string()));
    }

    #[tokio::test]
    async fn get_filter_options_falls_back_to_empty_lists_when_the_directory_is_unreachable() {
        // All three endpoints fail independently here, but per-endpoint
        // fallback (code review finding) means the function itself still
        // succeeds with empty lists rather than propagating just the first
        // endpoint's failure as a whole-function `Err`.
        let result = get_filter_options_at("http://127.0.0.1:65535").await;
        assert_eq!(result, Ok(FilterOptions::default()));
    }
}
