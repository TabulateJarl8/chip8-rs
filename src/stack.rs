#[derive(Debug)]
pub struct Stack {
    // CHIP-8 spec requires a stack that goes 16 levels deep
    memory: [u16; 16],
    stack_pointer: u8,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            memory: [0; 16],
            stack_pointer: 0,
        }
    }

    pub fn push(&mut self, value: u16) {
        self.memory[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.memory[self.stack_pointer as usize]
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}
