extern crate rand;

#[cfg(test)]
#[path = "./cpu_test.rs"]
mod cpu_test;

use rand::Rng;

pub struct Cpu {
    /// Address Register
    pub i: u16,
    /// Program Counter
    pub pc: u16,
    /// Stack
    stack: [u16; 16],
    /// Stack pointer
    sp: usize,
    /// V Registers from V0 to VF
    pub regs: [u8; 16],
    /// Memory. Usually 4096 bytes
    pub mem: Vec<u8>,
    /// Graphics. 2048 pixels
    pub gfx: Vec<u8>,
    /// Delay timer
    pub delay: u8,
    /// Sound Timer
    pub sound: u8,
    /// Whether to draw graphics
    pub should_draw: bool,
    /// Last key pressed
    pub key_pressed: Option<u8>,
}

impl Cpu {
    pub fn new() -> Self {
        let mut mem = Cpu::fontset();
        mem.resize(0x1000, 0);

        Cpu {
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            regs: [0; 16],
            mem: mem,//vec![0; 0x1000],
            gfx: vec![0; 64 * 32],
            delay: 0,
            sound: 0,
            should_draw: false,
            key_pressed: None,
        }
    }

    pub fn load(&mut self, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            self.mem[0x200 + i] = *byte;
        }
    }

    pub fn step(&mut self) {
        self.should_draw = false;

        let instruction = self.load_16(self.pc);
        // println!("{:04x}: {:04x} {:?} I:{:04x} S:{:02x} {:?}", self.pc, instruction, self.regs, self.i, self.sp, self.stack);
        self.pc += 2;

        let a = instruction >> 12;
        let x = ((instruction >> 8) & 0xF) as usize;
        let y = ((instruction >> 4) & 0xF) as usize;
        let nnn = instruction & 0xFFF;
        let nn = (instruction & 0xFF) as u8;
        let n = (instruction & 0xF) as u8;

        // It might be nicer to match on a tuple of nibbles, like (a, x, y, n)
        match a {
            0x0 => {
                match n {
                    0 => {
                        self.gfx = vec![0; 64 * 32]; // FIXME: zero out instead of reallocating
                        self.should_draw = true;
                    },
                    _ => {
                        self.sp -= 1;
                        let address = self.stack[self.sp];
                        self.pc = address;
                    }
                }
            }
            0x1 => self.pc = nnn,
            0x2 => {
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },
            0x3 => if self.regs[x] == nn { self.pc += 2; },
            0x4 => if self.regs[x] != nn { self.pc += 2; },
            0x5 => if self.regs[x] == self.regs[y] { self.pc += 2; },
            0x6 => self.regs[x] = nn,
            0x7 => self.regs[x] = self.regs[x].wrapping_add(nn),
            0x8 => {
                match n {
                    0x0 => self.regs[x] = self.regs[y],
                    0x1 => self.regs[x] |= self.regs[y],
                    0x2 => self.regs[x] &= self.regs[y],
                    0x3 => self.regs[x] ^= self.regs[y],
                    0x4 => {
                        let res = self.regs[x] as u16 + self.regs[y] as u16;
                        if res > 255 {
                            self.regs[0xF] = 1;
                        } else {
                            self.regs[0xF] = 0;
                        }
                        self.regs[x] = res as u8;
                    },
                    0x5 => {
                        let x2 = self.regs[x];
                        let y = self.regs[y];
                        self.regs[0xF] = if y > x2 { 0 } else { 1 };
                        self.regs[x] = self.regs[x].wrapping_sub(y);
                    },
                    0x6 => {
                        self.regs[0xF] = self.regs[x] & 1;
                        self.regs[x] >>= 1;
                    },
                    0x7 => {
                        let x2 = self.regs[x];
                        let y = self.regs[y];
                        self.regs[0xF] = if y < x2 { 0 } else { 1 };
                        self.regs[x] = y.wrapping_sub(self.regs[x]);
                    },
                    0xE => {
                        self.regs[0xF] = self.regs[x] >> 7;
                        self.regs[x] <<= 1;
                    },
                    _ => unreachable!("Unrecognize opcode {:02x}", instruction),
                }
            },
            0x9 => if self.regs[x] != self.regs[y] { self.pc += 2; },
            0xA => self.i = nnn,
            0xB => self.pc = nnn + self.regs[0] as u16,
            0xC => {
                let mut rng = rand::thread_rng();
                self.regs[x] = nn & rng.gen::<u8>();
            },
            0xD => {
                self.regs[0xF] = 0;
                let x = self.regs[x] as u16;
                let y = self.regs[y] as u16;
                let height = n as u16;
                self.should_draw = true;

                for line_y in 0..height {
                    let pixel = self.mem[(line_y + self.i) as usize];

                    for line_x in 0..8 {
                        if pixel & (0x80 >> line_x) != 0 {
                            let offset = (x + line_x + ((y + line_y) * 64)) as usize;
                            if self.gfx[offset] == 1 { self.regs[0xF] = 1 } // collision
                            self.gfx[offset] ^= 1;
                        }
                    }
                }
            },
            0xE => {
                match nn {
                    0x9E => {
                        if let Some(key) = self.key_pressed {
                            if key == self.regs[x] {
                                self.pc += 2;
                            }
                        }
                    },
                    0xA1 => {
                        if let Some(key) = self.key_pressed {
                            if key != self.regs[x] {
                                self.pc += 2;
                            }
                        } else {
                            self.pc += 2;
                        }
                    }
                    _ => unimplemented!("Keys stuff. Opcode: {:04x}", instruction),
                }
            }
            0xF => {
                match nn {
                    // TODO: 0A
                    0x07 => self.regs[x] = self.delay,
                    0x15 => self.delay = self.regs[x],
                    0x18 => self.sound = self.regs[x],
                    0x1E => self.i += self.regs[x] as u16,
                    0x29 => self.i = self.regs[x] as u16 * 5,
                    0x33 => {
                        let i = self.i as usize;
                        let vx = self.regs[x];
                        self.mem[i] = vx / 100;
                        self.mem[i + 1] = (vx / 10) % 10;
                        self.mem[i + 2] = (vx % 100) % 10;
                    },
                    0x55 => {
                        for offset in 0..(x+1) {
                            let address = self.i + offset as u16;
                            self.mem[address as usize] = self.regs[offset];
                        }
                    }
                    0x65 => {
                        for offset in 0..(x+1) {
                            let address = self.i + offset as u16;
                            self.regs[offset] = self.mem[address as usize]
                        }
                    }
                    _ => panic!("Not implemented yet {:02x}", instruction),
                }
            }
            _ => panic!("Not implemented yet {:02x}", instruction),
        };

        if self.delay > 0 { self.delay -= 1 };
        if self.sound > 0 { print!("BEEP"); self.sound -= 1 }; // TODO: Emit beep
    }

    fn load_16(&self, address: u16) -> u16 {
        let hi = self.mem[address as usize] as u16;
        let lo = self.mem[address as usize + 1] as u16;
        (hi << 8) + lo
    } 

    #[allow(dead_code)]
    fn store_16(&mut self, address: u16, value: u16) {
        self.mem[address as usize] = (value >> 8) as u8;
        self.mem[address as usize + 1] = (value & 0xFF) as u8;
    }

    fn fontset() -> Vec<u8> {
        vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x80, 0xF0, 0x80, 0x80 
        ]
    }
}