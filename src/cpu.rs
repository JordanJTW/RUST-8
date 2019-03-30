// Copyright of Jordan Werthman (2019).

use log::*;

use crate::bus::Bus;
use crate::memory::Memory;
use crate::memory::USER_OFFSET;

// Models the CHIP-8 processor
//
// Given |memory| this processor interprets instructions in a ROM and
// executes one each tick() of the CPU. On each tick() the CPU interacts
// with I/O over the |bus|. The CHIP-8 CPU has 35 opcodes in total with
// an explaination of each found at: https://en.wikipedia.org/wiki/CHIP-8

enum PcOp {
    Next,        // Moves PC to the next instruction in memory
    Skip,        // Skips the current and next instruction
    Jump(usize), // Jumps to the given location in memory
    Halt,        // Remains on the current instruction
}

const REG_COUNT: usize = 16;

pub struct Cpu {
    pc: usize,
    reg: [u8; REG_COUNT],
    memory: Memory,
    i: usize,
}

impl Cpu {
    pub fn new(rom: &Vec<u8>) -> Cpu {
        Cpu {
            pc: USER_OFFSET,
            reg: [0; REG_COUNT],
            memory: Memory::new(rom),
            i: 0,
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        let instruction = self.memory.read_instruction(self.pc);
        match self.execute(instruction, bus) {
            PcOp::Next => self.pc = self.pc + 2,
            PcOp::Skip => self.pc = self.pc + 4,
            PcOp::Jump(addr) => self.pc = addr,
            PcOp::Halt => (),
        }
    }

    fn execute(&mut self, instruction: u16, bus: &mut Bus) -> PcOp {
        let nnn = instruction & 0xFFF;
        let nn = (instruction & 0xFF) as u8;
        let n = (instruction & 0xF) as u8;
        let x = ((instruction >> 8) & 0xF) as usize;
        let y = ((instruction >> 4) & 0xF) as usize;

        let instruction_exploded = (
            (instruction >> 12) & 0xF,
            (instruction >> 8) & 0xF,
            (instruction >> 4) & 0xF,
            (instruction >> 0) & 0xF,
        );

        info!("PC: {} Executing instruction: 0x{:04X}", self.pc, instruction);

        match instruction_exploded {
            // 0x00E0: Clears the screen
            (0x0, 0x0, 0xE, 0x0) => {
                info!("Clear Screen");
                bus.clear_display();
            }
            // 0x00EE: Returns from a subroutine
            (0x0, 0x0, 0xE, 0xE) => {
                info!("Return from subroutine");
                return PcOp::Jump(self.memory.pop_stack());
            }
            (0x0, _, _, _) => panic!("Calls to RCA 1802"),
            // 0x1NNN: goto NNN
            (0x1, _, _, _) => {
                info!("goto 0x{:03X}", nnn);
                return PcOp::Jump(nnn as usize);
            }
            // 0x2NNN: Calls subroutine at NNN
            (0x2, _, _, _) => {
                info!("Call: 0x{:03X}()", nnn);
                // Add 2 to the current PC so that we return to the instruction after
                // the current one; otherwise we end up calling the subroutine again.
                self.memory.push_stack(self.pc + 2);
                return PcOp::Jump(nnn as usize);
            }
            // 0x3XNN: Skips next instruction if VX equals NN
            (0x3, _, _, _) => {
                info!("Skip if Vx == NN");
                if self.reg[x] == nn {
                    return PcOp::Skip;
                }
            }
            // 0x4XNN: Skips next instruction if VX does not equals NN
            (0x4, _, _, _) => {
                info!("Skip if Vx != NN");
                if self.reg[x] != nn {
                    return PcOp::Skip;
                }
            }
            // 0x5XY0: Skips next instruction if VX equals VY
            (0x5, _, _, 0x0) => {
                info!("Skip if Vx == Vy");
                if self.reg[x] == self.reg[y] {
                    return PcOp::Skip;
                }
            }
            // 0x6XNN: Sets VX to NN
            (0x6, _, _, _) => {
                info!("Vx = NN");
                self.reg[x] = nn;
            }
            // 0x7XNN: Adds NN to VX (Carry flag is not changed)
            (0x7, _, _, _) => {
                info!("Vx += NN");
                self.reg[x] = self.reg[x].wrapping_add(nn);
            }
            // 0x8XY0: Asigns VX to the value of VY
            (0x8, _, _, 0x0) => {
                info!("Vx = Vy");
                self.reg[x] = self.reg[y];
            }
            // 0x8XY1: Sets VX to VX or VY (Bitwise OR operation)
            (0x8, _, _, 0x1) => {
                info!("Vx = Vx | Vy");
                self.reg[x] = self.reg[x] | self.reg[y];
            }
            // 0x8XY2: Sets VX to VX and VY (Bitwise AND operation)
            (0x8, _, _, 0x2) => {
                info!("Vx = Vx & Vy");
                self.reg[x] = self.reg[x] & self.reg[y];
            }
            // 0x8XY3: Sets VX to VX xor VY
            (0x8, _, _, 0x3) => {
                info!("Vx = Vx ^ Vy");
                self.reg[x] = self.reg[x] ^ self.reg[y];
            }
            // 0x8XY4: Adds VY to VX; VF is set to 1 when there's a carry,
            //         and to 0 when there isn't i.e. the flag is set if the
            //         result would exceed the max value of u8 (255).
            (0x8, _, _, 0x4) => {
                info!("Vx += Vy");
                self.reg[x] = self.reg[x].wrapping_add(self.reg[y]);

                let value: u16 = self.reg[x] as u16 + self.reg[y] as u16;
                self.reg[0xF] = if value > 255 { 1 } else { 0 };
            }
            // 0x8XY5: VY is subtracted from VX; VF is set to 0 when there's
            //         a borrow, and 1 when there isn't i.e. the flag is set if
            //         the result of the subtraction would be negative.
            (0x8, _, _, 0x5) => {
                info!("Vx -= Vy");
                self.reg[0xF] = if self.reg[x] > self.reg[y] { 1 } else { 0 };
                self.reg[x] = self.reg[x].wrapping_sub(self.reg[y]);
            }
            // 0x8XY6: Stores the least significant bit of VX in VF and then
            //         shifts VX to the right by 1
            (0x8, _, _, 0x6) => {
                info!("Vx >>= 1");
                self.reg[0x0f] = self.reg[x] & 1;
                self.reg[x] >>= 1;
            }
            // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a
            //         borrow, and 1 when there isn't i.e. the flag is set if
            //         the result of the subtraction would be negative.
            (0x8, _, _, 0x7) => {
                info!("Vx = Vy - Vx");
                self.reg[0xF] = if self.reg[y] > self.reg[x] { 1 } else { 0 };
                self.reg[x] = self.reg[y].wrapping_sub(self.reg[x]);
            }
            // 0x8XYE: Stores the most significant bit of VX in VF and then
            //         shifts VX to the left by 1
            (0x8, _, _, 0xE) => {
                info!("Vx <<= 1");
                self.reg[0x0f] = (self.reg[x] & 0x80) >> 7;
                self.reg[x] <<= 1;
            }
            // 0x9XY0: Skips the next instruction if VX doesn't equal VY
            (0x9, _, _, 0x0) => {
                info!("Skip if Vx != Vy");
                if self.reg[x] != self.reg[y] {
                    return PcOp::Skip;
                }
            }
            // 0xANNN: Sets I to the address NNN
            (0xA, _, _, _) => {
                info!("I = NNN");
                self.i = nnn as usize;
            }
            // 0xBNNN: Jumps to the address NNN plus V0
            (0xB, _, _, _) => {
                info!("PC = V0 + NNN");
                return PcOp::Jump((self.reg[0] as u16 + nnn) as usize);
            }
            // 0xCXNN: Sets VX to the result of a bitwise and operation on a
            //         random number (Typically: 0 to 255) and NN
            (0xC, _, _, _) => {
                info!("Vx = rand() & NN");
                self.reg[x] = 16; // rand::random::<u8>() & nn;
            }
            // 0xDXYN: Draws a sprite at coordinate (VX, VY) that has a width
            //         of 8 pixels and a height of N pixels. Each row of 8
            //         pixels is read as bit-coded starting from memory location
            //         I; I value doesn’t change after the execution of this
            //         instruction. As described above, VF is set to 1 if any
            //         screen pixels are flipped from set to unset when the
            //         sprite is drawn, and to 0 if that doesn’t happen
            (0xD, _, _, _) => {
                info!("draw(Vx,Vy,N)");
                let position = (self.reg[x] as usize, self.reg[y] as usize);
                let pixel_flip = bus.draw_display(&mut self.memory, self.i, position, n as usize);
                self.reg[0xf] = pixel_flip as u8;
            }
            // 0xEX9E: Skips the next instruction if the key stored in VX is
            //         pressed
            (0xE, _, 0x9, 0xE) => {
                info!("Skip if key() == Vx");
                if bus.check_key(self.reg[x]) {
                    return PcOp::Skip;
                }
            }
            // 0xEXA1: Skips the next instruction if the key stored in VX is
            //         not pressed
            (0xE, _, 0xA, 0x1) => {
                info!("Skip if key() != Vx");
                if !bus.check_key(self.reg[x]) {
                    return PcOp::Skip;
                }
            }
            // 0xFX07: Sets VX to the value of the delay timer
            (0xF, _, 0x0, 0x7) => {
                info!("Vx = delay_timer()");
                self.reg[x] = bus.delay_timer();
            }
            // 0xFX0A: A key press is awaited, and then stored in VX. (Blocking
            //         Operation. All instruction halted until next key event)
            (0xF, _, 0x0, 0xA) => {
                info!("Vx = get_key()");
                if let Some(key) = bus.any_key() {
                    info!("Set Vx to {:X}", key);
                    self.reg[x] = key;
                } else {
                    return PcOp::Halt;
                }
            }
            // 0xFX15: Sets the delay timer to Vx
            (0xF, _, 0x1, 0x5) => {
                info!("delay_timer(Vx)");
                bus.set_delay_timer(self.reg[x]);
            }
            // 0xFX18: Sets the sound timer to VX
            (0xF, _, 0x1, 0x8) => {
                info!("sound_timer(Vx)");
                bus.set_sound_timer(self.reg[x]);
            }
            // 0xFX1E: Adds VX to I
            (0xF, _, 0x1, 0xE) => {
                info!("I += Vx");
                self.i += self.reg[x] as usize;
            }
            // 0xFX29: Sets I to the location of the sprite for the character
            //         in VX. Characters 0-F (in hexadecimal) are represented
            //         by a 4x5 font
            (0xF, _, 0x2, 0x9) => {
                info!("I = sprite_addr[Vx]");
                self.i = self.reg[x] as usize * 5;
            }
            // 0xFX33: Stores the binary-coded decimal representation of VX,
            //         with the most significant of three digits at the address
            //         in I, the middle digit at I plus 1, and the least
            //         significant digit at I plus 2. (In other words, take
            //         the decimal representation of VX, place the hundreds
            //         digit in memory at location in I, the tens digit at
            //         location I+1, and the ones digit at location I+2.)
            (0xF, _, 0x3, 0x3) => {
                info!("Store BCD");
                let mut value = self.reg[x];
                for pos in 0..3 {
                    // Use integer division to separate each digit of |value|.
                    let magnitude = 10_u8.pow(2 - pos);
                    let digit = value / magnitude;

                    self.memory.data()[self.i + pos as usize] = digit;
                    value -= magnitude * digit;
                }
            }
            // 0xFX55: Stores V0 to VX (including VX) in memory starting at
            //         address I. The offset from I is increased by 1 for each
            //         value written, but I itself is left unmodified
            (0xF, _, 0x5, 0x5) => {
                info!("Store V0-X to address I");
                for pos in 0..x + 1 {
                    self.memory.data()[self.i + pos] = self.reg[pos];
                }
            }
            // 0xFX65: Fills V0 to VX (including VX) with values from memory
            //         starting at address I. The offset from I is increased by
            //         1 for each value written, but I itself is left unmodified
            (0xF, _, 0x6, 0x5) => {
                info!("Load V0-X from address I");
                for pos in 0..x + 1 {
                    self.reg[pos] = self.memory.data()[self.i + pos];
                }
            }
            (_, _, _, _) => panic!("Unknown instruction: 0x{:04X}", instruction),
        };

        PcOp::Next
    }
}
