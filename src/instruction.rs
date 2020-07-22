use crate::bit_splitter::BitSplitter;

/// A wrapper for addresses.
#[derive(Debug, PartialEq, Eq)]
pub struct Addr(u16);

/// A wrapper for registers.
#[derive(Debug, PartialEq, Eq)]
pub struct Reg(u8);

/// A wrapper for constants.
#[derive(Debug, PartialEq, Eq)]
pub struct Const(u8);

/// A single instruction from the CHIP-8 instruction set.
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    ClearScreen,
    Return,
    Goto(Addr),
    Call(Addr),
    IfRegEqConst(Reg, Const),
    IfRegNeqConst(Reg, Const),
    IfRegEqReg(Reg, Reg),
    SetRegToConst(Reg, Const),
    IncRegByConst(Reg, Const),
    SetRegToReg(Reg, Reg),
    BitwiseOr(Reg, Reg),
    BitwiseAnd(Reg, Reg),
    BitwiseXor(Reg, Reg),
    IncRegByReg(Reg, Reg),
    DecRegByReg(Reg, Reg),
    BitshiftRight(Reg)
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
            (8, x, y, 1) => Instruction::BitwiseOr(Reg(x), Reg(y)),
            (8, x, y, 2) => Instruction::BitwiseAnd(Reg(x), Reg(y)),
            (8, x, y, 3) => Instruction::BitwiseXor(Reg(x), Reg(y)),
            (8, x, y, 4) => Instruction::IncRegByReg(Reg(x), Reg(y)),
            (8, x, y, 5) => Instruction::DecRegByReg(Reg(x), Reg(y)),
            (8, x, _, 6) => Instruction::BitshiftRight(Reg(x)),
            _ => panic!("Unknown opcode!") // TODO: Use Option?
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
        assert_eq!(Instruction::BitwiseOr(Reg(0xD), Reg(0xE)), Instruction::from_u16(0x8DE1));
        assert_eq!(Instruction::BitwiseAnd(Reg(0xD), Reg(0xE)), Instruction::from_u16(0x8DE2));
        assert_eq!(Instruction::BitwiseXor(Reg(0xD), Reg(0xE)), Instruction::from_u16(0x8DE3));
        assert_eq!(Instruction::IncRegByReg(Reg(0xA), Reg(0xB)), Instruction::from_u16(0x8AB4));
        assert_eq!(Instruction::DecRegByReg(Reg(0xA), Reg(0xB)), Instruction::from_u16(0x8AB5));
        assert_eq!(Instruction::BitshiftRight(Reg(0xA)), Instruction::from_u16(0x8AB6));
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