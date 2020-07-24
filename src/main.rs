use std::path::PathBuf;
use structopt::StructOpt;
use chip_8::emulator::Emulator;
use termion::screen::AlternateScreen;
use std::io::{Write, stdout};

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

    // Get configuration and read input file
    let opt = Opt::from_args();
    let program = std::fs::read(opt.input)?;

    let mut screen = AlternateScreen::from(stdout());

    // Load instructions into emulator memory
    let mut emulator = Emulator::new();
    emulator.load(&program);


    // Start execution
    while emulator.step() {
        write!(screen, "{}", termion::cursor::Goto(1,1))?;
        write!(screen, "{}", emulator)?;
        std::thread::sleep(std::time::Duration::from_micros(1_000_000/60))
    }

    Ok(())
}