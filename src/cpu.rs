// Global constants matching the CHIP-8 hardware specification
const MEMORY_SIZE: usize = 4096;

// The first 512 bytes (0x000–0x1FF) were reserved for the original interpreter firmware.
// All user programs are loaded starting at 0x200.
const START_ADDRESS: u16 = 0x200;

// Standard CHIP-8 display resolution
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

// Built-in system font: 16 hexadecimal glyphs (0–F), each 5 bytes tall.
// Preloaded into memory at boot, starting at 0x050 per the CHIP-8 convention.
const FONT_SET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

/// Emulates the CHIP-8 CPU, memory, display, and I/O subsystems.
pub struct Cpu {
    /// 4096-byte flat memory address space (RAM + ROM mapped from 0x200)
    memory: [u8; MEMORY_SIZE],
    /// 16 general-purpose 8-bit data registers (V0–VF)
    v: [u8; 16],
    /// 16-bit Index Register (I): points to memory locations for sprite/font data
    i: u16,
    /// Program Counter: address of the next instruction to fetch
    pc: u16,
    /// 16-level Call Stack for subroutine return addresses
    stack: [u16; 16],
    /// Stack Pointer: index into the top of the call stack
    sp: u8,
    /// VRAM: 64x32 pixel display matrix. 0 = off (black), 1 = on (white)
    pub display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    /// I/O port map: boolean state of the 16-key hexadecimal keypad
    pub keypad: [bool; 16],
    /// General-purpose hardware timer, decremented at 60 Hz
    pub delay_timer: u8,
    /// Sound timer: buzzer is active while this value is greater than zero
    pub sound_timer: u8,
}

impl Cpu {
    /// Initializes the CPU to its power-on state and preloads the system font into memory.
    pub fn new() -> Self {
        let mut cpu = Self {
            memory: [0; MEMORY_SIZE],
            v: [0; 16],
            i: 0,
            pc: START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            display: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            keypad: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        };

        // Preload the built-in font set at the conventional address 0x050
        for (idx, &byte) in FONT_SET.iter().enumerate() {
            cpu.memory[0x050 + idx] = byte;
        }

        cpu
    }

    /// Copies ROM bytes into memory starting at 0x200 (the standard program entry point).
    pub fn load_rom(&mut self, data: &[u8]) {
        for (idx, &byte) in data.iter().enumerate() {
            let addr = START_ADDRESS as usize + idx;
            if addr < MEMORY_SIZE {
                self.memory[addr] = byte;
            }
        }
    }

