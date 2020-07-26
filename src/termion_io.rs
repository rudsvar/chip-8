use crate::emulator::{Input, Output};
use std::io::{Write, Stdout, stdout};
use std::sync::{Arc, Mutex, mpsc};
use std::time;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = SCREEN_WIDTH;
const TIMEOUT: time::Duration = time::Duration::from_millis(250);

pub struct TermionInput {
    key: Arc<Mutex<Option<(u8, time::SystemTime)>>>, // Latest pressed key
    waiting: Arc<Mutex<bool>>, // Wether we are waiting for a keypress or not
    receiver: mpsc::Receiver<(u8, time::SystemTime)>, // Channel for receiving blocking keypresses
}

impl TermionInput {
    pub fn new(key: Arc<Mutex<Option<(u8, time::SystemTime)>>>, waiting: Arc<Mutex<bool>>, receiver: mpsc::Receiver<(u8, time::SystemTime)>) -> TermionInput {
        TermionInput {
            key,
            waiting,
            receiver,
        }
    }
}

impl Input for TermionInput {
    fn get_key(&self) -> Option<u8> {
        match *self.key.lock().unwrap() {
            Some((key, timestamp)) => {
                if timestamp.elapsed().unwrap() < TIMEOUT {
                    log::info!("Key pressed is {}", key);
                    return Some(key);
                }
            },
            _ => {}
        }

        None
    }
    fn get_key_blocking(&self) -> u8 {
        let mut waiting = self.waiting.lock().unwrap();
        *waiting = true;
        let mut result = 0;
        for (key, timestamp) in self.receiver.recv() {
            if timestamp.elapsed().unwrap() < TIMEOUT {
                result = key;
                break;
            }
        }
        *waiting = false;
        result
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