use crate::util::bit_splitter::BitSplitter;

/// A wrapper for addresses.
#[derive(Debug, PartialEq, Eq)]
pub struct Addr(pub u16);

/// A wrapper for registers.
#[derive(Debug, PartialEq, Eq)]
pub struct Reg(pub u8);

/// A wrapper for constants.
#[derive(Debug, PartialEq, Eq)]
pub struct Const(pub u8);

/// A single instruction from the CHIP-8 instruction set.
/// Two bytes written in hexadecimal, with the following special characters:
/// - NNN: address
/// - NN: 8-bit constant
/// - N: 4-bit constant
/// - X and Y: 4-bit register identifier
/// - PC: Program counter
/// - I: 16 bit register for memory address
/// - VN: One of the 16 available variables (register identifiers)
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    ClearScreen, // 00E0
    Return, // 00EE
    Goto(Addr), // 1NNN
    Call(Addr), // 2NNN
    IfRegEqConst(Reg, Const), // 3XNN
    IfRegNeqConst(Reg, Const), // 4XNN
    IfRegEqReg(Reg, Reg), // 5XY0
    SetRegToConst(Reg, Const), // 6XNN
    IncRegByConst(Reg, Const), // 7XNN
    SetRegToReg(Reg, Reg), // 8XY0
    BitwiseOr(Reg, Reg), // 8XY1
    BitwiseAnd(Reg, Reg), // 8XY2
    BitwiseXor(Reg, Reg), // 8XY3
    IncRegByReg(Reg, Reg), // 8XY4
    DecRegByReg(Reg, Reg), // 8XY5
    BitshiftRight(Reg), // 8XY6
    SetVxVyMinusVx(Reg, Reg), // 8XY7
    BitshiftLeft(Reg), // 8XYE
    IfRegNeqReg(Reg, Reg), // 9XY0
    SetI(Addr), // ANNN
    SetPcToV0PlusAddr(Addr), // BNNN
    SetVxRand(Reg, Const), // CXNN
    Draw(Reg, Reg, Const), // DXYN
    IfKeyEqVx(Reg), // EX9E
    IfKeyNeqVx(Reg), // EXA1
    SetRegToDelayTimer(Reg), // FX07
    SetRegToGetKey(Reg), // FX0A
    SetDelayTimerToReg(Reg), // FX15
    SetSoundTimerToReg(Reg), // FX18
    AddRegToI(Reg), // FX1E
    SetIToSpriteAddrVx(Reg), // FX29
    SetIToBcdOfReg(Reg), // FX33
    RegDump(Reg), // FX55
    RegLoad(Reg) // FX65
}

impl Instruction {

    fn split_u16(value: u16) -> (u8, u8) {
        let left = (value & 0xFF00) >> 8;
        let right = value & 0x00FF;
        (left as u8, right as u8)
    }

    pub fn from_u16(value: u16) -> Instruction {
        let (left, right) = Self::split_u16(value);
        Instruction::from_two_u8(left, right)
    }

