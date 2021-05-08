mod chip8;
mod disasm;
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

    #[structopt(short, long, help = "Disassemble program and exit")]
    disassemble: bool,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let f = File::open(&opt.file)?;
    let program_rom: Vec<u8> = f.bytes().filter_map(|r| r.ok()).collect();

    if opt.disassemble {
        for line in disasm::disassemble(&program_rom, 0x200) {
            println!("{}", line);
        }
    } else {
        let mut emu = Emulator::new()?;

        emu.debug_print = opt.verbose;

        if let Some(clock) = opt.clock {
            emu.clock_period = if clock > 0. {
                Duration::from_secs_f64(1. / clock)
            } else {
                Duration::from_secs_f64(1e-9) // 1 GHz
            };
        }

        emu.cpu.load_rom(&program_rom)?;
        // emu.pause()?;

        while !emu.closing {
            emu.step()?;
        }

        emu.close();
    }

    Ok(())
}
