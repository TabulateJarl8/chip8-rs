use crate::{memory::Memory, sound::Speaker, stack::Stack, virtual_buffer::VirtualDisplay};

const START_ADDR: u16 = 0x200;

#[derive(Debug)]
pub struct Chip8 {
    memory: Memory,
    v_registers: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    program_counter: u16,
    index_register: u16,
    stack: Stack,
    window: VirtualDisplay,
    keys: [bool; 16],
    key_wait_register: Option<u8>,
    speaker: Option<Speaker>,
}

impl Chip8 {
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
            speaker: Speaker::new(),
        }
    }

    pub fn window(&self) -> &VirtualDisplay {
        &self.window
    }

    pub fn tick_cpu(&mut self) {
        // don't execute anything if waiting on a key release
        if self.key_wait_register.is_some() {
            return;
        }

        let opcode = self.fetch();
        self.execute(opcode);
    }

    pub fn press_key(&mut self, key_index: usize) {
        if key_index > 0xF {
            log::warn!("Discarding out of range keypress: {}", key_index);
            return;
        }

        log::debug!("Pressing key: {}", key_index);
        self.keys[key_index] = true;
    }

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

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        self.memory[start..start + data.len()].copy_from_slice(data);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            // timer is currently active

            if let Some(speaker) = &mut self.speaker && !speaker.is_playing() {
                speaker.start();
            }
            self.sound_timer -= 1;
        } else {
            // timer is not active, check if buzz is disabled
            if let Some(speaker) = &mut self.speaker && speaker.is_playing() {
                speaker.stop();
            }
        }
    }

    fn fetch(&mut self) -> u16 {
        let high = self.memory[self.program_counter as usize] as u16;
        let low = self.memory[(self.program_counter + 1) as usize] as u16;
        let opcode = (high << 8) | low;
        self.program_counter += 2;

        opcode
    }

    fn execute(&mut self, opcode: u16) {
        // log::debug!("PC: {}", self.program_counter);
        // log::debug!("Executing opcode: 0x{:04x}", opcode);

        let bit1 = (opcode & 0xF000) >> 12;
        let bit2 = (opcode & 0x0F00) >> 8;
        let bit3 = (opcode & 0x00F0) >> 4;
        let bit4 = opcode & 0x000F;

        match (bit1, bit2, bit3, bit4) {
            (0, 0, 0, 0) => (),
            (0, 0, 0xE, 0) => self.window.clear(),
            (0, 0, 0xE, 0xE) => self.program_counter = self.stack.pop(),
            (1, _, _, _) => {
                self.program_counter = opcode & 0xFFF;
            }
            (2, _, _, _) => {
                self.stack.push(self.program_counter);
                self.program_counter = opcode & 0xFFF;
            }
            (3, reg, _, _) => {
                if self.v_registers[reg as usize] as u16 == opcode & 0xFF {
                    self.program_counter += 2;
                }
            }
            (4, reg, _, _) => {
                if self.v_registers[reg as usize] as u16 != opcode & 0xFF {
                    self.program_counter += 2;
                }
            }
            (5, reg_x, reg_y, 0) => {
                if self.v_registers[reg_x as usize] == self.v_registers[reg_y as usize] {
                    self.program_counter += 2;
                }
            }
            (6, reg, _, _) => {
                self.v_registers[reg as usize] = (opcode & 0xFF) as u8;
            }
            (7, reg, _, _) => {
                let value = &mut self.v_registers[reg as usize];
                *value = (*value).wrapping_add((opcode & 0xFF) as u8);
            }
            (8, reg_x, reg_y, 0) => {
                self.v_registers[reg_x as usize] = self.v_registers[reg_y as usize];
            }
            (8, reg_x, reg_y, 1) => {
                self.v_registers[reg_x as usize] |= self.v_registers[reg_y as usize];
            }
            (8, reg_x, reg_y, 2) => {
                self.v_registers[reg_x as usize] &= self.v_registers[reg_y as usize];
            }
            (8, reg_x, reg_y, 3) => {
                self.v_registers[reg_x as usize] ^= self.v_registers[reg_y as usize];
            }
            (8, reg_x, reg_y, 4) => {
                let vx = self.v_registers[reg_x as usize];
                let vy = self.v_registers[reg_y as usize];

                self.v_registers[0xF] = vx.checked_add(vy).is_none().into();
                self.v_registers[reg_x as usize] = vx.wrapping_add(vy);
            }
            (8, reg_x, reg_y, 5) => {
                let vx = self.v_registers[reg_x as usize];
                let vy = self.v_registers[reg_y as usize];

                self.v_registers[0xF] = (vx > vy).into();
                self.v_registers[reg_x as usize] = vx.wrapping_sub(vy);
            }
            (8, reg_x, _, 6) => {
                let vx = self.v_registers[reg_x as usize];

                // overflow register gets the least significant bit since it's the one chopped off
                self.v_registers[0xF] = vx & 1;
                self.v_registers[reg_x as usize] >>= 1;
            }
            (8, reg_x, reg_y, 7) => {
                let vx = self.v_registers[reg_x as usize];
                let vy = self.v_registers[reg_y as usize];

                let (new_value, overflow) = vy.overflowing_sub(vx);

                self.v_registers[0xF] = (!overflow).into();
                self.v_registers[reg_x as usize] = new_value;
            }
            (8, reg_x, _, 0xE) => {
                let vx = self.v_registers[reg_x as usize];

                // set overflow register to most significant bit
                self.v_registers[0xF] = (vx >> 7) & 1;
                self.v_registers[reg_x as usize] <<= 1;
            }

            (9, reg_x, reg_y, 0) => {
                if self.v_registers[reg_x as usize] != self.v_registers[reg_y as usize] {
                    self.program_counter += 2;
                }
            }

            (0xA, _, _, _) => {
                self.index_register = opcode & 0xFFF;
            }

            (0xB, _, _, _) => {
                self.program_counter = self.v_registers[0] as u16 + (opcode & 0xFFF);
            }

            (0xC, reg_x, _, _) => {
                self.v_registers[reg_x as usize] = rand::random::<u8>() & (opcode & 0xFF) as u8;
            }

            (0xD, reg_x, reg_y, n) => {
                let sprite_addr = self.index_register as usize;
                let num_rows = n as usize;
                let sprite = &self.memory[sprite_addr..sprite_addr + num_rows];

                let x_coord = self.v_registers[reg_x as usize];
                let y_coord = self.v_registers[reg_y as usize];

                log::debug!(
                    "Drawing sprite of length {} at {}, {}",
                    sprite.len(),
                    x_coord,
                    y_coord,
                );

                if self
                    .window
                    .draw_sprite(x_coord as usize, y_coord as usize, num_rows, sprite)
                {
                    self.v_registers[0xF] = 1;
                } else {
                    self.v_registers[0xF] = 0;
                }
            }

            (0xE, reg_x, 9, 0xE) => {
                if self.keys[self.v_registers[reg_x as usize] as usize] {
                    self.program_counter += 2;
                }
            }

            (0xE, reg_x, 0xA, 1) => {
                if !self.keys[self.v_registers[reg_x as usize] as usize] {
                    self.program_counter += 2;
                }
            }

            (0xF, reg_x, 0, 7) => {
                self.v_registers[reg_x as usize] = self.delay_timer;
            }

            (0xF, reg_x, 0, 0xA) => self.key_wait_register = Some(reg_x as u8),

            (0xF, reg_x, 1, 5) => {
                self.delay_timer = self.v_registers[reg_x as usize];
            }

            (0xF, reg_x, 1, 8) => {
                self.sound_timer = self.v_registers[reg_x as usize];
            }

            (0xF, reg_x, 1, 0xE) => {
                self.index_register += self.v_registers[reg_x as usize] as u16;
            }

            (0xF, reg_x, 2, 9) => {
                self.index_register = self.v_registers[reg_x as usize] as u16 * 5;
            }

            (0xF, reg_x, 3, 3) => {
                let vx = self.v_registers[reg_x as usize];
                let i = self.index_register as usize;

                self.memory[i] = vx / 100;
                self.memory[i + 1] = (vx / 10) % 10;
                self.memory[i + 2] = vx % 10;
            }

            (0xF, reg_x, 5, 5) => {
                for reg in 0..=reg_x {
                    let addr = (self.index_register + reg) as usize;
                    self.memory[addr] = self.v_registers[reg as usize];
                }
            }

            (0xF, reg_x, 6, 5) => {
                for reg in 0..=reg_x {
                    let addr = (self.index_register + reg) as usize;
                    self.v_registers[reg as usize] = self.memory[addr];
                }
            }

            (_, _, _, _) => unimplemented!("Unimplemented opcode: 0x{:04x}", opcode),
        }
    }
}
