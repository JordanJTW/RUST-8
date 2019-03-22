// Copyright of Jordan Werthman (2019).

use log::*;

use crate::memory::Memory;

// Models the address bus for I/O
//
// Handles interfacing the CPU with the I/O used to render the screen or
// read in keypresses.

pub const WINDOW_HEIGHT: usize = 32;
pub const WINDOW_WIDTH: usize = 64;

const KEY_COUNT: usize = 16;
const PIXEL_COUNT: usize = WINDOW_HEIGHT * WINDOW_WIDTH;

const TIMER_FREQUENCY: f64 = 60.0;

pub struct Bus {
    display: [bool; PIXEL_COUNT],
    keys: [bool; KEY_COUNT],
    delay_timer: f64,
    sound_timer: f64,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            display: [false; PIXEL_COUNT],
            keys: [false; KEY_COUNT],
            delay_timer: 0.0,
            sound_timer: 0.0,
        }
    }

    pub fn display(&self) -> &[bool; PIXEL_COUNT] {
        return &self.display;
    }

    pub fn draw_display(
        &mut self,
        memory: &mut Memory,
        memory_offset: usize,
        (x, y): (usize, usize),
        height: usize,
    ) -> bool {
        let mut pixel_flipped = false;

        for dy in 0..height {
            for dx in 0..8 {
                let (x, y) = ((x + dx) % WINDOW_WIDTH, (y + dy) % WINDOW_HEIGHT);
                let byte: u8 = memory.data()[memory_offset + dy];

                let index: usize = y * WINDOW_WIDTH + x;
                let value: bool = ((byte << dx) & 0x80) != 0;

                if value {
                    pixel_flipped |= self.display[index];
                    self.display[index] ^= true;
                }
            }
        }

        if log_enabled!(Level::Info) {
            self.print_board();
        }
        return pixel_flipped;
    }

    pub fn clear_display(&mut self) {
        for i in 0..PIXEL_COUNT {
            self.display[i] = false;
        }
    }

    pub fn any_key(&self) -> Option<u8> {
        match self.keys.iter().enumerate().find(|&(_, &value)| value) {
            Some((key, _)) => Some(key as u8),
            None => None,
        }
    }

    pub fn check_key(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    pub fn set_key(&mut self, key: usize) {
        self.keys[key] = true;
    }

    pub fn clear_key(&mut self, key: usize) {
        self.keys[key] = false;
    }

    pub fn delay_timer(&self) -> u8 {
        self.delay_timer as u8
    }

    pub fn set_delay_timer(&mut self, duration: u8) {
        self.delay_timer = duration as f64;
    }

    pub fn sound_active(&self) -> bool {
        self.sound_timer > 0.0
    }

    pub fn set_sound_timer(&mut self, duration: u8) {
        self.sound_timer = duration as f64;
    }

    pub fn update_timers(&mut self, dt: f64) {
        if self.delay_timer > 0.0 {
            self.delay_timer -= dt * TIMER_FREQUENCY;
        }
        if self.sound_timer > 0.0 {
            self.sound_timer -= dt * TIMER_FREQUENCY;
        }
        if self.delay_timer < 0.0 {
            self.delay_timer = 0.0;
        }
        if self.sound_timer < 0.0 {
            self.sound_timer = 0.0;
        }
    }

    pub fn reset(&mut self) {
        self.clear_display();
        self.set_delay_timer(0);
        self.set_sound_timer(0);

        for key in 0..KEY_COUNT {
            self.clear_key(key);
        }
    }

    fn print_board(&self) {
        for y in 0..WINDOW_HEIGHT {
            for x in 0..WINDOW_WIDTH {
                let index: usize = y as usize * WINDOW_WIDTH + x as usize;
                print!("{}", if self.display[index] { "#" } else { "_" });
            }
            println!();
        }
    }
}
