use anyhow::anyhow;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use thiserror::Error;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_BUFFER_LENGTH: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

// Arbitrary bytes
const RNG_SEED: [u8; 32] = [
    0xBA, 0xD5, 0xEE, 0xD5, 0xBA, 0xD5, 0xEE, 0xD5, 0xBA, 0xD5, 0xEE, 0xD5, 0xBA, 0xD5, 0xEE, 0xD5,
    0xBA, 0xD5, 0xEE, 0xD5, 0xBA, 0xD5, 0xEE, 0xD5, 0xBA, 0xD5, 0xEE, 0xD5, 0xBA, 0xD5, 0xEE, 0xD5,
];

const ADDR_PROGRAM: u16 = 0x200;

const ADDR_CHARACTER: u16 = 0;
const SIZE_CHARACTER: u16 = 5;
const CHARACTER_ROM: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // '0'
    0x20, 0x60, 0x20, 0x20, 0x70, // '1'
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // '2'
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // '3'
    0x90, 0x90, 0xF0, 0x10, 0x10, // '4'
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // '5'
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // '6'
    0xF0, 0x10, 0x20, 0x40, 0x40, // '7'
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // '8'
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // '9'
    0xF0, 0x90, 0xF0, 0x90, 0x90, // 'A'
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // 'B'
    0xF0, 0x80, 0x80, 0x80, 0xF0, // 'C'
    0xE0, 0x90, 0x90, 0x90, 0xE0, // 'D'
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // 'E'
    0xF0, 0x80, 0xF0, 0x80, 0x80, // 'F'
];

#[derive(Debug, Error)]
pub enum Chip8Panic {
    #[error("attempted to return while stack pointer is 0")]
    StackUnderflow,

    #[error("attempted to push to full return stack")]
    StackOverflow,

    #[error("unknown opcode")]
    UnknownOpCode,
}

#[derive(Debug, Clone)]
pub struct Chip8 {
    /// Deterministic Random Number Generator
    pub rng: StdRng,

    /// General Purpose Registers
    ///
    /// V0 ~ VF
    pub v: [u8; 0x10],

    /// Memory Address Register
    ///
    /// Used to store memory addresses, so only lowest 12 bits are usually used.
    pub i: u16,

    /// Delay Timer Register
    ///
    /// Decrements every tick (60 Hz) until reaching 0.
    /// The delay timer is active whenever DT is non-zero.
    pub dt: u8,

    /// Sound Timer Register
    ///
    /// Decrements every tick (60 Hz) until reaching 0.
    /// The buzzer will sound whenever ST is non-zero.
    pub st: u8,

    /// Program Counter
    ///
    /// Stores the currently executing address.
    pub pc: u16,

    /// Stack Pointer
    ///
    /// Points to the topmost level of the stack.
    pub sp: u8,

    /// Stack
    ///
    /// Array of subroutine return addresses.
    pub stack: [u16; 0x10],

    /// RAM
    pub ram: [u8; 0x1000],

    /// Display 1-bit Buffer
    pub display: [bool; DISPLAY_BUFFER_LENGTH],

    /// Input keys
    ///
    /// Hex input keys '0' to 'F'
    pub keys: [bool; 0x10],

