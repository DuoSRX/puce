#[macro_use]
extern crate lazy_static;
extern crate sdl2;

mod audio;
mod cpu;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::{env, time, thread};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let mut file = File::open(path).unwrap();
    let mut rom = Vec::new();
    file.read_to_end(&mut rom).unwrap();

    let mut cpu = cpu::Cpu::new();
    cpu.load(rom);

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

    let beep = audio::Audio::new(&sdl_context);

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

        if cpu.should_beep {
            beep.start();
        } else {
            beep.stop();
        }

        //thread::sleep(time::Duration::from_secs(1) / 240);
    }
}

// Original hexadecimal keyboard layout:
// 123C
// 456D
// 789E
// A0BF
// Mapped to:
// 1234
// QWER
// ASDF
// ZXCV
fn keycode_to_u8(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),

        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),

        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),

        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None
    }
}
