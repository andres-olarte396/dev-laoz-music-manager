# 🎵 L4OZ Music Manager (TUI)

**dev-laoz-music-manager** es una aplicación CLI y TUI (Text User Interface) desarrollada en **Rust** diseñada para la indexación ultra rápida de bibliotecas musicales masivas (hasta 1,000,000+ de canciones), búsqueda instantánea, reproducción de audio nativa, **identificación acústica automática** y **renombrado estándar** de archivos completamente basados en consola.

---

## 🏗️ Arquitectura del Software

El proyecto fue diseñado utilizando estrictos principios de **Clean Architecture** (Arquitectura Limpia) y **Dominio Dirigido por Diseño (DDD)**.

El código fuente en la carpeta `src` está estructurado en 3 capas fundamentales:

1. **`domain/` (Dominio)**: Contiene la lógica pura de la aplicación.
   - `entities/`: Las estructuras base, como `Track` (Canción) y `AppConfig` (Configuración).
   - `ports/`: Las interfaces o traítos (traits) abstractos, como `TrackRepository`, lo que permite desenchufar motores de base de datos a futuro.

2. **`application/` (Casos de Uso)**: Contiene la lógica de negocio y las funcionalidades del sistema (independiente de la UI).
   - `tui.rs`: Controla la máquina de estados de la interfaz gráfica y los atajos de teclado.
   - `playback.rs`: Administra el hilo de audio en segundo plano (MPSC Channels).
   - `scan_library.rs`: Controla el proceso iterativo asíncrono para leer grandes directorios.
   - `identify_track.rs`: Identifica metadatos reales usando huella acústica Chromaprint + AcoustID.
   - `rename_track.rs`: Renombra archivos al formato estándar basado en tags existentes.

3. **`infrastructure/` (Infraestructura)**: Contiene todo el código que interactúa con el mundo exterior.
   - `cli/`: Parsea los comandos del sistema mediante `clap`.
   - `database/`: Concreción de SQLite usando `sqlx`.
   - `filesystem/`: Implementación de extractores como `Symphonia`, para leer la metadata ID3.
   - `config_loader.rs`: Carga y serializa `config.toml` usando `toml` + `serde`.

---

## 🛠️ Tecnologías Principales (Crates)

- **[Ratatui](https://github.com/ratatui-org/ratatui) & [Crossterm](https://github.com/crossterm-rs/crossterm)**: Para el dibujado de paneles gráficos en la terminal.
- **[Rodio](https://github.com/RustAudio/rodio) & [Symphonia](https://github.com/pdeljanov/Symphonia)**: Motores y decodificadores de audio.
- **[SQLx](https://github.com/launchbadge/sqlx)**: Toolkit asíncrono para SQLite empotrada.
- **[Tokio](https://github.com/tokio-rs/tokio)**: El _runtime_ asíncrono de Rust líder de la industria.
- **[Lofty](https://github.com/Serial-ATA/lofty-rs)**: Lectura y escritura de tags ID3, Vorbis, MP4 de audio.
- **[rusty-chromaprint](https://crates.io/crates/rusty-chromaprint)**: Generación de huellas acústicas en Rust puro (fallback sin fpcalc).
- **[Reqwest](https://crates.io/crates/reqwest)**: Cliente HTTP asíncrono para la API de AcoustID.
- **[Regex](https://crates.io/crates/regex)**: Detección de versiones (Acústica, En Vivo, Remix) en títulos.

---

## 🚀 Instalación y Despliegue

La documentación de instalación y generación de binarios se ha separado por sistema operativo:

- 🐧 **[Guía de Instalación para Linux](docs/install-linux.md)**
- 🪟 **[Guía de Compilación Cruzada para Windows](docs/install-windows.md)**

---

## ⚙️ Configuración (`config.toml`)

El archivo `config.toml` en la raíz del proyecto centraliza todas las opciones configurables:

```toml
[identify]
acoustid_key = "TU_KEY"        # Obtener gratis en https://acoustid.org/new-application
# fpcalc_path = "C:\\fpcalc.exe"  # Ruta a fpcalc (Chromaprint). Opcional.

[rename]
auto_detect_version = true     # Detecta Acústica/En Vivo/Remix automáticamente
# default_version = ""         # Fuerza una versión para todos los archivos
```

---

## 📖 Guía de Uso

### Flujo recomendado para organizar tu biblioteca

```bash
# 1. Identificar y corregir metadatos usando huella acústica
music-manager identify "C:\Users\Music" --save

# 2. Renombrar archivos al formato estándar basado en los tags corregidos
music-manager rename "C:\Users\Music"

# 3. Indexar la biblioteca en SQLite para búsqueda rápida
music-manager scan "C:\Users\Music"

# 4. Explorar y reproducir en la TUI interactiva
music-manager tui "C:\Users\Music"
```

Para todos los comandos y opciones completas consulta:

- 🕹️ **[Guía de Uso Rápido (Interactiva)](docs/usage-guide.md)**
- 📐 **[Diseño de la CLI](docs/cli-design.md)**
