# CHIP-8 Emulator

This project is my take on a CHIP-8 emulator implemented in Rust. While not complex at all, I had never really tried to make an emulator before so I figured this would be a good place to start. It is hardware accelerated and fully safe with deny unwrap/expect/panic. It also passes (to my knowledge) the [Timendus CHIP-8 Test Suite](https://github.com/Timendus/chip8-test-suite). I've also tried to make it as efficient as I could by using 1D arrays and trying to allocate memory as few times as possible.

This project makes use of [`winit`](https://github.com/rust-windowing/winit) and [`pixels`](https://github.com/parasyte/pixels) for the rendering, and [`rodio`](https://github.com/RustAudio/rodio) for the cross-platform audio. Sound support can be optionally compiled out with the `audio` feature flag.

### AI Usage

AI was used for a few documentation fragments and for improving logging within the crate.
