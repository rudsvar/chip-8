use crate::emulator::{Input, Output};
use crate::key_manager::KeyManager;

use std::io::{Write, Stdout, stdout};

use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = SCREEN_WIDTH;

pub struct TermionInput<'a> {
    key_manager: &'a KeyManager
}

impl TermionInput<'_> {
    pub fn new(key_manager: &KeyManager) -> TermionInput{
        TermionInput {
            key_manager
        }
    }
}

impl Input for TermionInput<'_> {
    
    fn get_key(&self) -> Option<u8> {
        let key = self.key_manager.get_key()?;
        key_to_u8(key)
    }

    fn get_key_blocking(&self) -> u8 {
        let key = self.key_manager.get_key_blocking();
        key_to_u8(key).unwrap() // TODO: Add predicate -^
    }
}

pub struct TermionOutput {
    screen: AlternateScreen<termion::raw::RawTerminal<Stdout>>,
    cells: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT]
}

impl TermionOutput {
    pub fn new() -> TermionOutput {
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        write!(screen, "{}", termion::cursor::Hide).unwrap();
        TermionOutput {
            screen,
            cells: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT]
        }
    }
}

impl Drop for TermionOutput {
    fn drop(&mut self) {
        write!(self.screen, "{}", termion::cursor::Hide).unwrap();
    }
}

impl Output for TermionOutput {

    fn set(&mut self, x: usize, y: usize, state: u8) {
        let old_state = &mut self.cells[y][x];
        if *old_state != state {
            *old_state = state;
            write!(self.screen, "{}", termion::cursor::Goto(2 * x as u16 + 1, y as u16 + 1)).unwrap();
            write!(self.screen, "{}", if state == 1 { "██" } else { "  " }).unwrap();
            self.screen.flush().unwrap();
        }
    }

    fn get(&self, x: usize, y: usize) -> u8 {
        self.cells[y][x]
    }

    fn clear(&mut self) {
        self.cells = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
        write!(self.screen, "{}", termion::clear::All).unwrap();
        self.screen.flush().unwrap();
    }

    fn refresh(&mut self) {
        write!(self.screen, "{}", termion::cursor::Goto(1, 1)).unwrap();
        for row in self.cells.iter() {
            for cell in row.iter() {
                write!(self.screen, "{}", if *cell == 1 { "██" } else { "  " } ).unwrap();
            }
            writeln!(self.screen, "").unwrap();
        }
        self.screen.flush().unwrap();
    }
}

fn key_to_u8(key: Key) -> Option<u8> {
    match key {
        Key::Char(c) => {
            c.to_digit(10)
                .filter(|c| *c <= 0xF)
                .map(|c| c as u8)
        }
        _ => None
    }
}