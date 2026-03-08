use crate::domain::entities::track::Track;
use crate::domain::ports::track_repository::{TrackRepository, RepositoryError};
use ulid::Ulid;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

// Un adaptador "Mock" (En memoria) temporal para que el MVP corra sin SQLite aún
pub struct InMemoryTrackRepository {
    count: AtomicUsize,
}

impl InMemoryTrackRepository {
    pub fn new() -> Self {
        Self { count: AtomicUsize::new(0) }
    }
}

#[async_trait]
impl TrackRepository for InMemoryTrackRepository {
    async fn save(&self, _track: &Track) -> Result<(), RepositoryError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        // Aquí no guardamos de verdad para ahorrar memoria en el demo, solo simulamos éxito
        Ok(())
    }

    async fn find_by_id(&self, _id: &Ulid) -> Result<Option<Track>, RepositoryError> {
        Ok(None)
    }

    async fn list_paginated(&self, _cursor: Option<Ulid>, _limit: u32) -> Result<Vec<Track>, RepositoryError> {
        Ok(vec![])
    }

    async fn search(&self, _query: &str, _limit: u32) -> Result<Vec<Track>, RepositoryError> {
        Ok(vec![])
    }

    async fn search_by_path(&self, _base_path: &str, _limit: u32) -> Result<Vec<Track>, RepositoryError> {
        Ok(vec![])
    }
}
