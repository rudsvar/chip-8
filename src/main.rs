use std::path::PathBuf;
use std::thread;
use std::time;
use std::sync::{Arc, Mutex, mpsc};

use structopt::StructOpt;

use chip_8::emulator::Emulator;
use chip_8::termion_io::{TermionInput, TermionOutput};

use termion::event::Key;
use termion::input::TermRead;

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