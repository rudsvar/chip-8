/// A structure for easily splitting an opcode
/// into different formats, such as a single `u16`,
/// two `u8`, four `u8` or similar.
pub struct BitSplitter(u8, u8);

impl BitSplitter {

    pub fn new(left: u8, right: u8) -> BitSplitter {
        BitSplitter(left, right)
    }

    /// Left-shift the first u8-component 8 bits,
    /// then take bitwise or with the second component
    /// in order to store the components in a u16.
    pub fn as_u16(&self) -> u16 {
        ((self.0 as u16) << 8) | self.1 as u16
    }

    /// Return the two u8-components as a tuple
    pub fn as_two_u8(&self) -> (u8, u8) {
        (self.0, self.1)
    }

    pub fn as_four_u8(&self) -> (u8, u8, u8, u8) {
        let four_last_bits_mask = 0x0F;
        (
            (self.0 >> 4) & four_last_bits_mask,
            (self.0 >> 0) & four_last_bits_mask,
            (self.1 >> 4) & four_last_bits_mask,
            (self.1 >> 0) & four_last_bits_mask
        )
    }

    pub fn last_8_bits(&self) -> u8 {
        self.1
    }

    pub fn last_12_bits(&self) -> u16 {
        self.as_u16() & 0x0FFF
    }
}