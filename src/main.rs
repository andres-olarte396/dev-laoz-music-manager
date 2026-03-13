pub mod domain;
pub mod application;
pub mod infrastructure;

use infrastructure::cli::{self, Commands};
use infrastructure::database::sqlite_repository::SqliteTrackRepository;
use infrastructure::filesystem::symphonia_extractor::SymphoniaExtractor;
use infrastructure::config_loader::ConfigLoader;
use application::use_cases::scan_library::ScanLibraryUseCase;
use application::use_cases::playback::PlaybackUseCase;
use application::use_cases::tui::TuiUseCase;
use application::use_cases::identify_track::IdentifyTrackUseCase;
use application::use_cases::rename_track::{RenameTrackUseCase, VersionTag};
use domain::entities::config::RenameConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Iniciando Music Manager...");
    
    let cli_args = cli::parse();

    // 1. Cargar configuración desde config.toml
    let config_path = ConfigLoader::get_default_config_path();
    let config = ConfigLoader::load(&config_path).await
        .unwrap_or_else(|e| {
            eprintln!("⚠️ No se pudo cargar config.toml: {}. Usando valores por defecto.", e);
            domain::entities::config::AppConfig::default()
        });

    // 2. DB (SQLite)
    let db_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("music-manager");
    std::fs::create_dir_all(&db_dir).unwrap_or_default();
    let db_path = db_dir.join("music.db");

    let pool_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path.to_str().unwrap_or("music.db"))
        .create_if_missing(true);
    let pool = sqlx::SqlitePool::connect_with(pool_options).await?;
    let repository = SqliteTrackRepository::new(pool).await?;
    let extractor = SymphoniaExtractor::new();

    // 3. Casos de uso
    let scan_use_case = ScanLibraryUseCase::new(repository.clone(), extractor);

    match &cli_args.command {
        Commands::Scan { path, deep, watch } => {
            println!("Ejecutando SCAN en {} (Deep: {}, Watch: {})", path, deep, watch);
            scan_use_case.execute(path, *deep).await?;
        }
        Commands::Status => {
            println!("Ejecutando STATUS...");
        }
        Commands::List { limit, after } => {
            println!("Ejecutando LIST (Límite: {}, Cursor: {:?})", limit, after);
            let parsed_cursor = match after {
                Some(c) => match std::str::FromStr::from_str(&c) {
                    Ok(id) => Some(id),
                    Err(_) => {
                        eprintln!("Error: Cursor inválido. Debe ser un ULID.");
                        return Ok(());
                    }
                },
                None => None,
            };

            use crate::domain::ports::track_repository::TrackRepository;
            match repository.list_paginated(parsed_cursor, *limit).await {
                Ok(tracks) => {
                    println!("\n🎶 Pistas en Biblioteca ({}):", tracks.len());
                    for (i, t) in tracks.iter().enumerate() {
                        let artist_str = t.artist.as_deref().unwrap_or("Unknown");
                        let title_str = t.title.as_deref().unwrap_or(&t.file_path);
                        println!("{:03} | ID: {} | {} - {}", i + 1, t.id, artist_str, title_str);
                    }
                }
                Err(e) => eprintln!("Error al consultar BD: {}", e),
            }
        }
        Commands::Tui { path } => {
            println!("Iniciando TUI Avanzada...");
            let use_case = TuiUseCase::new(repository.clone(), path.clone());
            if let Err(e) = use_case.execute().await {
                eprintln!("Error en TUI: {}", e);
            }
        }
        Commands::Doctor => { println!("Ejecutando DOCTOR..."); }
        Commands::Migrate { destino, mode, concurrent: _, dry_run } => {
            println!("Ejecutando MIGRATE hacia {} (Modo: {}, Dry-Run: {})", destino, mode, dry_run);
        }
        Commands::Play { criterio: _, id: _, artist: _, shuffle: _, daemon } => {
            println!("Ejecutando PLAY (Daemon: {})", daemon);
            let play_use_case = PlaybackUseCase::new();
            if let Err(e) = play_use_case.execute(&cli_args.command) {
                eprintln!("Error en reproducción: {}", e);
            }
        }
        Commands::Identify { path, save } => {
            let identify_use_case = IdentifyTrackUseCase::new(repository.clone(), config.identify.clone());
            if config.identify.acoustid_key.is_empty() {
                eprintln!("⚠️ No hay acoustid_key configurada en config.toml → [identify] acoustid_key = \"TU_KEY\"");
            }
            if let Err(e) = identify_use_case.execute(path, *save).await {
                eprintln!("Error identificando pista: {}", e);
            }
        }
        Commands::Rename { path, dry_run, version, no_version } => {
            // Los flags CLI tienen precedencia sobre config.toml
            let mut rename_config = config.rename.clone();

            if *no_version {
                rename_config.auto_detect_version = false;
                rename_config.default_version = None;
            } else if let Some(v) = version {
                let v_lower = v.to_lowercase();
                let label = match v_lower.as_str() {
                    "acustica" | "acústica" | "acoustic" => "Acústica",
                    "envivo" | "en-vivo" | "live"        => "En Vivo",
                    "remix"                              => "Remix",
                    "cover"                              => "Cover",
                    "instrumental"                       => "Instrumental",
                    "radio"                              => "Radio Edit",
                    "extended"                           => "Extended Mix",
                    "demo"                               => "Demo",
                    "remaster"                           => "Remaster",
                    other                                => other,
                };
                println!("🏷️ Versión forzada (CLI): {}", label);
                rename_config.default_version = Some(label.to_string());
            }

            let mode = if *dry_run { "[SIMULACIÓN]" } else { "[APLICANDO]" };
            println!("✏️ Rename {} en: {}", mode, path);

            let use_case = RenameTrackUseCase::new(rename_config);
            if let Err(e) = use_case.execute(path, *dry_run) {
                eprintln!("Error en rename: {}", e);
            }
        }
        _ => {
            println!("Comando de control remoto recibido (Seek, Pause, Resume, Next, Prev).");
        }
    }
    
    Ok(())
}
