// Copyright of Jordan Werthman (2019).

use piston_window::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;

mod cpu;

const WINDOW_SIZE: [u32; 2] = [500, 250];

fn main() {
    let buffer = read_file("brix.ch8").expect("File not found.");
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
            Key::D1 => Some(0x1),
            Key::Up | Key::D2 => Some(0x2),
            Key::D3 => Some(0x3),
            Key::Left | Key::Q => Some(0x4),
            Key::W => Some(0x5),
            Key::Right | Key::E => Some(0x6),
            Key::A => Some(0x7),
            Key::Down | Key::S => Some(0x8),
            Key::D => Some(0x9),
            Key::X => Some(0x0),
            Key::D4 => Some(0xA),
            Key::R => Some(0xB),
            Key::F => Some(0xC),
            Key::Z => Some(0xD),
            Key::C => Some(0xE),
            Key::V => Some(0xF),
            _ => None,
        }
    };

    let mut should_tick = false;
    while let Some(event) = window.next() {
        window.draw_2d(&event, |ctx, gfx| {
            clear(color::BLACK, gfx);

            let board = cpu.display();
            let dimen = ctx.get_view_size()[0] / 64.0;

            for x in 0..64 {
                for y in 0..32 {
                    if board[y * 64 + x] {
                        let location =
                            rectangle::square(x as f64 * dimen, y as f64 * dimen, dimen - 0.5);
                        rectangle(color::WHITE, location, ctx.transform, gfx);
                    }
                }
            }
        });

        if let Event::Input(input) = &event {
            match input {
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(key),
                    state,
                    ..
                }) => {
                    if let Some(keypad) = key_mapping(*key) {
                        match state {
                            ButtonState::Press => {
                                println!("Keypad set {:?}", keypad);
                                cpu.set_key(keypad);
                            }
                            ButtonState::Release => {
                                println!("Keypad clear {:?}", keypad);
                                cpu.clear_key(keypad);
                            }
                        }
                    }
                }
                Input::FileDrag(FileDrag::Drop(path)) => {
                    let filename: &str = path.to_str().expect("Invalid path.");
                    cpu = cpu::Cpu::new(read_file(filename).expect("File not found."));
                    should_tick = true;
                }
                _ => (),
            }
        }

        if let Some(args) = event.update_args() {
            cpu.update_timers(args.dt);
        }

        if should_tick {
           cpu.tick();
        }
    }
}

fn read_file(filename: &str) -> io::Result<(Vec<u8>)> {
    let mut file = File::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    println!("Loaded {} instructions.", buffer.len() / 2);

    Ok(buffer)
}
