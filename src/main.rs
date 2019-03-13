// Copyright of Jordan Werthman (2019).

extern crate piston_window;

use piston_window::*;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod cpu;

const WINDOW_SIZE: [u32; 2] = [800, 400];

fn main() {
    let buffer = read_file("clock.ch8").expect("File not found.");
    println!("Read data: {:?}", buffer);

    println!("Loaded {} instructions.", buffer.len() / 2);

    let mut cpu: cpu::Cpu = cpu::Cpu::new(buffer);

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("RUST-8", WINDOW_SIZE)
        .exit_on_esc(true)
        .opengl(opengl)
        // .fullscreen(true)
        .build()
        .unwrap();

    let key_mapping = |piston_key| {
        use keyboard::Key;

        match piston_key {
            Key::D1 => Some(cpu::Keypad::Key1),
            Key::Up | Key::D2 => Some(cpu::Keypad::Key2),
            Key::D3 => Some(cpu::Keypad::Key3),
            Key::Left | Key::Q => Some(cpu::Keypad::Key4),
            Key::W => Some(cpu::Keypad::Key5),
            Key::Right | Key::E => Some(cpu::Keypad::Key6),
            Key::A => Some(cpu::Keypad::Key7),
            Key::Down | Key::S => Some(cpu::Keypad::Key8),
            Key::D => Some(cpu::Keypad::Key9),
            Key::X => Some(cpu::Keypad::Key0),
            Key::D4 => Some(cpu::Keypad::KeyA),
            Key::R => Some(cpu::Keypad::KeyB),
            Key::F => Some(cpu::Keypad::KeyC),
            Key::Z => Some(cpu::Keypad::KeyD),
            Key::C => Some(cpu::Keypad::KeyE),
            Key::V => Some(cpu::Keypad::KeyF),
            _ => None,
        }
    };

    while let Some(e) = window.next() {
        window.draw_2d(&e, |ctx, gfx| {
            clear(color::BLACK, gfx);
            
            let board = cpu.display();
            let dimen = ctx.get_view_size()[0] / 64.0;

            for x in 0..63 {
                for y in 0..31 {
                    if board[y * 64 + x] {
                        let location = rectangle::square(
                                x as f64 * dimen, y as f64 * dimen, dimen - 0.5);
                        rectangle(color::WHITE, location, ctx.transform, gfx);
                    }
                }
            }
        });

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if let Some(keypad) = key_mapping(key) {
                println!("Send: {:?} -> {:?}", key, keypad);
                cpu.set_key(keypad);
            }
        }

        cpu.tick();
    }
}

fn read_file(filename: &str) -> io::Result<(Vec<u8>)> {
    let mut file = File::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}
