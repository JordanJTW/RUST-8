// Copyright of Jordan Werthman (2019).

#[derive(PartialEq)]
enum State {
    Running,
    KeyExpected,
}

pub struct Cpu {
    pc: usize,
    reg: [u8; 16],
    memory: Vec<u8>,
    last_key: Option<usize>,
    state: State,
    stack: [usize; 16],
    sp: usize,
    display: [bool; 64 * 32],
    keys: [bool; 16],
    i: usize,
    delay_timer: f64,
    sound_timer: f64,
}

impl Cpu {
    pub fn new(memory: Vec<u8>) -> Cpu {
        Cpu {
            pc: 0,
            reg: [0; 16],
            memory: memory,
            last_key: None,
            state: State::Running,
            stack: [0; 16],
            sp: 0,
            display: [false; 64 * 32],
            keys: [false; 16],
            i: 0,
            delay_timer: 0.0,
            sound_timer: 0.0,
        }
    }

    // TODO(jordanjtw): Remove |bool| hack to pause execution while debugging.
    pub fn tick(&mut self) -> bool {
        if self.state == State::Running {
            let instruction = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
            if self.execute(instruction) {
                self.pc = self.pc + 2;
                return true;
            }
        }
        return false;
    }

    pub fn display(&mut self) -> &[bool; 64 * 32] {
        return &self.display;
    }

    pub fn set_key(&mut self, key: usize) {
        self.last_key = Some(key);
        self.keys[key] = true;
        self.state = State::Running;
    }

    pub fn clear_key(&mut self, key: usize) {
        self.keys[key] = false;
    }

    pub fn update_timers(&mut self, dt: f64) {
        if self.delay_timer > 0.0 {
            self.delay_timer -= dt * 60.0;
        }
        if self.sound_timer > 0.0 {
            self.sound_timer -= dt * 60.0;
        }
        if self.delay_timer < 0.0 {
            self.delay_timer = 0.0;
        }
        if self.sound_timer < 0.0 {
            self.sound_timer = 0.0;
        }
    }

