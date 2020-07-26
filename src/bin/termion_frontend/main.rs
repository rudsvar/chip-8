use std::path::PathBuf;

use structopt::StructOpt;

use chip_8::emulator::emulator::Emulator;
use chip_8::emulator::key_manager::KeyManager;

mod termion_io;
use termion_io::{TermionInput, TermionOutput};

use termion::event::Key;

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

    let key_manager = KeyManager::new();

    // Load instructions into emulator memory
    let mut emulator = Emulator::with_io(
        TermionInput::new(&key_manager), 
        TermionOutput::new()
    );
    emulator.load(&program);

    // Start execution
    while key_manager.get_key() != Some(Key::Char('q')) {
        emulator.step();
        std::thread::sleep(std::time::Duration::from_millis(1_000/120));
    }

    Ok(())
}