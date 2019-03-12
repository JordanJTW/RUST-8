pub struct Cpu {
    pc: usize,
    reg: [u8; 16],
    memory: Vec<u8>,
}

impl Cpu {
    pub fn new(memory: Vec<u8>) -> Cpu {
        Cpu {
            pc: 0,
            reg: [0; 16],
            memory: memory,
        }
    }

    pub fn tick(&mut self) {
        let instruction = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        self.execute(instruction);
        self.pc = self.pc + 2;
    }

    fn execute(&mut self, instruction: u16) {
        let nnn = instruction & 0xFFF;
        let nn = (instruction & 0xFF) as u8;
        let x = ((instruction >> 8) & 0xF) as usize;
        let y = ((instruction >> 4) & 0xF) as usize;

        let instruction_exploded = (
            (instruction >> 12) & 0xF,
            (instruction >> 8) & 0xF,
            (instruction >> 4) & 0xF,
            (instruction >> 0) & 0xF,
        );

        println!("Executing instruction: 0x{:04X}", instruction);

        match instruction_exploded {
            (0x0, 0x0, 0xE, 0x0) => panic!("Clear Screen"),
            (0x0, 0x0, 0xE, 0xE) => panic!("Return from subroutine"),
            (0x0, _, _, _) => panic!("Calls to RCA 1802"),
            // 0x1NNN: goto NNN
            (0x1, _, _, _) => {
                println!("goto 0x{:03X}", nnn);
                self.pc = nnn as usize;
            }
            // 0x2NNN: Calls subroutine at NNN
            (0x2, _, _, _) => panic!("Call: 0x{:03X}()", nnn),
            // 0x3XNN: Skips next instruction if VX equals NN
            (0x3, _, _, _) => {
                println!("Skip if Vx == NN");
                if self.reg[x] == nn {
                    self.pc = self.pc + 2;
                }
            }
            // 0x4XNN: Skips next instruction if VX does not equals NN
            (0x4, _, _, _) => {
                println!("Skip if Vx != NN");
                if self.reg[x] != nn {
                    self.pc = self.pc + 2;
                }
            }
            // 0x5XY0: Skips next instruction if VX equals VY
            (0x5, _, _, 0x0) => {
                println!("Skip if Vx == Vy");
                if self.reg[x] == self.reg[y] {
                    self.pc = self.pc + 2;
                }
            }
            // 0x6XNN: Sets VX to NN
            (0x6, _, _, _) => {
                println!("Vx = NN");
                self.reg[x] = nn;
            }
            // 0x7XNN: Adds NN to VX (Carry flag is not changed)
            (0x7, _, _, _) => {
                println!("Vx += NN");
                self.reg[x] = self.reg[x] + nn;
            }
            // 0x8XY0: Asigns VX to the value of VY
            (0x8, _, _, 0x0) => {
                println!("Vx = Vy");
                self.reg[x] = self.reg[y];
            }
            // 0x8XY1: Sets VX to VX or VY (Bitwise OR operation)
            (0x8, _, _, 0x1) => {
                println!("Vx = Vx | Vy");
                self.reg[x] = self.reg[x] | self.reg[y];
            }
            // 0x8XY2: Sets VX to VX and VY (Bitwise AND operation)
            (0x8, _, _, 0x2) => {
                println!("Vx = Vx & Vy");
                self.reg[x] = self.reg[x] & self.reg[y];
            }
            // 0x8XY3: Sets VX to VX xor VY
            (0x8, _, _, 0x3) => {
                println!("Vx = Vx ^ Vy");
                self.reg[x] = self.reg[x] ^ self.reg[y];
            }
            // 0x8XY4: Adds VY to VX; VF is set to 1 when there's a carry,
            //         and to 0 when there isn't
            (0x8, _, _, 0x4) => {
                panic!("Vx += Vy");
                self.reg[x] = self.reg[x] + self.reg[y];
            }
            // 0x8XY5: VY is subtracted from VX; VF is set to 0 when there's
            //         a borrow, and 1 when there isn't
            (0x8, _, _, 0x5) => {
                panic!("Vx -= Vy");
                self.reg[x] = self.reg[x] - self.reg[y];
            }
            // 0x8XY6: Stores the least significant bit of VX in VF and then
            //         shifts VX to the right by 1
            (0x8, _, _, 0x6) => panic!("Vx >>= 1"),
            // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a
            //         borrow, and 1 when there isn't
            (0x8, _, _, 0x7) => {
                panic!("Vx = Vy - Vx");
                self.reg[x] = self.reg[y] - self.reg[x];
            }
            // 0x8XYE: Stores the most significant bit of VX in VF and then
            //         shifts VX to the left by 1
            (0x8, _, _, 0xE) => panic!("Vx <<= 1"),
            // 0x9XY0: Skips the next instruction if VX doesn't equal VY
            (0x9, _, _, 0x0) => {
                println!("Skip if Vx != Vy");
                if self.reg[x] != self.reg[y] {
                    self.pc = self.pc + 2;
                }
            }
            // 0xANNN: Sets I to the address NNN
            (0xA, _, _, _) => panic!("I = NNN"),
            // 0xBNNN: Jumps to the address NNN plus V0
            (0xB, _, _, _) => {
                println!("PC = V0 + NNN");
                self.pc = (self.reg[0] as u16 + nnn) as usize;
            }
            // 0xCXNN: Sets VX to the result of a bitwise and operation on a
            //         random number (Typically: 0 to 255) and NN
            (0xC, _, _, _) => panic!("Vx = rand() & NN"),
            // 0xDXYN: Draws a sprite at coordinate (VX, VY) that has a width
            //         of 8 pixels and a height of N pixels. Each row of 8
            //         pixels is read as bit-coded starting from memory location
            //         I; I value doesn’t change after the execution of this
            //         instruction. As described above, VF is set to 1 if any
            //         screen pixels are flipped from set to unset when the
            //         sprite is drawn, and to 0 if that doesn’t happen
            (0xD, _, _, _) => panic!("draw(Vx,Vy,N)"),
            // 0xEX9E: Skips the next instruction if the key stored in VX is
            //         pressed
            (0xE, _, 0x9, 0xE) => panic!("Skip if key() == Vx"),
            // 0xEXA1: Skips the next instruction if the key stored in VX is
            //         not pressed
            (0xE, _, 0xA, 0x1) => panic!("Skip if key() != Vx"),
            // 0xFX07: Sets VX to the value of the delay timer
            (0xF, _, 0x0, 0x7) => panic!("Vx = delay_timer()"),
            // 0xFX0A: A key press is awaited, and then stored in VX. (Blocking
            //         Operation. All instruction halted until next key event)
            (0xF, _, 0x0, 0xA) => {
                println!("Vx = get_key()");
            }
            // 0xFX15: Sets the delay timer to Vx
            (0xF, _, 0x1, 0x5) => panic!("delay_timer(Vx)"),
            // 0xFX18: Sets the sound timer to VX
            (0xF, _, 0x1, 0x8) => panic!("sound_timer(Vx)"),
            // 0xFX1E: Adds VX to I
            (0xF, _, 0x1, 0xE) => panic!("I += Vx"),
            // 0xFX29: Sets I to the location of the sprite for the character
            //         in VX. Characters 0-F (in hexadecimal) are represented
            //         by a 4x5 font
            (0xF, _, 0x2, 0x9) => panic!("I = sprite_addr[Vx]"),
            // 0xFX33: Stores the binary-coded decimal representation of VX,
            //         with the most significant of three digits at the address
            //         in I, the middle digit at I plus 1, and the least
            //         significant digit at I plus 2. (In other words, take
            //         the decimal representation of VX, place the hundreds
            //         digit in memory at location in I, the tens digit at
            //         location I+1, and the ones digit at location I+2.)
            (0xF, _, 0x3, 0x3) => panic!("Store BCD"),
            // 0xFX55: Stores V0 to VX (including VX) in memory starting at
            //         address I. The offset from I is increased by 1 for each
            //         value written, but I itself is left unmodified
            (0xF, _, 0x5, 0x5) => panic!("Store V0-X to address I"),
            // 0xFX65: Fills V0 to VX (including VX) with values from memory
            //         starting at address I. The offset from I is increased by
            //         1 for each value written, but I itself is left unmodified
            (0xF, _, 0x6, 0x5) => panic!("Load V0-X from address I"),
            (_, _, _, _) => panic!("Unknown instruction: 0x{:04X}", instruction),
        };
    }
}
