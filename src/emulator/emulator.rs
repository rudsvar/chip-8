//! The CHIP-8 emulator as described at https://en.wikipedia.org/wiki/CHIP-8#Virtual_machine_description.

use crate::emulator::input::{DummyInput, EmulatorInput};
use crate::emulator::instruction::*;
use crate::emulator::output::{DummyOutput, EmulatorOutput};

const MEM_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 256;
const PC_START: u16 = 0x200;
const FONT: [u8; 80] = [
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

pub struct Emulator<I: EmulatorInput, O: EmulatorOutput> {
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
    output: O,
}

impl Emulator<DummyInput, DummyOutput> {
    /// Create a new emulator with dummy input and output
    pub fn dummy() -> Emulator<DummyInput, DummyOutput> {
        Emulator::new(DummyInput, DummyOutput::new())
    }
}

impl<I: EmulatorInput, O: EmulatorOutput> Emulator<I, O> {
    /// Create a new emulator with input and output
    pub fn new(input: I, output: O) -> Self {
        let mut memory = [0; MEM_SIZE];

        // Load font
        memory[0..FONT.len()].copy_from_slice(&FONT);

        Emulator {
            memory,
            registers: [0; NUM_REGISTERS],
            delay_timer: 0,
            sound_timer: 0,
            i: 0,
            program_counter: PC_START,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],

            input,
            output,
        }
    }

    /// Copy a program into memory at 0x200.
    pub fn load(&mut self, program: &[u8]) {
        let pc = self.program_counter as usize;
        let len = std::cmp::min(program.len(), self.memory.len() - pc);
        self.memory[pc..pc + len].copy_from_slice(program);
    }

    /// Perform a single step, which will update timers,
    /// then load an instruction and execute it.
    pub fn step(&mut self) {
        // Each opcode is two bytes
        let left = self.memory[self.program_counter as usize];
        let right = self.memory[self.program_counter as usize + 1];
        let instruction = Instruction::from_two_u8(left, right);

        self.execute_single(instruction);
    }

    /// Execute many instructions in succession
    pub fn execute_many(&mut self, instructions: &[Instruction]) {
        for instruction in instructions {
            self.execute_single(*instruction);
        }
    }

    /// Execute a single instruction
    pub fn execute_single(&mut self, instruction: Instruction) {
        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        log::trace!("{:?}", instruction);

        self.program_counter += 2;

        match instruction {
            // Clear the screen
            Instruction::ClearScreen => {
                self.output.clear();
            }

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
                self.registers[x as usize] = self.registers[x as usize].overflowing_add(n).0;
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
            // Set VF to 1 if there is a carry, 0 otherwise.
            Instruction::IncRegByReg(Reg(x), Reg(y)) => {
                let x_value = self.registers[x as usize];
                let y_value = self.registers[y as usize];
                let (sum, overflow) = x_value.overflowing_add(y_value);
                self.registers[x as usize] = sum;
                self.registers[0xF] = if overflow { 1 } else { 0 };
            }

            // Decrement the value of a register by the value of another
            // Set VF to 0 if there is a borrow, 1 otherwise.
            Instruction::DecRegByReg(Reg(x), Reg(y)) => {
                let x_value = self.registers[x as usize];
                let y_value = self.registers[y as usize];
                let (sum, overflow) = x_value.overflowing_sub(y_value);
                self.registers[x as usize] = sum;
                self.registers[0xF] = if overflow { 0 } else { 1 };
            }

            // Store least significant bit in VF then shift right
            Instruction::BitshiftRight(Reg(x)) => {
                let value = self.registers[x as usize];
                self.registers[0xF] = value & 1;
                self.registers[x as usize] >>= 1;
            }

            // Set VF to 0 when there's a borrow, and 1 when there isn't.
            Instruction::SetVxVyMinusVx(Reg(x), Reg(y)) => {
                let x_value = self.registers[x as usize];
                let y_value = self.registers[y as usize];
                let (sum, borrow) = y_value.overflowing_sub(x_value);
                self.registers[x as usize] = sum;
                self.registers[0xF] = if borrow { 0 } else { 1 };
            }

            // Store most significant bit in VF then shift left
            Instruction::BitshiftLeft(Reg(x)) => {
                let value = self.registers[x as usize];
                self.registers[0xF] = (value & 0b10000000) >> 7;
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
                let sprite_data: &[u8] =
                    &self.memory[sprite_addr..sprite_addr + sprite_height as usize];

                // Write to screen
                let mut any_collisions = 0;
                for (h, row) in sprite_data.iter().enumerate() {
                    for w in 0..8 {
                        let new_pixel = row >> (7 - w) & 1; // Get bit number `bit_idx`
                        let old_pixel = self.output.get(x_coord + w, y_coord + h);
                        let xored_pixel = old_pixel ^ new_pixel; // XOR old pixel with new pixel
                        self.output.set(x_coord + w, y_coord + h, xored_pixel); // Save xor'ed pixel

                        // Set pixel was unset, so we set the collision flag
                        if old_pixel == 1 && xored_pixel == 0 {
                            any_collisions = 1;
                        }
                    }
                }

                // Set VF collision flag
                self.registers[0xF] = any_collisions;
            }

            // Skip if the key in Vx is pressed
            Instruction::IfKeyEqVx(Reg(x)) => {
                if self.input.get_key() == Some(self.registers[x as usize]) {
                    self.program_counter += 2;
                }
            }

            // Skip if the key in Vx isn't pressed
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

            // Set i to character address. Each font element is 5 bytes wide.
            Instruction::SetIToSpriteAddrVx(Reg(x)) => {
                self.i = 5 * self.registers[x as usize] as u16;
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
                self.memory[i] = (hundredths % 10) as u8;
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
    use test_case::test_case;

    const X: u8 = 0xA;
    const Y: u8 = 0xB;

    #[test]
    fn clear_screen_clears_screen() {
        let mut emulator = Emulator::dummy();
        emulator.output.set(0, 0, 1);
        emulator.output.set(4, 8, 2);
        emulator.output.set(3, 5, 3);
        emulator.execute_single(Instruction::ClearScreen);
        assert_eq!(emulator.output.get(0, 0), 0);
        assert_eq!(emulator.output.get(4, 8), 0);
        assert_eq!(emulator.output.get(3, 5), 0);
    }

    #[test_case(0x200; "when addr is 0x200")]
    #[test_case(0x250; "when addr is 0x250")]
    #[test_case(0x350; "when addr is 0x350")]
    fn goto_goes_to(addr: u16) {
        let mut emulator = Emulator::dummy();
        emulator.execute_single(Instruction::Goto(Addr(addr)));
        assert_eq!(emulator.program_counter, addr);
    }

    #[test]
    fn return_after_call_is_neutral() {
        // Create emulator
        let mut emulator = Emulator::dummy();
        assert_eq!(emulator.program_counter, 0x200);

        // Write program with call and return
        let program = [
            0x22, 0x06, // 0x00, jump to 0x206
            0x00, 0x00, // 0x02
            0x00, 0x00, // 0x04
            0x00, 0xEE, // 0x06, return
        ];
        emulator.load(&program);

        // Run the program
        emulator.step(); // Call 0x206
        assert_eq!(emulator.program_counter, 0x206);
        emulator.step(); // Return to 202
        assert_eq!(emulator.program_counter, 0x202);
    }

    #[test_case(0, 0 => 0x204; "skips when equal")]
    #[test_case(3, 8 => 0x202; "does not skip when not equal")]
    fn if_reg_eq_const(reg_value: u8, const_value: u8) -> u16 {
        let mut emulator = Emulator::dummy();
        emulator.registers[X as usize] = reg_value;
        emulator.execute_single(Instruction::IfRegEqConst(Reg(X), Const(const_value)));
        emulator.program_counter
    }

    #[test_case(0, 0 => 0x202; "does not skip when equal")]
    #[test_case(3, 8 => 0x204; "skips when not equal")]
    fn if_reg_neq_const(reg_value: u8, const_value: u8) -> u16 {
        let mut emulator = Emulator::dummy();
        emulator.registers[X as usize] = reg_value;
        emulator.execute_single(Instruction::IfRegNeqConst(Reg(X), Const(const_value)));
        emulator.program_counter
    }

    #[test]
    fn if_reg_eq_reg() {
        let mut emulator = Emulator::dummy();

        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(3)),
            Instruction::SetRegToConst(Reg(Y), Const(5)),
        ]);

        // Should not skip instruction
        assert_eq!(emulator.program_counter, 0x204);
        emulator.execute_single(Instruction::IfRegEqReg(Reg(X), Reg(Y)));
        assert_eq!(emulator.program_counter, 0x206);

        // Should skip instruction
        emulator.execute_single(Instruction::SetRegToConst(Reg(Y), Const(3)));
        assert_eq!(emulator.program_counter, 0x208);
        emulator.execute_single(Instruction::IfRegEqReg(Reg(X), Reg(Y)));
        assert_eq!(emulator.program_counter, 0x20C);
    }

    #[test]
    fn set_reg_to_const() {
        let mut emulator = Emulator::dummy();
        let value = 7;
        assert_eq!(emulator.registers[X as usize], 0);
        emulator.execute_single(Instruction::SetRegToConst(Reg(X), Const(value)));
        assert_eq!(emulator.registers[X as usize], value);
    }

    #[test]
    fn inc_reg_by_const() {
        let mut emulator = Emulator::dummy();
        let value = 7;
        assert_eq!(emulator.registers[X as usize], 0);
        emulator.execute_single(Instruction::IncRegByConst(Reg(X), Const(value)));
        assert_eq!(emulator.registers[X as usize], value);
        emulator.execute_single(Instruction::IncRegByConst(Reg(X), Const(value)));
        assert_eq!(emulator.registers[X as usize], 2 * value);
    }

    #[test]
    fn set_reg_to_reg() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(4)),
            Instruction::SetRegToConst(Reg(Y), Const(8)),
            Instruction::SetRegToReg(Reg(X), Reg(Y)),
        ]);
        assert_eq!(emulator.registers[X as usize], 8);
    }

    #[test]
    fn bitwise_or() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(0xA), Const(0b0101)),
            Instruction::SetRegToConst(Reg(0xB), Const(0b1100)),
            Instruction::BitwiseOr(Reg(0xA), Reg(0xB)),
        ]);
        assert_eq!(emulator.registers[0xA], 0b1101);
    }

    #[test]
    fn bitwise_and() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(0xA), Const(0b0101)),
            Instruction::SetRegToConst(Reg(0xB), Const(0b1101)),
            Instruction::BitwiseAnd(Reg(0xA), Reg(0xB)),
        ]);
        assert_eq!(emulator.registers[0xA], 0b0101);
    }

    #[test]
    fn bitwise_xor() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(0xA), Const(0b010101)),
            Instruction::SetRegToConst(Reg(0xB), Const(0b110111)),
            Instruction::BitwiseXor(Reg(0xA), Reg(0xB)),
        ]);
        assert_eq!(emulator.registers[0xA], 0b100010);
    }

    #[test_case(3, 7 => (10, 0); "case 1")]
    #[test_case(6, 24 => (30, 0); "case 2")]
    #[test_case(78, 10 => (88, 0); "case 3")]
    #[test_case(75, 240 => (59, 1); "when result overflows")]
    fn inc_reg_by_reg(x_value: u8, y_value: u8) -> (u8, u8) {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(x_value)),
            Instruction::SetRegToConst(Reg(Y), Const(y_value)),
            Instruction::IncRegByReg(Reg(X), Reg(Y)),
        ]);
        (emulator.registers[X as usize], emulator.registers[0xF])
    }

    #[test]
    fn dec_reg_by_reg() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(10)),
            Instruction::SetRegToConst(Reg(Y), Const(7)),
            Instruction::DecRegByReg(Reg(X), Reg(Y)),
        ]);
        assert_eq!(emulator.registers[X as usize], 3);
    }

    #[test]
    fn dec_reg_by_reg_underflow() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(5)),
            Instruction::SetRegToConst(Reg(Y), Const(45)),
            Instruction::DecRegByReg(Reg(X), Reg(Y)),
        ]);
        assert_eq!(emulator.registers[X as usize], 216);
    }

    #[test]
    fn bitshift_right() {
        let mut emulator = Emulator::dummy();

        let value = 0b00001011;
        emulator.execute_single(Instruction::SetRegToConst(Reg(X), Const(value)));

        emulator.execute_single(Instruction::BitshiftRight(Reg(X)));
        assert_eq!(emulator.registers[X as usize], value >> 1);
        assert_eq!(emulator.registers[0xF], 1);
        emulator.execute_single(Instruction::BitshiftRight(Reg(X)));
        assert_eq!(emulator.registers[X as usize], value >> 2);
        assert_eq!(emulator.registers[0xF], 1);
        emulator.execute_single(Instruction::BitshiftRight(Reg(X)));
        assert_eq!(emulator.registers[X as usize], value >> 3);
        assert_eq!(emulator.registers[0xF], 0);
    }

    #[test]
    fn set_vx_vy_minus_vx() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(12)),
            Instruction::SetRegToConst(Reg(Y), Const(14)),
            Instruction::SetVxVyMinusVx(Reg(X), Reg(Y)),
        ]);
        assert_eq!(emulator.registers[X as usize], 2);
        assert_eq!(emulator.registers[0xF], 1);
    }

    #[test]
    fn set_vx_vy_minus_vx_borrow() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(20)),
            Instruction::SetRegToConst(Reg(Y), Const(14)),
            Instruction::SetVxVyMinusVx(Reg(X), Reg(Y)),
        ]);
        assert_eq!(emulator.registers[X as usize], 250);
        assert_eq!(emulator.registers[0xF], 0);
    }

    #[test]
    fn bitshift_left() {
        let mut emulator = Emulator::dummy();

        let value = 0b10110111;
        emulator.execute_single(Instruction::SetRegToConst(Reg(X), Const(value)));

        emulator.execute_single(Instruction::BitshiftLeft(Reg(X)));
        assert_eq!(emulator.registers[X as usize], value << 1);
        assert_eq!(emulator.registers[0xF], 1);
        emulator.execute_single(Instruction::BitshiftLeft(Reg(X)));
        assert_eq!(emulator.registers[X as usize], value << 2);
        assert_eq!(emulator.registers[0xF], 0);
        emulator.execute_single(Instruction::BitshiftLeft(Reg(X)));
        assert_eq!(emulator.registers[X as usize], value << 3);
        assert_eq!(emulator.registers[0xF], 1);
    }

    #[test]
    fn if_reg_neq_reg() {
        let mut emulator = Emulator::dummy();

        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(3)),
            Instruction::SetRegToConst(Reg(Y), Const(5)),
        ]);

        // Should skip instruction
        assert_eq!(emulator.program_counter, 0x204);
        emulator.execute_single(Instruction::IfRegNeqReg(Reg(X), Reg(Y)));
        assert_eq!(emulator.program_counter, 0x208);

        // Should not skip instruction
        emulator.execute_single(Instruction::SetRegToConst(Reg(Y), Const(3)));
        assert_eq!(emulator.program_counter, 0x20A);
        emulator.execute_single(Instruction::IfRegNeqReg(Reg(X), Reg(Y)));
        assert_eq!(emulator.program_counter, 0x20C);
    }

    #[test]
    fn set_i() {
        let mut emulator = Emulator::dummy();
        assert_eq!(emulator.i, 0x0);
        emulator.execute_single(Instruction::SetI(Addr(0x232)));
        assert_eq!(emulator.i, 0x232);
    }

    #[test]
    fn set_pc_to_v0_plus_addr() {
        let mut emulator = Emulator::dummy();
        let v0 = 7;
        let addr = 0x400;
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(0x0), Const(v0)),
            Instruction::SetPcToV0PlusAddr(Addr(addr)),
        ]);
        assert_eq!(emulator.program_counter, v0 as u16 + addr);
    }

    #[test]
    fn set_vx_rand() {
        let mut emulator = Emulator::dummy();
        for _ in 0..10_000 {
            emulator.execute_single(Instruction::SetVxRand(Reg(X), Const(0x0F)));
            let value = emulator.registers[X as usize];
            assert!(value < 2u8.pow(4));
        }
    }

    #[test]
    fn draw() {
        let mut emulator = Emulator::dummy();
        let program = [0b01001111, 0b01111001, 0b00101011, 0b01010110];
        emulator.load(&program);
        emulator.execute_many(&[
            Instruction::SetI(Addr(0x200)),
            Instruction::SetRegToConst(Reg(X), Const(0)),
            Instruction::SetRegToConst(Reg(Y), Const(0)),
            Instruction::Draw(Reg(X), Reg(Y), Const(program.len() as u8)),
        ]);
        for (h, row) in program.iter().enumerate() {
            for w in 0..8 {
                assert_eq!(emulator.output.get(w, h), (row >> (7 - w)) & 1);
            }
        }
    }

    /// Input that always presses a given key.
    struct ConstantInput(u8);
    impl EmulatorInput for ConstantInput {
        fn get_key(&self) -> Option<u8> {
            Some(self.0)
        }
        fn get_key_blocking(&self) -> u8 {
            self.0
        }
    }

    #[test]
    fn if_key_eq_vx() {
        let mut emulator = Emulator::new(ConstantInput(0), DummyOutput::new());

        // Skip since both are 0
        emulator.execute_single(Instruction::IfKeyEqVx(Reg(X)));
        assert_eq!(emulator.program_counter, 0x204);

        // Don't skip. Input is 0, Vx is 5
        emulator.registers[0xA] = 5;
        emulator.execute_single(Instruction::IfKeyEqVx(Reg(X)));
        assert_eq!(emulator.program_counter, 0x206);
    }

    #[test]
    fn if_key_neq_vx() {
        let mut emulator = Emulator::new(ConstantInput(0), DummyOutput::new());

        // Don't skip since both are 0
        emulator.execute_single(Instruction::IfKeyNeqVx(Reg(X)));
        assert_eq!(emulator.program_counter, 0x202);

        // Skip since they are different
        emulator.registers[0xA] = 5;
        emulator.execute_single(Instruction::IfKeyNeqVx(Reg(X)));
        assert_eq!(emulator.program_counter, 0x206);
    }

    #[test]
    fn set_reg_to_delay_timer() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(123)),
            Instruction::SetDelayTimerToReg(Reg(X)),
        ]);
        assert_eq!(emulator.delay_timer, 123);
        emulator.execute_single(Instruction::SetRegToDelayTimer(Reg(Y)));
        // Decreases one in previous step
        assert_eq!(emulator.registers[Y as usize], 122);
    }

    #[test]
    fn set_reg_to_get_key() {
        let mut emulator = Emulator::new(ConstantInput(9), DummyOutput::new());
        emulator.execute_single(Instruction::SetRegToGetKey(Reg(X)));
        assert_eq!(emulator.registers[X as usize], 9);
    }

    #[test]
    fn set_delay_timer() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(97)),
            Instruction::SetDelayTimerToReg(Reg(X)),
        ]);
        assert_eq!(emulator.delay_timer, 97);
    }

    #[test]
    fn set_sound_timer() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(97)),
            Instruction::SetSoundTimerToReg(Reg(X)),
        ]);
        assert_eq!(emulator.sound_timer, 97);
    }

    #[test]
    fn add_reg_to_i() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(32)),
            Instruction::SetI(Addr(32)),
            Instruction::AddRegToI(Reg(X)),
        ]);
        assert_eq!(emulator.i, 64);
    }

    #[test]
    fn set_i_to_sprite_addr_vx() {
        let mut emulator = Emulator::dummy();
        let sprite_no = 8;
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(sprite_no)),
            Instruction::SetIToSpriteAddrVx(Reg(X)),
        ]);
        assert_eq!(emulator.i, 5 * sprite_no as u16); // Each sprite is 5 bytes wide
    }

    #[test]
    fn set_i_to_bcd_of_reg() {
        let mut emulator = Emulator::dummy();
        emulator.execute_many(&[
            Instruction::SetRegToConst(Reg(X), Const(184)),
            Instruction::SetIToBcdOfReg(Reg(X)),
        ]);
        assert_eq!(emulator.memory[emulator.i as usize], 1);
        assert_eq!(emulator.memory[emulator.i as usize + 1], 8);
        assert_eq!(emulator.memory[emulator.i as usize + 2], 4);
    }

    #[test]
    fn reg_dump() {
        let mut emulator = Emulator::dummy();

        // Set V0..V0xF to their index
        for i in 0..=0xF {
            emulator.execute_single(Instruction::SetRegToConst(Reg(i), Const(i)));
        }

        // Dump all up to Vx
        emulator.execute_single(Instruction::SetI(Addr(0x200)));
        emulator.execute_single(Instruction::RegDump(Reg(X)));

        // All register values up to x should be in memory
        for offset in 0..X {
            assert_eq!(
                emulator.memory[emulator.i as usize + offset as usize],
                offset
            );
        }

        // The others should not have been dumped
        for offset in X + 1..=0xF {
            assert_eq!(emulator.memory[emulator.i as usize + offset as usize], 0);
        }
    }

    #[test]
    fn reg_load() {
        let mut emulator = Emulator::dummy();

        // Add some data
        let mut data = Vec::new();
        for offset in 0..=0xF {
            data.push(offset);
        }
        emulator.load(&data);

        // Load into registers
        emulator.execute_single(Instruction::SetI(Addr(0x200)));
        emulator.execute_single(Instruction::RegLoad(Reg(X)));

        // The others should not have been dumped
        for offset in 0..=X {
            assert_eq!(emulator.registers[offset as usize], offset);
        }

        // The others should not have been dumped
        for offset in X + 1..=0xF {
            assert_eq!(emulator.registers[offset as usize], 0);
        }
    }
}
