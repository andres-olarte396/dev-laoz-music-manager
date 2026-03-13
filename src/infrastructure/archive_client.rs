use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use urlencoding::encode;

use crate::domain::entities::config::ArchiveOrgConfig;
use crate::domain::ports::archive::{ArchiveItem, ArchiveService};

// Structs for internal deserialization
#[derive(Deserialize, Debug)]
struct SearchResponse {
    response: SearchResponseData,
}

#[derive(Deserialize, Debug)]
struct SearchResponseData {
    docs: Vec<ArchiveItem>,
}

#[derive(Deserialize, Debug)]
struct MetadataResponse {
    files: Vec<MetadataFile>,
}

#[derive(Deserialize, Debug)]
struct MetadataFile {
    name: String,
    format: Option<String>,
}

pub struct ArchiveClient {
    client: Client,
    config: ArchiveOrgConfig,
}

impl ArchiveClient {
    pub fn new(config: ArchiveOrgConfig, timeout_secs: u64) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self { client, config })
    }
}

#[async_trait]
impl ArchiveService for ArchiveClient {
    async fn search_audio(&self, query: &str, limit: u32) -> anyhow::Result<Vec<ArchiveItem>> {
        let actual_limit = if limit > 0 { limit } else { self.config.max_results };
        let url = format!(
            "{}?q={}+AND+mediatype:audio&output=json&rows={}",
            self.config.advanced_search_url,
            encode(query),
            actual_limit
        );

        let res = self.client.get(&url).send().await?;
        let search_res: SearchResponse = res.json().await?;
        
        Ok(search_res.response.docs)
    }

    async fn get_item_files(&self, identifier: &str) -> anyhow::Result<Vec<String>> {
        let url = format!("https://archive.org/metadata/{}", identifier);
        let res = self.client.get(&url).send().await?;
        let metadata_res: MetadataResponse = res.json().await?;

        // Filter files that are likely audio (MP3, OGG, FLAC)
        let mut audio_files = Vec::new();
        for file in metadata_res.files {
            if let Some(format) = file.format {
                if format.to_lowercase().contains("mp3") 
                   || format.to_lowercase().contains("ogg")
                   || format.to_lowercase().contains("flac") {
                    audio_files.push(file.name);
                }
            } else if file.name.ends_with(".mp3") || file.name.ends_with(".ogg") || file.name.ends_with(".flac") {
                 audio_files.push(file.name);
            }
        }

        Ok(audio_files)
    }

    async fn download_file(
        &self,
        identifier: &str,
        filename: &str,
        dest: &std::path::Path,
    ) -> anyhow::Result<()> {
        let url = format!("{}/{}/{}", self.config.download_base_url, identifier, filename);
        let mut res = self.client.get(&url).send().await?;
        
        if !res.status().is_success() {
            anyhow::bail!("Failed to download file, HTTP status: {}", res.status());
        }

        let mut file = tokio::fs::File::create(dest).await?;
        while let Some(chunk) = res.chunk().await? {
            tokio::io::copy(&mut chunk.as_ref(), &mut file).await?;
        }

        Ok(())
    }
}
