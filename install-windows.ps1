param(
    [switch]$InstallTarget = $true
)

# Cambiar al directorio donde esta el script
Push-Location $PSScriptRoot

Write-Host "Iniciando compilacion de Music Manager para Windows..." -ForegroundColor Cyan

# Comprobar si Cargo este instalado
if (-Not (Get-Command -Name cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: 'cargo' no esta instalado." -ForegroundColor Red
    Write-Host "Por favor instala Rust primero."
    exit 1
}

if ($InstallTarget) {
    # Cuando se compila de forma nativa en Windows normalmente se usa MSVC
}

# Compilar en modo Release para Windows
Write-Host "Compilando binario multiplataforma (.exe)..." -ForegroundColor Yellow
cargo build --release

$OutputDir = "target/release"
$BinaryPath = "$OutputDir/music-manager.exe"

if (Test-Path $BinaryPath) {
    Write-Host "Compilacion exitosa!" -ForegroundColor Green
    
    $InstallDir = "$env:LOCALAPPDATA\MusicManager"
    if (-Not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    }
    
    Write-Host "Copiando binario a la carpeta de Instalacion ($InstallDir)..." -ForegroundColor Yellow
    Copy-Item -Path $BinaryPath -Destination "$InstallDir\music-manager.exe" -Force
    
    $InstalledBinary = "$InstallDir\music-manager.exe"
    
    # Obtener el directorio de Musica predeterminado de Windows
    $MusicFolder = [Environment]::GetFolderPath('MyMusic')
    
    # Crear acceso directo en el Escritorio
    $DesktopFolder = [Environment]::GetFolderPath('Desktop')
    $ShortcutPath = Join-Path -Path $DesktopFolder -ChildPath "Music Manager.lnk"
    
    Write-Host "Creando acceso directo en tu Escritorio redirigido a $MusicFolder..." -ForegroundColor Cyan
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $InstalledBinary
    $Arguments = 'tui "' + $MusicFolder + '"'
    $Shortcut.Arguments = $Arguments
    $Shortcut.WorkingDirectory = $InstallDir
    $Shortcut.Description = "Gestor Masivo de Bibliotecas Musicales"
    $Shortcut.Save()
    
    Write-Host ""
    Write-Host "La instalacion finalizo exitosamente." -ForegroundColor Green
    Write-Host "Puedes iniciar la aplicacion haciendo doble clic en el acceso directo 'Music Manager' de tu Escritorio."
} else {
    Write-Host "Hubo un error durante la compilacion." -ForegroundColor Red
}
