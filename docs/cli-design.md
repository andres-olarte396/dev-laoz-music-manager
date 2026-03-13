# Interfaz de Línea de Comandos (CLI)

Este documento define la estructura de comandos, argumentos y flujos de interacción para el usuario final del **Music Manager CLI**. Se utiliza un diseño jerárquico anidado, el estándar de facto en herramientas modernas (como `git`, `cargo`, o `docker`).

## Estructura Base

El ejecutable se asume nombrado `music-manager` o su alias corto `mm`.
Todos los comandos siguen el patrón:
`music-manager <VERBO> [OBJETO] [OPCIONES]`

## 1. Gestión de la Biblioteca (Escaneo y Sincronización)

Comandos enfocados en descubrir y registrar archivos de música en la base de datos local SQLite.

### `scan`

Escanea un directorio recursivamente en busca de archivos de audio soportados (ej. `.mp3`, `.flac`) y los añade a la base de datos.

- **Uso:** `music-manager scan <PATH> [OPCIONES]`
- **Opciones:**
  - `--fast`: (Por defecto) Escanea usando los metadatos del sistema de archivos (fecha de modificación, tamaño) para detectar cambios.
  - `--deep`: Lee los _tags_ ID3 de cada archivo y recalcula el _hash_ criptográfico para detectar corrupciones o ediciones invisibles al sistema de archivos.
  - `--watch`: Mantiene un daemon en segundo plano que escucha eventos inotify/fs-events para añadir pistas nuevas al vuelo.
- **Ejemplo:** `music-manager scan /home/user/Music --deep`

### `status`

Muestra un reporte analítico del estado actual de la biblioteca.

- **Uso:** `music-manager status`
- **Salida Esperada:**

  ```text
  🎵 Music Manager Status
  -------------------------
  Total Tracks:     14,502
  Total Size:       124.5 GB
  Corrupted/Missing: 12 tracks (Run `music-manager doctor` to fix)
  Last Scan:        2 hours ago
  ```

## 2. Herramientas de Reparación y Mantenimiento

### `doctor`

Analiza la base de datos contra el disco duro real buscando inconsistencias (rutas rotas, archivos borrados, _hashes_ que no coinciden).

- **Uso:** `music-manager doctor`
- **Salida Interactiva:** Presentará una tabla de los errores encontrados y ofrecerá un prompt interactivo (`[Y/n]`) para eliminarlos de la BD o intentar "re-linkearlos" si se movieron.

## 3. Consultas y Paginación (Lectura)

### `list`

Muestra las pistas añadidas, demostrando el poder de la Paginación Keyset.

- **Uso:** `music-manager list [OPCIONES]`
- **Opciones:**
  - `--limit <N>`: Cantidad de tracks por página (Default: 50).
  - `--after <ULID>`: El cursor para obtener la siguiente página.
  - `--format <json|table>`: Formato de salida (Default: table).
- **Ejemplo de Salida (Table):**

  ```text
  ID                     | Title             | Artist        | Format
  -------------------------------------------------------------------
  01HNGYXZ7A8B9C0D1E2F...| Bohemian Rhapsody | Queen         | FLAC
  01HNGYYA1B2C3D4E5F6G...| Hotel California  | Eagles        | MP3
  ```

## 4. Reproducción de Audio en Terminal

Dado que el motor cuenta con soporte para leer binarios, podemos incluir un reproductor integrado para escuchar las pistas sin salir de la consola, aprovechando librerías puente de audio en Rust (ej: `rodio`).

### `play`

Reproduce una canción, un álbum, o una lista de resultados con controles básicos interactivos en la terminal o como servicio en segundo plano (daemon).

- **Uso:** `music-manager play <CRITERIO> [OPCIONES]`
- **Opciones:**
  - `--id <ULID>`: Reproduce una pista exacta por su ID.
  - `--artist <NOMBRE>`: Reproduce todas las pistas de un artista.
  - `--shuffle`: Reproduce los resultados en orden aleatorio.
  - `--daemon`: Inicia la reproducción en segundo plano y devuelve el control de la terminal al usuario de inmediato.

### Control de Reproducción (Modo Daemon/Background)

Si la reproducción se está ejecutando en segundo plano, o quieres controlarla desde otra pestaña de la terminal, puedes usar estos comandos de control remoto sin interrumpir tu trabajo actual:

- **Canción Anterior / Siguiente:**
  - `music-manager prev`: Regresa a la canción anterior en la lista de reproducción.
  - `music-manager next`: Salta a la siguiente canción en la cola.

- **Adelantar / Retroceder (Seek):**
  - `music-manager seek +<SEGUNDOS>`: Adelanta la canción (ej. `music-manager seek +15` para adelantar 15 segundos).
  - `music-manager seek -<SEGUNDOS>`: Retrocede la canción (ej. `music-manager seek -10` para regresar 10 segundos).

- **Pausar / Reanudar:**
  - `music-manager pause`: Pausa la reproducción actual.
  - `music-manager resume`: Reanuda la reproducción desde donde se pausó.

- **Experiencia de Terminal Interactiva (In-Place):** Si **NO** usas `--daemon`, tu terminal se convertirá temporalmente en un minirreproductor TUI. Allí podrás usar la barra espaciadora para `[Pause/Play]`, flechas `[<- ->]` para adelantar o retroceder la canción en vivo, y las letras `[N/P]` para ir a la siguiente/previa.

## 5. Migración Asíncrona Masiva

El núcleo pesado del sistema: mover/copiar miles de archivos a una nueva estructura de carpetas de forma segura.

### `migrate`

Mueve o copia la biblioteca entera a un nuevo destino (ej. un disco duro externo o un NAS) reestructurando por "Artista/Álbum" y calculando _hashes_ BLAKE3 en streaming.

