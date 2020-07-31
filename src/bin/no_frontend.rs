use std::path::PathBuf;

use structopt::StructOpt;

use chip_8::emulator::Emulator;
use chip_8::emulator::{input::DummyInput, output::DummyOutput};

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

    // Load instructions into emulator memory
    let mut emulator = Emulator::<DummyInput, DummyOutput>::new();
    emulator.load(&program);

    // Start execution
    loop {
        emulator.step();
        std::thread::sleep(std::time::Duration::from_millis(1_000 / 60));
    }
}
