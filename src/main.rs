use std::path::PathBuf;
use structopt::StructOpt;
use chip_8::emulator::Emulator;

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
    let data = std::fs::read(opt.input)?;

    // Load instructions into emulator memory
    let mut emulator = Emulator::new(&data);
    emulator.execute(); // Start execution
    
    Ok(())
}