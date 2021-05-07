mod chip8;
mod emu;
mod window;

use std::{fs::File, io::Read};

use emu::Emulator;

fn main() -> anyhow::Result<()> {
    let mut emu = Emulator::new()?;

    let f = File::open("roms/UFO")?;
    let data: Vec<u8> = f.bytes().filter_map(|r| r.ok()).collect();

    emu.cpu.load_rom(&data)?;
    // emu.pause()?;

    while !emu.closing {
        emu.step()?;
    }

    emu.close()?;
    Ok(())
}
