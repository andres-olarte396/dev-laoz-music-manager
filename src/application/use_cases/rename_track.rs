use std::path::{Path, PathBuf};
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use crate::domain::entities::config::RenameConfig;

/// Versiones especiales detectadas en el nombre o tags de la pista
#[derive(Debug, Clone, PartialEq)]
pub enum VersionTag {
    Acoustic,
    Live,
    Remix,
    Cover,
    Instrumental,
    Radio,
    Extended,
    Demo,
    Remaster,
    Custom(String),
}

impl VersionTag {
    /// Detecta una versión a partir de texto libre (nombre de archivo o tag)
    pub fn detect(text: &str) -> Option<Self> {
        let lower = text.to_lowercase();
        // Orden importa: más específico primero
        if lower.contains("acoustic") || lower.contains("acústic") || lower.contains("acustic") {
            Some(Self::Acoustic)
        } else if lower.contains("en vivo") || lower.contains("live") || lower.contains("directo") || lower.contains("concierto") {
            Some(Self::Live)
        } else if lower.contains("remix") || lower.contains("rmx") {
            Some(Self::Remix)
        } else if lower.contains("cover") || lower.contains("versión") && lower.contains("cover") {
            Some(Self::Cover)
        } else if lower.contains("instrumental") {
            Some(Self::Instrumental)
        } else if lower.contains("radio edit") || lower.contains("radio mix") {
            Some(Self::Radio)
        } else if lower.contains("extended") || lower.contains("extended mix") {
            Some(Self::Extended)
        } else if lower.contains("demo") {
            Some(Self::Demo)
        } else if lower.contains("remaster") || lower.contains("remastered") {
            Some(Self::Remaster)
        } else {
            None
        }
    }

    /// Texto a mostrar entre paréntesis
    pub fn label(&self) -> &str {
        match self {
            Self::Acoustic   => "Acústica",
            Self::Live       => "En Vivo",
            Self::Remix      => "Remix",
            Self::Cover      => "Cover",
            Self::Instrumental => "Instrumental",
            Self::Radio      => "Radio Edit",
            Self::Extended   => "Extended Mix",
            Self::Demo       => "Demo",
            Self::Remaster   => "Remaster",
            Self::Custom(s)  => s.as_str(),
        }
    }
}

/// Metadatos de una pista extraídos para el renombrado
#[derive(Debug)]
pub struct TrackNamingInfo {
    pub artist: String,
    pub title: String,
    pub track_number: Option<u32>,
    pub version: Option<VersionTag>,
}

/// Resultado de un renombrado (real o simulado)
#[derive(Debug)]
pub struct RenameResult {
    pub original: PathBuf,
    pub new_name: PathBuf,
    pub renamed: bool,
    pub skipped: bool,
    pub reason: Option<String>,
}

pub struct RenameTrackUseCase {
    pub config: RenameConfig,
}

impl RenameTrackUseCase {
    pub fn new(config: RenameConfig) -> Self {
        Self { config }
    }

    /// Ejecuta el renombrado sobre un archivo o directorio completo.
    /// `dry_run = true` → solo muestra los cambios sin aplicarlos.
    pub fn execute(&self, path: &str, dry_run: bool) -> anyhow::Result<Vec<RenameResult>> {
        let p = Path::new(path);
        if !p.exists() {
            anyhow::bail!("La ruta '{}' no existe.", path);
        }

        let mut results = Vec::new();

        if p.is_dir() {
            println!("📁 Procesando directorio: {}", path);
            let audio_exts = ["mp3","flac","m4a","wma","wav","ogg","aac","opus","alac","aiff"];
            let files: Vec<_> = walkdir::WalkDir::new(p)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().is_file() && {
                        let ext = e.path().extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                        audio_exts.contains(&ext.as_str())
                    }
                })
                .collect();

            let total = files.len();
            println!("📊 Total de archivos encontrados: {}", total);

            for (i, entry) in files.iter().enumerate() {
                print!("[{}/{}] ", i + 1, total);
                let result = self.process_file(entry.path(), dry_run);
                results.push(result);
            }

            let renamed  = results.iter().filter(|r| r.renamed).count();
            let previews = results.iter().filter(|r| !r.renamed && !r.skipped).count();
            let skipped  = results.iter().filter(|r| r.skipped).count();
            let errors   = results.iter().filter(|r| r.skipped && r.reason.as_deref() != Some("nombre ya correcto")).count();