- **Uso:** `music-manager migrate <DESTINO> [OPCIONES]`
- **Opciones:**
  - `--mode <copy|move>`: Si debe copiar en origen o mover los archivos (Default: copy).
  - `--structure <PATTERN>`: Patrón de las carpetas destino. Ej: `{artist}/{album}/{title}.{ext}`.
  - `--concurrent <N>`: Número de _workers_ asíncronos en el pool (Default: Número de núcleos de CPU lógicos).
  - `--dry-run`: Simula la migración completa, mostrando inconsistencias o sobrescrituras accidentales, pero sin tocar el disco duro destino.
- **Ejemplo Visual (Progreso de la Consola):**

  ```text
  🚀 Migrating 14,502 tracks to /media/usb_drive/Music...
  [===>-------------] 25% (3,625 / 14,502) | 120 MB/s | ETA: 4m 12s
  ✅ Successfully processed 01HNG... (The Beatles - Hey Jude.flac)
  ⚠️ Warning: Duplicate detected for "Queen - Radio Ga Ga.mp3" -> Skipped.
  ```

## Resumen de la Experiencia de Usuario (CLI UX)

Para garantizar un estándar "Senior", la CLI integrará:

1. **Barras de Progreso Reales:** Usando crates como `indicatif` para mostrar ETA asíncronos reales durante `scan` y `migrate`.
2. **Colorización:** Salidas semánticas (Rojo = Errores, Amarillo = Advertencias/Skipped, Verde = Éxito, Azul = Tablas/Info).
3. **JSON Output:** Todo comando tendrá el flag opcional estandarizado `--json` pensado para que otros scripts (o una futura capa API) consuman las salidas crudas desde consola sin tener que parsear tablas textuales.

---

## 6. Identificación Acústica (AcoustID)

Permite identificar los metadatos reales de una canción ignorando el nombre del archivo, usando su **huella acústica** (chromaprint fingerprint) y la base de datos pública de **AcoustID / MusicBrainz**.

### `identify`

Calcula la huella acústica de un archivo o de todos los archivos de un directorio y consulta la API de AcoustID para obtener artista, título y álbum reales.

- **Uso:** `music-manager identify <PATH> [OPCIONES]`
- **Opciones:**
  - `--save`: Sobrescribe los tags ID3/Vorbis del archivo con los metadatos recuperados.
- **Ejemplo (archivo individual):**
  ```bash
  music-manager identify "cancion_mal_nombrada.mp3" --save
  ```
- **Ejemplo (directorio completo en background):**
  ```bash
  music-manager identify "C:\Users\ANDRES\Music" --save
  ```
- **Salida de ejemplo:**
  ```text
  📊 Total de archivos de audio encontrados: 25220
  [1/25220] cancion.mp3
     🔍 Calculando huella acústica...
     🌐 Consultando AcoustID...
     ✅ Encontrado: Bizarrap - Shakira: Bzrp Music Sessions, Vol. 53
     💾 Metadatos guardados.
  ────────────────────────────────────────────────────
  🎉 Identificación finalizada
     📁 Archivos procesados : 25220/25220
     ✅ Identificados        : 17097
     💾 Guardados            : 14258
     ⚠️  No encontrados       : 7967
     ❌ Errores              : 2995
  ────────────────────────────────────────────────────
  ```
- **Configuración** (en `config.toml`):
  ```toml
  [identify]
  acoustid_key = "TU_KEY"      # Obtener en https://acoustid.org/new-application
  # fpcalc_path = "C:\\ruta\\fpcalc.exe"  # Opcional si fpcalc no está en PATH
  ```

---

## 7. Renombrado Estándar de Archivos

Renombra archivos de audio aplicando un formato consistente basado en los tags ID3/Vorbis del archivo. Detecta automáticamente versiones especiales (acústica, en vivo, remix...).

### `rename`

- **Formato de salida:** `{Artista} - {NúmeroPista} {Título} ({Versión}).{ext}`
  - El número de pista y la versión son **opcionales** (solo se incluyen si existen en los tags o se detectan).
- **Uso:** `music-manager rename <PATH> [OPCIONES]`
- **Opciones:**
  - `--dry-run`: Muestra los cambios propuestos **sin aplicarlos** (simulación segura).
  - `--version <TIPO>`: Fuerza una versión específica para todos los archivos. Valores: `acustica`, `envivo`, `remix`, `cover`, `instrumental`, `radio`, `extended`, `demo`, `remaster`.
  - `--no-version`: Desactiva la detección automática de versión.
- **Ejemplos:**
  ```bash
  # Ver cambios sin aplicar
  music-manager rename "C:\Users\Music" --dry-run

  # Renombrar directorio completo
  music-manager rename "C:\Users\Music"

  # Forzar versión para convenciones de álbumes en vivo
  music-manager rename "C:\Conciertos" --version envivo
  ```
- **Salida de ejemplo:**
  ```text
  📊 Total de archivos encontrados: 25220
  [1/25220] ✅ Renombrado:   005.SHAKIRA    BZRP Music Sessions 5.mp3 →
                              Bizarrap - 01 Shakira- Bzrp Music Sessions, Vol. 53.mp3
  ────────────────────────────────────────────────────
  🎉 Renombrado finalizado
     📁 Total archivos    : 25220
     ✅ Renombrados       : 21518
     ⏩ Sin cambios       : 1919
     ❌ Errores           : 1783
  ────────────────────────────────────────────────────
  ```
- **Configuración** (en `config.toml`):
  ```toml
  [rename]
  auto_detect_version = true   # Detecta Acústica/En Vivo/Remix automáticamente
  # default_version = ""       # Fuerza versión global (vacío = deshabilitado)
  ```