    /// Display dirty flag
    ///
    /// Set when the display buffer has changed.
    pub display_dirty: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            rng: StdRng::from_entropy(),
            v: [0; 0x10],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0,
            sp: 0,
            stack: [0; 0x10],
            ram: [0; 0x1000],
            display: [false; DISPLAY_BUFFER_LENGTH],
            keys: [false; 0x10],
            display_dirty: false,
        };

        chip8.reset();

        chip8
    }

    pub fn status(&self) -> String {
        let vx_str = self
            .v
            .iter()
            .map(|x| format!("{:02X}", *x))
            .collect::<Vec<_>>()
            .join(" ");

        format!(
            "{:04X}: {:04X} I={:04X} Vx=[{}] DT={:02X}",
            self.pc,
            self.mem_read_opcode(self.pc),
            self.i,
            vx_str,
            self.dt
        )
    }

    pub fn display_width(&self) -> usize {
        DISPLAY_WIDTH
    }

    pub fn display_height(&self) -> usize {
        DISPLAY_HEIGHT
    }

    pub fn reset(&mut self) {
        self.rng = SeedableRng::from_seed(RNG_SEED);

        self.i = 0;
        self.dt = 0;
        self.st = 0;
        self.pc = ADDR_PROGRAM;
        self.sp = 0;

        fill_array(&mut self.v, 0);
        fill_array(&mut self.stack, 0);
        fill_array(&mut self.display, false);
        fill_array(&mut self.keys, false);

        fill_array(&mut self.ram, 0);
        self.mem_write_slice(ADDR_CHARACTER, &CHARACTER_ROM)
            .unwrap();

        self.display_dirty = true;
    }

    pub fn set_key(&mut self, key: u8) {
        let key = (key & 0xf) as usize;
        self.keys[key] = true;
    }

    pub fn step(&mut self) -> Result<(), Chip8Panic> {
        let opcode = self.mem_read_opcode(self.pc);

        self.execute_opcode(opcode)?;

        fill_array(&mut self.keys, false);

        Ok(())
    }

    pub fn timer_tick(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
        }
    }

    fn v<T: Into<usize>>(&mut self, x: T) -> &mut u8 {
        &mut self.v[x.into()]
    }

    fn execute_opcode(&mut self, opcode: u16) -> Result<(), Chip8Panic> {
        let nnn = opcode & 0x0fff;
        let kk = (opcode & 0x00ff) as u8;

        match split_opcode(opcode) {
            (0x0, 0x0, 0xE, 0x0) => {
                // CLS: Clear the display

                fill_array(&mut self.display, false);
                self.display_dirty = true;

                self.pc += 2;

                Ok(())
            }
            (0x0, 0x0, 0xE, 0xE) => {
                // RET: Return from a subroutine

                if self.sp == 0 {
                    Err(Chip8Panic::StackUnderflow)
                } else {
                    self.pc = self.stack[usize::from(self.sp)];
                    self.sp -= 1;
                    self.pc += 2;

                    Ok(())
                }
            }
            (0x1, _x, _y, _z) => {
                // JP addr: Jump to address

                self.pc = nnn;

                Ok(())
            }
            (0x2, _x, _y, _z) => {
                // CALL addr: Call subroutine at address

                if self.sp >= 0xff {
                    Err(Chip8Panic::StackOverflow)
                } else {
                    self.sp += 1;
                    self.stack[usize::from(self.sp)] = self.pc;
                    self.pc = nnn;

                    Ok(())
                }
            }
            (0x3, x, _y, _z) => {
                // SE Vx, kk: Skip next instruction if Vx = kk

                if *self.v(x) == kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                Ok(())
            }
            (0x4, x, _y, _z) => {
                // SNE Vx, kk: Skip next instruction if Vx != kk

                if *self.v(x) != kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                Ok(())
            }
            (0x5, x, y, 0x0) => {
                // SE Vx, Vy: Skip next instruction if Vx == Vy
                if *self.v(x) == *self.v(y) {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                Ok(())
            }
            (0x6, x, _y, _z) => {
                // LD Vx, kk: Vx = kk

                *self.v(x) = kk;
                self.pc += 2;

                Ok(())
            }
            (0x7, x, _y, _z) => {
                // ADD Vx, kk: Vx = Vx + kk

                *self.v(x) = (*self.v(x)).wrapping_add(kk);
                self.pc += 2;

                Ok(())
            }
            (0x8, x, y, 0x0) => {
                // LD Vx, Vy: Set Vx = Vy
                *self.v(x) = *self.v(y);
                self.pc += 2;
                Ok(())
            }
            (0x8, x, y, 0x1) => {
                // OR Vx, Vy: Set Vx = Vx OR Vy
                *self.v(x) = *self.v(x) | *self.v(y);
                self.pc += 2;
                Ok(())
            }
            (0x8, x, y, 0x2) => {
                // AND Vx, Vy: Set Vx = Vx AND Vy
                *self.v(x) = *self.v(x) & *self.v(y);
                self.pc += 2;
                Ok(())
            }
            (0x8, x, y, 0x3) => {
                // XOR Vx, Vy: Set Vx = Vx XOR Vy
                *self.v(x) = *self.v(x) ^ *self.v(y);
                self.pc += 2;
                Ok(())
            }
            (0x8, x, y, 0x4) => {
                // ADD Vx, Vy: Set Vx = Vx + Vy, set VF = carry
                let (sum, ovf) = (*self.v(x)).overflowing_add(*self.v(y));
                self.v[0xf] = if ovf { 1 } else { 0 };
                *self.v(x) = sum;

                self.pc += 2;
                Ok(())
            }
            (0x8, x, y, 0x5) => {
                // SUB Vx, Vy: Set Vx = Vx - Vy, set VF = NOT borrow

                // Note: VF is assigned first, so it could change Vx - Vy if x or y is F
                // If this doesn't matter, then we could use overflowing_sub() instead
                self.v[0xf] = if *self.v(x) > *self.v(y) { 1 } else { 0 };
                *self.v(x) = (*self.v(x)).wrapping_sub(*self.v(y));

                self.pc += 2;
                Ok(())
            }
            (0x8, x, _y, 0x6) => {
                // SHR Vx|Vy: Set Vx = Vx >> 1, set VF = shifted-out bit

                // Compatibility note: Some machines may use Vx = Vy >> 1

                self.v[0xf] = *self.v(x) & 1;
                *self.v(x) = *self.v(x) >> 1;

                self.pc += 2;
                Ok(())
            }
            (0x8, x, y, 0x7) => {
                // SUBN Vx, Vy: Set Vx = Vy - Vx, set VF = NOT borrow

                // Note: VF is assigned first, so it could change Vx - Vy if x or y is F
                // If this doesn't matter, then we could use overflowing_sub() instead
                self.v[0xf] = if *self.v(y) > *self.v(x) { 1 } else { 0 };
                *self.v(x) = (*self.v(y)).wrapping_sub(*self.v(x));

                self.pc += 2;
                Ok(())
            }
            (0x8, x, _y, 0xE) => {
                // SHL Vx|Vy: Set Vx = Vx << 1, set VF = shifted-out bit

                // Compatibility note: Some machines may use Vx = Vy << 1

                self.v[0xf] = if *self.v(x) & 0x80 == 0 { 0 } else { 1 };
                *self.v(x) = *self.v(x) << 1;

                self.pc += 2;
                Ok(())
            }
            (0x9, x, y, 0x0) => {
                // SNE Vx, Vy: Skip next instruction if Vx != Vy

                if *self.v(x) != *self.v(y) {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                Ok(())
            }
            (0xA, _x, _y, _z) => {
                // LD I, addr: Set I = nnn

                self.i = nnn;
                self.pc += 2;

                Ok(())
            }
            (0xB, _x, _y, _z) => {
                // JP V0, addr: Jump to location nnn + V0

                self.pc = u16::from(self.v[0]) + nnn;

                Ok(())
            }
            (0xC, x, _y, _z) => {
                // RND Vx, kk: Random byte AND kk

                *self.v(x) = kk & ((self.rng.next_u32() & 0xff) as u8);
                self.pc += 2;

                Ok(())
            }
            (0xD, x, y, z) => {
                // DRW Vx, Vy, nibble:
                // Display n-byte sprite starting at memory location I at (Vx, Vy),
                // set VF = collision.

                let vx = usize::from(*self.v(x));
                let vy = usize::from(*self.v(y));
                let i = usize::from(self.i);

                self.v[0xf] = 0;

                for dy in 0..z {
                    let dy = usize::from(dy);

                    self.disp_toggle_sprite_row(vx, vy + dy, self.ram[i + dy]);
                }

                self.pc += 2;

                Ok(())
            }
            (0xE, x, 0x9, 0xE) => {
                // SKP Vx: Skip next instruction if key with value of Vx is pressed
                let key_idx = usize::from(*self.v(x) & 0xf);
                let key_pressed = self.keys[key_idx];

                if key_pressed {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                Ok(())
            }
            (0xE, x, 0xA, 0x1) => {
                // SKNP Vx: Skip next instruction if key with value of Vx is not pressed
                let key_idx = usize::from(*self.v(x) & 0xf);
                let key_pressed = self.keys[key_idx];

                if !key_pressed {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                Ok(())
            }
            (0xF, x, 0x0, 0x7) => {
                // LD Vx, DT: set Vx = DT

                *self.v(x) = self.dt;

                self.pc += 2;

                Ok(())
            }
            (0xF, x, 0x0, 0xA) => {
                // LD Vx, K: Wait for a key press, store value of key in Vx

                let key_pressed = self
                    .keys
                    .iter()
                    .enumerate()
                    .filter(|(_, is_pressed)| **is_pressed)
                    .map(|(i, _)| i)
                    .next();

                if let Some(key_pressed) = key_pressed {
                    *self.v(x) = key_pressed as u8;
                    self.pc += 2;
                }

                Ok(())
            }
            (0xF, x, 0x1, 0x5) => {
                // LD DT, Vx: Set DT = Vx

                self.dt = *self.v(x);

                self.pc += 2;

                Ok(())
            }
            (0xF, x, 0x1, 0x8) => {
                // LD ST, Vx: Set ST = Vx

                self.st = *self.v(x);

                self.pc += 2;

                Ok(())
            }
            (0xF, x, 0x1, 0xE) => {
                // ADD I, Vx: Set I = I + Vx

                self.i += u16::from(*self.v(x));

                self.pc += 2;

                Ok(())
            }
            (0xF, x, 0x2, 0x9) => {
                // LD F, Vx: Set I = location of sprite for digit Vx

                let char = u16::from(*self.v(x) & 0x0f);

                self.i = ADDR_CHARACTER + SIZE_CHARACTER * char;

                self.pc += 2;

                Ok(())
            }
            (0xF, x, 0x3, 0x3) => {
                // LD B, Vx: Store BCD repr of Vx in mem locations I, I+1, I+2

                let i = usize::from(self.i);
                let vx = *self.v(x);

                let hundreds = vx / 100 % 10;
                let tens = vx / 10 % 10;
                let ones = vx / 1 % 10;

                self.ram[i] = hundreds;
                self.ram[i + 1] = tens;
                self.ram[i + 2] = ones;

                self.pc += 2;

                Ok(())
            }
            (0xF, x, 0x5, 0x5) => {
                // LD [I], Vx: Store registers V0 through Vx in memory starting at I

                for di in 0_usize..=usize::from(x) {
                    let addr = (usize::from(self.i) + di) % self.ram.len();
                    self.ram[addr] = self.v[di];
                }

                self.pc += 2;

                Ok(())
            }
            (0xF, x, 0x6, 0x5) => {
                // LD Vx, [I]: Read registers V0 through Vx from memory starting at I

                for di in 0_usize..=usize::from(x) {
                    let addr = (usize::from(self.i) + di) % self.ram.len();
                    self.v[di] = self.ram[addr];
                }

                self.pc += 2;

                Ok(())
            }
            _ => Err(Chip8Panic::UnknownOpCode),
        }
    }

    fn disp_toggle_sprite_row(&mut self, x: usize, y: usize, s: u8) {
        for i in (0..8).rev() {
            if (s >> i) & 1 == 1 {
                self.disp_toggle_coord(x + 7 - i, y);
            }
        }
    }

    fn disp_toggle_coord(&mut self, x: usize, y: usize) {
        let idx = self.disp_coord_to_index(x, y);

        if self.display[idx] {
            self.v[0xf] = 1;
        }

        self.display[idx] = !self.display[idx];

        self.display_dirty = true;
    }

    fn disp_coord_to_index(&self, mut x: usize, mut y: usize) -> usize {
        x = x % self.display_width();
        y = y % self.display_height();

        y * self.display_width() + x
    }

    pub fn mem_read_opcode(&self, addr: u16) -> u16 {
        let msb: u16 = self.mem_read_byte(addr).into();
        let lsb: u16 = self.mem_read_byte(addr + 1).into();

        (msb << 8) | lsb
    }

    pub fn load_rom(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.mem_write_slice(ADDR_PROGRAM, data)?;

        Ok(())
    }

    fn mem_write_slice(&mut self, start: u16, slice: &[u8]) -> anyhow::Result<()> {
        let start = usize::from(start);

        if start + slice.len() >= self.ram.len() {
            return Err(anyhow!("mem_insert out-of-bounds"));
        }

        for (offset, val) in slice.iter().enumerate() {
            self.ram[start + offset] = *val;
        }

        Ok(())
    }

    // fn mem_write_byte(&mut self, addr: u16, val: u8) {
    //     let addr = usize::from(addr) % self.ram.len();
    //     self.ram[addr] = val;
    // }

    fn mem_read_byte(&self, addr: u16) -> u8 {
        let addr = usize::from(addr) % self.ram.len();
        self.ram[addr]
    }
}

fn fill_array<T: Copy>(a: &mut [T], val: T) {
    for x in a.iter_mut() {
        *x = val;
    }
}

pub fn split_opcode(op: u16) -> (u8, u8, u8, u8) {
    let hi = (op >> 8) as u8;
    let lo = op as u8;
    split_opcode2(hi, lo)
}

pub fn split_opcode2(hi: u8, lo: u8) -> (u8, u8, u8, u8) {
    ((hi & 0xf0) >> 4, hi & 0x0f, (lo & 0xf0) >> 4, lo & 0x0f)
}