    fn execute(&mut self, instruction: u16) -> bool {
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

        println!("Executing instruction: 0x{:04X}", instruction);

        match instruction_exploded {
            (0x0, 0x0, 0xE, 0x0) => println!("Clear Screen"),
            (0x0, 0x0, 0xE, 0xE) => {
                println!("Return from subroutine");
                self.sp = self.sp - 1;
                self.pc = self.stack[self.sp];
            }
            (0x0, _, _, _) => panic!("Calls to RCA 1802"),
            // 0x1NNN: goto NNN
            (0x1, _, _, _) => {
                println!("goto 0x{:03X}", nnn);
                // TODO(jordanjtw): Clean-up this PC fiddling.
                self.pc = (nnn as usize - 2) - 0x200;
            }
            // 0x2NNN: Calls subroutine at NNN
            (0x2, _, _, _) => {
                println!("Call: 0x{:03X}()", nnn);
                self.stack[self.sp] = self.pc;
                self.sp = self.sp + 1;
                // TODO(jordanjtw): Clean-up this PC fiddling.
                self.pc = (nnn as usize - 2) - 0x200;
            }
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
                self.reg[x] = self.reg[x].wrapping_add(nn);
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
            //         and to 0 when there isn't i.e. the flag is set if the
            //         result would exceed the max value of u8 (255).
            (0x8, _, _, 0x4) => {
                println!("Vx += Vy");
                self.reg[x] = self.reg[x].wrapping_add(self.reg[y]);

                let value: u16 = self.reg[x] as u16 + self.reg[y] as u16;
                self.reg[0xF] = if value > 255 { 1 } else { 0 };
            }
            // 0x8XY5: VY is subtracted from VX; VF is set to 0 when there's
            //         a borrow, and 1 when there isn't i.e. the flag is set if
            //         the result of the subtraction would be negative.
            (0x8, _, _, 0x5) => {
                println!("Vx -= Vy");
                self.reg[0xF] = if self.reg[x] > self.reg[y] { 1 } else { 0 };
                self.reg[x] = self.reg[x].wrapping_sub(self.reg[y]);
            }
            // 0x8XY6: Stores the least significant bit of VX in VF and then
            //         shifts VX to the right by 1
            (0x8, _, _, 0x6) => panic!("Vx >>= 1"),
            // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a
            //         borrow, and 1 when there isn't i.e. the flag is set if
            //         the result of the subtraction would be negative.
            (0x8, _, _, 0x7) => {
                println!("Vx = Vy - Vx");
                self.reg[0xF] = if self.reg[y] > self.reg[x] { 1 } else { 0 };
                self.reg[x] = self.reg[y].wrapping_sub(self.reg[x]);
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
            (0xA, _, _, _) => {
                println!("I = NNN");
                self.i = nnn as usize - 0x200;
            }
            // 0xBNNN: Jumps to the address NNN plus V0
            (0xB, _, _, _) => {
                println!("PC = V0 + NNN");
                // TODO(jordanjtw): Clean-up this PC fiddling.
                self.pc = (self.reg[0] as u16 + nnn) as usize - 0x200 - 2;
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
            (0xD, _, _, _) => {
                println!("draw(Vx,Vy,N)");
                let (x, y) = (self.reg[x], self.reg[y]);
                println!("draw(X={}, Y={}, H={})", x, y, n);

                for dy in 0..n {
                    for dx in 0..8 {
                        let (x, y) = (x + dx, y + dy);
                        let byte: u8 = self.memory[self.i + dy as usize];

                        if x >= 64 || y >= 32 {
                            continue;
                        }

                        let index: usize = y as usize * 64 + x as usize;
                        self.display[index] = ((byte << dx) & 0x80) != 0;
                    }
                }

                self.print_board();
            }
           // 0xEX9E: Skips the next instruction if the key stored in VX is
            //         pressed
            (0xE, _, 0x9, 0xE) => {
                println!("Skip if key() == Vx");
                if self.keys[self.reg[x] as usize] {
                    self.pc = self.pc + 2;
                }
            }
            // 0xEXA1: Skips the next instruction if the key stored in VX is
            //         not pressed
            (0xE, _, 0xA, 0x1) => {
                println!("Skip if key() != Vx");
                if !self.keys[self.reg[x] as usize] {
                    self.pc = self.pc + 2;
                }
            }
            // 0xFX07: Sets VX to the value of the delay timer
            (0xF, _, 0x0, 0x7) => {
                println!("Vx = delay_timer() {}", self.delay_timer as u8);
                self.reg[x] = self.delay_timer as u8;
            }
            // 0xFX0A: A key press is awaited, and then stored in VX. (Blocking
            //         Operation. All instruction halted until next key event)
            (0xF, _, 0x0, 0xA) => {
                println!("Vx = get_key()");
                self.state = State::KeyExpected;
                if let Some(key) = self.last_key {
                    println!("Set Vx to {:X}", key as u8);
                    self.reg[x] = key as u8;
                    self.last_key = None;
                }
            }
            // 0xFX15: Sets the delay timer to Vx
            (0xF, _, 0x1, 0x5) => {
                println!("delay_timer(Vx) {}", self.reg[x]);
                self.delay_timer = self.reg[x] as f64;
            }
            // 0xFX18: Sets the sound timer to VX
            (0xF, _, 0x1, 0x8) => println!("sound_timer(Vx)"),
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
            (0xF, _, 0x3, 0x3) => {
                println!("Store BCD");
                let mut value = self.reg[x];
                for pos in 0..3 {
                    // Use integer division to separate each digit of |value|.
                    let magnitude = 10_u8.pow(2 - pos);
                    let digit = value / magnitude;

                    self.memory[self.i + pos as usize] = digit;
                    value -= magnitude * digit;
                }
            }
            // 0xFX55: Stores V0 to VX (including VX) in memory starting at
            //         address I. The offset from I is increased by 1 for each
            //         value written, but I itself is left unmodified
            (0xF, _, 0x5, 0x5) => {
                println!("Store V0-X to address I");
                for pos in 0..x {
                    self.memory[self.i + pos] = self.reg[pos];
                }
            }
            // 0xFX65: Fills V0 to VX (including VX) with values from memory
            //         starting at address I. The offset from I is increased by
            //         1 for each value written, but I itself is left unmodified
            (0xF, _, 0x6, 0x5) => {
                println!("Load V0-X from address I");
                for pos in 0..x {
                    self.reg[pos] = self.memory[self.i + pos];
                }
            }
            (_, _, _, _) => panic!("Unknown instruction: 0x{:04X}", instruction),
        };

        true
    }

    fn print_board(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                let index: usize = y as usize * 64 + x as usize;
                print!("{}", if self.display[index] { "#" } else { "_" });
            }
            println!();
        }
    }
}
