//! Application state and event handling.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use image::{DynamicImage, GenericImageView};
use ratatui::widgets::ListState;

use crate::ascii::{image_to_ascii, showcase_art, AsciiArt, AsciiOptions};
use crate::scanner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Showcase,
    Help,
}

pub struct App {
    pub should_quit: bool,
    pub mode: Mode,
    pub dir: PathBuf,
    pub images: Vec<PathBuf>,
    pub list_state: ListState,
    pub opts: AsciiOptions,
    pub preview: Option<AsciiArt>,
    pub preview_error: Option<String>,
    pub status: String,
    pub flash: Option<(String, Instant)>,
    pub scroll: u16,
    pub tick: u64,
    pub last_tick: Instant,
    /// Last preview panel size we fitted to.
    pub panel: (u16, u16),
    /// Zoom on top of auto-fit (1.0 = fill panel).
    pub zoom: f32,
    /// Source image aspect (height / width in pixels). Showcase uses 1.0.
    pub source_aspect: f32,
    /// Cached decoded image for the current selection.
    cached_image: Option<DynamicImage>,
    cached_path: Option<PathBuf>,
}

impl App {
    pub fn new(dir: PathBuf) -> anyhow::Result<Self> {
        let mut images = scanner::list_images(&dir)?;
        if images.is_empty() {
            images = scanner::list_images_recursive(&dir, 200)?;
        }

        let mut list_state = ListState::default();
        if !images.is_empty() {
            list_state.select(Some(0));
        }

        let mut app = Self {
            should_quit: false,
            mode: if images.is_empty() {
                Mode::Showcase
            } else {
                Mode::Browse
            },
            dir,
            images,
            list_state,
            opts: AsciiOptions::default(),
            preview: None,
            preview_error: None,
            status: String::new(),
            flash: None,
            scroll: 0,
            tick: 0,
            last_tick: Instant::now(),
            panel: (0, 0),
            zoom: 1.0,
            source_aspect: 1.0,
            cached_image: None,
            cached_path: None,
        };

        app.load_selected_image();
        app.status = format!(
            "{} images · press ? for help · w to save",
            app.images.len()
        );
        Ok(app)
    }

    pub fn selected_path(&self) -> Option<&Path> {
        self.list_state
            .selected()
            .and_then(|i| self.images.get(i))
            .map(|p| p.as_path())
    }

    fn load_selected_image(&mut self) {
        self.cached_image = None;
        self.cached_path = None;
        self.source_aspect = 1.0;

        if self.mode == Mode::Showcase || self.images.is_empty() {
            return;
        }

        let Some(path) = self.selected_path().map(|p| p.to_path_buf()) else {
            return;
        };

        match image::open(&path) {
            Ok(img) => {
                let (iw, ih) = img.dimensions();
                if iw > 0 {
                    self.source_aspect = ih as f32 / iw as f32;
                }
                self.cached_path = Some(path);
                self.cached_image = Some(img);
            }
            Err(e) => {
                self.preview_error = Some(e.to_string());
                self.status = format!("error: {e}");
            }
        }
    }

    /// ASCII output rows/cols ratio for the current content.
    fn ascii_aspect(&self) -> f32 {
        // Character cells are taller than wide — bake that into layout.
        let cell = self.opts.aspect.clamp(0.2, 1.0);
        if self.mode == Mode::Showcase || self.images.is_empty() || self.cached_image.is_none() {
            // Procedural orb is authored as ~square after cell correction.
            cell
        } else {
            self.source_aspect * cell
        }
    }

    /// Fit ASCII width/height into the preview panel. Call from the UI each frame.
    pub fn fit_to_panel(&mut self, cols: u16, rows: u16) {
        let cols = cols.max(8);
        let rows = rows.max(4);

        let aspect = self.ascii_aspect().max(0.05);
        // Contain: pick the largest width that still fits height.
        let max_from_height = (rows as f32) / aspect;
        let fitted = (cols as f32).min(max_from_height);
        let width = (fitted * self.zoom).floor().clamp(12.0, 400.0) as u32;

        let size_changed = self.panel != (cols, rows);
        let width_changed = self.opts.width != width;
        self.panel = (cols, rows);

        if size_changed || width_changed || self.preview.is_none() {
            self.opts.width = width;
            self.refresh_preview();
        }
    }

