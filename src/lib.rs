extern crate rand;

/* memory */
/* mem map taken from http://devernay.free.fr/hacks/chip8/C8TECH10.HTM */
/* Memory Map:
   +---------------+= 0xFFF (4095) End of Chip-8 RAM
   |               |
   |               |
   |               |
   |               |
   |               |
   | 0x200 to 0xFFF|
   |     Chip-8    |
   | Program / Data|
   |     Space     |
   |               |
   |               |
   |               |
   +- - - - - - - -+= 0x600 (1536) Start of ETI 660 Chip-8 programs
   |               |
   |               |
   |               |
   +---------------+= 0x200 (512) Start of most Chip-8 programs
   | 0x000 to 0x1FF|
   | Reserved for  |
   |  interpreter  |
   +---------------+= 0x000 (0) Start of Chip-8 RAM
*/
pub struct Chip8 {
    // For better or worse, use matrix notation for now.
    // The screen is 64x32 pixels, i.e 64 wide 32 tall. There are 32 rows,
    // 64 columns. I try to stick to that notation here but it should probably
    // be changed to screen[width][height] like most graphics apis seem to be.
    // Each entry represents if the pixel is currently set (i.e. is white)
    // or is not set (i.e. is black, the background).
    pub screen: [[bool; 64]; 32],
    pub keys: [bool; 0xF + 1], // Input is a hex keyboard
    memory: [u8; 4096],
    registers: [u8; 16],
    instruction_reg: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip = Chip8 {
            screen: [[false; 64]; 32],
            keys: [false; 0xF + 1],
            memory: [0; 4096],
            registers: [0; 16],
            instruction_reg: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
        };
        chip.clear_screen();
        chip.load_fontset();
        chip
    }

    // Clear the screen
    fn clear_screen(&mut self) {
        for (_i, row) in self.screen.iter_mut().enumerate() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }

    // Decrements timers at 60 Hz if not 0
    // When sound_timer is non zero the chip-8 buzzer sounds
    pub fn update_timers(&mut self) {
        if self.delay_timer != 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer != 0 {
            self.sound_timer -= 1;
        }
    }

    // Loads a buffer into memory at location 0x200
    // which is where ROM data starts for chip-8
    pub fn load_rom(&mut self, buffer: Vec<u8>) {
        for (i, data) in buffer.iter().enumerate() {
            self.memory[0x200 + i] = *data;
        }
    }

    // Loads the fontset chip 8 provides into memory
    fn load_fontset(&mut self) {
        // Load the fontset into the beginning of the interpreter memory
        let fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        for (i, font) in fontset.iter().enumerate() {
            self.memory[i] = *font;
        }
    }

    // Runs an instruction, updating registers as necessary
    pub fn cycle(&mut self) {
        let mut opcode: u16 = (self.memory[self.pc as usize] as u16) << 8;
        opcode |= self.memory[(self.pc + 1) as usize] as u16;

        // Calculate indexes of registers derived from opcode
        let index: usize = ((opcode & 0x0F00) >> 8) as usize;
        let index_x: usize = index;
        let index_y: usize = ((opcode & 0x00F0) >> 4) as usize;

        // Calculate variable used in some opcodes
        let kk: u8 = (opcode & 0x00FF) as u8;

        // Increment program counter now
        self.pc += 2;

        // Giant switch statement to determine opcode
        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => self.clear_screen(),
                    0x00EE => {
                        // Saves top of stack to program counter
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => panic!("Unknown instruction: {:x}", opcode),
                }
            }
            0x1000 => self.pc = opcode & 0x0FFF, // Jump to 0x0nnn
            0x2000 => {
                // Push the program counter onto stack and then jump to 0x0nnn
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            }
            0x3000 => {
                // Skip next instruction if condition met
                if kk == self.registers[index] {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // Skip next instruction if condition met
                if kk != self.registers[index] {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // Skip next instruction if condition met
                if self.registers[index_x] == self.registers[index_y] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                self.registers[index] = kk;
            }
            0x7000 => {
                self.registers[index] = self.registers[index].wrapping_add(kk);
            }
            0x8000 => {
                match opcode & 0x000F {
                    // Binary operators
                    0x0000 => {
                        self.registers[index_x] = self.registers[index_y];
                    }
                    0x0001 => {
                        self.registers[index_x] |= self.registers[index_y];
                    }
                    0x0002 => {
                        self.registers[index_x] &= self.registers[index_y];
                    }
                    0x0003 => {
                        self.registers[index_x] ^= self.registers[index_y];
                    }
                    // Overflow aware operators
                    0x0004 => {
                        let (vx, vf) =
                            self.registers[index_x].overflowing_add(self.registers[index_y]);
                        self.registers[index_x] = vx;
                        self.registers[0xF] = vf as u8;
                    }
                    0x0005 => {
                        let (vx, vf) =
                            self.registers[index_x].overflowing_sub(self.registers[index_y]);
                        self.registers[index_x] = vx;
                        self.registers[0xF] = vf as u8;
                    }
                    0x0006 => {
                        // TODO: Carefully confirm this is not supposed to be index_y
                        self.registers[0xF] = self.registers[index] & 0x01;
                        self.registers[index] >>= 1;
                    }
                    0x0007 => {
                        let (vx, vf) =
                            self.registers[index_y].overflowing_sub(self.registers[index_x]);
                        self.registers[index_x] = vx;
                        self.registers[0xF] = vf as u8;
                    }
                    0x000E => {
                        // Conflicting docs TODO
                        self.registers[0xF] = self.registers[index] >> 7;
                        self.registers[index] <<= 1;
                    }
                    _ => panic!("Unknown instruction: {:x}", opcode),
                }
            }
            0x9000 => {
                // Skip instruction if condition met
                if self.registers[index_x] != self.registers[index_y] {
                    self.pc += 2;
                }
            }
            0xA000 => self.instruction_reg = 0x0FFF & opcode,
            0xB000 => self.pc = (0x0FFF & opcode) + self.registers[0] as u16,
            0xC000 => self.registers[index] = rand::random::<u8>() & kk, // random generator
            0xD000 => {
                // Draw a sprite, detecting collision
                let height = (0x000F & opcode) as u8;
                let x = self.registers[index_x];
                let y = self.registers[index_y];
                self.registers[0xF] = 0; // No collision detected initially

                // Walk the length of the sprite (corresponding to height)
                for current_height in 0..height {
                    // Walk each sprite byte from MSB to LSB (corresponding to width)
                    for current_width in 0..8 {
                        // Isolate the current bit
                        let sprite_byte: u8 =
                            self.memory[(self.instruction_reg + current_height as u16) as usize];
                        let mut pixel: u8 = 0x80 & (sprite_byte << current_width);
                        pixel >>= 7;

                        // Check for collision
                        let y_offset = ((current_height + y) % 32) as usize;
                        let x_offset = ((current_width + x) % 64) as usize;
                        if self.screen[y_offset][x_offset] as u8 & pixel == 1 {
                            self.registers[0xF] = 1;
                        }
                        // Pixels are XOR'ed onto the screen
                        if self.screen[y_offset][x_offset] as u8 ^ pixel == 1 {
                            self.screen[y_offset][x_offset] = true;
                        } else {
                            self.screen[y_offset][x_offset] = false;
                        }
                    }
                }
            }
            0xE000 => match opcode & 0x00FF {
                0x009E => {
                    if self.keys[self.registers[index] as usize] {
                        self.pc += 2;
                    }
                }
                0x00A1 => {
                    if !self.keys[self.registers[index] as usize] {
                        self.pc += 2;
                    }
                }
                _ => panic!("Unknown instruction: {:x}", opcode),
            },
            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => self.registers[index] = self.delay_timer,
                    0x000A => {
                        // This should "block" until a key is pressed, storing the key
                        // We "block" by rolling back the PC to this instruction again
                        // allowing us to give control back to the main game loop to
                        // grab any keyboard updates
                        self.pc -= 2;
                        for (i, key) in self.keys.iter().enumerate() {
                            if *key {
                                self.registers[index] = i as u8;
                            }
                        }
                    }
                    0x0015 => self.delay_timer = self.registers[index],
                    0x0018 => self.sound_timer = self.registers[index],
                    0x001E => self.instruction_reg += self.registers[index] as u16,
                    0x0029 => {
                        // set I = location of sprite registers[index]
                        let character = self.registers[index];
                        self.instruction_reg = character as u16 * 5;
                    }
                    0x0033 => {
                        // The interpreter takes the decimal value of Vx, and places
                        // the hundreds digit in memory at location in I, the tens digit at
                        // location I+1, and the ones digit at location I+2.
                        let hundreds = self.registers[index] / 100;
                        let tens = (self.registers[index] / 10) % 10;
                        let ones = self.registers[index] % 10;
                        self.memory[self.instruction_reg as usize] = hundreds;
                        self.memory[self.instruction_reg as usize + 1] = tens;
                        self.memory[self.instruction_reg as usize + 2] = ones;
                    }
                    0x0055 => {
                        let end = index + 1;
                        for i in 0..end {
                            self.memory[self.instruction_reg as usize + i] = self.registers[i];
                        }
                    }
                    0x0065 => {
                        let end = index + 1;
                        for i in 0..end {
                            self.registers[i] = self.memory[self.instruction_reg as usize + i];
                        }
                    }
                    _ => panic!("Unknown instruction: {:x}", opcode),
                }
            }
            _ => panic!("Unknown instruction: {:x}", opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to convert opcode vector to u8 vector
    fn opcodes_to_buffer(opcodes: &Vec<u16>) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        for opcode in opcodes.iter() {
            buffer.push(((opcode & 0xFF00) >> 8) as u8);
            buffer.push((opcode & 0x00FF) as u8);
        }
        buffer
    }

    #[test]
    fn test_clear_screen_works() {
        let mut chip8 = Chip8::new();
        chip8.screen[1][2] = true;
        chip8.clear_screen();
        assert!(!chip8.screen[1][2]);
    }

    #[test]
    fn test_load_rom() {
        let mut chip8 = Chip8::new();
        let rom: Vec<u16> = vec![0x0001, 0x0203]; // Random ROM
        chip8.load_rom(opcodes_to_buffer(&rom));
        assert_eq!(chip8.memory[0x200], 0x00);
        assert_eq!(chip8.memory[0x201], 0x01);
        assert_eq!(chip8.memory[0x202], 0x02);
        assert_eq!(chip8.memory[0x203], 0x03);
    }

    #[test]
    fn test_clear_screen_instruction() {
        let mut chip8 = Chip8::new();
        chip8.screen[1][2] = true;
        let rom: Vec<u16> = vec![0x00E0]; // Clear screen instruction
        chip8.load_rom(opcodes_to_buffer(&rom));

        assert!(chip8.screen[1][2]);
        chip8.cycle();
        assert_eq!(chip8.pc, 0x202);
        assert!(!chip8.screen[1][2]);
    }

    #[test]
    fn test_push_and_pop_stack() {
        let mut chip8 = Chip8::new();
        let rom: Vec<u16> = vec![0x2666]; // Push pc to stack, jump to 0x666
        chip8.load_rom(opcodes_to_buffer(&rom));

        // Lazily insert a pop stack at the jump address
        chip8.memory[0x666] = 0x00;
        chip8.memory[0x667] = 0xEE;
        chip8.cycle();
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.stack[0], 0x202);
        assert_eq!(chip8.pc, 0x666);
        chip8.cycle();
        assert_eq!(chip8.sp, 0);
        assert_eq!(chip8.pc, 0x202);
    }

    #[test]
    fn test_jump() {
        let mut chip8 = Chip8::new();
        let rom: Vec<u16> = vec![0x1666]; // Push pc to stack, jump to 0x666
        chip8.load_rom(opcodes_to_buffer(&rom));
        chip8.cycle();
        assert_eq!(chip8.pc, 0x666);
    }
}