    /// Decrements the hardware timers by one tick. Must be called at exactly 60 Hz.
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Buzzer signal would be emitted here in a full audio implementation
            }
            self.sound_timer -= 1;
        }
    }

    /// Executes one complete Fetch-Decode-Execute CPU cycle.
    pub fn tick(&mut self) {
        // --- FETCH ---
        // Read two consecutive bytes from memory and merge them into a 16-bit opcode
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte  = self.memory[(self.pc + 1) as usize] as u16;
        let opcode    = (high_byte << 8) | low_byte;

        // Advance the program counter before execution (each instruction is 2 bytes)
        self.pc += 2;

        // --- DECODE ---
        // Isolate the most significant nibble to identify the instruction family
        let nibble = (opcode & 0xF000) >> 12;

        // --- EXECUTE ---
        match nibble {
            0x0 => {
                if opcode == 0x00E0 {
                    // 00E0: CLS — Clear the display
                    self.display = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
                } else if opcode == 0x00EE {
                    // 00EE: RET — Return from subroutine
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
            },

            0x1 => {
                // 1NNN: JP addr — Unconditional jump to address NNN
                let addr = opcode & 0x0FFF;
                self.pc = addr;
            },

            0x2 => {
                // 2NNN: CALL addr — Push current PC onto the stack and jump to NNN
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                let addr = opcode & 0x0FFF;
                self.pc = addr;
            },

            0x3 => {
                // 3XNN: SE Vx, byte — Skip next instruction if Vx == NN
                let x  = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                if self.v[x] == nn { self.pc += 2; }
            },

            0x4 => {
                // 4XNN: SNE Vx, byte — Skip next instruction if Vx != NN
                let x  = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                if self.v[x] != nn { self.pc += 2; }
            },

            0x5 => {
                // 5XY0: SE Vx, Vy — Skip next instruction if Vx == Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] == self.v[y] { self.pc += 2; }
            },

            0x6 => {
                // 6XNN: LD Vx, byte — Load immediate value NN into register Vx
                let x  = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] = nn;
            },

            0x7 => {
                // 7XNN: ADD Vx, byte — Add NN to Vx (wraps on overflow, no carry flag)
                let x  = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] = self.v[x].wrapping_add(nn);
            },

            0x8 => {
                // 8XYN: ALU operations — sub-opcode N selects the operation
                let x      = ((opcode & 0x0F00) >> 8) as usize;
                let y      = ((opcode & 0x00F0) >> 4) as usize;
                let sub_op = opcode & 0x000F;

                match sub_op {
                    0x0 => self.v[x] = self.v[y],        // LD:  Vx = Vy
                    0x1 => self.v[x] |= self.v[y],        // OR:  Vx |= Vy
                    0x2 => self.v[x] &= self.v[y],        // AND: Vx &= Vy
                    0x3 => self.v[x] ^= self.v[y],        // XOR: Vx ^= Vy

                    0x4 => {
                        // ADD: Vx += Vy. VF = 1 on unsigned overflow, 0 otherwise.
                        let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x]    = result;
                        self.v[0xF] = if overflow { 1 } else { 0 };
                    },

                    0x5 => {
                        // SUB: Vx -= Vy. VF = 1 if Vx >= Vy (no borrow), 0 if borrow occurred.
                        let (result, underflow) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x]    = result;
                        self.v[0xF] = if underflow { 0 } else { 1 };
                    },

                    0x6 => {
                        // SHR: Logical shift right by 1. VF = evicted LSB.
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                    },

                    0x7 => {
                        // SUBN: Vx = Vy - Vx. VF = 1 if Vy >= Vx (no borrow).
                        let (result, underflow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x]    = result;
                        self.v[0xF] = if underflow { 0 } else { 1 };
                    },

                    0xE => {
                        // SHL: Logical shift left by 1. VF = evicted MSB.
                        self.v[0xF] = (self.v[x] >> 7) & 1;
                        self.v[x] <<= 1;
                    },

                    _ => {} // Reserved / undefined
                }
            },

            0x9 => {
                // 9XY0: SNE Vx, Vy — Skip next instruction if Vx != Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] != self.v[y] { self.pc += 2; }
            },

            0xA => {
                // ANNN: LD I, addr — Set the Index Register to NNN
                let addr = opcode & 0x0FFF;
                self.i = addr;
            },

            0xC => {
                // CXNN: RND Vx, byte — Vx = random_byte AND NN
                let x  = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                let rand_byte = rand::random::<u8>();
                self.v[x] = rand_byte & nn;
            },

            0xD => {
                // DXYN: DRW Vx, Vy, nibble — Draw an N-row sprite at (Vx, Vy).
                // Sprites are XOR'd onto the display. VF = 1 if any pixel is erased (collision).
                let x      = ((opcode & 0x0F00) >> 8) as usize;
                let y      = ((opcode & 0x00F0) >> 4) as usize;
                let height = (opcode & 0x000F) as u16;

                // Wrap origin coordinates to screen bounds
                let origin_x = self.v[x] as usize % DISPLAY_WIDTH;
                let origin_y = self.v[y] as usize % DISPLAY_HEIGHT;

                self.v[0xF] = 0; // Reset collision flag

                for row in 0..height {
                    let sprite_byte = self.memory[(self.i + row) as usize];

                    for col in 0..8usize {
                        let pixel = (sprite_byte >> (7 - col)) & 1;

                        if pixel == 1 {
                            let pixel_x = origin_x + col;
                            let pixel_y = origin_y + row as usize;

                            // Clip pixels that exceed the display boundary
                            if pixel_x < DISPLAY_WIDTH && pixel_y < DISPLAY_HEIGHT {
                                if self.display[pixel_y][pixel_x] == 1 {
                                    self.v[0xF] = 1; // Collision detected
                                }
                                self.display[pixel_y][pixel_x] ^= 1; // XOR pixel
                            }
                        }
                    }
                }
            },

            0xE => {
                // Keypad-conditional skip instructions
                let x      = ((opcode & 0x0F00) >> 8) as usize;
                let key_op = opcode & 0x00FF;

                match key_op {
                    0x9E => {
                        // EX9E: SKP Vx — Skip if key[Vx] is pressed
                        if self.keypad[self.v[x] as usize] { self.pc += 2; }
                    },
                    0xA1 => {
                        // EXA1: SKNP Vx — Skip if key[Vx] is not pressed
                        if !self.keypad[self.v[x] as usize] { self.pc += 2; }
                    },
                    _ => {}
                }
            },

            0xF => {
                // Miscellaneous system instructions (timers, font, BCD, memory)
                let x      = ((opcode & 0x0F00) >> 8) as usize;
                let sub_op = opcode & 0x00FF;

                match sub_op {
                    0x07 => {
                        // FX07: LD Vx, DT — Read delay timer into Vx
                        self.v[x] = self.delay_timer;
                    },
                    0x0A => {
                        // FX0A: LD Vx, K — Halt execution until a key is pressed.
                        // Implementation: rewind PC to re-execute this instruction next tick.
                        let mut key_pressed = false;
                        for btn in 0..16usize {
                            if self.keypad[btn] {
                                self.v[x]    = btn as u8;
                                key_pressed = true;
                                break;
                            }
                        }
                        if !key_pressed {
                            self.pc -= 2; // Stall: repeat this instruction next cycle
                        }
                    },
                    0x15 => {
                        // FX15: LD DT, Vx — Set delay timer to Vx
                        self.delay_timer = self.v[x];
                    },
                    0x18 => {
                        // FX18: LD ST, Vx — Set sound timer to Vx
                        self.sound_timer = self.v[x];
                    },
                    0x1E => {
                        // FX1E: ADD I, Vx — Advance the Index Register by Vx
                        self.i += self.v[x] as u16;
                    },
                    0x29 => {
                        // FX29: LD F, Vx — Point I to the font sprite for digit Vx
                        // Font base at 0x050; each glyph is 5 bytes wide
                        self.i = 0x050 + (self.v[x] as u16 * 5);
                    },
                    0x33 => {
                        // FX33: LD B, Vx — Store BCD representation of Vx at I, I+1, I+2
                        let value    = self.v[x];
                        let hundreds = value / 100;
                        let tens     = (value / 10) % 10;
                        let units    = value % 10;
                        self.memory[self.i as usize]       = hundreds;
                        self.memory[(self.i + 1) as usize] = tens;
                        self.memory[(self.i + 2) as usize] = units;
                    },
                    0x55 => {
                        // FX55: LD [I], Vx — Dump registers V0..Vx into memory at I
                        for idx in 0..=x {
                            self.memory[self.i as usize + idx] = self.v[idx];
                        }
                    },
                    0x65 => {
                        // FX65: LD Vx, [I] — Load registers V0..Vx from memory at I
                        for idx in 0..=x {
                            self.v[idx] = self.memory[self.i as usize + idx];
                        }
                    },
                    _ => {} // Reserved / undefined
                }
            },

            _ => {} // Unknown instruction family — silently ignored
        }
    }
}