    pub fn refresh_preview(&mut self) {
        self.scroll = 0;
        self.preview_error = None;

        if self.mode == Mode::Showcase || self.images.is_empty() || self.cached_image.is_none() {
            // Match panel height when possible for a snug showcase fill.
            let mut art = showcase_art(self.opts.width, self.opts.style, self.opts.color);
            if self.panel.1 > 0 {
                let target_h = self.panel.1 as usize;
                if art.height > target_h {
                    art.rows.truncate(target_h);
                    art.height = target_h;
                }
            }
            self.preview = Some(art);
            self.status = format!(
                "showcase · {}×{} · zoom {:.0}% · w save · ? help",
                self.opts.width,
                self.preview.as_ref().map(|a| a.height).unwrap_or(0),
                self.zoom * 100.0
            );
            return;
        }

        let img = self.cached_image.as_ref().unwrap();
        let art = image_to_ascii(img, &self.opts);
        let name = self
            .cached_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("?");
        self.status = format!(
            "{name} · {}×{} · {} · zoom {:.0}% · w save · ? help",
            art.width,
            art.height,
            self.opts.style.name(),
            self.zoom * 100.0
        );
        self.preview = Some(art);
    }

    pub fn select_next(&mut self) {
        if self.images.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.images.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
        self.load_selected_image();
        self.panel = (0, 0); // force refit for new aspect
        self.refresh_preview();
    }

