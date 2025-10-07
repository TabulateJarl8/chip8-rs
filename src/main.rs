use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey, SmolStr},
    window::{Window, WindowId},
};

use crate::emulator::Chip8;

mod emulator;
mod memory;
mod stack;
mod virtual_buffer;

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    emulator: Chip8,
}

impl App {
    fn new(program_data: Vec<u8>) -> Self {
        let mut emulator = Chip8::new();
        emulator.load(&program_data);

        Self {
            window: None,
            pixels: None,
            emulator,
        }
    }

    fn draw(&mut self) {
        if let Some(pixels) = &mut self.pixels {
            let frame = pixels.frame_mut();
            let buffer = self.emulator.window().to_framebuffer();

            // copy RGBA bytes
            for (dest, src) in frame.chunks_exact_mut(4).zip(buffer.iter()) {
                dest.copy_from_slice(&src.to_be_bytes());
            }

            if let Err(e) = pixels.render() {
                log::error!("Rending failed: {:?}", e);
            }
        }
    }

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
        let emu_window = self.emulator.window();
        let width = emu_window.scaled_width() as u32;
        let height = emu_window.scaled_height() as u32;

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("CHIP-8 Emulator")
                        .with_inner_size(LogicalSize::new(width, height)),
                )
                .unwrap(),
        );

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, Arc::clone(&window));
        let pixels = Pixels::new(width, height, surface_texture).expect("could not create pixels");

        self.pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::debug!("Close requested, stopping...");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        logical_key,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                if let Key::Named(NamedKey::Escape) = logical_key {
                    event_loop.exit();
                    return;
                } else if let Key::Character(str) = logical_key {
                    if let Some(key_index) = Self::map_key_to_index(str) {
                        match state {
                            ElementState::Pressed => self.emulator.press_key(key_index),
                            ElementState::Released => self.emulator.release_key(key_index),
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.emulator.tick();
                self.draw();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let filename = std::env::args()
        .nth(1)
        .expect("Expected path to a CHIP-8 program");
    let data = std::fs::read(filename).expect("Expected a path to a CHIP-8 program");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(data);
    event_loop.run_app(&mut app).unwrap();
}
