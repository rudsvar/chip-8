use chip_8::emulator::{input::EmulatorInput, output::EmulatorOutput};

use super::key_manager::KeyManager;

use crossterm::event::KeyCode;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use std::io::{stdout, Write};

const SCREEN_WIDTH: usize = 128;
const SCREEN_HEIGHT: usize = 64;

pub struct CrosstermInput<'a> {
    key_manager: &'a KeyManager,
}

impl CrosstermInput<'_> {
    pub fn new(key_manager: &KeyManager) -> CrosstermInput {
        CrosstermInput { key_manager }
    }
}

impl EmulatorInput for CrosstermInput<'_> {
    fn get_key(&self) -> Option<u8> {
        let key = self.key_manager.get_key()?;
        key_to_u8(key)
    }

    fn get_key_blocking(&self) -> u8 {
        loop {
            let key = self.key_manager.get_key_blocking();
            if let Some(i) = key_to_u8(key) {
                return i;
            }
        }
    }
}

pub struct CrosstermOutput {
    cells: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

impl CrosstermOutput {
    pub fn new() -> CrosstermOutput {
        execute!(stdout(), EnterAlternateScreen);
        execute!(stdout(), cursor::Hide);
        terminal::enable_raw_mode();
        let bottom = SCREEN_HEIGHT + 2;
        let right = SCREEN_WIDTH + 2;
        for y in 1..=bottom {
            for x in 1..=right {
                if y == 1 || y == bottom || x == 1 || x == right {
                    let c = if y == 1 && x == 1 {
                        '┏'
                    } else if y == 1 && x == right {
                        '┓'
                    } else if y == bottom && x == 1 {
                        '┗'
                    } else if y == bottom && x == right {
                        '┛'
                    } else if y == 1 || y == bottom {
                        '━'
                    } else if x == 1 || x == right {
                        '┃'
                    } else {
                        'X'
                    };
                    execute!(stdout(), cursor::MoveTo(x as u16, y as u16));
                    write!(stdout(), "{}", c).unwrap();
                }
            }
        }
        CrosstermOutput {
            cells: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
        }
    }

    fn draw(&mut self, x: usize, y: usize, state: u8) {
        execute!(stdout(), cursor::MoveTo(2 * x as u16 + 2, y as u16 + 2));
        write!(stdout(), "{}", if state == 1 { "██" } else { "  " }).unwrap();
    }
}

impl Drop for CrosstermOutput {
    fn drop(&mut self) {
        terminal::disable_raw_mode();
        execute!(stdout(), LeaveAlternateScreen);
        execute!(stdout(), cursor::Show);
    }
}

impl EmulatorOutput for CrosstermOutput {
    fn set(&mut self, x: usize, y: usize, state: u8) {
        let old_state = &mut self.cells[y][x];
        if *old_state != state {
            *old_state = state;
            self.draw(x, y, state);
            stdout().flush();
        }
    }

    fn get(&self, x: usize, y: usize) -> u8 {
        self.cells[y][x]
    }

    fn clear(&mut self) {
        self.cells = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
        execute!(stdout(), Clear(ClearType::All));
        stdout().flush();
    }

    fn refresh(&mut self) {
        execute!(stdout(), cursor::MoveTo(1, 1));
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                self.draw(x, y, self.cells[y][x]);
            }
        }
        stdout().flush();
    }
}

fn key_to_u8(key: KeyCode) -> Option<u8> {
    match key {
        KeyCode::Char(c) => c.to_digit(10).filter(|c| *c <= 0xF).map(|c| c as u8),
        _ => None,
    }
}
