# rust8
A chip-8 emulator written in rust.
Currently a work in progress, but is "functional".

## Installation
### Dependencies
- rust
- sdl2 development libraries
	- https://github.com/Rust-SDL2/rust-sdl2#linux
	- (Windows should work, but I haven't tried)
	- Also maybe possible to just use the rust-sdl2 bundle feature
### Installing from source
- git clone
- run the program
```
git clone https://github.com/tung44/rust8
cd rust8
cargo run ./path/to/rom
```
ROMs can easily be found online under public domain.
Controls are hardcoded to 1-4, q-r, a-f, z-v. This maps to the 4x4
hexadecimal keyboard the chip-8 uses.
#### chip-8 keyboard original layout
```
-----------------
| 1 | 2 | 3 | C |
-----------------
| 4 | 5 | 6 | D |
-----------------
| 7 | 8 | 9 | E |
-----------------
| A | 0 | B | F |
-----------------
```

## Contribution Ideas
 - Unit test each instruction
 - Update documentation further
 - General cleanup of code
 - Implement debug print of system for panics
 - Implement GUI for loading ROMs
 - Implement start/stop
 - Implement realtime debugger/disassembler
 - Implement runtime options (frequency, colors, audio, controls)

