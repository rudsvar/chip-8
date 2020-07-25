use std::path::PathBuf;
use structopt::StructOpt;
use chip_8::emulator::Emulator;
use termion::screen::AlternateScreen;
use std::io::{Write, stdout};

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
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
    let mut stdout = AlternateScreen::from(stdout());

    write!(stdout, "{}", termion::cursor::Hide)?;

    // Start execution
    while emulator.step() {
        let screen = emulator.get_screen();
        for &(x, y) in emulator.get_screen_update_locations() {
            write!(stdout, "{}", termion::cursor::Goto(x as u16 + 1, y as u16 + 1))?;
            write!(stdout, "{}", if screen[y][x] == 1 { "##" } else { "  " })?;
        }
        stdout.flush()?;
        std::thread::sleep(std::time::Duration::from_micros(1_000_000/60))
    }

    Ok(())
}