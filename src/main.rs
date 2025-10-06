use minifb::{Key, KeyRepeat, Window, WindowOptions};

use crate::emulator::Chip8;

mod emulator;
mod memory;
mod stack;
mod virtual_buffer;

fn main() {
    let mut emulator = Chip8::new();

    let mut window = Window::new(
        "CHIP-8 Emulator",
        emulator.window().scaled_width(),
        emulator.window().scaled_height(),
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(60);

    let data = include_bytes!("1-chip8-logo.ch8");
    emulator.load(data);

    println!("{:?}", emulator);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        emulator.press_keys(&window.get_keys_pressed(KeyRepeat::No));
        emulator.release_keys(&window.get_keys_released());

        emulator.tick();
        println!("{:?}", emulator);

        let emu_window = emulator.window();
        window
            .update_with_buffer(
                &emu_window.to_framebuffer(),
                emu_window.scaled_width(),
                emu_window.scaled_height(),
            )
            .unwrap();
    }
}
