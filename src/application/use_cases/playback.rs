use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use rodio::{Decoder, OutputStream, Sink};
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use walkdir::WalkDir;

use crate::infrastructure::cli::Commands;

pub struct PlaybackUseCase;

impl PlaybackUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, command: &Commands) -> anyhow::Result<()> {
        if let Commands::Play { criterio, id: _, artist: _, shuffle: _, daemon: _ } = command {
            let path_str = criterio.as_deref().unwrap_or(".");
            let target_path = Path::new(path_str);

            if !target_path.exists() {
                anyhow::bail!("La ruta especificada no existe: {}", path_str);
            }

            let mut playlist: Vec<PathBuf> = Vec::new();

            // 1. Construir la Playlist (Archivos o Directorios completos)
            if target_path.is_file() {
                playlist.push(target_path.to_path_buf());
            } else if target_path.is_dir() {
                println!("🔍 Escaneando directorio en busca de audio...");
                for entry in WalkDir::new(target_path).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                        let extensions = ["mp3", "flac", "m4a", "wma", "wav", "ogg", "aac", "opus", "alac", "aiff"];
                        if extensions.contains(&ext.as_str()) {
                            playlist.push(path.to_path_buf());
                        }
                    }
                }
            }

            if playlist.is_empty() {
                anyhow::bail!("No se encontraron archivos de audio válidos en la ruta.");
            }

            playlist.sort();

            println!("🎵 Playlist lista con {} pistas.", playlist.len());
            
            let page_size = 50;
            let total_pages = (playlist.len() + page_size - 1) / page_size;
            let mut current_page = 0;
            let mut start_index = 0;

            loop {
                println!("\n--- 📄 Página {} / {} ---", current_page + 1, total_pages);
                let start = current_page * page_size;
                let end = ((current_page + 1) * page_size).min(playlist.len());
                
                for (i, track) in playlist[start..end].iter().enumerate() {
                    let global_index = start + i + 1;
                    let filename = track.file_name().unwrap_or_default().to_string_lossy();
                    println!("{:4}. {}", global_index, filename);
                }
                
                println!("\n[n] Siguiente pág | [p] Anterior pág | Escribe un NÚMERO (1-{}) para reproducir | [q] Salir", playlist.len());
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                
                if input == "q" {
                    println!("⏹️ Saliendo...");
                    return Ok(());
                } else if input == "n" {
                    if current_page + 1 < total_pages {
                        current_page += 1;
                    } else {
                        println!("⚠️ Ya estás en la última página.");
                    }
                } else if input == "p" {
                    if current_page > 0 {
                        current_page -= 1;
                    } else {
                        println!("⚠️ Ya estás en la primera página.");
                    }
                } else if let Ok(num) = input.parse::<usize>() {
                    if num >= 1 && num <= playlist.len() {
                        start_index = num - 1;
                        break;
                    } else {
                        println!("⚠️ Número fuera de rango. Selecciona entre 1 y {}.", playlist.len());
                    }
                } else {
                    println!("⚠️ Entrada no reconocida. Ingresa un número, 'n', 'p' o 'q'.");
                }
            }

            println!("Teclas de control: [p] Pausar/Reanudar | [n] Siguiente | [q] Salir");

            // 2. Inicializar el motor de audio alsa
            let (_stream, stream_handle) = OutputStream::try_default()
                .map_err(|e| anyhow::anyhow!("Error al abrir el dispositivo de audio: {}", e))?;
            
            let sink = Sink::try_new(&stream_handle)?;

            // 3. Iterar por cada canción en la Playlist
            for (index, file_path) in playlist.iter().enumerate().skip(start_index) {
                println!("\n▶️ Reproduciendo [{}/{}]: {}", 
                    index + 1, 
                    playlist.len(), 
                    file_path.file_name().unwrap_or_default().to_string_lossy()
                );

                let file = BufReader::new(File::open(file_path)?);
                let source = match Decoder::new(file) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("⚠️ Saltando archivo corrupto/no soportado: {}", e);
                        continue;
                    }
                };

                sink.append(source);
                sink.play();

                // 4. Bucle TUI Interactivo (crossterm) en lugar de un Sleep gigante
                let mut skip_track = false;
                let mut should_quit = false;

                while !sink.empty() {
                    // Poll con un pequeño timeout para no quemar CPU
                    if event::poll(Duration::from_millis(150))? {
                        // Procesamos solo eventos de presionar tecla
                        if let Event::Key(key_event) = event::read()? {
                            if key_event.kind == KeyEventKind::Press {
                                match key_event.code {
                                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                                        println!("⏹️ Saliendo del reproductor...");
                                        should_quit = true;
                                        break; // Salimos del while de crossterm
                                    }
                                    KeyCode::Char('p') | KeyCode::Char('P') => {
                                        if sink.is_paused() {
                                            println!("▶️ Reanudando...");
                                            sink.play();
                                        } else {
                                            println!("⏸️ Pausado");
                                            sink.pause();
                                        }
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') => {
                                        println!("⏭️ Saltando a siguiente pista...");
                                        skip_track = true;
                                        break; // Salimos del while actual
                                    }
                                    _ => {} // Ignorar otras teclas
                                }
                            }
                        }
                    }
                }

                if should_quit {
                    sink.stop(); // Detiene el audio por completo
                    break;       // Rompe el `for` de la playlist
                }

                if skip_track {
                    // Detiene exclusivamente la pista actual y pasa al siguiente iterador del for
                    sink.stop();
                    // Limpiamos el buffer del sink para la siguiente iteración
                    sink.clear();
                }
            }

            println!("🏁 Lista de reproducción finalizada.");
        }
        Ok(())
    }
}
