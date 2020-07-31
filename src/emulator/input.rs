/// Represents an input device that is capable of providing keys in the range 0..0xF.
pub trait EmulatorInput {
    fn get_key(&self) -> Option<u8>;
    fn get_key_blocking(&self) -> u8;
}

/// An input device that never provides any input
pub struct DummyInput;

impl EmulatorInput for DummyInput {
    fn get_key(&self) -> Option<u8> {
        None
    }
    fn get_key_blocking(&self) -> u8 {
        0
    }
}
