// Copyright of Jordan Werthman (2019).

use log::*;
use wasm_bindgen::prelude::*;

use crate::bus::Bus;
use crate::cpu::Cpu;

use crate::bus::WINDOW_WIDTH;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
  	#[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

use log::{Record, Level, Metadata};

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            console_log!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

use log::LevelFilter;

static LOGGER: ConsoleLogger = ConsoleLogger;

#[wasm_bindgen]
pub struct Chip8 {
    bus: Bus,
    cpu: Option<Cpu>,
}

#[wasm_bindgen]
impl Chip8 {
	#[wasm_bindgen(constructor)]
    pub fn new() -> Chip8 {
		console_error_panic_hook::set_once();

		log::set_logger(&LOGGER)
        	.map(|()| log::set_max_level(LevelFilter::Info)).expect("logger");

        info!("Random: {}", random());

        Chip8 {
        	bus: Bus::new(),
        	cpu: None,
        }
    }

    pub fn check_pixel(&self, x: usize, y: usize) -> bool {
        return self.bus.display()[y as usize * WINDOW_WIDTH + x as usize];
    }

    pub fn load(&mut self, rom: Vec<u8>) {
    	info!("Loaded {} instructions.", rom.len() / 2);
    	self.cpu = Some(Cpu::new(&rom));
    }

    pub fn update(&mut self, timer_delta: f64) {
        self.bus.update_timers(timer_delta);
    }

    pub fn tick(&mut self) {
    	if let Some(ref mut cpu) = self.cpu {
    		console_log!("CPU tick!");
    		cpu.tick(&mut self.bus);
    	} else {
    		console_log!("No CPU :(");
    	}
    }
}
