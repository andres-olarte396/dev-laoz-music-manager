use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct Track {
    pub id: Ulid,
    pub file_path: String,
    
    // Metadata ID3 / Vorbis
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    
    // Technical metadata
    pub duration_seconds: Option<u64>,
}

impl Track {
    // Constructor completo para cuando extraemos metadata exitosamente
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        file_path: String,
        title: Option<String>,
        artist: Option<String>,
        album_artist: Option<String>,
        album: Option<String>,
        year: Option<u32>,
        genre: Option<String>,
        duration_seconds: Option<u64>,
    ) -> Self {
        Self {
            id: Ulid::new(),
            file_path,
            title,
            artist,
            album_artist,
            album,
            year,
            genre,
            duration_seconds,
        }
    }

    // Constructor de fallback (si el extractor falla)
    pub fn fallback(file_path: String, filename_as_title: String) -> Self {
        Self {
            id: Ulid::new(),
            file_path,
            title: Some(filename_as_title),
            artist: Some("Unknown".to_string()),
            album_artist: None,
            album: Some("Unknown Album".to_string()),
            year: None,
            genre: None,
            duration_seconds: None,
        }
    }
}
