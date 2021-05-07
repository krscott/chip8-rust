use std::sync::{Arc, Mutex};

use crate::chip8::{self, Chip8};
use crate::window::{self, WindowHandle};
use minifb::{Key, WindowOptions};

const COLOR_ON: u32 = u32::MAX;
const COLOR_OFF: u32 = 0;

const TITLE: &str = "Chip8 Rust Emulator";

pub struct Emulator {
    pub cpu: Chip8,
    pub window: WindowHandle,
    pub display_buffer: Arc<Mutex<Vec<u32>>>,
    pub paused: bool,
    pub closing: bool,
}

impl Emulator {
    pub fn new() -> anyhow::Result<Self> {
        let cpu = Chip8::new();

        let width = cpu.display_width();
        let height = cpu.display_height();

        let window_options = WindowOptions {
            resize: true,
            scale: minifb::Scale::X16,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..Default::default()
        };

        let window = window::spawn(TITLE, width, height, window_options)?;

        let display_buffer = Arc::new(Mutex::new(
            (0..width * height)
                .enumerate()
                .map(|(i, _)| if i % 2 == 0 { u32::MAX } else { 0 })
                .collect(),
        ));

        Ok(Emulator {
            cpu,
            window,
            display_buffer,
            paused: false,
            closing: false,
        })
    }

    pub fn step(&mut self) -> anyhow::Result<()> {
        if !self.closing {
            self.read_inputs()?;
        }

        if !self.paused && !self.closing {
            self.cpu_step()?;
        }

        self.update_buffer()?;

        Ok(())
    }

    fn read_inputs(&mut self) -> anyhow::Result<()> {
        for key in self.window.get_keys()? {
            match key {
                Key::Escape => {
                    self.quit()?;
                }
                Key::Space => {
                    if self.paused {
                        self.unpause()?;
                    } else {
                        self.pause()?;
                    }
                }
                Key::Enter => {
                    self.pause()?;
                    self.cpu_step()?;
                }

                Key::X => self.cpu.set_key(0x0),
                Key::Key1 => self.cpu.set_key(0x1),
                Key::Key2 => self.cpu.set_key(0x2),
                Key::Key3 => self.cpu.set_key(0x3),
                Key::Q => self.cpu.set_key(0x4),
                Key::W => self.cpu.set_key(0x5),
                Key::E => self.cpu.set_key(0x6),
                Key::A => self.cpu.set_key(0x7),
                Key::S => self.cpu.set_key(0x8),
                Key::D => self.cpu.set_key(0x9),
                Key::Z => self.cpu.set_key(0xA),
                Key::C => self.cpu.set_key(0xB),
                Key::Key4 => self.cpu.set_key(0xC),
                Key::R => self.cpu.set_key(0xD),
                Key::F => self.cpu.set_key(0xE),
                Key::V => self.cpu.set_key(0xF),

                _ => {}
            }
        }

        Ok(())
    }

    fn cpu_step(&mut self) -> anyhow::Result<()> {
        match self.cpu.step() {
            Ok(()) => {}
            Err(e) => {
                match e {
                    chip8::Chip8Panic::StackUnderflow => {
                        println!("Error: Stack Overflow");
                    }
                    chip8::Chip8Panic::StackOverflow => {
                        println!("Error: Stack Overflow");
                    }
                    chip8::Chip8Panic::UnknownOpCode => {
                        println!(
                            "Error: Unknown opcode {:#04x}",
                            self.cpu.mem_read_opcode(self.cpu.pc)
                        );
                    }
                }

                self.pause()?;
            }
        }

        Ok(())
    }

    fn update_buffer(&mut self) -> anyhow::Result<()> {
        {
            let mut window_buffer = self.display_buffer.lock().unwrap();

            for (i, b) in window_buffer.iter_mut().enumerate() {
                *b = match self.cpu.display[i] {
                    true => COLOR_ON,
                    false => COLOR_OFF,
                };
            }
        }

        self.window.update_buffer(self.display_buffer.clone())?;

        Ok(())
    }

    pub fn pause(&mut self) -> anyhow::Result<()> {
        self.paused = true;
        self.window.set_title(format!("PAUSED - {}", TITLE))?;

        Ok(())
    }

    pub fn unpause(&mut self) -> anyhow::Result<()> {
        self.paused = true;
        self.window.set_title(TITLE.into())?;

        Ok(())
    }

    pub fn quit(&mut self) -> anyhow::Result<()> {
        self.closing = true;
        self.window.set_title(format!("CLOSING - {}", TITLE))?;

        Ok(())
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.window.close()?;

        Ok(())
    }
}
