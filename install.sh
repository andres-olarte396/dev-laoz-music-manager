#!/bin/bash

# Generador e Instalador para Linux
# Compila dev-laoz-music-manager y crea el enlace en la terminal global

set -e

echo "🎵 Iniciando compilación de Music Manager..."

# Verificar si Cargo está instalado
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: 'cargo' no está instalado."
    echo "Por favor instala Rust primero usando: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Compilar en modo Release
echo "📦 Compilando en modo Release..."
cargo build --release

# Directorio de instalación local de usuario Linux
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

echo "🚚 Moviendo binario a $BIN_DIR..."
cp target/release/music-manager "$BIN_DIR/music-manager"

# Asegurar permisos de ejecución
chmod +x "$BIN_DIR/music-manager"

echo "✅ Instalación completada con éxito."
echo ""
echo "▶️ Prueba tu nueva aplicación global ejecutando:"
echo "   music-manager tui \"$HOME/Music\""
echo ""
echo "Nota: Si el comando no funciona directamente, asegúrate de que '$BIN_DIR' esté en tu PATH."
