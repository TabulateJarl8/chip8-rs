use crate::{memory::Memory, stack::Stack, virtual_buffer::VirtualDisplay};

#[cfg(feature = "audio")]
use crate::sound::Speaker;

/// Where the user program should be loaded into memory, and what the program counter is
/// initialized to
const START_ADDR: u16 = 0x200;

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct Quirks: u8 {
        /// The AND, OR and XOR opcodes (`8xy1`, `8xy2` and `8xy3`) reset the flags register to zero
        const VF_RESET = 0b00001;
        /// The save and load opcodes (`Fx55` and `Fx65`) increment the index register
        const MEMORY = 0b00010;
        /// Sprites drawn at the bottom edge of the screen get clipped instead of wrapping around to the top of the screen
        const CLIPPING = 0b00100;
        /// The shift opcodes (`8xy6` and `8xyE`) only operate on `vX` instead of storing the shifted version of `vY` in `vX`
        const SHIFTING = 0b01000;
        /// The "jump to some address plus `v0`" instruction (`Bnnn`) doesn't use `v0`, but `vX` instead where `X` is the highest nibble of `nnn`
        const JUMPING = 0b10000;
    }
}

impl Default for Quirks {
    fn default() -> Self {
        Self::VF_RESET | Self::CLIPPING | Self::MEMORY
    }
}


/// The main emulator state
#[derive(Debug)]
pub struct Chip8 {
    /// The RAM
    memory: Memory,
    /// 16 VX registers
    v_registers: [u8; 16],
    /// Delay timer
    delay_timer: u8,
    /// Sound timer
    sound_timer: u8,
    /// Program counter
    program_counter: u16,
    /// Special index (I) register
    index_register: u16,
    /// The stack
    stack: Stack,
    /// The virtualized window display buffer
    window: VirtualDisplay,
    /// Array of which keys are currently pressed
    keys: [bool; 16],
    /// This is Some when we are waiting on a keypress from the FX0A instruction
    key_wait_register: Option<u8>,
    /// Optional audio support
    #[cfg(feature = "audio")]
    speaker: Option<Speaker>,
    quirks: Quirks,
}

impl Chip8 {
    /// Creates a new CHIP-8 emulator with default values
    pub fn new() -> Self {
        Self {
            window: VirtualDisplay::new(20),
            v_registers: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            program_counter: START_ADDR,
            index_register: 0,
            stack: Stack::new(),
            memory: Memory::new(),
            keys: [false; 16],
            key_wait_register: None,
            #[cfg(feature = "audio")]
            speaker: Speaker::new(),
            quirks: Default::default(),
        }
    }

    /// Consumes self and adds the given quirks, overriding anything set in the previous self
    pub fn override_quirks(mut self, quirks: Quirks) -> Self {
        self.quirks = quirks;
        self
    }

    /// Consumes self and adds the given additional quirks on top of the default ones
    pub fn add_quirks(mut self, quirks: Quirks) -> Self {
        self.quirks |= quirks;
        self
    }

    /// Returns a reference to the held window
    pub fn window(&self) -> &VirtualDisplay {
        &self.window
    }

    /// Ticks the CPU and runs the Von Neumann decode-execute cycle
    ///
    /// Note that this doesn't do anything if currently waiting on a keypress from the user. See
    /// [`Self::key_wait_register`]
    pub fn tick_cpu(&mut self) {
        // don't execute anything if waiting on a key release
        if self.key_wait_register.is_some() {
            log::trace!("Waiting for keypress, skipping CPU tick");
            return;
        }

        let opcode = self.fetch();
        self.execute(opcode);
    }

    /// Register a key as currently pressed within the emulator. Accepts a key index in the range of `0x0..=0xF`
    pub fn press_key(&mut self, key_index: usize) {
        if key_index > 0xF {
            log::warn!("Discarding out of range keypress: {}", key_index);
            return;
        }

        log::debug!("Pressing key: {}", key_index);
        self.keys[key_index] = true;
    }

    /// Register a key as currently released within the emulator. Accepts a key index in the range of `0x0..=0xF`
    pub fn release_key(&mut self, key_index: usize) {
        if key_index > 0xF {
            log::warn!("Discarding out of range key release: {}", key_index);
            return;
        }

        log::debug!("Releasing key: {}", key_index);
        self.keys[key_index] = false;

        if let Some(reg_x) = self.key_wait_register {
            log::debug!("Writing key index to register {}", reg_x);
            self.v_registers[reg_x as usize] = key_index as u8;
            self.key_wait_register = None;
        }
    }

