pub struct Chip8 {
    memory: [u8; 4096],
    v_registers: [u8; 15],
    delay_timer: u8,
    sound_timer: u8,
    program_counter: u16,
    stack_pointer: u8,
    index_register: u16,
    // TODO: change this
    stack: Vec<u8>,
}
