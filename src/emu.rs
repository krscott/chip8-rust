use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use crate::{
    chip8::{self, Chip8},
    window::{self, WindowHandle},
};
use minifb::Key;

const COLOR_ON: u32 = u32::MAX;
const COLOR_OFF: u32 = 0;

const TITLE: &str = "Chip8 Rust Emulator";

const DEFAULT_CLOCK_S: f64 = 1. / 100_000.;

pub struct Emulator {
    pub cpu: Chip8,
    pub window_handle: WindowHandle,
    pub key_map: HashMap<Key, u8>,
    pub clock_period: Duration,
    pub timer_time: SystemTime,
    pub paused: bool,
    pub closing: bool,
    pub debug_print: bool,
}

impl Emulator {
    pub fn new() -> anyhow::Result<Self> {
        let cpu = Chip8::new();

        let width = cpu.display_width();
        let height = cpu.display_height();

        let window_handle = window::spawn(TITLE.into(), width, height);

        Ok(Emulator {
            cpu,
            window_handle,
            key_map: default_key_map(),
            clock_period: Duration::from_secs_f64(DEFAULT_CLOCK_S),
            timer_time: SystemTime::now(),
            paused: false,
            closing: false,
            debug_print: false,
        })
    }

    pub fn step(&mut self) -> anyhow::Result<()> {
        if self.window_handle.is_closing() {
            self.quit();
        }

        if !self.closing {
            self.read_inputs()?;
        }

        if !self.paused && !self.closing {
            if self.debug_print {
                println!("{}", self.cpu.status());
            }

            self.cpu_step()?;
        }

        if self.cpu.display_dirty {
            self.cpu.display_dirty = false;

            self.update_window();
        }

        if self.window_handle.take_updated() {
            self.cpu.timer_tick();
        }

        spin_sleep::sleep(self.clock_period);

        Ok(())
    }

    fn read_inputs(&mut self) -> anyhow::Result<()> {
        // Read Key Presses
        if let Some(keys_pressed) = self.window_handle.get_keys_pressed() {
            for key in keys_pressed {
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
        }

        // Read Mapped Keys
        if let Some(keys) = self.window_handle.get_keys() {
            for key in keys {
                if let Some(code) = self.key_map.get(&key) {
                    self.cpu.set_key(*code);
                }
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

    fn update_window(&mut self) {
        for (i, b) in self
            .window_handle
            .get_display_buffer_mut()
            .iter_mut()
            .enumerate()
        {
            *b = match self.cpu.display[i] {
                true => COLOR_ON,
                false => COLOR_OFF,
            };
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
        self.window_handle.set_title(format!("PAUSED - {}", TITLE));
    }

    pub fn unpause(&mut self) {
        self.paused = true;
        self.window_handle.set_title(TITLE.into());
    }

    pub fn quit(&mut self) {
        self.closing = true;
        self.window_handle.set_title(format!("CLOSING - {}", TITLE));
    }

    pub fn close(self) {
        self.window_handle.close();
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
