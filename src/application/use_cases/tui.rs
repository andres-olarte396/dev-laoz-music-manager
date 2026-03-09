use std::fs::{self, File};
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::domain::ports::track_repository::TrackRepository;
use crate::application::use_cases::playback::PlaybackUseCase;
use crate::domain::entities::track::Track;

pub enum PlayerCommand {
    Quit,
    Play(Track),
    Pause,
    Resume,
}

enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq, Clone, Copy)]
enum AppFocus {
    Explorer,
    Search,
    Library,
}

pub struct ExplorerEntry {
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub name: String,
}

struct App<R: TrackRepository> {
    input: String,
    input_mode: InputMode,
    results: Vec<Track>,
    list_state: ListState,
    
    // File Explorer State
    current_dir: std::path::PathBuf,
    dir_entries: Vec<ExplorerEntry>,
    explorer_state: ListState,

    focus: AppFocus,
    repository: R,
    current_track: Option<Track>,
    is_playing: bool,
    playback_start: Option<std::time::Instant>,
    accumulated_play_time: std::time::Duration,
}

impl<R: TrackRepository> App<R> {
    fn new(repository: R, start_path: Option<String>) -> App<R> {
        let current_dir = start_path
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")));
        
        let mut app = App {
            input: String::new(),
            input_mode: InputMode::Normal,
            results: Vec::new(),
            list_state: ListState::default(),
            
            current_dir,
            dir_entries: Vec::new(),
            explorer_state: ListState::default(),
            
            focus: AppFocus::Search, // Default focus
            repository,
            current_track: None,
            is_playing: false,
            playback_start: None,
            accumulated_play_time: std::time::Duration::ZERO,
        };
        app.load_directory();
        app
    }

