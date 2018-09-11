extern crate rand;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn clear_screen_works() {
        let mut screen = Screen::new();
        screen.bitmap[1][2] = true;
        screen.clear_screen();
        assert_eq!(screen.bitmap[1][2], false);
    }
}

pub struct Screen {
    bitmap: [[bool; 64]; 64],
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            bitmap: [[false; 64]; 64],
        }
    }

    fn clear_screen(&mut self) {
        for (_i, row) in self.bitmap.iter_mut().enumerate() {
            for col in row.iter_mut() {
                *col = false;
            }
        }
    }
}

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
pub struct Chip8<'a> {
    screen: &'a mut Screen,
    memory: [u8; 4096],
    registers: [u8; 16],
    instruction_reg: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
}

impl<'a> Chip8<'a> {
    pub fn new(screen: &mut Screen) -> Chip8 {
        let mut chip = Chip8 {
            screen: screen,
            memory: [0; 4096],
            registers: [0; 16],
            instruction_reg: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
        };
        chip.screen.clear_screen();
        chip.load_fontset();
        chip
    }

    // Decrements timers if not 0
    fn update_timers(&mut self) {
        if self.delay_timer != 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer != 0 {
            self.sound_timer -= 1;
        }
    }

    // Loads a buffer into memory at location 0x200
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

        self.update_timers();

        // Giant switch statement to determine opcode
        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => self.screen.clear_screen(),
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
            0xA000 => self.instruction_reg = 0x00FF & opcode,
            0xB000 => self.pc = (0x0FFF & opcode) + self.registers[0] as u16,
            0xC000 => self.registers[index] = rand::random::<u8>() & kk, // random generator
            0xD000 => {
                println!("TODO 0xD000");
                // Draw a sprite, detecting collision
                    /*
                    uint8_t height = 0x000F & opcode;
                    uint8_t x = Vx[indexX];
                    uint8_t y = Vx[indexY];
                    Vx[0xF] = 0;  // No collision to start with initially

                    // Walk the sprite length (corresponding to height)
                    for (int currentHeight = 0; currentHeight < height; ++currentHeight)
                    {
                        // Walk each sprite byte from MSB to LSB (corresponding to width)
                        for (int currentWidth = 0; currentWidth < 8; ++currentWidth)
                        {
                            // Isolate the current bit
                            uint8_t spriteByte = memory[I+currentHeight];
                            uint8_t pixel = 0x80 & (spriteByte << currentWidth);
                            pixel = pixel >> 7;

                            // Check for collision
                            if (mScreen[(currentHeight+y)%32][(currentWidth+x)%64] & pixel)
                            {
                                Vx[0xF] = 1;
                            }
                            mScreen[(currentHeight+y)%32][(currentWidth+x)%64] ^= pixel;
                        }
                    }
                    Q_EMIT drawSignal();
                    break;
                     */            }
            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        println!("TODO 0xEx9E");
                        // If key pressed skip instruction
                            /*
                            if (mKeys[Vx[index]])
                            {
                                PC += 2;
                            }
                            */                    }
                    0x00A1 => {
                        println!("TODO 0xExA1");
                        // If key not pressed skip instruction
                            /*
                            if (!mKeys[Vx[index]])
                            {
                                PC += 2;
                            }
                            */                    }
                    _ => panic!("Unknown instruction: {:x}", opcode),
                }
            }
            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => self.registers[index] = self.delay_timer,
                    0x000A => println!("TODO 0xFx0A"),
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
