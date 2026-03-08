# 🐧 Guía de Instalación y Generación (Linux)

Esta guía detalla los pasos requeridos para compilar, instalar y utilizar **dev-laoz-music-manager** nativamente en tu entorno Linux (como Pop!_OS, Ubuntu, Debian, etc.).

## 1. Requisitos Previos

Antes de comenzar, debes asegurarte de tener instalado [Rust](https://www.rust-lang.org/tools/install) en tu sistema.

Verifica tu instalación:
```bash
cargo --version
rustc --version
```
Si no lo tienes instalado, ejecuta el siguiente comando oficial en tu terminal:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Cierra tu terminal y vuelve a abrirla para recargar las variables de entorno.

## 2. Generación del Binario usando el Script Automatizado

El repositorio cuenta con un script preparado (`install.sh`) que agiliza el proceso completo.

1. Abre tu terminal y ubícate en la raíz de este proyecto:
   ```bash
   cd /ruta/a/dev-laoz-music-manager/
   ```
2. Otorga permisos de ejecución al script (si aún no los tiene):
   ```bash
   chmod +x install.sh
   ```
3. Ejecuta el instalador:
   ```bash
   ./install.sh
   ```

### ¿Qué hace el script?
* Usa `cargo build --release` para compilar todo el código fuente. El flag `--release` garantiza que el ejecutable resultante se beneficie de todas las optimizaciones de procesamiento que Rust ofrece (haciendo que el audio y el escaneo sean rapidísimos).
* Toma el ejecutable compilado (ubicado en `target/release/music-manager`) y lo **copia** a la carpeta `~/.local/bin/music-manager`.

## 3. Configuración del PATH Global

Para que puedas llamar a `music-manager` desde *cualquier* lugar de tu terminal (sin tener que estar en la carpeta del repositorio), la ruta `~/.local/bin` debe ser reconocida por tu sistema operativo.

Si al terminar la instalación ejecutas:
```bash
music-manager --version
```
Y recibes el error **`no se encontró la orden`**, necesitas agregar esa ruta a tu archivo `~/.bashrc` (o `~/.zshrc`).

Ejecuta este comando:
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## 4. Instalación Manual (Sin Script)

Si prefieres hacerlo paso a paso a mano:

1. Estando en la carpeta del proyecto:
   ```bash
   cargo build --release
   ```
2. Crea el directorio bin local (por si no existe):
   ```bash
   mkdir -p ~/.local/bin
   ```
3. Mueve el archivo:
   ```bash
   cp target/release/music-manager ~/.local/bin/music-manager
   ```

¡Felicidades! Tienes tu propio gestor de música compilado en tu Linux. Dirígete a la [Guía de Uso (usage-guide.md)](usage-guide.md) para los siguientes pasos.
