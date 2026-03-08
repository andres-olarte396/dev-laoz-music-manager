# 🎵 L4OZ Music Manager (TUI)

**dev-laoz-music-manager** es una aplicación CLI y TUI (Text User Interface) desarrollada en **Rust** diseñada para la indexación ultra rápida de bibliotecas musicales masivas (hasta 1,000,000+ de canciones), búsqueda instantánea y reproducción de audio nativa completamente basada en consola, manteniéndose liviana, asíncrona y no bloqueante.

---

## 🏗️ Arquitectura del Software

El proyecto fue diseñado utilizando estrictos principios de **Clean Architecture** (Arquitectura Limpia) y **Dominio Dirigido por Diseño (DDD)**. 

El código fuente en la carpeta `src` está estructurado en 3 capas fundamentales:

1. **`domain/` (Dominio)**: Contiene la lógica pura de la aplicación.
   * `entities/`: Las estructuras base, como `Track` (Canción).
   * `ports/`: Las interfaces o traítos (traits) abstractos, como `TrackRepository` (Repositorio de Canciones), lo que permite desenchufar motores de base de datos a futuro.

2. **`application/` (Casos de Uso)**: Contiene la lógica de negocio y las funcionalidades del sistema (independiente de la UI).
   * `tui.rs`: Controla la máquina de estados de la interfaz gráfica y los atajos de teclado.
   * `playback.rs`: Administra el hilo de audio en segundo plano (MPSC Channels).
   * `scan_library.rs`: Controla el proceso iterativo asíncrono para leer grandes directorios buscando archivos musicales.

3. **`infrastructure/` (Infraestructura)**: Contiene todo el código que interactúa con el mundo exterior.
   * `cli/`: Parsea los comandos del sistema mediante `clap`.
   * `database/`: Concreción de SQLite usando `sqlx`. Se encarga de transformar los `Track` del dominio a consultas SQL crudas.
   * `filesystem/`: Implementación de extractores como `Symphonia`, para leer la metadata ID3 de los MP3 sin cargarlos por completo a RAM.

---

## 🛠️ Tecnologías Principales (Crates)

El proyecto depende de ecosistemas robustos del mundo de Rust:

* **[Ratatui](https://github.com/ratatui-org/ratatui) & [Crossterm](https://github.com/crossterm-rs/crossterm)**: Para el dibujado de paneles gráficos en la terminal y captura de teclas en modo "Raw".
* **[Rodio](https://github.com/RustAudio/rodio) & [Symphonia](https://github.com/pdeljanov/Symphonia)**: Motores y decodificadores de audio para reproducir la música de manera eficiente por hardware.
* **[SQLx](https://github.com/launchbadge/sqlx)**: Un toolkit asíncrono para la conexión a bases de datos SQLite empotrada de manera puramente asíncrona (`tokio`).
* **[Tokio](https://github.com/tokio-rs/tokio)**: El *runtime* asíncrono de Rust líder de la industria.
* **[MPSC Channels]**: Utilizados para comunicar los paneles asíncronos de la UI con el hilo "bloqueante" del hardware de sonido.

---

## 🚀 Instalación y Despliegue

La documentación de instalación y generación de binarios se ha separado por sistema operativo para mayor claridad:

* 🐧 **[Guía de Instalación para Linux](docs/install-linux.md)**: Instrucciones para compilar y agregar el comando al PATH global en sistemas basados en Linux.
* 🪟 **[Guía de Compilación Cruzada para Windows](docs/install-windows.md)**: Instrucciones para generar el archivo `.exe` desde Linux usando `mingw-w64` y llevarlo a Microsoft Windows.

---

## 📖 Guía de Uso

Para aprender a escanear tu biblioteca musical a la base de datos SQLite y descubrir todos los atajos de teclado de la Interfaz TUI (Navegador, Búsqueda y Resultados), dirígete a la documentación principal:

* 🕹️ **[Guía de Uso Rápido (Interactiva)](docs/usage-guide.md)**

