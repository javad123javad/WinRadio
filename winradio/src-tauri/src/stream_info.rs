//! DNS resolution backing Story 2.4's Stream Info Tile — the tile's only
//! genuinely fetched field (codec/bitrate/country render synchronously off
//! the already-cached `Station`, per Approach). Mirrors `directory.rs`'s
//! "pure parse split from the network call" shape: `extract_host_and_port`
//! is a pure, unit-testable URL parse; `resolve_ip` wraps it with the actual
//! (blocking) DNS lookup.
//!
//! Uses `http::Uri` (already a direct dependency for the HTTP stack) rather
//! than the fuller `url` crate — radio stream URLs are plain
//! `http(s)://host[:port]/path` and don't need `url`'s userinfo/fragment
//! handling (Design Notes). DNS resolution uses blocking
//! `std::net::ToSocketAddrs` inside `tokio::task::spawn_blocking` rather than
//! adding tokio's `net` feature, keeping the dependency surface minimal.

use std::net::ToSocketAddrs;

/// Parses `url`'s host and port, defaulting the port by scheme (443 for
/// `https`, 80 otherwise) when the URL doesn't specify one explicitly.
/// Returns `None` for a malformed URL or one with no host at all (e.g. a
/// bare path) — the caller resolves that to the tile's "unavailable" IP
/// state, never a `playback-error` (same scope isolation as Location/
/// Weather).
pub fn extract_host_and_port(url: &str) -> Option<(String, u16)> {
    let uri: http::Uri = url.parse().ok()?;
    let host = uri.host()?.to_string();
    let port = uri.port_u16().unwrap_or(match uri.scheme_str() {
        Some("https") => 443,
        _ => 80,
    });
    Some((host, port))
}

/// Resolves `url`'s host to an IP address via DNS. The actual (blocking)
/// lookup runs on `spawn_blocking` since `std::net::ToSocketAddrs` has no
/// async equivalent without adding tokio's `net` feature (Design Notes). A
/// malformed URL, a URL with no host, or a lookup failure/empty result all
/// surface as `Err` — resolved by the caller to the "unavailable" IP state,
/// never a `playback-error`.
pub async fn resolve_ip(url: &str) -> Result<String, String> {
    let (host, port) =
        extract_host_and_port(url).ok_or_else(|| "Could not determine stream host".to_string())?;

    tokio::task::spawn_blocking(move || {
        (host.as_str(), port)
            .to_socket_addrs()
            .map_err(|e| e.to_string())?
            .next()
            .map(|addr| addr.ip().to_string())
            .ok_or_else(|| "DNS lookup returned no addresses".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_host_and_port_returns_the_explicit_port_when_present() {
        assert_eq!(
            extract_host_and_port("http://example.com:8000/stream"),
            Some(("example.com".to_string(), 8000))
        );
    }

    #[test]
    fn extract_host_and_port_defaults_to_80_for_http_with_no_explicit_port() {
        assert_eq!(
            extract_host_and_port("http://example.com/stream"),
            Some(("example.com".to_string(), 80))
        );
    }

    #[test]
    fn extract_host_and_port_defaults_to_443_for_https_with_no_explicit_port() {
        assert_eq!(
            extract_host_and_port("https://example.com/stream"),
            Some(("example.com".to_string(), 443))
        );
    }

    #[test]
    fn extract_host_and_port_returns_none_for_a_malformed_url() {
        assert_eq!(extract_host_and_port("not a url at all"), None);
    }

    #[test]
    fn extract_host_and_port_returns_none_when_there_is_no_host() {
        // A bare path is a syntactically valid `http::Uri` (a
        // relative-reference) but carries no host at all.
        assert_eq!(extract_host_and_port("/just/a/path"), None);
    }

    #[tokio::test]
    async fn resolve_ip_fails_for_a_malformed_url_without_ever_attempting_dns() {
        let result = resolve_ip("not a url at all").await;
        assert!(result.is_err());
    }
}