    pub fn from_two_u8(left: u8, right: u8) -> Instruction {
        let opcode = BitSplitter::new(left, right);
        match opcode.as_four_u8() {
            (0, 0, 0xE, 0) => Instruction::ClearScreen,
            (0, 0, 0xE, 0xE) => Instruction::Return,
            (1, _, _, _) => Instruction::Goto(Addr(opcode.last_12_bits())),
            (2, _, _, _) => Instruction::Call(Addr(opcode.last_12_bits())),
            (3, x, _, _) => Instruction::IfRegEqConst(Reg(x), Const(opcode.last_8_bits())),
            (4, x, _, _) => Instruction::IfRegNeqConst(Reg(x), Const(opcode.last_8_bits())),
            (5, x, y, 0) => Instruction::IfRegEqReg(Reg(x), Reg(y)),
            (6, x, _, _) => Instruction::SetRegToConst(Reg(x), Const(opcode.last_8_bits())),
            (7, x, _, _) => Instruction::IncRegByConst(Reg(x), Const(opcode.last_8_bits())),
            (8, x, y, 0) => Instruction::SetRegToReg(Reg(x), Reg(y)),
            (8, x, y, 1) => Instruction::BitwiseOr(Reg(x), Reg(y)),
            (8, x, y, 2) => Instruction::BitwiseAnd(Reg(x), Reg(y)),
            (8, x, y, 3) => Instruction::BitwiseXor(Reg(x), Reg(y)),
            (8, x, y, 4) => Instruction::IncRegByReg(Reg(x), Reg(y)),
            (8, x, y, 5) => Instruction::DecRegByReg(Reg(x), Reg(y)),
            (8, x, _, 6) => Instruction::BitshiftRight(Reg(x)),
            (8, x, y, 7) => Instruction::SetVxVyMinusVx(Reg(x), Reg(y)),
            (8, x, _, 0xE) => Instruction::BitshiftLeft(Reg(x)),
            (9, x, y, 0) => Instruction::IfRegNeqReg(Reg(x), Reg(y)),
            (0xA, _, _, _) => Instruction::SetI(Addr(opcode.last_12_bits())),
            (0xB, _, _, _) => Instruction::SetPcToV0PlusAddr(Addr(opcode.last_12_bits())),
            (0xC, x, _, _) => Instruction::SetVxRand(Reg(x), Const(opcode.last_8_bits())),
            (0xD, x, y, n) => Instruction::Draw(Reg(x), Reg(y), Const(n)),
            (0xE, x, 9, 0xE) => Instruction::IfKeyEqVx(Reg(x)),
            (0xE, x, 0xA, 1) => Instruction::IfKeyNeqVx(Reg(x)),
            (0xF, x, 0, 7) => Instruction::SetRegToDelayTimer(Reg(x)),
            (0xF, x, 0, 0xA) => Instruction::SetRegToGetKey(Reg(x)),
            (0xF, x, 1, 5) => Instruction::SetDelayTimerToReg(Reg(x)),
            (0xF, x, 1, 8) => Instruction::SetSoundTimerToReg(Reg(x)),
            (0xF, x, 1, 0xE) => Instruction::AddRegToI(Reg(x)),
            (0xF, x, 2, 9) => Instruction::SetIToSpriteAddrVx(Reg(x)),
            (0xF, x, 3, 3) => Instruction::SetIToBcdOfReg(Reg(x)),
            (0xF, x, 5, 5) => Instruction::RegDump(Reg(x)),
            (0xF, x, 6, 5) => Instruction::RegLoad(Reg(x)),
            _ => {
                // TODO: Use Option?
                log::error!("Unknown opcode {:#06x}", opcode.as_u16());
                panic!("Unknown opcode!")
            }
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn opcodes_are_parsed_correctly() {
        assert_eq!(Instruction::ClearScreen, Instruction::from_u16(0x00E0));
        assert_eq!(Instruction::Return, Instruction::from_u16(0x00EE));
        assert_eq!(Instruction::Goto(Addr(0x25)), Instruction::from_u16(0x1025));
        assert_eq!(Instruction::Call(Addr(0x37)), Instruction::from_u16(0x2037));
        assert_eq!(Instruction::IfRegEqConst(Reg(0xA), Const(8)), Instruction::from_u16(0x3A08));
        assert_eq!(Instruction::IfRegNeqConst(Reg(0xA), Const(8)), Instruction::from_u16(0x4A08));
        assert_eq!(Instruction::IfRegNeqConst(Reg(0xA), Const(8)), Instruction::from_u16(0x4A08));
        assert_eq!(Instruction::SetRegToConst(Reg(0xB), Const(0x23)), Instruction::from_u16(0x6B23));
        assert_eq!(Instruction::IncRegByConst(Reg(0xC), Const(0xA1)), Instruction::from_u16(0x7CA1));
        assert_eq!(Instruction::SetRegToReg(Reg(0xA), Reg(0xB)), Instruction::from_u16(0x8AB0));
        assert_eq!(Instruction::BitwiseOr(Reg(0xD), Reg(0xE)), Instruction::from_u16(0x8DE1));
        assert_eq!(Instruction::BitwiseAnd(Reg(0xD), Reg(0xE)), Instruction::from_u16(0x8DE2));
        assert_eq!(Instruction::BitwiseXor(Reg(0xD), Reg(0xE)), Instruction::from_u16(0x8DE3));
        assert_eq!(Instruction::IncRegByReg(Reg(0xA), Reg(0xB)), Instruction::from_u16(0x8AB4));
        assert_eq!(Instruction::DecRegByReg(Reg(0xA), Reg(0xB)), Instruction::from_u16(0x8AB5));
        assert_eq!(Instruction::BitshiftRight(Reg(0xA)), Instruction::from_u16(0x8AB6));
        assert_eq!(Instruction::SetVxVyMinusVx(Reg(0xA), Reg(0xB)), Instruction::from_u16(0x8AB7));
        assert_eq!(Instruction::BitshiftLeft(Reg(0xA)), Instruction::from_u16(0x8A0E));
        assert_eq!(Instruction::IfRegNeqReg(Reg(0xA), Reg(0xB)), Instruction::from_u16(0x9AB0));
        assert_eq!(Instruction::SetI(Addr(0x25)), Instruction::from_u16(0xA025));
        assert_eq!(Instruction::SetPcToV0PlusAddr(Addr(0x25)), Instruction::from_u16(0xB025));
        assert_eq!(Instruction::SetVxRand(Reg(0xA), Const(0x23)), Instruction::from_u16(0xCA23));
        assert_eq!(Instruction::Draw(Reg(0xA), Reg(0xB), Const(0xC)), Instruction::from_u16(0xDABC));
        assert_eq!(Instruction::IfKeyEqVx(Reg(0xA)), Instruction::from_u16(0xEA9E));
        assert_eq!(Instruction::IfKeyNeqVx(Reg(0xA)), Instruction::from_u16(0xEAA1));
        assert_eq!(Instruction::SetRegToDelayTimer(Reg(0xA)), Instruction::from_u16(0xFA07));
        assert_eq!(Instruction::SetRegToGetKey(Reg(0xA)), Instruction::from_u16(0xFA0A));
        assert_eq!(Instruction::SetDelayTimerToReg(Reg(0xA)), Instruction::from_u16(0xFA15));
        assert_eq!(Instruction::SetSoundTimerToReg(Reg(0xA)), Instruction::from_u16(0xFA18));
        assert_eq!(Instruction::AddRegToI(Reg(0xA)), Instruction::from_u16(0xFA1E));
        assert_eq!(Instruction::SetIToSpriteAddrVx(Reg(0xA)), Instruction::from_u16(0xFA29));
        assert_eq!(Instruction::SetIToBcdOfReg(Reg(0xA)), Instruction::from_u16(0xFA33));
        assert_eq!(Instruction::RegDump(Reg(0xA)), Instruction::from_u16(0xFA55));
        assert_eq!(Instruction::RegLoad(Reg(0xA)), Instruction::from_u16(0xFA65));
    }

    #[test]
    fn from_two_u8_equals_from_u16() {
        assert_eq!(Instruction::from_two_u8(0x12, 0x34), Instruction::from_u16(0x1234));
        assert_eq!(Instruction::from_two_u8(0x2F, 0x2F), Instruction::from_u16(0x2F2F));
        assert_eq!(Instruction::from_two_u8(0x10, 0x20), Instruction::from_u16(0x1020));

    }

    #[test]
    fn split_u16_test() {
        assert_eq!((0x12, 0x34), Instruction::split_u16(0x1234));
        assert_eq!((0xFF, 0xFF), Instruction::split_u16(0xFFFF));
        assert_eq!((0x00, 0x00), Instruction::split_u16(0x0000));
        assert_eq!((0xF0, 0xF0), Instruction::split_u16(0xF0F0));

    }
}