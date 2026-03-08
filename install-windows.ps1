param(
    [switch]$InstallTarget = $true
)

# Cambiar al directorio donde está el script
Push-Location $PSScriptRoot

Write-Host "🎵 Iniciando compilación de Music Manager para Windows..." -ForegroundColor Cyan

# Comprobar si Cargo está instalado
if (-Not (Get-Command -Name cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: 'cargo' no está instalado." -ForegroundColor Red
    Write-Host "Por favor instala Rust primero."
    exit 1
}

if ($InstallTarget) {
    # Cuando se compila de forma nativa en Windows normalmente se usa MSVC (por defecto).
    # Solo agregaremos el target x86_64-pc-windows-gnu si explícitamente se requiere,
    # pero como está fallando por falta de MinGW, usaremos el default nativo de la máquina.
}

# Compilar en modo Release para Windows
Write-Host "📦 Compilando binario multiplataforma (.exe)..." -ForegroundColor Yellow
cargo build --release

$OutputDir = "target/release"
$BinaryPath = "$OutputDir/music-manager.exe"

if (Test-Path $BinaryPath) {
    Write-Host "✅ ¡Compilación exitosa!" -ForegroundColor Green
    Write-Host "El ejecutable de Windows se encuentra en: $BinaryPath"
    Write-Host ""
    Write-Host "▶️ Puedes copiar ese archivo .exe a cualquier máquina con Windows y ejecutarlo."
} else {
    Write-Host "❌ Hubo un error durante la compilación." -ForegroundColor Red
}
