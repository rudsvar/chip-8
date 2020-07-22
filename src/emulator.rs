//! The CHIP-8 emulator as described at https://en.wikipedia.org/wiki/CHIP-8#Virtual_machine_description.

use crate::instruction::Instruction;

pub struct Emulator {
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: usize,
    screen: [[bool; 64]; 32]
}

impl Emulator {

    /// Construct an emulator from 
    pub fn new(memory: &[u8]) -> Emulator {
        let mut emulator = Emulator {
            memory: [0; 4096],
            registers: [0; 16],
            pc: 512,
            screen: [[false; 64]; 32]
        };
        for i in 0..std::cmp::min(memory.len(), emulator.memory.len()) {
            emulator.memory[i] = memory[i];
        }
        emulator
    }

    /// Perform a single step by executing one instruction
    pub fn step(&mut self) -> bool {
        // Each opcode is two bytes
        let left = self.memory[self.pc];
        let right = self.memory[self.pc + 1];
        let instruction = Instruction::from_two_u8(left, right);
        self.pc += 2;

        match instruction {
            _ => {
                println!("Stepped");
                true
            }
        }
    }

    pub fn execute(&mut self) {
        while self.step() {};
    }
}