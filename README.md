# CHIP-8 Emulator

This project is my take on a CHIP-8 emulator implemented in Rust. While not complex at all, I had never really tried to make an emulator before so I figured this would be a good place to start. It is hardware accelerated and fully safe with deny unwrap/expect/panic. It also passes (to my knowledge) the [Timendus CHIP-8 Test Suite](https://github.com/Timendus/chip8-test-suite). I've also tried to make it as efficient as I could by using 1D arrays and trying to allocate memory as few times as possible. It took me about 9 1/2 hours to full complete, including documentation/cleanup of the code.

This project makes use of [`winit`](https://github.com/rust-windowing/winit) and [`pixels`](https://github.com/parasyte/pixels) for the rendering, and [`rodio`](https://github.com/RustAudio/rodio) for the cross-platform audio. Sound support can be optionally compiled out with the `audio` feature flag.

## Quirks

This emulator supports the following list of quirks taken from Timendus' test suite:

- vF Reset - The AND, OR and XOR opcodes (`8xy1`, `8xy2`, and `8xy3`) reset the flags register to zero
- Memory - The save and load opcodes (`Fx55` and `Fx65`) increment the index register
- Display Wait - Drawing sprites to the display waits for the vertical blank interrupt, limiting their speed to max 60 sprites per second
- Clipping - Sprites drawn at the bottom edge of the screen get clipped instead of wrapping around to the top of the screen
- Shifting - The shift opcodes (`8xy6` and `8xyE`) only operate on `vX` instead of storing the shifted version of `vY` in `vX`
- Jumping - The "jump to some address plus `v0`" instruction (`Bnnn`) doesn't use `v0`, but `vX` instead where `X` is the highest nibble of `nnn`

### AI Usage

AI was used for a few documentation fragments and for improving logging within the crate.
