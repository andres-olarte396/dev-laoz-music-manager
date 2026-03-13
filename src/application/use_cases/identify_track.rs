use std::fs::File;
use std::path::Path;
use reqwest::Client;
use serde_json::Value;

use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;

use rusty_chromaprint::{Configuration, Fingerprinter, FingerprintCompressor};
use data_encoding::BASE64URL_NOPAD;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, TagExt, TagType, ItemKey, TagItem};
use lofty::probe::Probe;
use lofty::config::WriteOptions;

use crate::domain::ports::track_repository::TrackRepository;
use crate::domain::ports::metadata_extractor::MetadataExtractor;
use crate::domain::entities::track::Track;
use crate::domain::entities::config::IdentifyConfig;

pub struct IdentifyTrackUseCase<R: TrackRepository + Clone + Send + Sync + 'static> {
    pub repository: R,
    pub config: IdentifyConfig,
}

impl<R: TrackRepository + Clone + Send + Sync + 'static> IdentifyTrackUseCase<R> {
    pub fn new(repository: R, config: IdentifyConfig) -> Self {
        Self { repository, config }
    }

    pub async fn execute(&self, path: &str, save: bool) -> anyhow::Result<()> {
        let p = Path::new(path);
        if !p.exists() {
            anyhow::bail!("La ruta {} no existe.", path);
        }

        if p.is_dir() {
            println!("📁 Procesando directorio: {}", path);
            let path_string = path.to_string();
            let repo = self.repository.clone();
            let task_config = self.config.clone();
            
            // Reconstruimos UseCase para enviarlo al subproceso de tokio
            let use_case = IdentifyTrackUseCase::new(repo, task_config);
            
            let handle = tokio::spawn(async move {
                println!("🚀 Inicializando el procesamiento para: {}", path_string);
                if let Err(e) = use_case.process_directory(&path_string, save).await {
                    eprintln!("Error procesando directorio: {}", e);
                }
            });
            
            println!("✅ Tarea iniciada para consultar AcoustID. Esperando a que termine...");
            if let Err(e) = handle.await {
                eprintln!("Error en la tarea de procesamiento: {}", e);
            }
        } else {
            self.process_file(p, save).await?;
        }

        Ok(())
    }

    async fn process_directory(&self, dir_path: &str, save: bool) -> anyhow::Result<()> {
        let root = Path::new(dir_path);
        let extensions = ["mp3", "flac", "m4a", "wma", "wav", "ogg", "aac", "opus", "alac", "aiff"];

        // Recopilar archivos primero para mostrar el total
        let files: Vec<_> = walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file() && {
                    let ext = e.path().extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    extensions.contains(&ext.as_str())
                }
            })
            .collect();

        let total = files.len();
        let mut processed = 0usize;
        let mut identified = 0usize;
        let mut saved = 0usize;
        let mut not_found = 0usize;
        let mut errors = 0usize;

        println!("📊 Total de archivos de audio encontrados: {}", total);

        for entry in &files {
            processed += 1;
            let current_path = entry.path();
            println!(
                "\n[{}/{}] {}",
                processed, total,
                current_path.file_name().unwrap_or_default().to_string_lossy()
            );

            println!("   🔍 Calculando huella acústica...");
            let fp_result = self.calculate_fingerprint(current_path);
            
            match fp_result {
                Ok((duration, fingerprint)) => {
                    println!("   🌐 Consultando AcoustID...");
                    match self.lookup_acoustid(duration, &fingerprint).await {
                        Ok(Some((title, artist, album))) => {
                            identified += 1;
                            println!("   ✅ Encontrado: {} - {}", artist, title);
                            if save {
                                if let Err(e) = self.save_metadata_to_file(current_path, &title, &artist, &album) {
                                    eprintln!("   ❌ Error guardando metadatos: {}", e);
                                    errors += 1;
                                } else {
                                    saved += 1;
                                    println!("   💾 Metadatos guardados.");
                                }
                            }
                        }
                        Ok(None) => {
                            not_found += 1;
                            println!("   ⚠️ No encontrado en AcoustID.");
                        }
                        Err(e) => {
                            errors += 1;
                            eprintln!("   ❌ Error AcoustID: {}", e);
                        }
                    }
                }
                Err(e) => {
                    errors += 1;
                    eprintln!("   ❌ Error fingerprint: {}", e);
                }
            }
        }

        println!("\n{}", "─".repeat(60));
        println!("🎉 Identificación finalizada en: {}", dir_path);
        println!("   📁 Archivos procesados : {}/{}", processed, total);
        println!("   ✅ Identificados        : {}", identified);
        if save {
            println!("   💾 Guardados            : {}", saved);
        }
        println!("   ⚠️  No encontrados       : {}", not_found);
        if errors > 0 {
            println!("   ❌ Errores              : {}", errors);
        }
        println!("{}", "─".repeat(60));
        Ok(())
    }

    async fn process_file(&self, file_path: &Path, save: bool) -> anyhow::Result<()> {
        println!("🔍 Calculando huella acústica para {}...", file_path.display());
        let (duration_secs, fingerprint) = self.calculate_fingerprint(file_path)?;
        
        println!("🌐 Buscando metadatos en AcoustID...");
        if let Some((title, artist, album)) = self.lookup_acoustid(duration_secs, &fingerprint).await? {
            println!("✅ Metadatos Encontrados para {}:", file_path.display());
            println!("   🎵 Título: {}", title);
            println!("   👤 Artista: {}", artist);
            println!("   💿 Álbum: {}", album);

            if save {
                println!("💾 Guardando metadatos en el archivo original...");
                if let Err(e) = self.save_metadata_to_file(file_path, &title, &artist, &album) {
                    eprintln!("   ❌ Error guardando metadatos: {}", e);
                } else {
                    println!("🔄 Actualizando base de datos...");
                    let path_str = file_path.to_string_lossy().to_string();
                    let mut track = crate::infrastructure::filesystem::symphonia_extractor::SymphoniaExtractor::new()
                        .extract_metadata(&path_str);
                    track.title = Some(title);
                    track.artist = Some(artist);
                    track.album = Some(album);
                    self.repository.save(&track).await?;
                }
            }
        }
        Ok(())
    }

    fn calculate_fingerprint(&self, path: &Path) -> anyhow::Result<(u64, String)> {
        // Try fpcalc (official Chromaprint tool) first — produces the exact format AcoustID expects
        if let Ok((duration, fingerprint)) = self.calculate_fingerprint_fpcalc(path) {
            return Ok((duration, fingerprint));
        }
        // Fallback to rusty-chromaprint
        self.calculate_fingerprint_rust(path)
    }

    fn calculate_fingerprint_fpcalc(&self, path: &Path) -> anyhow::Result<(u64, String)> {
        // Prioridad: config.toml > FPCALC_PATH env > PATH > LOCALAPPDATA
        let mut fpcalc_candidates = vec![
            self.config.fpcalc_path.clone().unwrap_or_default(),
            std::env::var("FPCALC_PATH").unwrap_or_default(),
            "fpcalc".to_string(),
            r"C:\Program Files\fpcalc\fpcalc.exe".to_string(),
            format!("{}\\fpcalc.exe", std::env::var("LOCALAPPDATA").unwrap_or_default()),
        ];
        fpcalc_candidates.dedup();

        for candidate in &fpcalc_candidates {
            if candidate.is_empty() { continue; }
            let output = std::process::Command::new(candidate)
                .arg(path)
                .arg("-length").arg("120")
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    let mut duration = 0u64;
                    let mut fingerprint = String::new();
                    for line in text.lines() {
                        if let Some(v) = line.strip_prefix("DURATION=") {
                            duration = v.trim().parse().unwrap_or(0);
                        } else if let Some(v) = line.strip_prefix("FINGERPRINT=") {
                            fingerprint = v.trim().to_string();
                        }
                    }
                    if !fingerprint.is_empty() && duration > 0 {
                        return Ok((duration, fingerprint));
                    }
                }
            }
        }
        anyhow::bail!("fpcalc no encontrado o falló");
    }

    fn calculate_fingerprint_rust(&self, path: &Path) -> anyhow::Result<(u64, String)> {
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }
        
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
        
        let mut format = probed.format;
        let track = format.default_track().ok_or_else(|| anyhow::anyhow!("No default track found"))?;
        let track_id = track.id;
        
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())?;
        
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
        
        let mut printer = Fingerprinter::new(&Configuration::preset_test1());
        printer.start(sample_rate, channels as u32)?;
        
        let mut sample_buf = None;
        let mut decoded_seconds = 0.0f64;
        
        while decoded_seconds < 120.0 {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(_) => break,
            };
            if packet.track_id() != track_id { continue; }
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    if sample_buf.is_none() {
                        let spec = *decoded.spec();
                        let duration = decoded.capacity() as u64;
                        sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                    }
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(decoded);
                        let samples = buf.samples();
                        printer.consume(samples);
                        decoded_seconds += (samples.len() as f64 / channels as f64) / sample_rate as f64;
                    }
                }
                Err(_) => continue,
            }
        }
        
        printer.finish();
        let fp = printer.fingerprint();
        
        // Chromaprint compressed format (best effort match with fpcalc)
        let config = Configuration::preset_test1();
        let compressor = FingerprintCompressor::from(&config);
        let compressed = compressor.compress(fp);
        let b64 = BASE64URL_NOPAD.encode(&compressed);
        
        Ok((decoded_seconds as u64, b64))
    }

    async fn lookup_acoustid(&self, duration: u64, fingerprint: &str) -> anyhow::Result<Option<(String, String, String)>> {
        let client = Client::new();
        let mut retries = 3;
        let mut json: Value = Value::Null;

        let key = self.config.acoustid_key.as_str();
        let url = "https://api.acoustid.org/v2/lookup";
        let duration_str = duration.to_string();
        let form_data = [
            ("client", key),
            ("meta", "recordings releasegroups compress"),
            ("duration", duration_str.as_str()),
            ("fingerprint", fingerprint),
        ];

        while retries > 0 {
            match client.post(url).form(&form_data).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        json = res.json().await.unwrap_or(Value::Null);
                        break;
                    } else if res.status().as_u16() == 429 {
                        println!("   ⏳ AcoustID Rate Limit excedido, esperando 2 segundos...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        retries -= 1;
                        continue;
                    } else {
                        println!("   ❌ Error consultando AcoustID API: {}", res.status());
                        if let Ok(text) = res.text().await {
                             println!("   Detalle: {}", text);
                        }
                        return Ok(None);
                    }
                }
                Err(e) => {
                     println!("   ❌ Error de red con AcoustID API: {}. Reintentando...", e);
                     tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                     retries -= 1;
                }
            }
        }

        if json == Value::Null {
             return Ok(None);
        }
        
        if json["status"].as_str() != Some("ok") {
            println!("   ⚠️ AcoustID API retornó un error o límite excedido.");
            return Ok(None);
        }

        let results_opt = json["results"].as_array();
        let results = match results_opt {
            Some(r) => r,
            None => {
                println!("   ⚠️ Sin formato de resultados válido.");
                return Ok(None);
            }
        };

        if results.is_empty() {
            println!("   ⚠️ No se encontraron coincidencias para esta huella acústica.");
            return Ok(None);
        }

        let best_match = &results[0];
        let recordings_opt = best_match["recordings"].as_array();
        let recordings = match recordings_opt {
            Some(r) => r,
            None => {
                println!("   ⚠️ Sin grabaciones.");
                return Ok(None);
            }
        };

        if recordings.is_empty() {
            println!("   ⚠️ Grabaciones vacías.");
            return Ok(None);
        }

        let recording = &recordings[0];
        let title = recording["title"].as_str().unwrap_or("Desconocido").to_string();
        
        let artists = recording["artists"].as_array();
        let artist = if let Some(a) = artists {
            if !a.is_empty() {
                a[0]["name"].as_str().unwrap_or("Desconocido").to_string()
            } else {
                "Desconocido".to_string()
            }
        } else {
            "Desconocido".to_string()
        };

        let releasegroups = recording["releasegroups"].as_array();
        let album = if let Some(rg) = releasegroups {
            if !rg.is_empty() {
                rg[0]["title"].as_str().unwrap_or("Desconocido").to_string()
            } else {
                "Desconocido".to_string()
            }
        } else {
            "Desconocido".to_string()
        };

        Ok(Some((title, artist, album)))
    }

    fn save_metadata_to_file(&self, path: &Path, title: &str, artist: &str, album: &str) -> anyhow::Result<()> {
        let mut tagged_file = Probe::open(path)?.read()?;
        
        let tag_type = tagged_file.primary_tag_type();
        
        let tag = match tagged_file.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => {
                if let Some(first_tag) = tagged_file.first_tag_mut() {
                    first_tag
                } else {
                    tagged_file.insert_tag(lofty::tag::Tag::new(tag_type));
                    tagged_file.primary_tag_mut().unwrap()
                }
            }
        };

        tag.insert_text(ItemKey::TrackTitle, title.to_string());
        tag.insert_text(ItemKey::TrackArtist, artist.to_string());
        tag.insert_text(ItemKey::AlbumTitle, album.to_string());

        tag.save_to_path(path, WriteOptions::new())?;

        Ok(())
    }
}
