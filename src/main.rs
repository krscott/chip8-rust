mod chip8;
mod emu;

use std::{fs::File, io::Read, path::PathBuf};
use structopt::StructOpt;

use emu::Emulator;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let mut emu = Emulator::new()?;

    let f = File::open(&opt.file)?;
    let data: Vec<u8> = f.bytes().filter_map(|r| r.ok()).collect();

    emu.cpu.load_rom(&data)?;
    // emu.pause()?;

    let mut last_pc = u16::MAX;

    while !emu.closing {
        if last_pc != emu.cpu.pc {
            last_pc = emu.cpu.pc;
            println!("{}", emu.cpu.status());
        }

        emu.step()?;
    }

    Ok(())
}
