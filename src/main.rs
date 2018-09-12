extern crate rust8;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    println!("Hey world!");
    let filename = &args[1];
    let mut f = File::open(filename).expect("File not found");
    let mut buffer: Vec<u8> = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    let screen = Rc::new(RefCell::new(rust8::Screen::new()));
    let mut chip8 = rust8::Chip8::new(Rc::clone(&screen));
    chip8.load_rom(buffer);

    loop {
        chip8.cycle();
    }
}
