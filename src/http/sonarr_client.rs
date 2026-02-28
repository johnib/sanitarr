use super::ResponseExt;
use anyhow::Ok;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder, Url};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A client for interacting with Sonarr API.
/// https://sonarr.tv/docs/api/
pub struct SonarrClient {
    client: Client,
    base_url: Url,
}

impl SonarrClient {
    pub fn new(base_url: &str, api_key: &str) -> anyhow::Result<Self> {
        let mut base_url = Url::parse(base_url)?;
        base_url.set_path("/api/v3/");

        let default_headers = auth_headers(api_key)?;
        let client = ClientBuilder::new()
            .default_headers(default_headers)
            .build()?;

        Ok(Self { client, base_url })
    }

    /// Get the series IDs for a given TVDB ID.
    /// https://sonarr.tv/docs/api/#/Series/get_api_v3_series
    pub async fn series_by_tvdb_id(&self, provider_id: &str) -> anyhow::Result<Vec<SeriesInfo>> {
        let url = self.base_url.join("series")?;
        let response = self
            .client
            .get(url)
            .query(&[("tvdbId", provider_id)])
            .send()
            .await?
            .handle_error()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Get all tags.
    pub async fn tags(&self) -> anyhow::Result<Vec<Tag>> {
        let url = self.base_url.join("tag")?;
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .handle_error()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Get all episodes for a series
    /// https://sonarr.tv/docs/api/#/Episode/get_api_v3_episode
    pub async fn episodes_by_series(&self, series_id: u64) -> anyhow::Result<Vec<EpisodeInfo>> {
        let url = self.base_url.join("episode")?;
        let response = self
            .client
            .get(url)
            .query(&[("seriesId", series_id)])
            .send()
            .await?
            .handle_error()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Delete an episode file by its ID
    /// https://sonarr.tv/docs/api/#/EpisodeFile/delete_api_v3_episodefile__id_
    pub async fn delete_episode_file(&self, episode_file_id: u64) -> anyhow::Result<()> {
        let url = self
            .base_url
            .join("episodefile/")?
            .join(&episode_file_id.to_string())?;
        self.client
            .delete(url)
            .send()
            .await?
            .handle_error()
            .await?;
        Ok(())
    }

    /// Unmonitor an episode (prevent Sonarr from re-downloading)
    /// https://sonarr.tv/docs/api/#/Episode/put_api_v3_episode__id_
    pub async fn unmonitor_episode(&self, episode_id: u64) -> anyhow::Result<()> {
        // First, get the current episode data
        let url = self
            .base_url
            .join("episode/")?
            .join(&episode_id.to_string())?;

        let mut episode: EpisodeInfo = self
            .client
            .get(url.clone())
            .send()
            .await?
            .handle_error()
            .await?
            .json()
            .await?;

        // Set monitored to false
        episode.monitored = false;

        // Update the episode
        self.client
            .put(url)
            .json(&episode)
            .send()
            .await?
            .handle_error()
            .await?;

        Ok(())
    }
}

fn auth_headers(api_key: &str) -> Result<HeaderMap, anyhow::Error> {
    let mut default_headers = HeaderMap::new();
    let mut header_value = HeaderValue::from_str(api_key)?;
    header_value.set_sensitive(true);
    default_headers.insert("x-api-key", header_value);
    Ok(default_headers)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesInfo {
    pub title: String,
    pub id: u64,
    pub tags: Option<Vec<u64>>,
}

impl Debug for SeriesInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.title, self.id)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeInfo {
    pub id: u64,
    pub series_id: u64,
    pub episode_file_id: Option<u64>,
    pub title: String,
    pub season_number: u32,
    pub episode_number: u32,
    pub monitored: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub label: String,
    pub id: u64,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_auth_headers() {
        let headers = super::auth_headers("abc-key").unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers.get("x-api-key").unwrap(), "abc-key");
    }
}
