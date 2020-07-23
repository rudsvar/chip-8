//! The CHIP-8 emulator as described at https://en.wikipedia.org/wiki/CHIP-8#Virtual_machine_description.

use crate::instruction::Instruction;

pub struct Emulator {
    memory: [u8; 4096],
    registers: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    I: u16,
    PC: u16,
    SP: u8,
    stack: [u16; 256],
    screen: [[bool; 64]; 32]
}

impl Emulator {

    /// Construct an emulator from 
    pub fn new(memory: &[u8]) -> Emulator {
        let mut emulator = Emulator {
            memory: [0; 4096],
            registers: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            I: 0,
            PC: 0x200,
            SP: 0,
            stack: [0; 256],
            screen: [[false; 64]; 32]
        };
        for i in 0..std::cmp::min(memory.len(), emulator.memory.len() - 0x200) {
            emulator.memory[emulator.PC as usize + i] = memory[i];
        }
        emulator
    }

    /// Perform a single step by executing one instruction
    pub fn step(&mut self) -> bool {
        // Each opcode is two bytes
        let left = self.memory[self.PC as usize];
        let right = self.memory[self.PC as usize + 1];
        let instruction = Instruction::from_two_u8(left, right);
        self.PC += 2;

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        match instruction {
            Instruction::Halt => false,
            i => {
                println!("Executing {:?}", i);
                true
            }
        }
    }

    pub fn execute(&mut self) {
        while self.step() {};
    }
}