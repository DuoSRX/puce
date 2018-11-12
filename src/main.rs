extern crate sdl2;

mod cpu;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::{time, thread};

fn main() {
    let mut cpu = cpu::Cpu::new();
    let rom = include_bytes!("../roms/TANK");

    for (i, byte) in rom.iter().enumerate() {
        cpu.mem[0x200 + i] = *byte;
    }

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
                    cpu.key_pressed = None;
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
    }
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
