//! The CHIP-8 emulator as described at https://en.wikipedia.org/wiki/CHIP-8#Virtual_machine_description.

use crate::instruction::*;

const MEM_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 256;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const EMPTY_SCREEN: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT] = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
const PC_START: u16 = 0x200;

pub struct Emulator {
    memory: [u8; MEM_SIZE],
    registers: [u8; NUM_REGISTERS],
    delay_timer: u8,
    sound_timer: u8,
    i: u16,
    program_counter: u16,
    stack_pointer: u8,
    stack: [u16; STACK_SIZE],
    screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]
}

impl Emulator {

    /// Create a new emulator
    pub fn new() -> Emulator {
        Emulator {
            memory: [0; MEM_SIZE],
            registers: [0; NUM_REGISTERS],
            delay_timer: 0,
            sound_timer: 0,
            i: 0,
            program_counter: PC_START,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            screen: EMPTY_SCREEN
        }
    }

    /// Copy a program into memory
    pub fn load(&mut self, program: &[u8]) {
        for i in 0..std::cmp::min(program.len(), self.memory.len() - 0x200) {
            self.memory[self.program_counter as usize + i] = program[i];
        }
    }

    /// Perform a single step, which will update timers,
    /// then load an instruction and execute it.
    pub fn step(&mut self) -> bool {

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

        self.program_counter += 2; // TODO: Make this more error resistant?

        self.execute_single(instruction)
    }

    /// Execute a single instruction
    fn execute_single(&mut self, instruction: Instruction) -> bool {
        match instruction {
            Instruction::Halt => return false,
            Instruction::ClearScreen => self.screen = EMPTY_SCREEN,
            Instruction::Return => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize]; // Jump back via stack
            }
            Instruction::Call(Addr(addr)) => {
                self.stack[self.stack_pointer as usize] = self.program_counter; // Store current address
                self.program_counter = addr; // Jump to addr
                self.stack_pointer += 1;
            }
            i => {
                println!("Unimplemented instruction {:?}", i);
            }
        };

        true
    }

    pub fn execute(&mut self) {
        while self.step() {};
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn halts_upon_reaching_halt_instruction() {
        let mut emulator = Emulator::new();
        assert_eq!(emulator.execute_single(Instruction::Halt), false);
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