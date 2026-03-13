use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppConfig {
    pub archive_org: ArchiveOrgConfig,
    pub library: LibraryConfig,
    pub network: NetworkConfig,
    pub identify: IdentifyConfig,
    pub rename: RenameConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArchiveOrgConfig {
    pub advanced_search_url: String,
    pub download_base_url: String,
    pub max_results: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LibraryConfig {
    pub download_path: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NetworkConfig {
    pub timeout_seconds: u64,
    pub max_concurrent_downloads: u32,
}

/// Configuración para el comando `identify`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IdentifyConfig {
    /// API Key de AcoustID (obtener en https://acoustid.org/new-application)
    pub acoustid_key: String,
    /// Ruta al binario fpcalc (Chromaprint). Si está vacío se busca en PATH y LOCALAPPDATA.
    pub fpcalc_path: Option<String>,
}

/// Configuración para el comando `rename`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RenameConfig {
    /// Versión forzada para el renombrado (vacío = auto-detectar).
    /// Valores: acustica, envivo, remix, cover, instrumental, radio, extended, demo, remaster
    pub default_version: Option<String>,
    /// Si es true, deshabilita la detección automática de versión desde el título/nombre de archivo
    pub auto_detect_version: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            archive_org: ArchiveOrgConfig {
                advanced_search_url: "https://archive.org/advancedsearch.php".to_string(),
                download_base_url: "https://archive.org/download".to_string(),
                max_results: 50,
            },
            library: LibraryConfig {
                download_path: None,
            },
            network: NetworkConfig {
                timeout_seconds: 30,
                max_concurrent_downloads: 3,
            },
            identify: IdentifyConfig {
                acoustid_key: String::new(),
                fpcalc_path: None,
            },
            rename: RenameConfig {
                default_version: None,
                auto_detect_version: true,
            },
        }
    }
}
