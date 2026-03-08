use crate::domain::entities::track::Track;
use ulid::Ulid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Not found")]
    NotFound,
}

#[async_trait::async_trait]
pub trait TrackRepository: Send + Sync {
    async fn save(&self, track: &Track) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &Ulid) -> Result<Option<Track>, RepositoryError>;
    // Cursor-based pagination port
    async fn list_paginated(&self, cursor: Option<Ulid>, limit: u32) -> Result<Vec<Track>, RepositoryError>;
    // Dynamic text-based search port
    async fn search(&self, query: &str, limit: u32) -> Result<Vec<Track>, RepositoryError>;
    // Sub-path filtering for the file explorer
    async fn search_by_path(&self, base_path: &str, limit: u32) -> Result<Vec<Track>, RepositoryError>;
}
