use std::path::PathBuf;
use structopt::StructOpt;
use chip_8::emulator::Emulator;
use std::io::{Write, stdout};

use termion::screen::AlternateScreen;
use termion::event::{Key, Event};
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

    // Load instructions into emulator memory
    let mut emulator = Emulator::new();
    emulator.load(&program);

    // Create alternate screen to draw on
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let stdin = termion::async_stdin();
    let mut events = stdin.events();

    write!(stdout, "{}", termion::cursor::Hide)?;

    let now = std::time::SystemTime::now();
    let no_keys_pressed = [false; 0xF];
    let mut keys_pressed = no_keys_pressed;
    let mut timestamps = [now; 0xF];

    // Start execution
    while emulator.step(&keys_pressed) {

        // Buffer keypresses for a little while
        for (is_pressed, timestamp) in keys_pressed.iter_mut().zip(timestamps.iter_mut()) {
            if *is_pressed && timestamp.elapsed().expect("System time error") > std::time::Duration::from_millis(500) {
                *is_pressed = false;
                *timestamp = std::time::SystemTime::now();
            }
        }

        // Check keys pressed
        match events.next() {
            Some(Ok(Event::Key(key))) => {
                match key {
                    Key::Char('q') => { break; }
                    Key::Ctrl('c') => { break; }
                    Key::Ctrl('l') => {
                        write!(stdout, "{}", termion::clear::All)?;
                        emulator.flag_all_pixels_for_redrawing();
                    }
                    Key::Char(c) => {
                        if let Some(int_value) = c.to_digit(10) {
                            if (0..0xF).contains(&int_value) {
                                keys_pressed[int_value as usize] = true;
                                timestamps[int_value as usize] = std::time::SystemTime::now();
                            }
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        }

        // Write the virtual screen contents to the terminal
        let screen = emulator.get_screen();
        for &(x, y) in emulator.get_screen_update_locations() {
            write!(stdout, "{}", termion::cursor::Goto(2 * x as u16 + 1, y as u16 + 1))?;
            write!(stdout, "{}", if screen[y][x] == 1 { "##" } else { "  " })?;
        }
        stdout.flush()?;

        // Sleep until the next frame
        std::thread::sleep(std::time::Duration::from_millis(1_000/120));
    }

    Ok(())
}