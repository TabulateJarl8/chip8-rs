use minifb::{Key, Window, WindowOptions};

use crate::virtual_buffer::{PixelState, VirtualDisplay};

mod emulator;
mod virtual_buffer;

fn main() {
    let mut buffer = VirtualDisplay::new(20);

    let mut window = Window::new(
        "CHIP-8 Emulator",
        buffer.scaled_width(),
        buffer.scaled_height(),
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(60);

    for i in 0..64 {
        if i % 2 == 0 {
            buffer.set_pixel(i, 0, PixelState::On);
        }
    }

    buffer.set_pixel(5, 5, PixelState::On);
    buffer.set_pixel(6, 5, PixelState::On);
    buffer.set_pixel(7, 5, PixelState::On);
    buffer.set_pixel(5, 6, PixelState::On);
    buffer.set_pixel(6, 6, PixelState::On);
    buffer.set_pixel(7, 6, PixelState::On);
    buffer.set_pixel(5, 7, PixelState::On);
    buffer.set_pixel(6, 7, PixelState::On);
    buffer.set_pixel(7, 7, PixelState::On);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(
                buffer.to_framebuffer(),
                buffer.scaled_width(),
                buffer.scaled_height(),
            )
            .unwrap();
    }
}
