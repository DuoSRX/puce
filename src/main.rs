extern crate rand;
extern crate rustbox;
extern crate sdl2;

// use std::io::{Write, stdout, stdin};
use std::{time, thread};
use rand::Rng;
// use std::error::Error;
// use std::default::Default;

// use rustbox::{RustBox};
// use rustbox::Key;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

#[derive(Debug)]
struct Cpu {
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
    delay: u8,
    /// Sound Timer
    sound: u8,
    /// Whether to draw graphics
    should_draw: bool,
    key_pressed: Option<u8>,
}

impl Cpu {
    fn new() -> Self {
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

    fn step(&mut self) {
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

        match a {
            0x0 => {
                match n {
                    0 => self.gfx = vec![0; 64 * 32], // FIXME: zero out instead of reallocating
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
                        let res = (self.regs[x] as u32).wrapping_sub(self.regs[y] as u32);
                        if res & 0x100 == 0 {
                            self.regs[0xF] = 0;
                        } else {
                            self.regs[0xF] = 1;
                        }
                        self.regs[x] = res as u8;
                    },
                    0x6 => {
                        self.regs[0xF] = self.regs[x] & 1;
                        self.regs[x] >>= 1;
                    },
                    0x7 => unimplemented!("VY-VX"),
                    0xE => {
                        self.regs[0xF] = self.regs[x] & 8;
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
                        if pixel & (0b1000_0000 >> line_x) != 0 {
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
                    _ => unimplemented!("keys stuff"),
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

    // fn store_16(&mut self, address: u16, value: u16) {
    //     self.mem[address as usize] = (value >> 8) as u8;
    //     self.mem[address as usize + 1] = (value & 0xFF) as u8;
    // }

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

fn main() {
    let mut cpu = Cpu::new();
    let rom = include_bytes!("../roms/PONG");

    for (i, byte) in rom.iter().enumerate() {
        cpu.mem[0x200 + i] = *byte;
    }

    // let rustbox = match RustBox::init(Default::default()) {
    //     Result::Ok(v) => v,
    //     Result::Err(e) => panic!("{}", e),
    // };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("wat", 64 * 8, 32 * 8)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().accelerated().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyUp { .. } => {
                    //cpu.key_pressed = None;
                },
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    cpu.key_pressed = keycode_to_u8(keycode);
                } 
                _ => ()
            }
        }

        cpu.step();

        if cpu.should_draw {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(255, 210, 0));

            for row in 0..32 {
                for column in 0..64 {
                    match cpu.gfx[(column + row * 64) as usize] {
                        1 => canvas.fill_rect(Rect::new(column * 8, row * 8, 8, 8)),
                        _ => Ok(()),
                    }.unwrap();
                };
            }

            canvas.present();
        }

        thread::sleep(time::Duration::from_secs(1) / 240);

        // rustbox.present();
        // match rustbox.poll_event(false) {
        //     Ok(rustbox::Event::KeyEvent(key)) => {
        //         match key {
        //             Key::Char('q') => { break 'running; }
        //             _ => {}
        //         }
        //     },
        //     Err(e) => panic!("{}", e.description()),
        //     _ => {}
        // }
    }

    // thread::sleep(time::Duration::from_secs(1));
    // write!(screen, "{}", termion::cursor::Show).unwrap();
    // print!("{}", termion::cursor::Show);
}

fn keycode_to_u8(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => Some(1),
        Keycode::Num2 => Some(2),
        Keycode::Num4 => Some(4),
        Keycode::Num6 => Some(6),
        Keycode::Num8 => Some(8),
        Keycode::C => Some(0xC),
        Keycode::D => Some(0xD),
        _ => None
    }
}

// let prog = vec![
//     0x6100, // LD V1, 0 ; x
//     0x620A, // LD V2, 0 ; y
//     0x6307, // LD V3, 3 ; number to draw
//     0x6400, // LD V4, 0 ; character counter
//     0x650A, // LD V5, 0 ; character to draw
//     0x9340, // SKIP if X != Y
//     0x1220, // JUMP TO EXIT $220
//     0xF529, // SPR V5
//     0xD125, // DRAW V1 V2 5 ; draw x=v1, y=v2, height=5
//     0x7105, // ADD V1, 5
//     0x7401, // ADD V4, 1
//     0x7501, // ADD V5, 1
//     0x120A, // JMP to loop $210
//     0x00E0  // CLS
// ];

// for (i, item) in prog.iter().enumerate() {
//     self.store_16(0x200 + (i as u16) * 2, (*item) as u16);
// }

// self.store_16(0x200, 0x600F);
// self.store_16(0x202, 0xF029);
// self.store_16(0x204, 0xD005);