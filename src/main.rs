mod chip8;
mod emu;
mod window;

use std::{fs::File, io::Read, path::PathBuf, time::Duration};
use structopt::StructOpt;

use emu::Emulator;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str), help = "Input ROM file")]
    file: PathBuf,

    #[structopt(short, long, help = "Print debug messages")]
    verbose: bool,

    #[structopt(short, long, help = "Clock speed (Hz)")]
    clock: Option<f64>,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let mut emu = Emulator::new()?;

    emu.debug_print = opt.verbose;

    if let Some(clock) = opt.clock {
        emu.clock_period = Duration::from_secs_f64(1. / clock);
    }

    let f = File::open(&opt.file)?;
    let data: Vec<u8> = f.bytes().filter_map(|r| r.ok()).collect();

    emu.cpu.load_rom(&data)?;
    // emu.pause()?;

    while !emu.closing {
        emu.step()?;
    }

    emu.close();

    Ok(())
}