    /// Load ROM data into the emulator. Does not clear previously loaded data.
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        self.memory[start..start + data.len()].copy_from_slice(data);
    }

    /// Tick the timers if they are greater than 0. This should happen at a rate of 60Hz
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
            log::trace!("Delay timer ticked, new value: {}", self.delay_timer);
        }

        if self.sound_timer > 0 {
            // timer is currently active

            #[cfg(feature = "audio")]
            if let Some(speaker) = &mut self.speaker && !speaker.is_playing() {
                speaker.start();
            }
            self.sound_timer -= 1;
            log::trace!("Sound timer ticked, new value: {}", self.sound_timer);
        } else {
            // timer is not active, check if buzz is disabled
            #[cfg(feature = "audio")]
            if let Some(speaker) = &mut self.speaker && speaker.is_playing() {
                speaker.stop();
            }
        }
    }

    /// Fetch the current instruction pointed to by [`Self::program_counter`] from memory
    fn fetch(&mut self) -> u16 {
        let high = self.memory[self.program_counter as usize] as u16;
        let low = self.memory[(self.program_counter + 1) as usize] as u16;
        let opcode = (high << 8) | low;
        self.program_counter += 2;

        opcode
    }

    /// Executes an instruction
    fn execute(&mut self, opcode: u16) {
        log::trace!("Executing opcode: 0x{:04x}", opcode);

        let bit1 = (opcode & 0xF000) >> 12;
        let bit2 = (opcode & 0x0F00) >> 8;
        let bit3 = (opcode & 0x00F0) >> 4;
        let bit4 = opcode & 0x000F;

        match (bit1, bit2, bit3, bit4) {
            (0, 0, 0, 0) => (),
            (0, 0, 0xE, 0) => {
                log::trace!("CLS");
                self.window.clear()
            }
            (0, 0, 0xE, 0xE) => {
                let addr = self.stack.pop();
                log::trace!("RET to 0x{:04x}", addr);
                self.program_counter = addr;
            }
            (1, _, _, _) => {
                let addr = opcode & 0xFFF;
                log::trace!("JP 0x{:04x}", addr);
                self.program_counter = addr;
            }
            (2, _, _, _) => {
                let addr = opcode & 0xFFF;
                log::trace!("CALL 0x{:04x}", addr);
                self.stack.push(self.program_counter);
                self.program_counter = addr;
            }
            (3, reg, _, _) => {
                let val = (opcode & 0xFF) as u8;
                log::trace!("SE V{:X}, {}", reg, val);
                if self.v_registers[reg as usize] == val {
                    self.program_counter += 2;
                }
            }
            (4, reg, _, _) => {
                let val = (opcode & 0xFF) as u8;
                log::trace!("SNE V{:X}, {}", reg, val);
                if self.v_registers[reg as usize] != val {
                    self.program_counter += 2;
                }
            }
            (5, reg_x, reg_y, 0) => {
                log::trace!("SE V{:X}, V{:X}", reg_x, reg_y);
                if self.v_registers[reg_x as usize] == self.v_registers[reg_y as usize] {
                    self.program_counter += 2;
                }
            }
            (6, reg, _, _) => {
                let val = (opcode & 0xFF) as u8;
                log::trace!("LD V{:X}, {}", reg, val);
                self.v_registers[reg as usize] = val;
            }
            (7, reg, _, _) => {
                let val = (opcode & 0xFF) as u8;
                log::trace!("ADD V{:X}, {}", reg, val);
                let value = &mut self.v_registers[reg as usize];
                *value = (*value).wrapping_add(val);
            }
            (8, reg_x, reg_y, 0) => {
                log::trace!("LD V{:X}, V{:X}", reg_x, reg_y);
                self.v_registers[reg_x as usize] = self.v_registers[reg_y as usize];
            }
            (8, reg_x, reg_y, 1) => {
                log::trace!("OR V{:X}, V{:X}", reg_x, reg_y);
                self.v_registers[reg_x as usize] |= self.v_registers[reg_y as usize];

                if self.quirks.contains(Quirks::VF_RESET) {
                    // quirk: reset the vF register to 0 on OR
                    self.v_registers[0xF] = 0;
                }
            }
            (8, reg_x, reg_y, 2) => {
                log::trace!("AND V{:X}, V{:X}", reg_x, reg_y);
                self.v_registers[reg_x as usize] &= self.v_registers[reg_y as usize];

                if self.quirks.contains(Quirks::VF_RESET) {
                    // quirk: reset the vF register to 0 on AND
                    self.v_registers[0xF] = 0;
                }
            }
            (8, reg_x, reg_y, 3) => {
                log::trace!("XOR V{:X}, V{:X}", reg_x, reg_y);
                self.v_registers[reg_x as usize] ^= self.v_registers[reg_y as usize];

                if self.quirks.contains(Quirks::VF_RESET) {
                    // quirk: reset the vF register to 0 on XOR
                    self.v_registers[0xF] = 0;
                }
            }
            (8, reg_x, reg_y, 4) => {
                log::trace!("ADD V{:X}, V{:X}", reg_x, reg_y);
                let vx = self.v_registers[reg_x as usize];
                let vy = self.v_registers[reg_y as usize];

                self.v_registers[reg_x as usize] = vx.wrapping_add(vy);
                self.v_registers[0xF] = vx.checked_add(vy).is_none().into();
            }
            (8, reg_x, reg_y, 5) => {
                log::trace!("SUB V{:X}, V{:X}", reg_x, reg_y);
                let vx = self.v_registers[reg_x as usize];
                let vy = self.v_registers[reg_y as usize];

                let (value, overflow) = vx.overflowing_sub(vy);

                self.v_registers[reg_x as usize] = value;
                self.v_registers[0xF] = (!overflow).into();
            }
            (8, reg_x, reg_y, 6) => {
                log::trace!("SHR V{:X}", reg_x);

                let shifted = if self.quirks.contains(Quirks::SHIFTING) {
                    // shifting quirk: only modifies vX
                    let shifted_x = self.v_registers[reg_x as usize] >> 1;
                    self.v_registers[reg_x as usize] = shifted_x;
                    shifted_x
                } else {
                    // normal behavior: sets vX to vY and then shifts vX
                    let shifted_y = self.v_registers[reg_y as usize] >> 1;
                    self.v_registers[reg_x as usize] = shifted_y;
                    shifted_y
                };

                // overflow register gets the least significant bit since it's the one chopped off
                self.v_registers[0xF] = shifted & 1;
            }
            (8, reg_x, reg_y, 7) => {
                log::trace!("SUBN V{:X}, V{:X}", reg_x, reg_y);
                let vx = self.v_registers[reg_x as usize];
                let vy = self.v_registers[reg_y as usize];

                let (new_value, overflow) = vy.overflowing_sub(vx);

                self.v_registers[reg_x as usize] = new_value;
                self.v_registers[0xF] = (!overflow).into();
            }
            (8, reg_x, reg_y, 0xE) => {
                log::trace!("SHL V{:X}", reg_x);

                let shifted = if self.quirks.contains(Quirks::SHIFTING) {
                    // shifting quirk: only modifies vX
                    let shifted_x = self.v_registers[reg_x as usize] << 1;
                    self.v_registers[reg_x as usize] = shifted_x;
                    shifted_x
                } else {
                    // normal behavior: sets vX to vY and then shifts vX
                    let shifted_y = self.v_registers[reg_y as usize] << 1;
                    self.v_registers[reg_x as usize] = shifted_y;
                    shifted_y
                };

                // set overflow register to most significant bit
                self.v_registers[0xF] = (shifted >> 7) & 1;
            }

            (9, reg_x, reg_y, 0) => {
                log::trace!("SNE V{:X}, V{:X}", reg_x, reg_y);
                if self.v_registers[reg_x as usize] != self.v_registers[reg_y as usize] {
                    self.program_counter += 2;
                }
            }

            (0xA, _, _, _) => {
                let val = opcode & 0xFFF;
                log::trace!("LD I, 0x{:04x}", val);
                self.index_register = val;
            }

            (0xB, reg_x, _, _) => {
                let val = opcode & 0xFFF;
                log::trace!("JP V0, 0x{:04x}", val);

                let reg_value = if self.quirks.contains(Quirks::JUMPING) {
                    // jumping quirk: adds vX instead of v0
                    self.v_registers[reg_x as usize]
                } else {
                    // normal behavior: adds v0
                    self.v_registers[0]
                };

                self.program_counter = reg_value as u16 + val;
            }

            (0xC, reg_x, _, _) => {
                let val = (opcode & 0xFF) as u8;
                let random_byte = rand::random::<u8>();
                log::trace!("RND V{:X}, {}", reg_x, val);
                self.v_registers[reg_x as usize] = random_byte & val;
            }

            (0xD, reg_x, reg_y, n) => {
                let x_coord = self.v_registers[reg_x as usize];
                let y_coord = self.v_registers[reg_y as usize];
                log::trace!(
                    "DRW V{:X}, V{:X}, {} (draw {} rows at ({}, {}))",
                    reg_x, reg_y, n, n, x_coord, y_coord
                );

                let sprite_addr = self.index_register as usize;
                let num_rows = n as usize;
                let sprite = &self.memory[sprite_addr..sprite_addr + num_rows];

                if self
                    .window
                    .draw_sprite(x_coord as usize, y_coord as usize, num_rows, sprite, self.quirks.contains(Quirks::CLIPPING))
                {
                    self.v_registers[0xF] = 1;
                } else {
                    self.v_registers[0xF] = 0;
                }
            }

            (0xE, reg_x, 9, 0xE) => {
                log::trace!("SKP V{:X}", reg_x);
                if self.keys[self.v_registers[reg_x as usize] as usize] {
                    self.program_counter += 2;
                }
            }

            (0xE, reg_x, 0xA, 1) => {
                log::trace!("SKNP V{:X}", reg_x);
                if !self.keys[self.v_registers[reg_x as usize] as usize] {
                    self.program_counter += 2;
                }
            }

            (0xF, reg_x, 0, 7) => {
                log::trace!("LD V{:X}, DT", reg_x);
                self.v_registers[reg_x as usize] = self.delay_timer;
            }

            (0xF, reg_x, 0, 0xA) => {
                log::trace!("LD V{:X}, K (waiting for key)", reg_x);
                self.key_wait_register = Some(reg_x as u8);
            }

            (0xF, reg_x, 1, 5) => {
                log::trace!("LD DT, V{:X}", reg_x);
                self.delay_timer = self.v_registers[reg_x as usize];
            }

            (0xF, reg_x, 1, 8) => {
                log::trace!("LD ST, V{:X}", reg_x);
                self.sound_timer = self.v_registers[reg_x as usize];
            }

            (0xF, reg_x, 1, 0xE) => {
                log::trace!("ADD I, V{:X}", reg_x);
                self.index_register += self.v_registers[reg_x as usize] as u16;
            }

            (0xF, reg_x, 2, 9) => {
                log::trace!("LD F, V{:X}", reg_x);
                self.index_register = self.v_registers[reg_x as usize] as u16 * 5;
            }

            (0xF, reg_x, 3, 3) => {
                log::trace!("LD B, V{:X}", reg_x);
                let vx = self.v_registers[reg_x as usize];
                let i = self.index_register as usize;

                self.memory[i] = vx / 100;
                self.memory[i + 1] = (vx / 10) % 10;
                self.memory[i + 2] = vx % 10;
            }

            (0xF, reg_x, 5, 5) => {
                log::trace!("LD [I], V{:X}", reg_x);
                for reg in 0..=reg_x {
                    let addr = (self.index_register + reg) as usize;
                    self.memory[addr] = self.v_registers[reg as usize];
                }

                if self.quirks.contains(Quirks::MEMORY) {
                    // quirk: save and load opcodes increment the index register
                    self.index_register += reg_x + 1;
                }
            }

            (0xF, reg_x, 6, 5) => {
                log::trace!("LD V{:X}, [I]", reg_x);
                for reg in 0..=reg_x {
                    let addr = (self.index_register + reg) as usize;
                    self.v_registers[reg as usize] = self.memory[addr];
                }

                if self.quirks.contains(Quirks::MEMORY) {
                    // quirk: save and load opcodes increment the index register
                    self.index_register += reg_x + 1;
                }
            }

            (_, _, _, _) => log::error!("Unimplemented opcode: 0x{:04x}", opcode),
        }
    }
}
