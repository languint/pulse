use serde::Deserialize;
use thiserror::Error;

const LRCLIB_BASE: &str = "https://lrclib.net";
const USER_AGENT: &str = "Pulse/0.1.0 (https://github.com/pulse-player/pulse)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricsLookup {
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub duration_secs: u32,
}

#[derive(Debug, Error)]
pub enum LyricsFetchError {
    #[error("lyrics not found for this track")]
    NotFound,
    #[error("network request failed: {0}")]
    Request(String),
    #[error("response could not be parsed: {0}")]
    Parse(String),
}

#[derive(Debug, Deserialize)]
struct LrcLibResponse {
    #[serde(default, rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
    #[serde(default, rename = "plainLyrics")]
    plain_lyrics: Option<String>,
}

/// Fetches lyrics from LRCLIB using track metadata.
///
/// Tries the cached endpoint first, then the full lookup endpoint.
pub async fn fetch_lrclib_lyrics(lookup: &LyricsLookup) -> Result<String, LyricsFetchError> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|error| LyricsFetchError::Request(error.to_string()))?;

    if let Some(content) = request_lrclib(&client, "/api/get-cached", lookup).await? {
        return Ok(content);
    }

    request_lrclib(&client, "/api/get", lookup)
        .await?
        .ok_or(LyricsFetchError::NotFound)
}

async fn request_lrclib(
    client: &reqwest::Client,
    path: &str,
    lookup: &LyricsLookup,
) -> Result<Option<String>, LyricsFetchError> {
    let url = format!("{LRCLIB_BASE}{path}");
    let response = client
        .get(url)
        .query(&[
            ("track_name", lookup.track_name.as_str()),
            ("artist_name", lookup.artist_name.as_str()),
            ("album_name", lookup.album_name.as_str()),
            ("duration", &lookup.duration_secs.to_string()),
        ])
        .send()
        .await
        .map_err(|error| LyricsFetchError::Request(error.to_string()))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(LyricsFetchError::Request(format!(
            "LRCLIB returned {status}: {body}"
        )));
    }

    let payload: LrcLibResponse = response
        .json()
        .await
        .map_err(|error| LyricsFetchError::Parse(error.to_string()))?;

    Ok(pick_lyrics_content(payload))
}

fn pick_lyrics_content(payload: LrcLibResponse) -> Option<String> {
    payload
        .synced_lyrics
        .filter(|value| !value.trim().is_empty())
        .or_else(|| payload.plain_lyrics.filter(|value| !value.trim().is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_synced_lyrics() {
        let content = pick_lyrics_content(LrcLibResponse {
            synced_lyrics: Some("[00:01.00]Synced".into()),
            plain_lyrics: Some("Plain".into()),
        })
        .expect("content");

        assert!(content.contains("Synced"));
    }

    #[test]
    fn falls_back_to_plain_lyrics() {
        let content = pick_lyrics_content(LrcLibResponse {
            synced_lyrics: None,
            plain_lyrics: Some("Plain line".into()),
        })
        .expect("content");

        assert_eq!(content, "Plain line");
    }
}
