use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey, SmolStr},
    window::{Window, WindowId},
};

use crate::emulator::Chip8;

/// Emulated CPU should default to a rate of 700Hz
const TARGET_CPU_FREQ: u64 = 700;
/// Timers should be ticked at a rate of 60Hz
const TIMER_FREQ: u64 = 60;

/// The Application GUI
pub struct App {
    /// The Application's window
    window: Option<Arc<Window>>,
    /// The application's rendering plane
    pixels: Option<Pixels<'static>>,
    /// The emulator
    emulator: Chip8,
    /// The last time the CPU was ticked. Used for frequency emulation.
    last_cpu_time: Instant,
    /// The last time the timers were ticked. Used for frequency emulation.
    last_timer_time: Instant,
}

impl App {
    /// Construct a new application with given ROM data
    pub fn new(program_data: Vec<u8>) -> Self {
        let mut emulator = Chip8::new();
        emulator.load(&program_data);

        Self {
            window: None,
            pixels: None,
            emulator,
            last_cpu_time: Instant::now(),
            last_timer_time: Instant::now(),
        }
    }

    /// Renders the virtual window to the [`Self::pixels`] plane. Actual redrawing is deferred to
    /// [`Self::about_to_wait`]
    fn draw(&mut self) {
        if let Some(pixels) = &mut self.pixels {
            let frame = pixels.frame_mut();
            self.emulator.window().render_to_buffer(frame);

            if let Err(e) = pixels.render() {
                log::error!("Rending failed: {:?}", e);
            }
        }
    }

    /// Maps a given character to a CHIP-8 key index
    fn map_key_to_index(key_text: SmolStr) -> Option<usize> {
        match key_text.as_ref() {
            "1" => Some(0x1),
            "2" => Some(0x2),
            "3" => Some(0x3),
            "4" => Some(0xC),
            "q" => Some(0x4),
            "w" => Some(0x5),
            "e" => Some(0x6),
            "r" => Some(0xD),
            "a" => Some(0x7),
            "s" => Some(0x8),
            "d" => Some(0x9),
            "f" => Some(0xE),
            "z" => Some(0xA),
            "x" => Some(0x0),
            "c" => Some(0xB),
            "v" => Some(0xF),
            _ => None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Construct the default window and pixels rendering plane
        let emu_window = self.emulator.window();
        let width = emu_window.scaled_width() as u32;
        let height = emu_window.scaled_height() as u32;

        // The window is an Arc in order to have an owned shared reference with the pixels plane
        log::info!("Creating window ({}x{})", width, height);
        let window = Arc::new(
            match event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("CHIP-8 Emulator")
                        .with_inner_size(LogicalSize::new(width, height)),
                ) {
                    Ok(w) => w,
                    Err(e) => {
                        log::error!("Error constructing window: {:?}", e);
                        std::process::exit(1);
                    },
                }
        );

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, Arc::clone(&window));
        let pixels = match Pixels::new(width, height, surface_texture) {
            Ok(p) => p,
            Err(e) => {
                log::error!("Error constructing pixel buffer: {:?}", e);
                std::process::exit(1);
            },
        };

        self.pixels = Some(pixels);
        self.window = Some(window);

        // reset the cpu and timer times
        self.last_cpu_time = Instant::now();
        self.last_timer_time = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::debug!("Close requested, stopping...");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    // filter for non-repeated keypresses
                    KeyEvent {
                        state,
                        logical_key,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                log::trace!("Keyboard Input: {:?}, {:?}", logical_key, state);
                if let Key::Named(NamedKey::Escape) = logical_key {
                    // close the application on escape
                    event_loop.exit();
                } else if let Key::Character(str) = logical_key
                    && let Some(key_index) = Self::map_key_to_index(str) {
                        match state {
                            ElementState::Pressed => self.emulator.press_key(key_index),
                            ElementState::Released => self.emulator.release_key(key_index),
                        }
                }
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // CPU clock timer
        let cpu_time = Duration::from_secs_f64(1.0 / TARGET_CPU_FREQ as f64);
        while self.last_cpu_time.elapsed() >= cpu_time {
            self.emulator.tick_cpu();
            self.last_cpu_time += cpu_time;
        }

        // Timers run at 60Hz
        let timer_time = Duration::from_secs_f64(1.0 / TIMER_FREQ as f64);
        if self.last_timer_time.elapsed() >= timer_time {
            self.emulator.tick_timers();
            self.last_timer_time = Instant::now();
        }

        // Request redraw and sleep until next event
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
