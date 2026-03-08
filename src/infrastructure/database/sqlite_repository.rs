use crate::domain::entities::track::Track;
use crate::domain::ports::track_repository::{TrackRepository, RepositoryError};
use ulid::Ulid;
use sqlx::{SqlitePool, Row};
use async_trait::async_trait;
use std::str::FromStr;

#[derive(Clone)]
pub struct SqliteTrackRepository {
    pool: SqlitePool,
}

impl SqliteTrackRepository {
    pub async fn new(pool: SqlitePool) -> Result<Self, sqlx::Error> {
        // Inicializar la tabla en SQLite de manera autogestionada si no existe
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                file_path TEXT UNIQUE NOT NULL,
                title TEXT,
                artist TEXT,
                album_artist TEXT,
                album TEXT,
                year INTEGER,
                genre TEXT,
                duration_seconds INTEGER
            );"
        )
        .execute(&pool)
        .await?;
        
        // Crear un índice para la paginación keyset basada en ULID (que es secuencial)
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tracks_id ON tracks (id);")
            .execute(&pool)
            .await?;
            
        Ok(Self { pool })
    }
}

#[async_trait]
impl TrackRepository for SqliteTrackRepository {
    async fn save(&self, track: &Track) -> Result<(), RepositoryError> {
        let id_str = track.id.to_string();
        
        sqlx::query(
            "INSERT INTO tracks (id, file_path, title, artist, album_artist, album, year, genre, duration_seconds)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(file_path) DO UPDATE SET
                title=excluded.title,
                artist=excluded.artist,
                album_artist=excluded.album_artist,
                album=excluded.album,
                year=excluded.year,
                genre=excluded.genre,
                duration_seconds=excluded.duration_seconds;"
        )
        .bind(&id_str)
        .bind(&track.file_path)
        .bind(&track.title)
        .bind(&track.artist)
        .bind(&track.album_artist)
        .bind(&track.album)
        .bind(track.year.map(|y| y as i64))
        .bind(&track.genre)
        .bind(track.duration_seconds.map(|d| d as i64))
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &Ulid) -> Result<Option<Track>, RepositoryError> {
        let id_str = id.to_string();
        
        let row = sqlx::query("SELECT * FROM tracks WHERE id = ?")
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if let Some(row) = row {
            let track_id_str: String = row.get("id");
            let track_id = Ulid::from_str(&track_id_str).map_err(|_| RepositoryError::Database("Invalid ULID".into()))?;
            let year: Option<i64> = row.get("year");
            let duration: Option<i64> = row.get("duration_seconds");

            let track = Track {
                id: track_id,
                file_path: row.get("file_path"),
                title: row.get("title"),
                artist: row.get("artist"),
                album_artist: row.get("album_artist"),
                album: row.get("album"),
                year: year.map(|y| y as u32),
                genre: row.get("genre"),
                duration_seconds: duration.map(|d| d as u64),
            };
            Ok(Some(track))
        } else {
            Ok(None)
        }
    }

    async fn list_paginated(&self, cursor: Option<Ulid>, limit: u32) -> Result<Vec<Track>, RepositoryError> {
        let query_str = match &cursor {
            Some(_) => "SELECT * FROM tracks WHERE id > ? ORDER BY id ASC LIMIT ?",
            None => "SELECT * FROM tracks ORDER BY id ASC LIMIT ?",
        };

        let mut query = sqlx::query(query_str);

        if let Some(c) = cursor {
            let cursor_str = c.to_string();
            query = query.bind(cursor_str);
        }

        let rows = query.bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for row in rows {
            let track_id_str: String = row.get("id");
            let track_id = Ulid::from_str(&track_id_str).map_err(|_| RepositoryError::Database("Invalid ULID".into()))?;
            let year: Option<i64> = row.get("year");
            let duration: Option<i64> = row.get("duration_seconds");

            tracks.push(Track {
                id: track_id,
                file_path: row.get("file_path"),
                title: row.get("title"),
                artist: row.get("artist"),
                album_artist: row.get("album_artist"),
                album: row.get("album"),
                year: year.map(|y| y as u32),
                genre: row.get("genre"),
                duration_seconds: duration.map(|d| d as u64),
            });
        }

        Ok(tracks)
    }

    async fn search(&self, query_str: &str, limit: u32) -> Result<Vec<Track>, RepositoryError> {
        let like_query = format!("%{}%", query_str);

        let rows = sqlx::query(
            "SELECT * FROM tracks 
             WHERE title LIKE ? OR artist LIKE ? OR album LIKE ? OR file_path LIKE ?
             ORDER BY artist ASC, album ASC, title ASC 
             LIMIT ?"
        )
        .bind(&like_query)
        .bind(&like_query)
        .bind(&like_query)
        .bind(&like_query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for row in rows {
            let track_id_str: String = row.get("id");
            let track_id = Ulid::from_str(&track_id_str).map_err(|_| RepositoryError::Database("Invalid ULID".into()))?;
            let year: Option<i64> = row.get("year");
            let duration: Option<i64> = row.get("duration_seconds");

            tracks.push(Track {
                id: track_id,
                file_path: row.get("file_path"),
                title: row.get("title"),
                artist: row.get("artist"),
                album_artist: row.get("album_artist"),
                album: row.get("album"),
                year: year.map(|y| y as u32),
                genre: row.get("genre"),
                duration_seconds: duration.map(|d| d as u64),
            });
        }

        Ok(tracks)
    }

    async fn search_by_path(&self, base_path: &str, limit: u32) -> Result<Vec<Track>, RepositoryError> {
        // If base_path is "/music/", we want "file_path LIKE '/music/%'"
        let like_query = format!("{}%", base_path);

        let rows = sqlx::query(
            "SELECT * FROM tracks 
             WHERE file_path LIKE ?
             ORDER BY artist ASC, album ASC, title ASC 
             LIMIT ?"
        )
        .bind(&like_query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for row in rows {
            let track_id_str: String = row.get("id");
            let track_id = Ulid::from_str(&track_id_str).map_err(|_| RepositoryError::Database("Invalid ULID".into()))?;
            let year: Option<i64> = row.get("year");
            let duration: Option<i64> = row.get("duration_seconds");

            tracks.push(Track {
                id: track_id,
                file_path: row.get("file_path"),
                title: row.get("title"),
                artist: row.get("artist"),
                album_artist: row.get("album_artist"),
                album: row.get("album"),
                year: year.map(|y| y as u32),
                genre: row.get("genre"),
                duration_seconds: duration.map(|d| d as u64),
            });
        }

        Ok(tracks)
    }
}
