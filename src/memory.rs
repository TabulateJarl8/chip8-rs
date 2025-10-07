use std::ops::{Index, IndexMut, Range};

/// The size of the CHIP-8 RAM
const MEMORY_SIZE: usize = 4096;

/// Font for characters `0x0`-`0xF`
const FONT_BYTES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// Represents the CHIP-8's memory
#[derive(Debug)]
pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Memory {
    /// Constructs a new [`Memory`] with the font loaded at address 0
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];
        // some programs expect font maps to be at 0x000
        memory[..FONT_BYTES.len()].copy_from_slice(&FONT_BYTES);

        Self { memory }
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        self.memory.index(index)
    }
}

impl Index<Range<usize>> for Memory {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        self.memory.index(index)
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.memory.index_mut(index)
    }
}

impl IndexMut<Range<usize>> for Memory {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        self.memory.index_mut(index)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
