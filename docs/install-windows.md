# 🪟 Guía de Compilación Nativa (Windows)

Esta guía detalla los pasos para compilar versiones ejecutables nativas para plataformas Microsoft Windows (`.exe`) directamente desde una máquina Windows.

## 1. Requisitos Previos (En tu Windows)

Para compilar la aplicación en Windows requerimos:
1. Instalar **Rust** y su gestor de paquetes **Cargo**.
   Puedes instalarlo desde PowerShell ejecutando:
   ```powershell
   Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
   .\rustup-init.exe -y
   ```
2. **Reiniciar tu terminal:** Después de la instalación, es obligatorio cerrar tu ventana de PowerShell y abrirla nuevamente para que los comandos `rustc` y `cargo` estén disponibles (o forzar su carga con `$env:Path = ...`).

## 2. Generación Automatizada usando PowerShell

El repositorio incluye un script escrito en PowerShell que orquesta la compilación para Windows de forma nativa.

1. Abre la consola de comandos de PowerShell en tu sistema.
2. Ejecuta el script de construcción de Windows desde la raíz del ecosistema o desde la misma carpeta de la herramienta:
   ```powershell
   .\tools\dev-laoz-music-manager\install-windows.ps1
   ```

### ¿Qué hace internamente este script?
1. Se posiciona en la carpeta correcta del código fuente (`Push-Location`).
2. Confirma que Cargo esté disponible.
3. Llama a Cargo para compilar usando el compilador nativo de la máquina:
   `cargo build --release`
4. Informa al usuario en qué lugar se almacenó el producto `.exe`.

## 3. Empleo en Sistema Windows Real

El resultado final de ese comando será un ejecutable ubicado exactamente aquí:
```text
/ruta/al/repo/tools/dev-laoz-music-manager/target/release/music-manager.exe
```

Ese archivo `music-manager.exe` es 100% independiente.

*   Puedes llevarlo en una llave USB y usarlo en cualquier computadora Windows.
*   En la máquina Windows, abre `PowerShell` o el `Símbolo del sistema` (cmd).
*   Para ejecutarlo en PowerShell desde la carpeta en que se encuentra, es necesario poner `.\` delante:
    ```powershell
    cd .\tools\dev-laoz-music-manager\target\release\
    
    .\music-manager.exe scan "C:\Users\Nombre\Music\"
    .\music-manager.exe tui "C:\Users\Nombre\Music\"
    ```

Siguiente paso recomendado: Revisar la [Guía de Uso (usage-guide.md)](usage-guide.md).
