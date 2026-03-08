# 🪟 Guía de Generación (Windows Cross-Compilation)

Esta guía detalla los pasos para compilar versiones ejecutables nativas para plataformas Microsoft Windows (`.exe`) desde una máquina anfitrión Linux.

La ventaja de este proceso de **compilación cruzada** es que puedes empaquetar y entregar un instalador para otro sistema operativo sin necesidad de arrancar máquinas virtuales de Windows ni salir de tu entorno Laoz local.

## 1. Requisitos Previos (En tu Linux)

Para compilar hacia la arquitectura de Windows requerimos:
1. Instalar la cadena de herramientas `rustup target x86_64-pc-windows-gnu`.
2. Una herramienta llamada `mingw-w64` en Linux que funciona como enlazador para los archivos binarios compilados de Windows.

Abre tu terminal en Pop!_OS / Ubuntu y ejecuta:
```bash
sudo apt update
sudo apt install mingw-w64 -y
```

## 2. Generación Automatizada usando Powershell

El repositorio incluye un script escrito en Powershell que orquesta la compilación para Windows.

1. Abre la consola de comandos de Powershell en tu sistema local.
   ```bash
   pwsh
   ```
2. Ejecuta el script de construcción de Windows situado en la raíz del proyecto:
   ```powershell
   ./install-windows.ps1
   ```

### ¿Qué hace internamente este script de Windows?
1. Confirma si necesitas descargar el target de compilación y ejecuta:
   `rustup target add x86_64-pc-windows-gnu`.
2. Llama a Cargo dándole la arquitectura extraña objetivo:
   `cargo build --release --target x86_64-pc-windows-gnu`.
3. Informa al usuario en qué lugar se almacenó el producto `.exe`.

## 3. Empleo en Sistema Windows Real

El resultado final de ese comando de arriba será un ejecutable ubicado exactamente aquí:
```text
/ruta/a/dev-laoz-music-manager/target/x86_64-pc-windows-gnu/release/music-manager.exe
```

Ese archivo `music-manager.exe` es 100% independiente.

*   Puedes llevarlo en una llave USB y pegarlo en una computadora Windows.
*   En la máquina Windows, abre `PowerShell` o el `Símbolo del sistema` (cmd).
*   Llama al gestor desde allí y usa los mismos comandos que en Linux:
    ```powershell
    music-manager.exe scan "C:\Users\Nombre\Music\"
    music-manager.exe tui "C:\Users\Nombre\Music\"
    ```

Siguiente paso recomendado: Revisar la [Guía de Uso (usage-guide.md)](usage-guide.md).
