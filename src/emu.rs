use std::collections::HashMap;

use crate::chip8::{self, Chip8};
use minifb::{Key, Window, WindowOptions};

const COLOR_ON: u32 = u32::MAX;
const COLOR_OFF: u32 = 0;

const TITLE: &str = "Chip8 Rust Emulator";

pub struct Emulator {
    pub cpu: Chip8,
    pub window: Window,
    pub display_buffer: Vec<u32>,
    pub key_map: HashMap<Key, u8>,
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

        let mut window = Window::new(TITLE, width, height, window_options)?;

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(1_000_000 / 60)));

        let display_buffer = (0..width * height)
            .enumerate()
            .map(|(i, _)| if i % 2 == 0 { u32::MAX } else { 0 })
            .collect();

        Ok(Emulator {
            cpu,
            window,
            display_buffer,
            key_map: default_key_map(),
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
        // Read Key Presses
        for key in self
            .window
            .get_keys_pressed(minifb::KeyRepeat::Yes)
            .unwrap_or_default()
        {
            match key {
                Key::Escape => {
                    self.quit();
                }
                Key::Space => {
                    if self.paused {
                        self.unpause();
                    } else {
                        self.pause();
                    }
                }
                Key::Enter => {
                    self.pause();
                    self.cpu_step()?;
                }

                _ => {}
            }
        }

        // Read Mapped Keys
        for key in self.window.get_keys().unwrap_or_default() {
            if let Some(code) = self.key_map.get(&key) {
                self.cpu.set_key(*code);
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

                self.pause();
            }
        }

        Ok(())
    }

    fn update_buffer(&mut self) -> anyhow::Result<()> {
        for (i, b) in self.display_buffer.iter_mut().enumerate() {
            *b = match self.cpu.display[i] {
                true => COLOR_ON,
                false => COLOR_OFF,
            };
        }

        if self.window.is_open() {
            self.window.update_with_buffer(
                &self.display_buffer,
                self.cpu.display_width(),
                self.cpu.display_height(),
            )?;
        } else {
            self.quit();
        }

        Ok(())
    }

    pub fn pause(&mut self) {
        self.paused = true;
        self.window.set_title(&format!("PAUSED - {}", TITLE));
    }

    pub fn unpause(&mut self) {
        self.paused = true;
        self.window.set_title(TITLE.into());
    }

    pub fn quit(&mut self) {
        self.closing = true;
        self.window.set_title(&format!("CLOSING - {}", TITLE));
    }
}

fn default_key_map() -> HashMap<Key, u8> {
    let mut key_map = HashMap::new();

    key_map.insert(Key::X, 0x0);
    key_map.insert(Key::Key1, 0x1);
    key_map.insert(Key::Key2, 0x2);
    key_map.insert(Key::Key3, 0x3);
    key_map.insert(Key::Q, 0x4);
    key_map.insert(Key::W, 0x5);
    key_map.insert(Key::E, 0x6);
    key_map.insert(Key::A, 0x7);
    key_map.insert(Key::S, 0x8);
    key_map.insert(Key::D, 0x9);
    key_map.insert(Key::Z, 0xA);
    key_map.insert(Key::C, 0xB);
    key_map.insert(Key::Key4, 0xC);
    key_map.insert(Key::R, 0xD);
    key_map.insert(Key::F, 0xE);
    key_map.insert(Key::V, 0xF);

    key_map
}
