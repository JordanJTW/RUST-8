// Copyright of Jordan Werthman (2019).

use std::fs::File;
use std::io;
use std::io::prelude::*;

mod cpu;

fn main() {
    let buffer = read_file("clock.ch8").expect("File not found.");
    println!("Read data: {:?}", buffer);

    println!("Loaded {} instructions.", buffer.len() / 2);

    let mut cpu: cpu::Cpu = cpu::Cpu::new(buffer);

    loop {
        cpu.tick();
    }
}

fn read_file(filename: &str) -> io::Result<(Vec<u8>)> {
    let mut file = File::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}
