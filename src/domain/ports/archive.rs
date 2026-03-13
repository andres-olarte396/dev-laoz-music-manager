use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArchiveItem {
    pub identifier: String,
    pub title: Option<String>,
    pub creator: Option<String>,
    pub mediatype: Option<String>,
}

#[async_trait]
pub trait ArchiveService: Send + Sync {
    /// Busca items de audio en archive.org
    async fn search_audio(&self, query: &str, limit: u32) -> anyhow::Result<Vec<ArchiveItem>>;

    /// Obtiene información de los archivos de un item (canciones MP3, OGG, etc.)
    async fn get_item_files(&self, identifier: &str) -> anyhow::Result<Vec<String>>;

    /// Descarga un archivo específico
    async fn download_file(
        &self,
        identifier: &str,
        filename: &str,
        dest: &std::path::Path,
    ) -> anyhow::Result<()>;
}
