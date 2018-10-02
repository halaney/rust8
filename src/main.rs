extern crate rust8;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;

use sdl2::audio::AudioCallback;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;

fn main() {
    // Read in game
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut f = File::open(filename).expect("File not found");
    let mut buffer: Vec<u8> = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    let mut chip8 = rust8::Chip8::new();
    chip8.load_rom(buffer);

    // Setup the window
    if !sdl2::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "0") {
        println!("Failed to set render scaling method");
    }
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })
        .unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rust8", 640, 320)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(64, 32).unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Calculate some constants
    let mut ticks = 0;
    let clock_frequency = 500; // Hz
    let framerate = 60;
    let clock_period = 1000 / clock_frequency; // milliseconds
    let ticks_per_frame = (1000 / framerate) / clock_period;

    // Run the game loop
    'running: loop {
        ticks += 1;

        // Handle events from the user
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        update_keys(&event_pump, &mut chip8);

        chip8.cycle();

        // the 60 Hz mark
        if ticks == ticks_per_frame {
            ticks = 0;
            chip8.update_timers();

            // Handle audio
            if chip8.sound_timer == 0 {
                device.pause();
            } else {
                device.resume();
            }

            // Draw black to buffer
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();

            // Update with current video buffer
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            for (y, row) in chip8.screen.iter().enumerate() {
                for (x, pixel) in row.iter().enumerate() {
                    if *pixel {
                        canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
                    }
                }
            }

            // Draw to screen
            canvas.present();
        }

        // Sleep till next cycle
        ::std::thread::sleep(Duration::from_millis(clock_period));
    }
}

// Update the chip8's keys
fn update_keys(e: &sdl2::EventPump, chip8: &mut rust8::Chip8) {
    for key in &mut chip8.keys {
        *key = false;
    }
    for scancode in e.keyboard_state().pressed_scancodes() {
        // This basically hardcodes the controls to use a pseudo
        // hexadecimal input. Basically a 4x4 keyboard, starting with
        // 1-4, then Q-R, A-F, Z-V
        match scancode {
            Scancode::Num1 => {
                chip8.keys[1] = true;
            }
            Scancode::Num2 => {
                chip8.keys[2] = true;
            }
            Scancode::Num3 => {
                chip8.keys[3] = true;
            }
            Scancode::Num4 => {
                chip8.keys[0xC] = true;
            }
            Scancode::Q => {
                chip8.keys[4] = true;
            }
            Scancode::W => {
                chip8.keys[5] = true;
            }
            Scancode::E => {
                chip8.keys[6] = true;
            }
            Scancode::R => {
                chip8.keys[0xD] = true;
            }
            Scancode::A => {
                chip8.keys[7] = true;
            }
            Scancode::S => {
                chip8.keys[8] = true;
            }
            Scancode::D => {
                chip8.keys[9] = true;
            }
            Scancode::F => {
                chip8.keys[0xE] = true;
            }
            Scancode::Z => {
                chip8.keys[0xA] = true;
            }
            Scancode::X => {
                chip8.keys[0] = true;
            }
            Scancode::C => {
                chip8.keys[0xB] = true;
            }
            Scancode::V => {
                chip8.keys[0xF] = true;
            }
            _ => {}
        }
    }
}

// Totally copy pasted example from the sdl2 docs
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
