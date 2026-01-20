//! Main egui application for the Praymodoro desktop companion.
//!
//! This module handles the UI rendering, sprite loading, and user interactions
//! through a transparent, draggable window that displays saint characters and
//! a countdown timer.

use crate::settings::save_settings;
use crate::state::{AppState, PomodoroMode};
use crate::tray::{TrayAction, TrayManager};
use egui::{Color32, Pos2, Rect, Sense, Vec2};
use image::imageops::FilterType;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Base width of the companion window in pixels.
const BASE_WIDTH: f32 = 160.0;

/// Base height of the companion window in pixels.
const BASE_HEIGHT: f32 = 395.0;

/// Maximum width for sprite textures loaded into GPU memory.
///
/// Original sprites are 590x1455, but we resize to 295x728 (half size)
/// to save GPU memory while maintaining quality at up to 200% scale.
const MAX_SPRITE_WIDTH: u32 = 295;

/// Maximum height for sprite textures loaded into GPU memory.
const MAX_SPRITE_HEIGHT: u32 = 728;

/// The main egui application struct for Praymodoro.
///
/// Manages the UI rendering, sprite caching, tray icon integration, and
/// responds to user interactions through both the window and tray menu.
pub struct PrayomodoroApp {
    /// Shared application state (synchronized with timer thread).
    state: Arc<Mutex<AppState>>,
    /// System tray icon manager.
    tray: Option<TrayManager>,
    /// Cached character sprite textures (key: "character_sprite").
    textures: HashMap<String, egui::TextureHandle>,
    /// Cached timer background texture.
    timer_bg: Option<egui::TextureHandle>,
    /// Last character name (used to detect character changes and clear caches).
    last_character: String,
}

impl PrayomodoroApp {
    /// Creates a new Praymodoro application instance.
    ///
    /// Initializes the system tray icon and sets up the initial character.
    /// Must be called on the main thread.
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        // Create tray on main thread
        let tray = TrayManager::new();

        let initial_character = {
            let s = state.lock();
            s.character.clone()
        };

        Self {
            state,
            tray: Some(tray),
            textures: HashMap::new(),
            timer_bg: None,
            last_character: initial_character,
        }
    }

    /// Loads a character sprite texture, with caching.
    ///
    /// Searches multiple locations for the sprite asset and resizes it to
    /// [`MAX_SPRITE_WIDTH`] x [`MAX_SPRITE_HEIGHT`] to conserve GPU memory.
    ///
    /// # Arguments
    ///
    /// * `ctx` - egui context for texture loading
    /// * `character` - Character identifier (e.g., "augustine-of-hippo")
    /// * `sprite` - Sprite name (e.g., "work", "quick-break", "idle")
    ///
    /// # Returns
    ///
    /// The texture handle if successfully loaded, or `None` if not found.
    fn load_texture(
        &mut self,
        ctx: &egui::Context,
        character: &str,
        sprite: &str,
    ) -> Option<egui::TextureHandle> {
        let key = format!("{}_{}", character, sprite);
        if let Some(tex) = self.textures.get(&key) {
            return Some(tex.clone());
        }

        // Try to load from assets directory
        let asset_path = format!("assets/characters/{}/{}.png", character, sprite);

        // First try relative to executable
        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;

        // Try multiple locations
        let paths_to_try = [
            exe_dir.join(&asset_path),
            exe_dir.join("../Resources").join(&asset_path),
            std::path::PathBuf::from(&asset_path),
            std::path::PathBuf::from(format!(
                "../assets/characters/{}/{}.png",
                character, sprite
            )),
            // For development - run from project root
            std::path::PathBuf::from(format!(
                "src-egui/assets/characters/{}/{}.png",
                character, sprite
            )),
        ];

        for path in &paths_to_try {
            if let Ok(image_data) = std::fs::read(path) {
                if let Ok(image) = image::load_from_memory(&image_data) {
                    // Resize to save GPU memory (590x1455 -> 295x728)
                    let resized = if image.width() > MAX_SPRITE_WIDTH || image.height() > MAX_SPRITE_HEIGHT {
                        image.resize(MAX_SPRITE_WIDTH, MAX_SPRITE_HEIGHT, FilterType::Lanczos3)
                    } else {
                        image
                    };

                    let rgba = resized.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = rgba.into_raw();

                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    let texture =
                        ctx.load_texture(&key, color_image, egui::TextureOptions::default());

                    self.textures.insert(key, texture.clone());
                    return Some(texture);
                }
            }
        }

        None
    }

    /// Loads the timer background texture from embedded assets.
    ///
    /// The timer background is cached after the first load.
    fn load_timer_bg(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if let Some(ref tex) = self.timer_bg {
            return Some(tex.clone());
        }

        // Load timer background from embedded bytes
        let timer_bytes = include_bytes!("../assets/ui/timer-rectangle.png");
        if let Ok(image) = image::load_from_memory(timer_bytes) {
            let rgba = image.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.into_raw();

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            let texture = ctx.load_texture("timer_bg", color_image, egui::TextureOptions::default());

            self.timer_bg = Some(texture.clone());
            return Some(texture);
        }

        None
    }

    /// Handles actions triggered from the system tray menu.
    ///
    /// Updates application state and sends viewport commands in response to
    /// user interactions with the tray icon menu.
    fn handle_tray_action(&mut self, action: TrayAction, ctx: &egui::Context) {
        match action {
            TrayAction::ToggleVisibility => {
                let mut s = self.state.lock();
                s.visible = !s.visible;
                let visible = s.visible;
                drop(s);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(visible));
            }
            TrayAction::SetCharacter(char_name) => {
                let mut s = self.state.lock();
                s.character = char_name;
                s.settings.character = s.character.clone();
                drop(s);
                let s = self.state.lock();
                save_settings(&s.settings);
            }
            TrayAction::SetScale(scale) => {
                let mut s = self.state.lock();
                s.scale = scale;
                s.settings.window.scale = s.scale;
                let new_size = Vec2::new(BASE_WIDTH * s.scale, BASE_HEIGHT * s.scale);
                drop(s);
                let s = self.state.lock();
                save_settings(&s.settings);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(new_size));
            }
            TrayAction::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            TrayAction::None => {}
        }
    }
}

