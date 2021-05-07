mod chip8;
mod emu;

use std::{fs::File, io::Read};

use emu::Emulator;

fn main() -> anyhow::Result<()> {
    let mut emu = Emulator::new()?;

    let f = File::open("roms/UFO")?;
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
