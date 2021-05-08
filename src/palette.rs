pub const BUILTIN_PALETTES: [(&str, &str); 10] = [
    ("000000", "ffffff"),
    ("f0f6f0", "222323"),
    ("280e0b", "ffecc9"),
    ("363636", "ececec"),
    ("10368f", "ff8e42"),
    ("210009", "00ffae"),
    ("40318e", "88d7de"),
    ("040612", "d400ff"),
    ("3f291e", "fdca55"),
    ("2b0000", "cc0e13"),
];

pub fn builtin(index: usize) -> (u32, u32) {
    let (off, on) = BUILTIN_PALETTES[index % BUILTIN_PALETTES.len()];
    (from_hex(off), from_hex(on))
}

pub fn from_hex(hex: &str) -> u32 {
    let hex = format!("{:06}", hex.trim().trim_matches('#'));

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or_default();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or_default();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or_default();

    from_u8_rgb(r, g, b)
}

pub fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}
