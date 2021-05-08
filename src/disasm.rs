// Hex bytes

pub fn disassemble(rom: &[u8], offset: u16) -> Vec<String> {
    rom.chunks(2)
        .enumerate()
        .map(|(i, opcode)| {
            let op_hex = opcode
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            let mnemonic = if opcode.len() == 2 {
                match split_opcode(opcode[0], opcode[1]) {
                    (0x0, 0x0, 0xE, 0x0) => format!("CLS"),
                    (0x0, 0x0, 0xE, 0xE) => format!("RET"),
                    (0x0, x, y, z) => format!("SYS {:X}{:X}{:X}", x, y, z),
                    (0x1, x, y, z) => format!("JP {:X}{:X}{:X}", x, y, z),
                    (0x2, x, y, z) => format!("CALL {:X}{:X}{:X}", x, y, z),
                    (0x3, x, y, z) => format!("SE V{:X}, {:X}{:X}", x, y, z),
                    (0x4, x, y, z) => format!("SNE V{:X}, {:X}{:X}", x, y, z),
                    (0x5, x, y, 0x0) => format!("SE V{:X}, V{:X}", x, y),
                    (0x6, x, y, z) => format!("LD V{:X}, {:X}{:X}", x, y, z),
                    (0x7, x, y, z) => format!("ADD V{:X}, {:X}{:X}", x, y, z),
                    (0x8, x, y, 0x0) => format!("LD V{:X}, V{:X}", x, y),
                    (0x8, x, y, 0x1) => format!("OR V{:X}, V{:X}", x, y),
                    (0x8, x, y, 0x2) => format!("AND V{:X}, V{:X}", x, y),
                    (0x8, x, y, 0x3) => format!("XOR V{:X}, V{:X}", x, y),
                    (0x8, x, y, 0x4) => format!("ADD V{:X}, V{:X}", x, y),
                    (0x8, x, y, 0x5) => format!("SUB V{:X}, V{:X}", x, y),
                    (0x8, x, y, 0x6) => format!("SHR V{:X} {{, V{:X}}}", x, y),
                    (0x8, x, y, 0x7) => format!("SUBN V{:X}, V{:X}", x, y),
                    (0x8, x, y, 0xE) => format!("SHL V{:X} {{, V{:X}}}", x, y),
                    (0x9, x, y, 0x0) => format!("SNE V{:X}, V{:X}", x, y),
                    (0xA, x, y, z) => format!("LD I, {:X}{:X}{:X}", x, y, z),
                    (0xB, x, y, z) => format!("JP V0, {:X}{:X}{:X}", x, y, z),
                    (0xC, x, y, z) => format!("RND V{:X}, {:X}{:X}", x, y, z),
                    (0xD, x, y, z) => format!("DRW V{:X}, V{:X}, {:X}", x, y, z),
                    (0xE, x, 0x9, 0xE) => format!("SKP V{:X}", x),
                    (0xE, x, 0xA, 0x1) => format!("SKNP V{:X}", x),
                    (0xF, x, 0x0, 0x7) => format!("LD V{:X}, DT", x),
                    (0xF, x, 0x0, 0xA) => format!("LD V{:X}, K", x),
                    (0xF, x, 0x1, 0x5) => format!("LD DT, V{:X}", x),
                    (0xF, x, 0x1, 0x8) => format!("LD ST, V{:X}", x),
                    (0xF, x, 0x1, 0xE) => format!("ADD I, V{:X}", x),
                    (0xF, x, 0x2, 0x9) => format!("LD F, V{:X}", x),
                    (0xF, x, 0x3, 0x3) => format!("LD B, V{:X}", x),
                    (0xF, x, 0x5, 0x5) => format!("LD [I], V{:X}", x),
                    (0xF, x, 0x6, 0x5) => format!("LD V{:X}, [I]", x),
                    _ => format!(""),
                }
            } else {
                String::from("")
            };

            format!("{:04X}: {}  {}", i + usize::from(offset), op_hex, mnemonic)
        })
        .collect()
}

fn split_opcode(hi: u8, lo: u8) -> (u8, u8, u8, u8) {
    ((hi & 0xf0) >> 4, hi & 0x0f, (lo & 0xf0) >> 4, lo & 0x0f)
}
