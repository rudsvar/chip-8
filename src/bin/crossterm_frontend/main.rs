use std::path::PathBuf;

use structopt::StructOpt;

use chip_8::emulator::emulator::Emulator;
use chip_8::emulator::key_manager::KeyManager;

mod crossterm_io;
use crossterm_io::{CrosstermInput, CrosstermOutput};
use crossterm::event::KeyCode;

/// The program options.
#[derive(StructOpt)]
struct Opt {
    /// The program to execute
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> std::io::Result<()> {

    env_logger::init();

    // Get configuration and read input file
    let opt = Opt::from_args();
    log::info!("Executing {:?}", &opt.input);
    let program = std::fs::read(opt.input)?;

    let key_manager = KeyManager::new();

    // Load instructions into emulator memory
    let mut emulator = Emulator::with_io(
        CrosstermInput::new(&key_manager), 
        CrosstermOutput::new()
    );
    emulator.load(&program);

    // Start execution
    while key_manager.get_key() != Some(KeyCode::Char('q')) {
        emulator.step();
        std::thread::sleep(std::time::Duration::from_millis(1_000/120));
    }

    Ok(())
}