    fn load_directory(&mut self) {
        self.dir_entries.clear();
        
        // Vista de "Mi Equipo" / "Discos Locales" en Windows
        if self.current_dir.as_os_str().is_empty() {
            for drive in b'A'..=b'Z' {
                let drive_str = format!("{}:\\", drive as char);
                let path = std::path::PathBuf::from(&drive_str);
                if path.exists() {
                    self.dir_entries.push(ExplorerEntry {
                        path,
                        is_dir: true,
                        name: drive_str,
                    });
                }
            }
            self.explorer_state.select(Some(0));
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut valid_entries: Vec<ExplorerEntry> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    if let Ok(ft) = e.file_type() {
                        if ft.is_dir() {
                            return Some(ExplorerEntry {
                                path: e.path(),
                                is_dir: true,
                                name: e.file_name().to_string_lossy().to_string(),
                            });
                        }
                        if ft.is_file() {
                            let ext = e.path().extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                            let extensions = ["mp3", "flac", "m4a", "wma", "wav", "ogg", "aac", "opus", "alac", "aiff"];
                            if extensions.contains(&ext.as_str()) {
                                return Some(ExplorerEntry {
                                    path: e.path(),
                                    is_dir: false,
                                    name: e.file_name().to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                    None
                })
                .collect();
            // Sort alphabetical
            valid_entries.sort_by(|a, b| a.name.cmp(&b.name));
            self.dir_entries = valid_entries;
        }
        self.explorer_state.select(Some(0));
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.results.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        match self.focus {
            AppFocus::Library => self.list_state.select(Some(i)),
            AppFocus::Explorer => self.explorer_state.select(Some(i)),
            _ => {}
        }
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.results.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        match self.focus {
            AppFocus::Library => self.list_state.select(Some(i)),
            AppFocus::Explorer => self.explorer_state.select(Some(i)),
            _ => {}
        }
    }

    async fn perform_search(&mut self) {
        if self.input.is_empty() {
            // Si está vacío, cargar las primeras 10000 canciones genéricas (usando limit)
            if let Ok(tracks) = self.repository.list_paginated(None, 10000).await {
                self.results = tracks;
            }
        } else {
            // Realizar búsqueda en la base de datos
            if let Ok(tracks) = self.repository.search(&self.input, 10000).await {
                self.results = tracks;
            }
        }
        self.list_state.select(Some(0));
    }

    async fn update_library_from_explorer(&mut self) {
        let path_to_search = if let Some(i) = self.explorer_state.selected() {
            if let Some(entry) = self.dir_entries.get(i) {
                entry.path.to_string_lossy().to_string()
            } else {
                self.current_dir.to_string_lossy().to_string()
            }
        } else {
            self.current_dir.to_string_lossy().to_string()
        };

        if let Ok(tracks) = self.repository.search_by_path(&path_to_search, 10000).await {
            self.results = tracks;
        }
        self.list_state.select(Some(0));
    }
}

pub struct TuiUseCase<R: TrackRepository + Clone + Send + 'static> {
    pub repository: R,
    pub target_dir: Option<String>,
}

impl<R: TrackRepository + Clone + Send + 'static> TuiUseCase<R> {
    pub fn new(repository: R, target_dir: Option<String>) -> Self {
        Self { repository, target_dir }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn Error>> {
        // --- 1. Inicializar Canales y Background Thread de Audio ---
        let (tx, rx) = mpsc::channel::<PlayerCommand>();

        let _audio_thread = thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            loop {
                // Non-blocking try_recv until the sink is empty or we get a command
                if let Ok(cmd) = rx.recv() {
                    match cmd {
                        PlayerCommand::Quit => break,
                        PlayerCommand::Play(track) => {
                            sink.stop(); // Stop anything playing
                            sink.clear();
                            
                            if let Ok(file) = File::open(&track.file_path) {
                                let reader = BufReader::new(file);
                                if let Ok(source) = Decoder::new(reader) {
                                    sink.append(source);
                                    sink.play();
                                }
                            }
                        }
                        PlayerCommand::Pause => sink.pause(),
                        PlayerCommand::Resume => sink.play(),
                    }
                } else {
                    break;
                }
            }
        });

        // --- 2. Preparar el Estado de la Aplicación TUI ---
        let mut app = App::new(self.repository.clone(), self.target_dir.clone());
        // Llenar lista inicial basada en el explorador
        app.update_library_from_explorer().await;

        // --- 3. Inicializar Terminal ratatui ---
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // --- 4. Correr la UI State Loop ---
        let res = run_app(&mut terminal, &mut app, tx.clone()).await;

        // --- 5. Limpieza general ---
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("Error en la TUI: {:?}", err);
        }

        // Matar al backend (opcionalmente)
        let _ = tx.send(PlayerCommand::Quit);
        
        Ok(())
    }
}

