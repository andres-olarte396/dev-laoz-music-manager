// use crate::domain::entities::track::Track;
use crate::domain::ports::track_repository::TrackRepository;
use crate::domain::ports::metadata_extractor::MetadataExtractor;
use walkdir::WalkDir;
use std::path::Path;

pub struct ScanLibraryUseCase<R: TrackRepository, E: MetadataExtractor> {
    pub repository: R,
    pub extractor: E,
}

impl<R: TrackRepository, E: MetadataExtractor> ScanLibraryUseCase<R, E> {
    pub fn new(repository: R, extractor: E) -> Self {
        Self { repository, extractor }
    }

    pub async fn execute(&self, path: &str, _deep: bool) -> anyhow::Result<()> {
        let root = Path::new(path);
        if !root.exists() || !root.is_dir() {
            anyhow::bail!("Path '{}' is not a valid directory.", path);
        }

        println!("🔍 Iniciando escaneo en: {}", path);
        let mut count = 0;

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let current_path = entry.path();
                let extension = current_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                
                let extensions = ["mp3", "flac", "m4a", "wma", "wav", "ogg", "aac", "opus", "alac", "aiff"];
                // Filtro para todas las extensiones soportadas
                if extensions.contains(&extension.as_str()) {
                    let file_path = current_path.to_string_lossy().to_string();
                    
                    // Extraer Metadata usando el Puerto
                    let track = self.extractor.extract_metadata(&file_path);

                    // Intentamos guardarlo en el repositorio
                    if let Err(e) = self.repository.save(&track).await {
                        eprintln!("Error guardando track {}: {}", file_path, e);
                    } else {
                        count += 1;
                        if count % 100 == 0 {
                            println!("✅ Se han procesado {} pistas...", count);
                        }
                    }
                }
            }
        }

        println!("🎉 Escaneo completado. Total de pistas encontradas: {}", count);
        Ok(())
    }
}
