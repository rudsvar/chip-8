//! The CHIP-8 emulator as described at https://en.wikipedia.org/wiki/CHIP-8#Virtual_machine_description.

use crate::instruction::*;

const MEM_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 256;
// const SCREEN_WIDTH: usize = 128;
// const SCREEN_HEIGHT: usize = 128;
// type Screen = [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT];
// const EMPTY_SCREEN: Screen = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
const PC_START: u16 = 0x200;

pub trait Input {
    fn get_key(&self) -> Option<u8>;
    fn get_key_blocking(&self) -> u8;
}

pub struct DummyInput;

impl Input for DummyInput {
    fn get_key(&self) -> Option<u8> { None }
    fn get_key_blocking(&self) -> u8 { unimplemented!("Can't get blocking") }
}

pub trait Output {
    fn set(&mut self, x: usize, y: usize, state: u8);
    fn get(&self, x: usize, y: usize) -> u8;
    fn clear(&mut self);
    fn refresh(&mut self);
}

pub struct DummyOutput;

impl Output for DummyOutput {
    fn set(&mut self, _: usize, _: usize, state: u8) {}
    fn get(&self, _: usize, _: usize) -> u8 { 0 }
    fn clear(&mut self) {}
    fn refresh(&mut self) {}
}

pub struct Emulator<I: Input, O: Output> {
    // Standard fields
    memory: [u8; MEM_SIZE],
    registers: [u8; NUM_REGISTERS],
    delay_timer: u8,
    sound_timer: u8,
    i: u16,
    program_counter: u16,
    stack_pointer: u8,
    stack: [u16; STACK_SIZE],

    input: I,
    output: O
}

impl<I: Input, O: Output> Emulator<I, O> {

    /// Create a new emulator with input and output
    pub fn with_io(input: I, output: O) -> Emulator<I, O> {
        Emulator {
            memory: [0; MEM_SIZE],
            registers: [0; NUM_REGISTERS],
            delay_timer: 0,
            sound_timer: 0,
            i: 0,
            program_counter: PC_START,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            
            input,
            output
        }
    }

    /// Create a new emulator with dummy input and output
    pub fn new() -> Emulator<DummyInput, DummyOutput> {
        Emulator::with_io(DummyInput, DummyOutput)
    }

    /// Copy a program into memory at 0x200.
    pub fn load(&mut self, program: &[u8]) {
        for i in 0..std::cmp::min(program.len(), self.memory.len() - 0x200) {
            self.memory[self.program_counter as usize + i] = program[i];
        }
    }

    /// Perform a single step, which will update timers,
    /// then load an instruction and execute it.
    pub fn step(&mut self) {

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        // Each opcode is two bytes
        let left = self.memory[self.program_counter as usize];
        let right = self.memory[self.program_counter as usize + 1];
        let instruction = Instruction::from_two_u8(left, right);

        log::trace!("{:?}", instruction);

        self.program_counter += 2; // TODO: Make this more error resistant?

        self.execute_single(instruction);
    }

    /// Execute a single instruction
    fn execute_single(&mut self, instruction: Instruction) {
        match instruction {

            // Clear the screen
            Instruction::ClearScreen => {
                self.output.clear();
            },

            // Return to the previous call site via the stack.
            Instruction::Return => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize]; // Jump back via stack
            }

            // Go to a specific memory address
            Instruction::Goto(Addr(addr)) => {
                self.program_counter = addr;
            }

            // Store the current address on the stack, then jump to the specified address
            Instruction::Call(Addr(addr)) => {
                self.stack[self.stack_pointer as usize] = self.program_counter; // Store current address
                self.stack_pointer += 1;
                self.program_counter = addr; // Jump to addr
            }

            // If the register equals the constant, skip the next instruction
            Instruction::IfRegEqConst(Reg(x), Const(n)) => {
                if self.registers[x as usize] == n {
                    self.program_counter += 2;
                }
            }

            Instruction::IfRegNeqConst(Reg(x), Const(n)) => {
                if self.registers[x as usize] != n {
                    self.program_counter += 2;
                }
            }

