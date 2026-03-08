param(
    [switch]$InstallTarget = $true
)

Write-Host "🎵 Iniciando compilación de Music Manager para Windows..." -ForegroundColor Cyan

# Comprobar si Cargo está instalado
if (-Not (Get-Command -Name cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: 'cargo' no está instalado." -ForegroundColor Red
    Write-Host "Por favor instala Rust primero."
    exit 1
}

if ($InstallTarget) {
    Write-Host "🔧 Agregando target de Windows (x86_64-pc-windows-gnu)..." -ForegroundColor Yellow
    rustup target add x86_64-pc-windows-gnu
    
    # En Linux, se requiere Mingw-w64 para cross-compilar hacia Windows
    # sudo apt install mingw-w64
}

# Compilar en modo Release para Windows
Write-Host "📦 Compilando binario multiplataforma (.exe)..." -ForegroundColor Yellow
cargo build --release --target x86_64-pc-windows-gnu

$OutputDir = "target/x86_64-pc-windows-gnu/release"
$BinaryPath = "$OutputDir/music-manager.exe"

if (Test-Path $BinaryPath) {
    Write-Host "✅ ¡Compilación exitosa!" -ForegroundColor Green
    Write-Host "El ejecutable de Windows se encuentra en: $BinaryPath"
    Write-Host ""
    Write-Host "▶️ Puedes copiar ese archivo .exe a cualquier máquina con Windows y ejecutarlo."
} else {
    Write-Host "❌ Hubo un error durante la compilación." -ForegroundColor Red
}
