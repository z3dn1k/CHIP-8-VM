use minifb::{Key, Window, WindowOptions, Scale};

// --- CONSTANTS ---

const RAM_SIZE: usize = 4096;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

fn main() {
    let mut cpu = CPU::new();
    // Remember to put a real game ROM here when you are ready to play!
    cpu.load_rom("Pong (1 player).ch8"); 

    // Set up the window with 16x scaling
    let mut window = Window::new(
        "CHIP-8 Emulator",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    ).expect("Failed to create window");

    // Limit the window refresh rate to ~60 FPS
    window.set_target_fps(60);

    // minifb requires a u32 buffer (ARGB format)
    let mut buffer: Vec<u32> = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

    // --- THE GAME LOOP ---
    while window.is_open() && !window.is_key_down(Key::Escape) {
        
        // 0. Update Keypad State
        cpu.keys = [false; NUM_KEYS];
        for key in window.get_keys() {
            match key {
                Key::Key1 => cpu.keys[0x1] = true,
                Key::Key2 => cpu.keys[0x2] = true,
                Key::Key3 => cpu.keys[0x3] = true,
                Key::Key4 => cpu.keys[0xC] = true,
                Key::Q    => cpu.keys[0x4] = true,
                Key::W    => cpu.keys[0x5] = true,
                Key::E    => cpu.keys[0x6] = true,
                Key::R    => cpu.keys[0xD] = true,
                Key::A    => cpu.keys[0x7] = true,
                Key::S    => cpu.keys[0x8] = true,
                Key::D    => cpu.keys[0x9] = true,
                Key::F    => cpu.keys[0xE] = true,
                Key::Z    => cpu.keys[0xA] = true,
                Key::X    => cpu.keys[0x0] = true,
                Key::C    => cpu.keys[0xB] = true,
                Key::V    => cpu.keys[0xF] = true,
                _ => (),
            }
        }

        // 1. Run the CPU cycle 10 times (600Hz)
        for _ in 0..10 {
            cpu.tick();
        }

        // 2. Decrement the timers exactly once per frame (60Hz)
        cpu.tick_timers();

        // 3. Map the boolean CHIP-8 display to the u32 minifb buffer
        for (i, &pixel) in cpu.display.iter().enumerate() {
            buffer[i] = if pixel { 0xFFFFFF } else { 0x000000 };
        }

        // 4. Update the window with the new buffer (60Hz)
        window
            .update_with_buffer(&buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .expect("Failed to update window");
    }
}

// --- CPU STRUCT DEFINITION ---

pub struct CPU {
    pub ram: [u8; RAM_SIZE],
    pub v_reg: [u8; NUM_REGS],
    pub i_reg: u16,
    pub pc: u16,
    pub stack: [u16; STACK_SIZE],
    pub sp: u16,
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub keys: [bool; NUM_KEYS],
    pub delay_timer: u8,
    pub sound_timer: u8,
}

// --- CPU IMPLEMENTATION ---

impl CPU {
    pub fn new() -> Self {
        let mut new_cpu = Self {
            ram: [0; RAM_SIZE],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            pc: 0x200, 
            stack: [0; STACK_SIZE],
            sp: 0,
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };

        new_cpu.ram[0x050..0x0A0].copy_from_slice(&FONTSET);

        new_cpu
    }

    // --- MEMORY AND ROM METHODS ---

    pub fn load_rom(&mut self, filename: &str) {
        let rom_data = std::fs::read(filename).expect("Failed to read ROM file");
        
        assert!(rom_data.len() <= RAM_SIZE - 0x200, "ROM is too large for memory");
        
        let start = 0x200;
        let end = start + rom_data.len();
        
        self.ram[start..end].copy_from_slice(&rom_data);
    }

    // --- TIMER METHODS ---

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!"); 
            }
            self.sound_timer -= 1;
        }
    }

    // --- CYCLE CYCLE (FETCH, DECODE, EXECUTE) ---

    pub fn tick(&mut self) {
        // 1. Fetch
        let byte1 = self.ram[self.pc as usize] as u16;
        let byte2 = self.ram[(self.pc + 1) as usize] as u16;
        
        let opcode = (byte1 << 8) | byte2;

        // Move the Program Counter to the next instruction
        self.pc += 2;

        // 2. Decode - Extract common variables from the opcode
        let c   = (opcode & 0xF000) >> 12; // First digit (Instruction Category)
        let x   = (opcode & 0x0F00) >> 8;  // Second digit (Register X index)
        let y   = (opcode & 0x00F0) >> 4;  // Third digit (Register Y index)
        let d   = opcode & 0x000F;         // Fourth digit (N)
        
        let nn  = (opcode & 0x00FF) as u8; // Last two digits (NN)
        let nnn = opcode & 0x0FFF;         // Last three digits (NNN)

        // 3. Execute
        match (c, x, y, d) {
            // 00E0 - Clear Screen
            (0, 0, 0xE, 0) => {
                self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
            },
            // 00EE - Return from a subroutine
            (0, 0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            },
            // 1NNN - Jump to address NNN
            (1, _, _, _) => {
                self.pc = nnn;
            },
            // 2NNN - Call subroutine at NNN
            (2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },
            // 3XNN - Skip next instruction if VX == NN
            (3, _, _, _) => {
                if self.v_reg[x as usize] == nn {
                    self.pc += 2;
                }
            },
            // 4XNN - Skip next instruction if VX != NN
            (4, _, _, _) => {
                if self.v_reg[x as usize] != nn {
                    self.pc += 2;
                }
            },
            // 5XY0 - Skip next instruction if VX == VY
            (5, _, _, 0) => {
                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += 2;
                }
            },
            // 6XNN - Set register VX to NN
            (6, _, _, _) => {
                self.v_reg[x as usize] = nn;
            },
            // 7XNN - Add NN to register VX (Do not change carry flag)
            (7, _, _, _) => {
                self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn);
            },

            // --- 8-SERIES MATH AND LOGIC OPCODES ---
            // 8XY0 - Set VX to the value of VY
            (8, _, _, 0) => {
                self.v_reg[x as usize] = self.v_reg[y as usize];
            },
            // 8XY1 - Set VX to (VX OR VY)
            (8, _, _, 1) => {
                self.v_reg[x as usize] |= self.v_reg[y as usize];
            },
            // 8XY2 - Set VX to (VX AND VY)
            (8, _, _, 2) => {
                self.v_reg[x as usize] &= self.v_reg[y as usize];
            },
            // 8XY3 - Set VX to (VX XOR VY)
            (8, _, _, 3) => {
                self.v_reg[x as usize] ^= self.v_reg[y as usize];
            },
            // 8XY4 - Add VY to VX. Set VF to 1 if there's a carry, 0 if not.
            (8, _, _, 4) => {
                let (res, overflow) = self.v_reg[x as usize].overflowing_add(self.v_reg[y as usize]);
                self.v_reg[x as usize] = res;
                self.v_reg[0xF] = if overflow { 1 } else { 0 };
            },
            // 8XY5 - Subtract VY from VX. Set VF to 0 if there's a borrow, 1 if not.
            (8, _, _, 5) => {
                let (res, borrow) = self.v_reg[x as usize].overflowing_sub(self.v_reg[y as usize]);
                self.v_reg[x as usize] = res;
                self.v_reg[0xF] = if borrow { 0 } else { 1 };
            },
            // 8XY6 - Shift VX right by 1. Set VF to the least significant bit before the shift.
            (8, _, _, 6) => {
                let lsb = self.v_reg[x as usize] & 1;
                self.v_reg[x as usize] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            // 8XY7 - Subtract VX from VY. Set VF to 0 if there's a borrow, 1 if not.
            (8, _, _, 7) => {
                let (res, borrow) = self.v_reg[y as usize].overflowing_sub(self.v_reg[x as usize]);
                self.v_reg[x as usize] = res;
                self.v_reg[0xF] = if borrow { 0 } else { 1 };
            },
            // 8XYE - Shift VX left by 1. Set VF to the most significant bit before the shift.
            (8, _, _, 0xE) => {
                let msb = (self.v_reg[x as usize] >> 7) & 1;
                self.v_reg[x as usize] <<= 1;
                self.v_reg[0xF] = msb;
            },
            // --- END 8-SERIES ---

            // 9XY0 - Skip next instruction if VX != VY
            (9, _, _, 0) => {
                if self.v_reg[x as usize] != self.v_reg[y as usize] {
                    self.pc += 2;
                }
            },
            // ANNN - Set I register to NNN
            (0xA, _, _, _) => {
                self.i_reg = nnn;
            },
            // BNNN - Jump to address NNN + V0
            (0xB, _, _, _) => {
                self.pc = nnn + (self.v_reg[0] as u16);
            },
            // CXNN - Generate a random number AND NN, store in VX
            (0xC, _, _, _) => {
                let random_byte: u8 = rand::random();
                self.v_reg[x as usize] = random_byte & nn;
            },
            // DXYN - Draw Sprite
            (0xD, _, _, _) => {
                let start_x = self.v_reg[x as usize] as usize % DISPLAY_WIDTH;
                let start_y = self.v_reg[y as usize] as usize % DISPLAY_HEIGHT;
                let height = d as usize;
                
                self.v_reg[0xF] = 0; // Reset collision flag

                for row in 0..height {
                    let sprite_byte = self.ram[(self.i_reg as usize) + row];
                    for col in 0..8 {
                        let sprite_pixel = sprite_byte & (0x80 >> col);
                        if sprite_pixel != 0 {
                            let target_x = start_x + col;
                            let target_y = start_y + row;
                            
                            if target_x < DISPLAY_WIDTH && target_y < DISPLAY_HEIGHT {
                                let idx = target_y * DISPLAY_WIDTH + target_x;
                                if self.display[idx] {
                                    self.v_reg[0xF] = 1; // Collision detected
                                }
                                self.display[idx] ^= true;
                            }
                        }
                    }
                }
            },
            // EX9E - Skip next instruction if key with the value of VX is pressed
            (0xE, _, 9, 0xE) => {
                let key = self.v_reg[x as usize];
                if self.keys[key as usize] {
                    self.pc += 2;
                }
            },
            // EXA1 - Skip next instruction if key with the value of VX is not pressed
            (0xE, _, 0xA, 1) => {
                let key = self.v_reg[x as usize];
                if !self.keys[key as usize] {
                    self.pc += 2;
                }
            },
            // FX07 - Set VX to the value of the delay timer
            (0xF, _, 0, 7) => {
                self.v_reg[x as usize] = self.delay_timer;
            },
            // FX0A - Wait for a key press, store the value of the key in VX
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..NUM_KEYS {
                    if self.keys[i] {
                        self.v_reg[x as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                
                if !pressed {
                    self.pc -= 2;
                }
            },
            // FX15 - Set the delay timer to VX
            (0xF, _, 1, 5) => {
                self.delay_timer = self.v_reg[x as usize];
            },
            // FX18 - Set the sound timer to VX
            (0xF, _, 1, 8) => {
                self.sound_timer = self.v_reg[x as usize];
            },
            
            // --- FINAL MEMORY OPCODES ---
            // FX1E - Add VX to I
            (0xF, _, 1, 0xE) => {
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x as usize] as u16);
            },
            // FX29 - Set I to the location of the sprite for the character in VX
            (0xF, _, 2, 9) => {
                let char = self.v_reg[x as usize] as u16;
                self.i_reg = 0x050 + (char * 5);
            },
            // FX33 - Store the BCD representation of VX at memory addresses I, I+1, and I+2
            (0xF, _, 3, 3) => {
                let value = self.v_reg[x as usize];
                self.ram[self.i_reg as usize]       = value / 100;
                self.ram[(self.i_reg + 1) as usize] = (value / 10) % 10;
                self.ram[(self.i_reg + 2) as usize] = value % 10;
            },
            // FX55 - Store registers V0 through VX in memory starting at location I
            (0xF, _, 5, 5) => {
                for i in 0..=(x as usize) {
                    self.ram[(self.i_reg as usize) + i] = self.v_reg[i];
                }
            },
            // FX65 - Read registers V0 through VX from memory starting at location I
            (0xF, _, 6, 5) => {
                for i in 0..=(x as usize) {
                    self.v_reg[i] = self.ram[(self.i_reg as usize) + i];
                }
            },
            // Catch-all for unimplemented opcodes
            _ => unimplemented!("Opcode {:04X} not implemented yet!", opcode),
        }
    }
}