async fn run_app<B: Backend, R: TrackRepository>(
    terminal: &mut Terminal<B>,
    app: &mut App<R>,
    _tx: mpsc::Sender<PlayerCommand>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            return Ok(());
                        }
                        KeyCode::Tab => {
                            // Cycle Focus: Explorer -> Search -> Library
                            app.focus = match app.focus {
                                AppFocus::Explorer => AppFocus::Search,
                                AppFocus::Search => AppFocus::Library,
                                AppFocus::Library => AppFocus::Explorer,
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.focus == AppFocus::Explorer {
                                let i = match app.explorer_state.selected() {
                                    Some(i) => if i >= app.dir_entries.len().saturating_sub(1) { 0 } else { i + 1 },
                                    None => 0,
                                };
                                app.explorer_state.select(Some(i));
                                app.update_library_from_explorer().await;
                            } else if app.focus == AppFocus::Library {
                                app.next();
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.focus == AppFocus::Explorer {
                                let i = match app.explorer_state.selected() {
                                    Some(i) => if i == 0 { app.dir_entries.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                app.explorer_state.select(Some(i));
                                app.update_library_from_explorer().await;
                            } else if app.focus == AppFocus::Library {
                                app.previous();
                            }
                        }
                        KeyCode::Char('/') => {
                            app.focus = AppFocus::Search;
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Backspace => {
                            if app.focus == AppFocus::Explorer {
                                if !app.current_dir.as_os_str().is_empty() {
                                    if let Some(parent) = app.current_dir.parent() {
                                        app.current_dir = parent.to_path_buf();
                                    } else {
                                        // Ir a mis discos si estamos en la raíz (ej. E:\)
                                        app.current_dir = std::path::PathBuf::from("");
                                    }
                                    app.load_directory();
                                    app.update_library_from_explorer().await;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if app.focus == AppFocus::Library {
                                if let Some(i) = app.list_state.selected() {
                                    if let Some(track) = app.results.get(i) {
                                        app.current_track = Some(track.clone());
                                        app.is_playing = true;
                                        app.playback_start = Some(std::time::Instant::now());
                                        app.accumulated_play_time = std::time::Duration::ZERO;
                                        let _ = _tx.send(PlayerCommand::Play(track.clone()));
                                    }
                                }
                            } else if app.focus == AppFocus::Explorer {
                                if let Some(i) = app.explorer_state.selected() {
                                    if let Some(entry) = app.dir_entries.get(i) {
                                        if entry.is_dir {
                                            app.current_dir = entry.path.clone();
                                            app.load_directory();
                                            app.update_library_from_explorer().await;
                                        } else {
                                            // Quick Play from Explorer
                                            let dummy_track = Track {
                                                id: ulid::Ulid::new(),
                                                file_path: entry.path.to_string_lossy().to_string(),
                                                title: Some(entry.name.clone()),
                                                artist: Some("Local File".to_string()),
                                                album_artist: None,
                                                album: None,
                                                year: None,
                                                genre: None,
                                                duration_seconds: None,
                                            };
                                            app.current_track = Some(dummy_track.clone());
                                            app.is_playing = true;
                                            app.playback_start = Some(std::time::Instant::now());
                                            app.accumulated_play_time = std::time::Duration::ZERO;
                                            let _ = _tx.send(PlayerCommand::Play(dummy_track));
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('p') | KeyCode::Char('P') => {
                            if app.is_playing {
                                app.is_playing = false;
                                if let Some(start) = app.playback_start.take() {
                                    app.accumulated_play_time += start.elapsed();
                                }
                                let _ = _tx.send(PlayerCommand::Pause);
                            } else {
                                app.is_playing = true;
                                app.playback_start = Some(std::time::Instant::now());
                                let _ = _tx.send(PlayerCommand::Resume);
                            }
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            // Iniciar la búsqueda
                            app.perform_search().await;
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
                }
            }
        }
    }
}

fn ui<R: TrackRepository>(f: &mut Frame, app: &mut App<R>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(10),     // [NEW] File Explorer
            Constraint::Length(3),      // Search
            Constraint::Min(0),         // Library
            Constraint::Length(3),      // Now playing
        ].as_ref())
        .split(f.size());

    // -- 0. Explorer Bar --
    let (explorer_title, ex_border_color) = if app.focus == AppFocus::Explorer {
        (" ▶ Navegador Local [FOCO ACTIVO] ", Color::Green)
    } else {
        (" Navegador Local [Presiona TAB para activar] ", Color::DarkGray)
    };
    
    let ex_items: Vec<ListItem> = app.dir_entries.iter().map(|entry| {
        let icon = if entry.is_dir { "📁" } else { "🎵" };
        let name = &entry.name;
        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(Color::Yellow)),
            Span::raw(name.to_string()),
        ]))
    }).collect();

    let explorer_block = List::new(ex_items)
        .block(Block::default()
            .title(explorer_title)
            .title_bottom(format!(" Ruta: {} | [Enter] Abrir | [Retroceso] Subir ", if app.current_dir.as_os_str().is_empty() { "Discos Locales".to_string() } else { app.current_dir.display().to_string() }))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ex_border_color)))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(explorer_block, chunks[0], &mut app.explorer_state);

    // -- 1. Search Bar --
    let (search_title, search_border_color) = if app.focus == AppFocus::Search {
        (" ▶ Búsqueda SQLite [FOCO ACTIVO] ", Color::Green)
    } else {
        (" Búsqueda SQLite [Presiona TAB para activar] ", Color::DarkGray)
    };
    let input_text = match app.input_mode {
        InputMode::Normal => {
            Line::from(vec![
                Span::styled("Escribe '/' para buscar | 'q' salir | Flechas navegar", Style::default().fg(Color::DarkGray)),
                Span::raw(format!(" (Filtro actual: {})", app.input)),
            ])
        }
        InputMode::Editing => {
            Line::from(vec![
                Span::styled(">> Búsqueda: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(&app.input),
            ])
        }
    };

    let search_block = Paragraph::new(input_text)
        .block(Block::default().title(search_title).borders(Borders::ALL).border_style(Style::default().fg(search_border_color)))
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        });
    f.render_widget(search_block, chunks[1]); // Move chunks[1]

    if let InputMode::Editing = app.input_mode {
        f.set_cursor(chunks[1].x + 13 + app.input.len() as u16, chunks[1].y + 1);
    }

    // -- 2. Track List --
    let (lib_title, lib_border_color) = if app.focus == AppFocus::Library {
        (format!(" ▶ Resultados de la Base de Datos ({}) [FOCO ACTIVO] ", app.results.len()), Color::Green)
    } else {
        (format!(" Resultados de la Base de Datos ({}) [Presiona TAB para activar] ", app.results.len()), Color::DarkGray)
    };
    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|track| {
            let title = track.title.clone().unwrap_or_else(|| track.file_path.clone());
            let artist = track.artist.clone().unwrap_or_else(|| "Unknown".to_string());
            let album = track.album.clone().unwrap_or_else(|| "Unknown Album".to_string());
            let line = Line::from(vec![
                Span::styled(format!("[{}] ", track.id.to_string()), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:40} ", title), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!("👤 {:20} ", artist), Style::default().fg(Color::Cyan)),
                Span::styled(format!("💿 {:30}", album), Style::default().fg(Color::Magenta)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list_block = List::new(items)
        .block(Block::default().title(lib_title).borders(Borders::ALL).border_style(Style::default().fg(lib_border_color)))
        .highlight_style(
            Style::default()
                .bg(if app.focus == AppFocus::Library { Color::LightGreen } else { Color::DarkGray })
                .fg(if app.focus == AppFocus::Library { Color::Black } else { Color::White })
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    
    // Necesitamos pasar un estado mutable para que se dibuje la selección
    f.render_stateful_widget(list_block, chunks[2], &mut app.list_state); // Modify chunks[2]

    // -- 3. Playback Controls --
    let now_playing_text = match &app.current_track {
        Some(track) => {
            let mut elapsed = app.accumulated_play_time;
            if app.is_playing {
                if let Some(start) = app.playback_start {
                    elapsed += start.elapsed();
                }
            }
            let elapsed_secs = elapsed.as_secs();
            let duration_text = if let Some(total_secs) = track.duration_seconds {
                let e_secs = elapsed_secs.min(total_secs as u64); // Cap temporal
                format!("{:02}:{:02} / {:02}:{:02}", e_secs / 60, e_secs % 60, total_secs / 60, total_secs % 60)
            } else {
                format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60)
            };
            
            format!(" {} - {} ({}) [{}] ", 
                track.artist.as_deref().unwrap_or("Unknown"), 
                track.title.as_deref().unwrap_or("Unknown"),
                track.album.as_deref().unwrap_or("Unknown Album"),
                duration_text
            )
        },
        None => " Ninguna pista seleccionada ".to_string(),
    };

    let play_status = if app.is_playing { "▶️ Reproduciendo" } else { "⏸️ Pausado" };

    let player_block = Paragraph::new(format!("{} | {} | Controles: [Tab] Cambiar Foco, [Enter] Seleccionar, [p] Play/Pause", play_status, now_playing_text))
        .block(Block::default().title(" Now Playing ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().fg(Color::White));
    f.render_widget(player_block, chunks[3]); // Use chunks[3]
}
