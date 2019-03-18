// Copyright of Jordan Werthman (2019).

pub struct Bus {
    display: [bool; 64 * 32],
    keys: [bool; 16],
    delay_timer: f64,
    sound_timer: f64,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            display: [false; 64 * 32],
            keys: [false; 16],
            delay_timer: 0.0,
            sound_timer: 0.0,
        }
    }

    pub fn display(&mut self) -> &mut [bool; 64 * 32] {
        return &mut self.display;
    }

    pub fn clear_display(&mut self) {
        for i in 0..self.display.len() {
            self.display[i] = false;
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
}
