# 🕹️ Guía de Uso del L4OZ Music Manager

El Music Manager de LAOZ está diseñado para la terminal y no requiere ratón, solo teclado. Se compone de dos pasos fundamentales: indexar (scan) y utilizar visualmente la interfaz interactiva.

---

## 1. El Comando Scan (Poblar la Base de Datos)
Debido a la naturaleza asíncrona y al gran tamaño de algunas bibliotecas musicales, las canciones deben registrarse una primera vez en la base de datos central antes de reproducirse fluidamente por nombre o metadatos.

1. Identifica qué ruta contiene tus MP3s.
2. Abre tu terminal e indícale a `music-manager` donde vive tu colección.
   ```bash
   music-manager scan "/media/andres.olarte/Backup/Shared/Music/"
   ```
3. Espera a que termine. Podrás escanear cuantas carpetas gigantes necesites, se irán agregando a la misma base de datos `music.db` central transparente de tu ordenador.

---

## 2. El Interfaz Gráfico Interactivo (TUI)

Es la parte fundamental. Para arrancar la magia de la consola usa:
```bash
music-manager tui "/media/andres.olarte/Backup/Shared/Music/"
```
*(Si no provees una ruta al final, te intentará abrir en la ruta de tu propio servidor local).*

Al entrar, verás tres ventanas llamadas "Paneles". El elemento con bordes **Verdes (Foco Activo)** es donde está tu control.

### Controles Universales:
* **`Tab` (Tabulador):** Es la tecla principal. Cambia el anillo verde saltando del panel "Navegador Local", al de "Búsqueda" o a "Resultados".
* **`q` (o `Ctrl + C`):** Para matar la app completamente.

### 2.1 El Navegador Local (Panel Superior)
Este panel se salta todos los archivos de tu sistema para mostrar única y exclusivamente las **carpetas** subyacentes.
* Usa la **Flecha Abajo** y la **Flecha Arriba** de tu teclado guiando el cursor gris.
* Oprime **`Enter`** sobre cualquier carpeta y el sistema la navegará virtualmente.
* Oprime **`Retroceso` (`Backspace`)** si deseas devolverte hacia el directorio padre o superior.
* *Nota: Las canciones no indexadas en la base de datos no arrojarán resultados cuando selecciones alguna carpeta.*

### 2.2 La Barra de Búsqueda (Panel Intermedio)
Búsqueda instantánea de SQLite.
* Con el foco ahí (borde verde activo), pulsa el comando inicial **`/`**.
* Empieza a escribir. Por ejemplo `rock`. Todas las canciones de tu SQLite bajarán en tiempo real.
* Cuando halles la pista, para salir de edición aprieta la flecha `Abajo` o `Esc` e intercepta los resultados inferiores.

### 2.3 Los Resultados (Panel Inferior)
Donde vive la música.
* Recorre libremente la inmensa lista con **Flechas Arriba y Abajo** (`j`, `k`).
* Pulsa **`Enter`** en cualquier pista, se inicializará el canal con el altavoz y verás rodar tus canciones en el panel de `Now Playing` inferior.
* Pulsa **`p`** para Pausar la música. Toca **`p`** de nuevo para que siga de fondo.

> 🛈 **Pro-Tip:** ¿Por qué no arranca la canción si estoy pulsando Enter en tu carpeta principal en el panel Navegador arriba? Porque la TUI agrupa los archivos al usar la acción MPSC inferior. Es absolutamente necesario tabular abajo (o buscar) y disparar a las canciones directamente, ya que el panel de Exploración solo mira Estructuras Lógicas (Carpetas), no Estímulos Físicos de Sonido (MP3).
