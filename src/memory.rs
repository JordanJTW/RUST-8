// Copyright of Jordan Werthman (2019).

// Models the RAM used by the system.
//
// The memory is 4096 bytes with the first 512 bytes reserved for system use
// and the rest used for user memory and is loaded with the given ROM. In the
// reserved space [0x0-0x50) is used for the character graphics (numbers 0-F
// in hexidecimal) and [0x50-0x70) contains the call stack (saving return
// addressed) saving 16 16-byte addresses.

const FONT_OFFSET: usize = 0x0;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xf0, 0x80, 0xF0, // 2
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

const STACK_OFFSET: usize = 0x50;

pub const USER_OFFSET: usize = 0x200;

pub struct Memory {
    memory: [u8; 4096],
    stack_pointer: usize,
}

impl Memory {
    pub fn new(rom: &Vec<u8>) -> Memory {
        let mut memory = [0; 4096];
        for (i, byte) in FONT.iter().enumerate() {
            memory[FONT_OFFSET + i] = *byte;
        }

        for (i, byte) in rom.iter().enumerate() {
            memory[USER_OFFSET + i] = *byte;
        }

        Memory {
            memory: memory,
            stack_pointer: STACK_OFFSET,
        }
    }

    pub fn push_stack(&mut self, address: usize) {
        self.memory[self.stack_pointer] = ((address >> 8) & 0xFF) as u8;
        self.memory[self.stack_pointer + 1] = (address & 0xFF) as u8;
        self.stack_pointer += 2;
    }

    pub fn pop_stack(&mut self) -> usize {
        self.stack_pointer -= 2;
        let high_byte: usize = self.memory[self.stack_pointer] as usize;
        let low_byte: usize = self.memory[self.stack_pointer + 1] as usize;

        (high_byte << 8) | low_byte
    }

    pub fn read_instruction(&mut self, pc: usize) -> u16 {
        (self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16
    }

    pub fn data(&mut self) -> &mut [u8; 4096] {
        &mut self.memory
    }
}