    pub fn select_prev(&mut self) {
        if self.images.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) => self.images.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.list_state.select(Some(i));
        self.load_selected_image();
        self.panel = (0, 0);
        self.refresh_preview();
    }

    pub fn bump_zoom(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta).clamp(0.4, 1.5);
        self.panel = (0, 0); // force refit
        self.flash_msg(format!("zoom {:.0}%", self.zoom * 100.0));
    }

    pub fn cycle_style(&mut self, forward: bool) {
        self.opts.style = if forward {
            self.opts.style.next()
        } else {
            self.opts.style.prev()
        };
        self.refresh_preview();
        self.flash_msg(format!("style → {}", self.opts.style.name()));
    }

    pub fn toggle_color(&mut self) {
        self.opts.color = !self.opts.color;
        self.refresh_preview();
        self.flash_msg(if self.opts.color {
            "color on".into()
        } else {
            "color off".into()
        });
    }

    pub fn toggle_invert(&mut self) {
        self.opts.invert = !self.opts.invert;
        self.refresh_preview();
        self.flash_msg(if self.opts.invert {
            "inverted".into()
        } else {
            "normal".into()
        });
    }

    pub fn enter_showcase(&mut self) {
        self.mode = Mode::Showcase;
        self.panel = (0, 0);
        self.refresh_preview();
        self.flash_msg("showcase · w saves · esc back".into());
    }

    pub fn enter_browse(&mut self) {
        self.mode = Mode::Browse;
        self.load_selected_image();
        self.panel = (0, 0);
        self.refresh_preview();
    }

    pub fn save_ascii(&mut self) -> anyhow::Result<()> {
        let art = self
            .preview
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("nothing to save"))?;

        let name = match self.selected_path() {
            Some(p) if self.mode == Mode::Browse => p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("asscify")
                .to_string(),
            _ => "showcase".to_string(),
        };

        let out = self.dir.join(format!("{name}.ascii.txt"));
        std::fs::write(&out, art.as_plain_string())?;
        self.flash_msg(format!("saved → {}", out.file_name().unwrap().to_string_lossy()));
        self.status = format!("wrote {}  ·  plain ASCII text file", out.display());
        Ok(())
    }

    fn flash_msg(&mut self, msg: String) {
        self.flash = Some((msg, Instant::now()));
    }

    pub fn flash_text(&self) -> Option<&str> {
        self.flash.as_ref().and_then(|(msg, at)| {
            if at.elapsed() < Duration::from_secs(2) {
                Some(msg.as_str())
            } else {
                None
            }
        })
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.mode == Mode::Showcase && self.tick % 8 == 0 {
            let mut art = showcase_art(self.opts.width, self.opts.style, self.opts.color);
            if self.panel.1 > 0 && art.height > self.panel.1 as usize {
                art.rows.truncate(self.panel.1 as usize);
                art.height = self.panel.1 as usize;
            }
            let phase = (self.tick as f32) * 0.15;
            for (yi, row) in art.rows.iter_mut().enumerate() {
                for (xi, cell) in row.iter_mut().enumerate() {
                    if let ratatui::style::Color::Rgb(r, g, b) = cell.color {
                        let wave =
                            ((xi as f32 * 0.08 + yi as f32 * 0.12 + phase).sin() + 1.0) * 0.5;
                        let boost = (wave * 24.0) as u8;
                        cell.color = ratatui::style::Color::Rgb(
                            r.saturating_add(boost / 3),
                            g.saturating_add(boost / 3),
                            b.saturating_add(boost / 2),
                        );
                    }
                }
            }
            self.preview = Some(art);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if self.mode == Mode::Help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('h') => {
                    self.mode = if self.images.is_empty() {
                        Mode::Showcase
                    } else {
                        Mode::Browse
                    };
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.mode == Mode::Showcase && !self.images.is_empty() {
                    self.enter_browse();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('?') | KeyCode::Char('h') => self.mode = Mode::Help,
            KeyCode::Char('s') => {
                if self.mode == Mode::Showcase {
                    self.enter_browse();
                } else {
                    self.enter_showcase();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.mode == Mode::Browse {
                    self.select_next();
                } else {
                    self.scroll = self.scroll.saturating_add(1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.mode == Mode::Browse {
                    self.select_prev();
                } else {
                    self.scroll = self.scroll.saturating_sub(1);
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => self.bump_zoom(0.1),
            KeyCode::Char('-') | KeyCode::Char('_') => self.bump_zoom(-0.1),
            KeyCode::Char('0') => {
                self.zoom = 1.0;
                self.panel = (0, 0);
                self.flash_msg("zoom reset · fill panel".into());
            }
            KeyCode::Char('[') => self.cycle_style(false),
            KeyCode::Char(']') => self.cycle_style(true),
            KeyCode::Char('c') => self.toggle_color(),
            KeyCode::Char('i') => self.toggle_invert(),
            KeyCode::Char('w') => {
                if let Err(e) = self.save_ascii() {
                    self.flash_msg(format!("save failed: {e}"));
                }
            }
            KeyCode::Char('r') => {
                if let Ok(mut imgs) = scanner::list_images(&self.dir) {
                    if imgs.is_empty() {
                        imgs = scanner::list_images_recursive(&self.dir, 200).unwrap_or_default();
                    }
                    self.images = imgs;
                    if self.list_state.selected().is_none() && !self.images.is_empty() {
                        self.list_state.select(Some(0));
                    }
                    if let Some(i) = self.list_state.selected() {
                        if i >= self.images.len() {
                            self.list_state
                                .select(Some(self.images.len().saturating_sub(1)));
                        }
                    }
                    self.load_selected_image();
                    self.panel = (0, 0);
                    self.refresh_preview();
                    self.flash_msg(format!("refreshed · {} images", self.images.len()));
                }
            }
            KeyCode::Enter => {
                if self.mode == Mode::Browse {
                    self.enter_showcase();
                }
            }
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(5),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(5),
            _ => {}
        }
    }

    pub fn run(mut self, terminal: &mut ratatui::DefaultTerminal) -> anyhow::Result<()> {
        let tick_rate = Duration::from_millis(50);
        loop {
            terminal.draw(|frame| crate::ui::draw(frame, &mut self))?;

            let timeout = tick_rate.saturating_sub(self.last_tick.elapsed());
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key),
                    Event::Resize(_, _) => {
                        self.panel = (0, 0); // refit on next draw
                    }
                    _ => {}
                }
            }

            if self.last_tick.elapsed() >= tick_rate {
                self.on_tick();
                self.last_tick = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}
