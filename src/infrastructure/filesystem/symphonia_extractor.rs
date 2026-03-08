use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, MetadataRevision, StandardTagKey, Value};
use symphonia::core::probe::Hint;
use std::fs::File;
use std::path::Path;

use crate::domain::entities::track::Track;
use crate::domain::ports::metadata_extractor::MetadataExtractor;

pub struct SymphoniaExtractor;

impl SymphoniaExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl MetadataExtractor for SymphoniaExtractor {
    fn extract_metadata(&self, file_path_str: &str) -> Track {
        let path = Path::new(file_path_str);
        
        let filename_fallback = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        // 1. Abrir archivo
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Track::fallback(file_path_str.to_string(), filename_fallback),
        };

        // 2. Configurar el Probe de Symphonia
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        
        // Autodetectar extension
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }

        // 3. Probar el archivo buscando demuxers y metadata
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();

        let mut probed = match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
            Ok(p) => p,
            Err(_) => return Track::fallback(file_path_str.to_string(), filename_fallback),
        };

        let mut format = probed.format;
        let mut track_data = Track::fallback(file_path_str.to_string(), filename_fallback);

        // EXTRAER DURACIÓN DESDE EL TRACK INFO
        if let Some(track_info) = format.default_track() {
            if let Some(tb) = track_info.codec_params.time_base {
                if let Some(n_frames) = track_info.codec_params.n_frames {
                    let time = tb.calc_time(n_frames);
                    track_data.duration_seconds = Some(time.seconds);
                }
            }
        }

        // EXTRAER TAGS ID3 DEL METADATA REVISION ACTIVO
        if let Some(metadata) = format.metadata().current() {
            extract_tags_from_revision(metadata, &mut track_data);
        } else if let Some(metadata) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
             extract_tags_from_revision(metadata, &mut track_data);
        }

        track_data
    }
}

fn extract_tags_from_revision(revision: &MetadataRevision, track: &mut Track) {
    for tag in revision.tags() {
        if let Some(std_key) = tag.std_key {
            match std_key {
                StandardTagKey::TrackTitle => {
                    if let Value::String(s) = &tag.value {
                        track.title = Some(s.clone());
                    }
                }
                StandardTagKey::Artist => {
                    if let Value::String(s) = &tag.value {
                        track.artist = Some(s.clone());
                    }
                }
                StandardTagKey::AlbumArtist => {
                    if let Value::String(s) = &tag.value {
                        track.album_artist = Some(s.clone());
                    }
                }
                StandardTagKey::Album => {
                    if let Value::String(s) = &tag.value {
                        track.album = Some(s.clone());
                    }
                }
                StandardTagKey::Date => {
                    if let Value::String(s) = &tag.value {
                        // El formato suele ser "YYYY" o "YYYY-MM-DD"
                        let year_str = s.split('-').next().unwrap_or("").trim();
                        if let Ok(y) = year_str.parse::<u32>() {
                            track.year = Some(y);
                        }
                    }
                }
                StandardTagKey::Genre => {
                    if let Value::String(s) = &tag.value {
                        track.genre = Some(s.clone());
                    }
                }
                _ => {} // Ignorar los tags técnicos como ReplayGain por ahora
            }
        }
    }
}