            println!("\n{}", "─".repeat(60));
            println!("🎉 Renombrado finalizado en: {}", path);
            println!("   📁 Total archivos    : {}", total);
            if dry_run {
                println!("   🔍 Cambios posibles  : {}", previews);
                println!("   ⏩ Sin cambios       : {}", skipped);
            } else {
                println!("   ✅ Renombrados       : {}", renamed);
                println!("   ⏩ Sin cambios       : {}", total - renamed - errors);
                if errors > 0 {
                    println!("   ❌ Errores           : {}", errors);
                }
            }
            println!("{}", "─".repeat(60));
        } else {
            let result = self.process_file(p, dry_run);
            results.push(result);
        }

        Ok(results)
    }

    fn process_file(&self, path: &Path, dry_run: bool) -> RenameResult {
        match self.read_naming_info(path) {
            Ok(info) => {
                let ext = path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("mp3");
                let new_filename = build_filename(&info, ext);
                let new_path = path.parent()
                    .unwrap_or(Path::new("."))
                    .join(&new_filename);

                // Si el nombre no cambiaría, saltamos
                if path.file_name().map(|f| f.to_string_lossy().to_string()).as_deref() == Some(&new_filename) {
                    println!("⏩ Sin cambios:   {}", path.file_name().unwrap_or_default().to_string_lossy());
                    return RenameResult {
                        original: path.to_path_buf(),
                        new_name: new_path,
                        renamed: false,
                        skipped: true,
                        reason: Some("nombre ya correcto".into()),
                    };
                }

                if dry_run {
                    println!(
                        "🔍 [Simulación]  {} →\n                  {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        new_filename
                    );
                    RenameResult {
                        original: path.to_path_buf(),
                        new_name: new_path,
                        renamed: false,
                        skipped: false,
                        reason: None,
                    }
                } else {
                    // Manejo de colisión: agregar sufijo numérico si el destino existe
                    let final_path = resolve_collision(&new_path, ext);
                    match std::fs::rename(path, &final_path) {
                        Ok(_) => {
                            println!(
                                "✅ Renombrado:   {} →\n                  {}",
                                path.file_name().unwrap_or_default().to_string_lossy(),
                                final_path.file_name().unwrap_or_default().to_string_lossy()
                            );
                            RenameResult {
                                original: path.to_path_buf(),
                                new_name: final_path,
                                renamed: true,
                                skipped: false,
                                reason: None,
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Error renombrando {}: {}", path.display(), e);
                            RenameResult {
                                original: path.to_path_buf(),
                                new_name: new_path,
                                renamed: false,
                                skipped: true,
                                reason: Some(e.to_string()),
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ No se pudo leer metadatos de {}: {}", path.display(), e);
                RenameResult {
                    original: path.to_path_buf(),
                    new_name: path.to_path_buf(),
                    renamed: false,
                    skipped: true,
                    reason: Some(e.to_string()),
                }
            }
        }
    }

    /// Lee los tags del archivo de audio para construir la info de renombrado.
    fn read_naming_info(&self, path: &Path) -> anyhow::Result<TrackNamingInfo> {
        let tagged = Probe::open(path)?.read()?;

        let tag = tagged.primary_tag()
            .or_else(|| tagged.first_tag())
            .ok_or_else(|| anyhow::anyhow!("No se encontraron tags en el archivo"))?;

        let raw_artist = tag.artist().map(|s| s.to_string()).unwrap_or_default();
        let artist = if raw_artist.trim().chars().all(|c| c.is_ascii_digit()) || raw_artist.trim().is_empty() {
            "Desconocido".to_string()
        } else {
            raw_artist
        };

        let raw_title = tag.title()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Sin Título".to_string())
            });
        let title = strip_track_prefix(&raw_title);

        let track_number = tag.track();

        // ── Detectar versión: primero config.toml, después auto-detect ─────────
        let version = if !self.config.auto_detect_version {
            // Auto-detect deshabilitado
            self.config.default_version.as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| VersionTag::Custom(s.to_string()))
        } else if let Some(forced) = self.config.default_version.as_deref().filter(|s| !s.is_empty()) {
            // Versión forzada desde config
            Some(VersionTag::Custom(forced.to_string()))
        } else {
            // Auto-detect desde título y nombre de archivo
            let filename_stem = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            VersionTag::detect(&title).or_else(|| VersionTag::detect(&filename_stem))
        };

        Ok(TrackNamingInfo {
            artist: sanitize_filename(&artist),
            title: sanitize_filename(&title),
            track_number,
            version,
        })
    }
}

/// Construye el nombre del archivo según el formato estándar:
/// `{Artista} - {NúmeroPista} {Título} ({Versión}).{ext}`
/// El número de pista y la versión son opcionales.
fn build_filename(info: &TrackNamingInfo, ext: &str) -> String {
    let clean_title = strip_version_from_title(&info.title);

    let track_part = match info.track_number {
        Some(n) if n > 0 => format!("{:02} ", n),
        _ => String::new(),
    };

    let version_part = match &info.version {
        Some(v) => format!(" ({})", v.label()),
        None => String::new(),
    };

    format!("{} - {}{}{}.{}", info.artist, track_part, clean_title, version_part, ext)
}

/// Elimina prefijos numéricos de pista que pueden aparecer al inicio del título.
/// Ejemplos: "14. Título" → "Título", "01 - Título" → "Título", "3) Título" → "Título"
fn strip_track_prefix(title: &str) -> String {
    // Patrones: "NN. ", "NN - ", "NN) ", "NN " al inicio
    let re = regex::Regex::new(r"^\d{1,3}[\.\-\)]\s*").unwrap();
    re.replace(title, "").trim().to_string()
}


/// Elimina texto de versión redundante del final del título
/// ej: "Song Title (Acoustic)" → "Song Title"
fn strip_version_from_title(title: &str) -> String {
    // Eliminar contenido entre paréntesis al final si contiene palabras de versión
    let re_parenthesis = regex::Regex::new(
        r"\s*\((acoustic|acústic[ao]|acustic[ao]|en vivo|live|directo|remix|rmx|cover|instrumental|radio edit|radio mix|extended|demo|remasteriz|remaster)\w*\)\s*$"
    );
    if let Ok(re) = re_parenthesis {
        re.replace(title, "").trim().to_string()
    } else {
        title.to_string()
    }
}

/// Elimina caracteres inválidos para nombres de archivo en Windows/Linux
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .trim()
        .to_string()
}

/// Si ya existe un archivo con el mismo nombre, añade sufijo numerico (2), (3)...
fn resolve_collision(path: &Path, ext: &str) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new("."));
    let mut i = 2u32;
    loop {
        let candidate = parent.join(format!("{} ({}).{}", stem, i, ext));
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}
