extern crate rust8;

use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    println!("Hey world!");
    let filename = &args[1];
    let mut f = File::open(filename).expect("File not found");
    let mut buffer: Vec<u8> = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    let mut chip8 = rust8::Chip8::new();
    chip8.load_rom(buffer);

    loop {
        chip8.cycle();
    }
}
