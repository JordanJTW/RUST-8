// Copyright of Jordan Werthman (2019).

use env_logger;
use log::*;

use piston_window::*;

use std::fs::File;
use std::io;
use std::io::prelude::*;

use rust8::Bus;
use rust8::Cpu;

use rust8::WINDOW_HEIGHT;
use rust8::WINDOW_WIDTH;

// CRT Monitor green:
const PIXEL_COLOR: [f32; 4] = [0.0, 0.95, 0.0, 1.0];
const WINDOW_SIZE: [u32; 2] = [500, 250];

fn main() {
    env_logger::init();

    let mut bus: Bus = Bus::new();
    let mut cpu: Option<Cpu> = None;

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("RUST-8", WINDOW_SIZE)
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let key_mapping = |ref piston_key| {
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

            let board = bus.display();
            let dimen = ctx.get_view_size()[0] / WINDOW_WIDTH as f64;

            for x in 0..WINDOW_WIDTH {
                for y in 0..WINDOW_HEIGHT {
                    if board[y * WINDOW_WIDTH + x] {
                        let location = rectangle::square(x as f64 * dimen, y as f64 * dimen, dimen);
                        rectangle(PIXEL_COLOR, location, ctx.transform, gfx);
                    }
                }
            }
        });

        if let Event::Input(ref input) = event {
            match input {
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(Key::Space),
                    state: ButtonState::Press,
                    ..
                }) => {
                    should_tick = !should_tick;
                }
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(Key::Return),
                    state: ButtonState::Press,
                    ..
                }) => {
                    if let Some(ref mut cpu) = cpu {
                        cpu.tick(&mut bus);
                    }
                }
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(key),
                    state,
                    ..
                }) => {
                    if let Some(keypad) = key_mapping(*key) {
                        match state {
                            ButtonState::Press => {
                                info!("Keypad set {:?}", keypad);
                                bus.set_key(keypad);
                            }
                            ButtonState::Release => {
                                info!("Keypad clear {:?}", keypad);
                                bus.clear_key(keypad);
                            }
                        }
                    }
                }
                Input::FileDrag(FileDrag::Drop(path)) => {
                    let filename = path.to_str().expect("Invalid path");
                    let buffer = read_file(filename).expect("File not found.");

                    cpu = Some(Cpu::new(&buffer));
                    bus.reset();

                    should_tick = true;
                }
                _ => (),
            }
        }

        if let Some(ref args) = event.update_args() {
            bus.update_timers(args.dt);
        }

        if should_tick {
            if let Some(ref mut cpu) = cpu {
                cpu.tick(&mut bus);
            }
        }
    }
}

fn read_file(filename: &str) -> io::Result<(Vec<u8>)> {
    let mut file = File::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    info!("Loaded {} instructions.", buffer.len() / 2);

    Ok(buffer)
}
