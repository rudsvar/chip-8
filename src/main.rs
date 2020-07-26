use std::path::PathBuf;
use structopt::StructOpt;
use chip_8::emulator::Emulator;
use chip_8::emulator::{Input, Output};
use std::io::{Write, stdout, Stdout};
use std::thread;
use std::sync::{Arc, Mutex, mpsc};
use std::time;

use termion::screen::AlternateScreen;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};

/// A basic example
#[derive(StructOpt, Debug)]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Files to process
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

struct TermionInput {
    key: Arc<Mutex<Option<(u8, time::SystemTime)>>>, // Latest pressed key
    waiting: Arc<Mutex<bool>>, // Wether we are waiting for a keypress or not
    receiver: mpsc::Receiver<(u8, time::SystemTime)>, // Channel for receiving blocking keypresses
}

impl TermionInput {
    fn new(key: Arc<Mutex<Option<(u8, time::SystemTime)>>>, waiting: Arc<Mutex<bool>>, receiver: mpsc::Receiver<(u8, time::SystemTime)>) -> TermionInput {
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
                if timestamp.elapsed().unwrap() < time::Duration::from_millis(250) {
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
            if timestamp.elapsed().unwrap() < time::Duration::from_millis(250) {
                result = key;
                break;
            }
        }
        *waiting = false;
        result
    }
}

struct TermionOutput {
    screen: AlternateScreen<termion::raw::RawTerminal<Stdout>>,
    cells: [[u8; 128]; 128]
}

impl TermionOutput {
    fn new() -> TermionOutput {
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        write!(screen, "{}", termion::cursor::Hide).unwrap();
        TermionOutput {
            screen,
            cells: [[0; 128]; 128]
        }
    }
}

impl Output for TermionOutput {
    fn set(&mut self, x: usize, y: usize, state: u8) {
        let old_state = &mut self.cells[y][x];
        if *old_state != state {
            *old_state = state;
            write!(self.screen, "{}", termion::cursor::Goto(2 * x as u16 + 1, y as u16 + 1)).unwrap();
            write!(self.screen, "{}", if state == 1 { "##" } else { "  " }).unwrap();
            self.screen.flush().unwrap();
        }
    }
    fn get(&self, x: usize, y: usize) -> u8 {
        self.cells[y][x]
    }
    fn clear(&mut self) {
        self.cells = [[0; 128]; 128];
        write!(self.screen, "{}", termion::clear::All).unwrap();
        self.screen.flush().unwrap();
    }
    fn refresh(&mut self) {
        write!(self.screen, "{}", termion::cursor::Goto(1, 1)).unwrap();
        for row in self.cells.iter() {
            for cell in row.iter() {
                write!(self.screen, "{}", if *cell == 1 { "##" } else { "  " } ).unwrap();
            }
            writeln!(self.screen, "").unwrap();
        }
        self.screen.flush().unwrap();
    }
}

fn event_listener(sender: mpsc::Sender<(u8, time::SystemTime)>, key_pressed: Arc<Mutex<Option<(u8, time::SystemTime)>>>, waiting: Arc<Mutex<bool>>, stop: Arc<Mutex<bool>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let stdin = std::io::stdin();
        for event in stdin.keys() {
            match event {
                Ok(key) => {
                    match key {
                        Key::Char('q') | Key::Ctrl('c') => {
                            *stop.lock().unwrap() = true; // Tell the main thread to end
                            break; // Make this thread end
                        }
                        // Key::Ctrl('l') => {
                        //     write!(stdout, "{}", termion::clear::All)?;
                        //     emulator.flag_all_pixels_for_redrawing();
                        // }
                        Key::Char(c) => {
                            if let Some(int_value) = c.to_digit(10) {
                                if (0..0xF).contains(&int_value) {
                                    // Make two individual critical sections
                                    let key_with_timestamp = (int_value as u8, std::time::SystemTime::now());
                                    *key_pressed.lock().unwrap() = Some(key_with_timestamp);
                                    if *waiting.lock().unwrap() {
                                        sender.send(key_with_timestamp).unwrap();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    })
}

fn main() -> std::io::Result<()> {

    let logfile = FileAppender::builder()
        .build("log/output.log").unwrap();

    let config = Config::builder()
        .appender(Appender::builder()
            .build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Trace))
        .unwrap();

    log4rs::init_config(config).unwrap();

    // Get configuration and read input file
    let opt = Opt::from_args();
    log::info!("Executing {:?}", &opt.input);
    let program = std::fs::read(opt.input)?;

    // Create shared data
    let (sender, receiver) = mpsc::channel();
    let key = Arc::new(Mutex::new(None));
    let waiting = Arc::new(Mutex::new(false));
    let stop = Arc::new(Mutex::new(false));
    let event_listener = event_listener(sender, key.clone(), waiting.clone(), stop.clone());

    // Load instructions into emulator memory
    let mut emulator = Emulator::with_io(TermionInput::new(key.clone(), waiting.clone(), receiver), TermionOutput::new());
    emulator.load(&program);

    // Start execution
    while !*stop.lock().unwrap() {
        emulator.step();
        std::thread::sleep(std::time::Duration::from_millis(1_000/120));
    }

    event_listener.join().expect("Could not join event listener");

    Ok(())
}