impl eframe::App for PrayomodoroApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Fully transparent background
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll tray events on main thread
        if let Some(ref mut tray) = self.tray {
            let action = tray.poll_events(&self.state);
            self.handle_tray_action(action, ctx);
        }

        // Check if should quit
        {
            let s = self.state.lock();
            if s.should_quit {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }

        // Get current state
        let (mode, formatted_time, character, scale) = {
            let s = self.state.lock();
            (
                s.mode,
                s.formatted_time.clone(),
                s.character.clone(),
                s.scale,
            )
        };

        // Check if character changed - if so, clear old textures and request full redraw
        let character_changed = character != self.last_character;
        if character_changed {
            // Clear cached textures for the old character to free GPU memory
            let old_char = &self.last_character;
            self.textures.retain(|key, _| !key.starts_with(old_char));

            self.last_character = character.clone();
            ctx.request_repaint();
        }

        // Determine sprite to show
        let sprite = match mode {
            PomodoroMode::Work => "work",
            PomodoroMode::Rest => "quick-break",
        };

        // Load texture
        let texture = self.load_texture(ctx, &character, sprite);

        // Central panel with transparent background
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                // Use expected size based on scale, not available_size which can be wrong on first frame
                let expected_size = Vec2::new(BASE_WIDTH * scale, BASE_HEIGHT * scale);
                let available_size = ui.available_size();
                // Use the larger of expected or available to avoid tiny sprites on startup
                let size = Vec2::new(
                    available_size.x.max(expected_size.x),
                    available_size.y.max(expected_size.y),
                );
                let rect = Rect::from_min_size(Pos2::ZERO, size);

                // Handle dragging - use native OS drag for smooth movement
                let response = ui.allocate_rect(rect, Sense::drag());

                if response.drag_started() {
                    // Use native window drag - much smoother than manual position updates
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                // Clear the entire window area first (fixes ghosting on transparent windows)
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::ZERO,
                    Color32::TRANSPARENT,
                );

                // Draw character sprite
                if let Some(tex) = texture {
                    let image_size = tex.size_vec2();
                    let aspect = image_size.x / image_size.y;

                    // Scale to fit window while maintaining aspect ratio
                    let target_height = size.y * 0.85; // Leave room for timer
                    let target_width = target_height * aspect;

                    let sprite_size = Vec2::new(target_width.min(size.x), target_height);
                    let sprite_pos = Pos2::new(
                        (size.x - sprite_size.x) / 2.0,
                        size.y - sprite_size.y,
                    );

                    ui.painter().image(
                        tex.id(),
                        Rect::from_min_size(sprite_pos, sprite_size),
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                // Draw timer at bottom with parchment background
                // Original: 130px × 49px, positioned at bottom: 20px
                let timer_width = 130.0 * scale;
                let timer_height = 49.0 * scale;
                let timer_bottom_margin = 20.0 * scale;

                let timer_rect = Rect::from_min_size(
                    Pos2::new(
                        (size.x - timer_width) / 2.0,
                        size.y - timer_height - timer_bottom_margin,
                    ),
                    Vec2::new(timer_width, timer_height),
                );

                // Draw timer background image
                if let Some(timer_tex) = self.load_timer_bg(ctx) {
                    ui.painter().image(
                        timer_tex.id(),
                        timer_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                // Timer text - dark brown like original (#4a3728), serif font
                let timer_color = Color32::from_rgb(74, 55, 40);
                let font_size = 26.0 * scale;
                ui.painter().text(
                    timer_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &formatted_time,
                    egui::FontId::new(font_size, egui::FontFamily::Name("serif".into())),
                    timer_color,
                );
            });

        // Request repaint frequently to keep UI responsive
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
