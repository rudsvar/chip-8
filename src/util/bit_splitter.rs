/// A structure for easily splitting an opcode
/// into different formats, such as a single `u16`,
/// two `u8`, four `u8` or similar.
pub struct BitSplitter(u8, u8);

impl BitSplitter {

    pub fn from_u16(value: u16) -> BitSplitter {
        BitSplitter((value >> 8) as u8, (value & 0x00FF) as u8)
    }

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

    pub fn get(&self, start: usize, end: usize) -> Option<u16> {
        if end > 16 {
            return None;
        }
        let mut value = self.as_u16();
        value &= 0xFFFF >> start; // Remove start we don't want
        value >>= 16 - end; // Remove end we don't want and shift to lowest position
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_from_start_behaves_correctly() {
        assert_eq!(Some(0xFFFF), BitSplitter::from_u16(0xFFFF).get(0, 16));
        assert_eq!(Some(0xFFF), BitSplitter::from_u16(0xFFFF).get(0, 12));
        assert_eq!(Some(0xFF), BitSplitter::from_u16(0xFFFF).get(0, 8));
        assert_eq!(Some(0xF), BitSplitter::from_u16(0xFFFF).get(0, 4));
        assert_eq!(Some(0x1), BitSplitter::from_u16(0xFFFF).get(0, 1));
    }

    #[test]
    fn get_in_middle_behaves_correctly() {
        assert_eq!(Some(0xAB), BitSplitter::from_u16(0xABCD).get(0, 8));
        assert_eq!(Some(0xBC), BitSplitter::from_u16(0xABCD).get(4, 12));
        assert_eq!(Some(0xBCD), BitSplitter::from_u16(0xABCD).get(4, 16));
    }

    #[test]
    fn get_components() {
        assert_eq!(Some(0xA), BitSplitter::from_u16(0xABCD).get(0, 4));
        assert_eq!(Some(0xB), BitSplitter::from_u16(0xABCD).get(4, 8));
        assert_eq!(Some(0xC), BitSplitter::from_u16(0xABCD).get(8, 12));
        assert_eq!(Some(0xD), BitSplitter::from_u16(0xABCD).get(12, 16));
    }
}