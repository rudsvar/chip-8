use std::collections::HashMap;

/// Represents an output device that can be written to.
pub trait EmulatorOutput {
    fn set(&mut self, x: usize, y: usize, state: u8);
    fn get(&self, x: usize, y: usize) -> u8;
    fn clear(&mut self);
    fn refresh(&mut self);
}

/// A simple output device that keeps track of set coordinates.
pub struct DummyOutput {
    screen: HashMap<(usize, usize), u8>,
}

impl DummyOutput {
    pub fn new() -> DummyOutput {
        DummyOutput {
            screen: HashMap::new(),
        }
    }
}

impl Default for DummyOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl EmulatorOutput for DummyOutput {
    fn set(&mut self, x: usize, y: usize, state: u8) {
        self.screen.insert((x, y), state);
    }
    fn get(&self, x: usize, y: usize) -> u8 {
        match self.screen.get(&(x, y)) {
            Some(value) => *value,
            None => 0,
        }
    }
    fn clear(&mut self) {
        self.screen.clear();
    }
    fn refresh(&mut self) {}
}