            Instruction::IfRegEqReg(Reg(x), Reg(y)) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.program_counter += 2;
                }
            }

            Instruction::SetRegToConst(Reg(x), Const(n)) => {
                self.registers[x as usize] = n;
            }

            // Should this overflow?
            Instruction::IncRegByConst(Reg(x), Const(n)) => {
                self.registers[x as usize] = self.registers[x as usize] + n;
            }

            Instruction::SetRegToReg(Reg(x), Reg(y)) => {
                self.registers[x as usize] = self.registers[y as usize];
            }

            Instruction::BitwiseOr(Reg(x), Reg(y)) => {
                self.registers[x as usize] |= self.registers[y as usize];
            }

            Instruction::BitwiseAnd(Reg(x), Reg(y)) => {
                self.registers[x as usize] &= self.registers[y as usize];
            }

            Instruction::BitwiseXor(Reg(x), Reg(y)) => {
                self.registers[x as usize] ^= self.registers[y as usize];
            }

            // Increment the value of a register by the value of another
            // TODO: Set VF to 1 if there is a carry, 0 otherwise.
            Instruction::IncRegByReg(Reg(x), Reg(y)) => {
                self.registers[x as usize] += self.registers[y as usize];
            }

            // Decrement the value of a register by the value of another
            // TODO: Set VF to 0 if there is a borrow, 1 otherwise.
            Instruction::DecRegByReg(Reg(x), Reg(y)) => {
                self.registers[x as usize] -= self.registers[y as usize];
            }

            Instruction::BitshiftRight(Reg(x)) => {
                self.registers[x as usize] >>= 1;
            }

            // TODO: VF is set to 0 when there's a borrow, and 1 when there isn't.
            Instruction::SetVxVyMinusVx(Reg(x), Reg(y)) => {
                self.registers[x as usize] = self.registers[y as usize] - self.registers[x as usize];
            }

            Instruction::BitshiftLeft(Reg(x)) => {
                self.registers[x as usize] <<= 1;
            }

            Instruction::IfRegNeqReg(Reg(x), Reg(y)) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.program_counter += 2;
                }
            }

            Instruction::SetI(Addr(addr)) => {
                self.i = addr;
            }

            Instruction::SetPcToV0PlusAddr(Addr(addr)) => {
                self.program_counter = self.registers[0] as u16 + addr; 
            }

            Instruction::SetVxRand(Reg(x), Const(n)) => {
                self.registers[x as usize] = rand::random::<u8>() & n; 
            }

            // TOOD: Implement fully, with xor of pixels.
            Instruction::Draw(Reg(x), Reg(y), Const(sprite_height)) => {

                // Get coordinates
                let x_coord = self.registers[x as usize] as usize;
                let y_coord = self.registers[y as usize] as usize;

                // Get sprite, each row is 8 bits
                let sprite_addr = self.i as usize;
                let sprite_data: &[u8] = &self.memory[sprite_addr .. sprite_addr + sprite_height as usize];

                // Write to screen
                let mut any_collisions = 0;
                for y in 0 .. sprite_height as usize {
                    let row: u8 = sprite_data[y];
                    for x in 0 .. 8 {
                        let new_pixel = (row >> (8 - x)) & 1; // Get bit number `bit_idx`
                        let old_pixel = self.output.get(x_coord + x, y_coord + y);
                        let xored_pixel = old_pixel ^ new_pixel; // XOR old pixel with new pixel
                        self.output.set(x_coord + x, y_coord + y, xored_pixel); // Save xor'ed pixel

                        // Set pixel was unset, so we set the collision flag
                        if old_pixel == 1 && xored_pixel == 0 {
                            any_collisions = 1;
                        }
                    }
                }

                // Set VF collision flag
                self.registers[0xF] = any_collisions;
            }

            // TODO: Skip if the key in Vx is pressed
            Instruction::IfKeyEqVx(Reg(x)) => {
                if self.input.get_key() == Some(self.registers[x as usize]) {
                    self.program_counter += 2;
                }
            }

            // TODO: Skip if the key in Vx isn't pressed
            Instruction::IfKeyNeqVx(Reg(x)) => {
                if self.input.get_key() != Some(self.registers[x as usize]) {
                    self.program_counter += 2;
                }
            }

            Instruction::SetRegToDelayTimer(Reg(x)) => {
                self.registers[x as usize] = self.delay_timer;
            }

            // Get a key press (blocking)
            Instruction::SetRegToGetKey(Reg(x)) => {
                self.registers[x as usize] = self.input.get_key_blocking();
            }

            Instruction::SetDelayTimerToReg(Reg(x)) => {
                self.delay_timer = self.registers[x as usize];
            }

            Instruction::SetSoundTimerToReg(Reg(x)) => {
                self.sound_timer = self.registers[x as usize];
            }

            Instruction::AddRegToI(Reg(x)) => {
                self.i += self.registers[x as usize] as u16;
            }

            // TODO: Add font sprites?
            Instruction::SetIToSpriteAddrVx(Reg(_)) => {
                log::warn!("Unimplemented: SetIToSpriteAddrVx");
                self.i = 0x0; // Set to empty area
            }

            Instruction::SetIToBcdOfReg(Reg(x)) => {
                let i = self.i as usize;

                // Get ones place
                let ones = self.registers[x as usize];
                self.memory[i + 2] = (ones % 10) as u8;

                // Get tens place
                let tens = ones / 10;
                self.memory[i + 1] = (tens % 10) as u8;

                // Get hundredths place
                let hundredths = tens / 10;
                self.memory[i] = (hundredths / 10) as u8;
            }

            // Dump register values up to Vx
            Instruction::RegDump(Reg(x)) => {
                let i = self.i as usize;
                for reg_no in 0..=x as usize {
                    self.memory[i + reg_no] = self.registers[reg_no];
                }
            }
            
            // Load register values up to Vx
            Instruction::RegLoad(Reg(x)) => {
                let i = self.i as usize;
                for reg_no in 0..=x as usize {
                    self.registers[reg_no] = self.memory[i + reg_no];
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn clear_screen_clears_screen() {
        let mut emulator = Emulator::new();
        emulator.screen[0][0] = 1;
        emulator.execute_single(Instruction::ClearScreen);
        assert_eq!(emulator.screen[0][0], 0);
    }

    #[test]
    fn goto_goes_to() {
        let mut emulator = Emulator::new();
        emulator.execute_single(Instruction::Goto(Addr(0x250)));
        assert_eq!(emulator.program_counter, 0x250);
    }

    #[test]
    fn return_after_call_is_neutral() {
        // Create emulator
        let mut emulator = Emulator::new();
        assert_eq!(emulator.program_counter, 0x200);

        // Write program with call and return
        let program = [
            0x22, 0x06, // 0x00, jump to 0x206
            0x00, 0x00, // 0x02
            0x00, 0x00, // 0x04
            0x00, 0xEE  // 0x06, return
        ];
        emulator.load(&program);

        // Run the program
        emulator.step(); // Call 0x206
        assert_eq!(emulator.program_counter, 0x206);
        emulator.step(); // Return to 202
        assert_eq!(emulator.program_counter, 0x202);
    }
}