mod chip8;
mod window;

use std::sync::{Arc, Mutex};

use chip8::Chip8;
use minifb::{Key, WindowOptions};

fn main() -> anyhow::Result<()> {
    let cpu = Chip8::new();

    let width = cpu.display_width();
    let height = cpu.display_height();

    let window_options = WindowOptions {
        resize: true,
        scale: minifb::Scale::X16,
        scale_mode: minifb::ScaleMode::AspectRatioStretch,
        ..Default::default()
    };

    let window = window::spawn("Chip8 Rust Emulator", width, height, window_options)?;

    let buffer = Arc::new(Mutex::new(
        (0..width * height)
            .enumerate()
            .map(|(i, _)| if i % 2 == 0 { u32::MAX } else { 0 })
            .collect(),
    ));

    'outer: loop {
        window.update_buffer(buffer.clone())?;

        for key in window.get_keys()? {
            match key {
                Key::Escape => break 'outer,
                _ => {}
            }
        }
    }

    window.close()?;
    Ok(())
}
