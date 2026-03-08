use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "music-manager")]
#[command(author = "Senior Coder")]
#[command(version = "0.1.0")]
#[command(about = "Gestor Masivo de Bibliotecas Musicales", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Escanea un directorio en busca de archivos de audio
    Scan {
        path: String,
        
        /// Usar escaneo profundo (calcula hashes)
        #[arg(long)]
        deep: bool,

        /// Mantiene un daemon para escuchar cambios (No implementado en v1)
        #[arg(long)]
        watch: bool,
    },
    
    /// Muestra un reporte del estado de la biblioteca
    Status,

    /// Lista las pistas usando paginación Keyset
    List {
        #[arg(long, default_value_t = 50)]
        limit: u32,

        #[arg(long)]
        after: Option<String>,
    },

    /// Analiza la BD buscando inconsistencias
    Doctor,

    /// Mueve o copia la biblioteca a un nuevo destino (Streaming + Hashing)
    Migrate {
        destino: String,

        #[arg(long, default_value = "copy")]
        mode: String,

        #[arg(long)]
        concurrent: Option<usize>,

        #[arg(long)]
        dry_run: bool,
    },

    /// Reproduce pistas de audio en la terminal
    Play {
        criterio: Option<String>,

        #[arg(long)]
        id: Option<String>,

        #[arg(long)]
        artist: Option<String>,

        #[arg(long)]
        shuffle: bool,

        #[arg(long)]
        daemon: bool,
    },

    /// Adelanta o retrocede la reproducción actual (Daemon)
    Seek {
        segundos: String,
    },

    /// Pausa la reproducción actual
    Pause,

    /// Reanuda la reproducción actual
    Resume,

    /// Salta a la siguiente canción
    Next,

    /// Retrocede a la canción anterior
    Prev,

    /// Inicia la Interfaz Gráfica Avanzada en Terminal (Búsqueda + Reproducción simultánea)
    Tui {
        /// Directorio inicial para explorar (opcional)
        path: Option<String>,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
