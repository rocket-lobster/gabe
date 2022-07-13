/// Given a block of code separated into u8 values, interpret each byte as a valid Gameboy opcode,
/// and convert it and its operands into a human-readable mnemonic.
/// Note: This converts data naively, and assumes the initial start point is an opcode and not the
/// operand of a previous opcode or data. Ensure that the input starts on a known-good opcode,
/// and that the entire range is valid code, not data.
pub fn disassemble_block(data: Box<[u8]>, pc: u16) -> Vec<(u16, String)> {
    let mut iter = data.iter();
    let mut ret: Vec<(u16, String)> = vec![];
    let mut current_pc = pc;
    while let Some(opcode) = iter.next() {
        match opcode {
            0x00 => ret.push((current_pc, format!("{:02X}:\tnop", opcode).to_string())),
            0x76 => ret.push((current_pc, format!("{:02X}:\thalt", opcode).to_string())),
            0x10 => ret.push((current_pc, format!("{:02X}:\tstop", opcode).to_string())),
            0x3F => ret.push((current_pc, format!("{:02X}:\tccf", opcode).to_string())),
            0x37 => ret.push((current_pc, format!("{:02X}:\tscf", opcode).to_string())),
            0xF3 => ret.push((current_pc, format!("{:02X}:\tdi", opcode).to_string())),
            0xFB => ret.push((current_pc, format!("{:02X}:\tei", opcode).to_string())),
            0x06 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld b,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x0E => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld c,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x16 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld d,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x1E => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld e,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x26 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld h,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x2E => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld l,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x36 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld (hl),${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x3E => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x02 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (bc),a", opcode).to_string(),
            )),
            0x12 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (de),a", opcode).to_string(),
            )),
            0x0a => ret.push((
                current_pc,
                format!("{:02X}:\t ld a,(bc)", opcode).to_string(),
            )),
            0x1a => ret.push((
                current_pc,
                format!("{:02X}:\t ld a,(de)", opcode).to_string(),
            )),
            0x22 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl+),a", opcode).to_string(),
            )),
            0x32 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl-),a", opcode).to_string(),
            )),
            0x2a => ret.push((
                current_pc,
                format!("{:02X}:\t ld a,(hl+)", opcode).to_string(),
            )),
            0x3a => ret.push((
                current_pc,
                format!("{:02X}:\t ld a,(hl-)", opcode).to_string(),
            )),

            // LDH (a8),A
            0xE0 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld ($FF{:02X}),a", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            // LDH A,(a8)
            0xF0 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t ld a,($FF{:02X})", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xE2 => ret.push((
                current_pc,
                format!("{:02X}:\t ld ($FF00+c),a", opcode).to_string(),
            )),
            0xF2 => ret.push((
                current_pc,
                format!("{:02X}:\t ld a,($FF00+c)", opcode).to_string(),
            )),
            0x40 => ret.push((current_pc, format!("{:02X}:\t ld b,b", opcode).to_string())),
            0x41 => ret.push((current_pc, format!("{:02X}:\t ld b,c", opcode).to_string())),
            0x42 => ret.push((current_pc, format!("{:02X}:\t ld b,d", opcode).to_string())),
            0x43 => ret.push((current_pc, format!("{:02X}:\t ld b,e", opcode).to_string())),
            0x44 => ret.push((current_pc, format!("{:02X}:\t ld b,h", opcode).to_string())),
            0x45 => ret.push((current_pc, format!("{:02X}:\t ld b,l", opcode).to_string())),
            0x46 => ret.push((
                current_pc,
                format!("{:02X}:\t ld b,(hl)", opcode).to_string(),
            )),
            0x47 => ret.push((current_pc, format!("{:02X}:\t ld b,a", opcode).to_string())),
            0x48 => ret.push((current_pc, format!("{:02X}:\t ld c,b", opcode).to_string())),
            0x49 => ret.push((current_pc, format!("{:02X}:\t ld c,c", opcode).to_string())),
            0x4A => ret.push((current_pc, format!("{:02X}:\t ld c,d", opcode).to_string())),
            0x4B => ret.push((current_pc, format!("{:02X}:\t ld c,e", opcode).to_string())),
            0x4C => ret.push((current_pc, format!("{:02X}:\t ld c,h", opcode).to_string())),
            0x4D => ret.push((current_pc, format!("{:02X}:\t ld c,l", opcode).to_string())),
            0x4E => ret.push((
                current_pc,
                format!("{:02X}:\t ld c,(hl)", opcode).to_string(),
            )),
            0x4F => ret.push((current_pc, format!("{:02X}:\t ld c,a", opcode).to_string())),
            0x50 => ret.push((current_pc, format!("{:02X}:\t ld d,b", opcode).to_string())),
            0x51 => ret.push((current_pc, format!("{:02X}:\t ld d,c", opcode).to_string())),
            0x52 => ret.push((current_pc, format!("{:02X}:\t ld d,d", opcode).to_string())),
            0x53 => ret.push((current_pc, format!("{:02X}:\t ld d,e", opcode).to_string())),
            0x54 => ret.push((current_pc, format!("{:02X}:\t ld d,h", opcode).to_string())),
            0x55 => ret.push((current_pc, format!("{:02X}:\t ld d,l", opcode).to_string())),
            0x56 => ret.push((
                current_pc,
                format!("{:02X}:\t ld d,(hl)", opcode).to_string(),
            )),
            0x57 => ret.push((current_pc, format!("{:02X}:\t ld d,a", opcode).to_string())),
            0x58 => ret.push((current_pc, format!("{:02X}:\t ld e,b", opcode).to_string())),
            0x59 => ret.push((current_pc, format!("{:02X}:\t ld e,c", opcode).to_string())),
            0x5A => ret.push((current_pc, format!("{:02X}:\t ld e,d", opcode).to_string())),
            0x5B => ret.push((current_pc, format!("{:02X}:\t ld e,e", opcode).to_string())),
            0x5C => ret.push((current_pc, format!("{:02X}:\t ld e,h", opcode).to_string())),
            0x5D => ret.push((current_pc, format!("{:02X}:\t ld e,l", opcode).to_string())),
            0x5E => ret.push((
                current_pc,
                format!("{:02X}:\t ld e,(hl)", opcode).to_string(),
            )),
            0x5F => ret.push((current_pc, format!("{:02X}:\t ld e,a", opcode).to_string())),
            0x60 => ret.push((current_pc, format!("{:02X}:\t ld h,b", opcode).to_string())),
            0x61 => ret.push((current_pc, format!("{:02X}:\t ld h,c", opcode).to_string())),
            0x62 => ret.push((current_pc, format!("{:02X}:\t ld h,d", opcode).to_string())),
            0x63 => ret.push((current_pc, format!("{:02X}:\t ld h,e", opcode).to_string())),
            0x64 => ret.push((current_pc, format!("{:02X}:\t ld h,h", opcode).to_string())),
            0x65 => ret.push((current_pc, format!("{:02X}:\t ld h,l", opcode).to_string())),
            0x66 => ret.push((
                current_pc,
                format!("{:02X}:\t ld h,(hl)", opcode).to_string(),
            )),
            0x67 => ret.push((current_pc, format!("{:02X}:\t ld h,a", opcode).to_string())),
            0x68 => ret.push((current_pc, format!("{:02X}:\t ld l,b", opcode).to_string())),
            0x69 => ret.push((current_pc, format!("{:02X}:\t ld l,c", opcode).to_string())),
            0x6A => ret.push((current_pc, format!("{:02X}:\t ld l,d", opcode).to_string())),
            0x6B => ret.push((current_pc, format!("{:02X}:\t ld l,e", opcode).to_string())),
            0x6C => ret.push((current_pc, format!("{:02X}:\t ld l,h", opcode).to_string())),
            0x6D => ret.push((current_pc, format!("{:02X}:\t ld l,l", opcode).to_string())),
            0x6E => ret.push((
                current_pc,
                format!("{:02X}:\t ld l,(hl)", opcode).to_string(),
            )),
            0x6F => ret.push((current_pc, format!("{:02X}:\t ld l,a", opcode).to_string())),
            0x70 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl),b", opcode).to_string(),
            )),
            0x71 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl),c", opcode).to_string(),
            )),
            0x72 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl),d", opcode).to_string(),
            )),
            0x73 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl),e", opcode).to_string(),
            )),
            0x74 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl),h", opcode).to_string(),
            )),
            0x75 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl),l", opcode).to_string(),
            )),
            0x77 => ret.push((
                current_pc,
                format!("{:02X}:\t ld (hl),a", opcode).to_string(),
            )),
            0x78 => ret.push((current_pc, format!("{:02X}:\t ld a,b", opcode).to_string())),
            0x79 => ret.push((current_pc, format!("{:02X}:\t ld a,c", opcode).to_string())),
            0x7A => ret.push((current_pc, format!("{:02X}:\t ld a,d", opcode).to_string())),
            0x7B => ret.push((current_pc, format!("{:02X}:\t ld a,e", opcode).to_string())),
            0x7C => ret.push((current_pc, format!("{:02X}:\t ld a,h", opcode).to_string())),
            0x7D => ret.push((current_pc, format!("{:02X}:\t ld a,l", opcode).to_string())),
            0x7E => ret.push((
                current_pc,
                format!("{:02X}:\t ld a,(hl)", opcode).to_string(),
            )),
            0x7F => ret.push((current_pc, format!("{:02X}:\t ld a,a", opcode).to_string())),
            0x01 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t ld bc,(${:02X}{:02X})",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x11 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t ld de,(${:02X}{:02X})",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x21 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t ld hl,(${:02X}{:02X})",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x31 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t ld sp,(${:02X}{:02X})",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xEA => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t ld (${:02X}{:02X}),a",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xFA => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t ld a,(${:02X}{:02X})",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x08 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t ld (${:02X}{:02X}),sp",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xF9 => ret.push((
                current_pc,
                format!("{:02X}:\t ld sp,hl", opcode).to_string(),
            )),
            0x80 => ret.push((current_pc, format!("{:02X}:\t add a,b", opcode).to_string())),
            0x81 => ret.push((current_pc, format!("{:02X}:\t add a,c", opcode).to_string())),
            0x82 => ret.push((current_pc, format!("{:02X}:\t add a,d", opcode).to_string())),
            0x83 => ret.push((current_pc, format!("{:02X}:\t add a,e", opcode).to_string())),
            0x84 => ret.push((current_pc, format!("{:02X}:\t add a,h", opcode).to_string())),
            0x85 => ret.push((current_pc, format!("{:02X}:\t add a,l", opcode).to_string())),
            0x86 => ret.push((
                current_pc,
                format!("{:02X}:\t add a,(hl)", opcode).to_string(),
            )),
            0x87 => ret.push((current_pc, format!("{:02X}:\t add a,a", opcode).to_string())),
            0xC6 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t add a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x88 => ret.push((current_pc, format!("{:02X}:\t adc a,b", opcode).to_string())),
            0x89 => ret.push((current_pc, format!("{:02X}:\t adc a,c", opcode).to_string())),
            0x8A => ret.push((current_pc, format!("{:02X}:\t adc a,d", opcode).to_string())),
            0x8B => ret.push((current_pc, format!("{:02X}:\t adc a,e", opcode).to_string())),
            0x8C => ret.push((current_pc, format!("{:02X}:\t adc a,h", opcode).to_string())),
            0x8D => ret.push((current_pc, format!("{:02X}:\t adc a,l", opcode).to_string())),
            0x8E => ret.push((
                current_pc,
                format!("{:02X}:\t adc a,(hl)", opcode).to_string(),
            )),
            0x8F => ret.push((current_pc, format!("{:02X}:\t adc a,a", opcode).to_string())),
            0xCE => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t adc a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xE8 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t add sp,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x09 => ret.push((
                current_pc,
                format!("{:02X}:\t add hl,bc", opcode).to_string(),
            )),
            0x19 => ret.push((
                current_pc,
                format!("{:02X}:\t add hl,de", opcode).to_string(),
            )),
            0x29 => ret.push((
                current_pc,
                format!("{:02X}:\t add hl,hl", opcode).to_string(),
            )),
            0x39 => ret.push((
                current_pc,
                format!("{:02X}:\t add hl,sp", opcode).to_string(),
            )),
            0x90 => ret.push((current_pc, format!("{:02X}:\t sub a,b", opcode).to_string())),
            0x91 => ret.push((current_pc, format!("{:02X}:\t sub a,c", opcode).to_string())),
            0x92 => ret.push((current_pc, format!("{:02X}:\t sub a,d", opcode).to_string())),
            0x93 => ret.push((current_pc, format!("{:02X}:\t sub a,e", opcode).to_string())),
            0x94 => ret.push((current_pc, format!("{:02X}:\t sub a,h", opcode).to_string())),
            0x95 => ret.push((current_pc, format!("{:02X}:\t sub a,l", opcode).to_string())),
            0x96 => ret.push((
                current_pc,
                format!("{:02X}:\t sub a,(hl)", opcode).to_string(),
            )),
            0x97 => ret.push((current_pc, format!("{:02X}:\t sub a,a", opcode).to_string())),
            0xD6 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t sub a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x98 => ret.push((current_pc, format!("{:02X}:\t sbc a,b", opcode).to_string())),
            0x99 => ret.push((current_pc, format!("{:02X}:\t sbc a,c", opcode).to_string())),
            0x9A => ret.push((current_pc, format!("{:02X}:\t sbc a,d", opcode).to_string())),
            0x9B => ret.push((current_pc, format!("{:02X}:\t sbc a,e", opcode).to_string())),
            0x9C => ret.push((current_pc, format!("{:02X}:\t sbc a,h", opcode).to_string())),
            0x9D => ret.push((current_pc, format!("{:02X}:\t sbc a,l", opcode).to_string())),
            0x9E => ret.push((
                current_pc,
                format!("{:02X}:\t sbc a,(hl)", opcode).to_string(),
            )),
            0x9F => ret.push((current_pc, format!("{:02X}:\t sbc a,a", opcode).to_string())),
            0xDE => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t sbc a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xA0 => ret.push((current_pc, format!("{:02X}:\t and a,b", opcode).to_string())),
            0xA1 => ret.push((current_pc, format!("{:02X}:\t and a,c", opcode).to_string())),
            0xA2 => ret.push((current_pc, format!("{:02X}:\t and a,d", opcode).to_string())),
            0xA3 => ret.push((current_pc, format!("{:02X}:\t and a,e", opcode).to_string())),
            0xA4 => ret.push((current_pc, format!("{:02X}:\t and a,h", opcode).to_string())),
            0xA5 => ret.push((current_pc, format!("{:02X}:\t and a,l", opcode).to_string())),
            0xA6 => ret.push((
                current_pc,
                format!("{:02X}:\t and a,(hl)", opcode).to_string(),
            )),
            0xA7 => ret.push((current_pc, format!("{:02X}:\t and a,a", opcode).to_string())),
            0xE6 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t and a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xA8 => ret.push((current_pc, format!("{:02X}:\t xor a,b", opcode).to_string())),
            0xA9 => ret.push((current_pc, format!("{:02X}:\t xor a,c", opcode).to_string())),
            0xAA => ret.push((current_pc, format!("{:02X}:\t xor a,d", opcode).to_string())),
            0xAB => ret.push((current_pc, format!("{:02X}:\t xor a,e", opcode).to_string())),
            0xAC => ret.push((current_pc, format!("{:02X}:\t xor a,h", opcode).to_string())),
            0xAD => ret.push((current_pc, format!("{:02X}:\t xor a,l", opcode).to_string())),
            0xAE => ret.push((
                current_pc,
                format!("{:02X}:\t xor a,(hl)", opcode).to_string(),
            )),
            0xAF => ret.push((current_pc, format!("{:02X}:\t xor a,a", opcode).to_string())),
            0xEE => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t xor a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xB0 => ret.push((current_pc, format!("{:02X}:\t or a,b", opcode).to_string())),
            0xB1 => ret.push((current_pc, format!("{:02X}:\t or a,c", opcode).to_string())),
            0xB2 => ret.push((current_pc, format!("{:02X}:\t or a,d", opcode).to_string())),
            0xB3 => ret.push((current_pc, format!("{:02X}:\t or a,e", opcode).to_string())),
            0xB4 => ret.push((current_pc, format!("{:02X}:\t or a,h", opcode).to_string())),
            0xB5 => ret.push((current_pc, format!("{:02X}:\t or a,l", opcode).to_string())),
            0xB6 => ret.push((
                current_pc,
                format!("{:02X}:\t or a,(hl)", opcode).to_string(),
            )),
            0xB7 => ret.push((current_pc, format!("{:02X}:\t or a,a", opcode).to_string())),
            0xF6 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t or a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xB8 => ret.push((current_pc, format!("{:02X}:\t cp a,b", opcode).to_string())),
            0xB9 => ret.push((current_pc, format!("{:02X}:\t cp a,c", opcode).to_string())),
            0xBA => ret.push((current_pc, format!("{:02X}:\t cp a,d", opcode).to_string())),
            0xBB => ret.push((current_pc, format!("{:02X}:\t cp a,e", opcode).to_string())),
            0xBC => ret.push((current_pc, format!("{:02X}:\t cp a,h", opcode).to_string())),
            0xBD => ret.push((current_pc, format!("{:02X}:\t cp a,l", opcode).to_string())),
            0xBE => ret.push((
                current_pc,
                format!("{:02X}:\t cp a,(hl)", opcode).to_string(),
            )),
            0xBF => ret.push((current_pc, format!("{:02X}:\t cp a,a", opcode).to_string())),
            0xFE => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t cp a,${:02X}", opcode, a1, a1).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x2F => ret.push((current_pc, format!("{:02X}:\t cpl a", opcode).to_string())),
            0x04 => ret.push((current_pc, format!("{:02X}:\t inc b", opcode).to_string())),
            0x0C => ret.push((current_pc, format!("{:02X}:\t inc c", opcode).to_string())),
            0x14 => ret.push((current_pc, format!("{:02X}:\t inc d", opcode).to_string())),
            0x1C => ret.push((current_pc, format!("{:02X}:\t inc e", opcode).to_string())),
            0x24 => ret.push((current_pc, format!("{:02X}:\t inc h", opcode).to_string())),
            0x2C => ret.push((current_pc, format!("{:02X}:\t inc l", opcode).to_string())),
            0x34 => ret.push((
                current_pc,
                format!("{:02X}:\t inc (hl)", opcode).to_string(),
            )),
            0x3C => ret.push((current_pc, format!("{:02X}:\t inc a", opcode).to_string())),
            0x05 => ret.push((current_pc, format!("{:02X}:\t dec b", opcode).to_string())),
            0x0D => ret.push((current_pc, format!("{:02X}:\t dec c", opcode).to_string())),
            0x15 => ret.push((current_pc, format!("{:02X}:\t dec d", opcode).to_string())),
            0x1D => ret.push((current_pc, format!("{:02X}:\t dec e", opcode).to_string())),
            0x25 => ret.push((current_pc, format!("{:02X}:\t dec h", opcode).to_string())),
            0x2D => ret.push((current_pc, format!("{:02X}:\t dec l", opcode).to_string())),
            0x35 => ret.push((
                current_pc,
                format!("{:02X}:\t dec (hl)", opcode).to_string(),
            )),
            0x3D => ret.push((current_pc, format!("{:02X}:\t dec a", opcode).to_string())),
            0x03 => ret.push((current_pc, format!("{:02X}:\t inc bc", opcode).to_string())),
            0x13 => ret.push((current_pc, format!("{:02X}:\t inc de", opcode).to_string())),
            0x23 => ret.push((current_pc, format!("{:02X}:\t inc hl", opcode).to_string())),
            0x33 => ret.push((current_pc, format!("{:02X}:\t inc sp", opcode).to_string())),
            0x0B => ret.push((current_pc, format!("{:02X}:\t dec bc", opcode).to_string())),
            0x1B => ret.push((current_pc, format!("{:02X}:\t dec de", opcode).to_string())),
            0x2B => ret.push((current_pc, format!("{:02X}:\t dec hl", opcode).to_string())),
            0x3B => ret.push((current_pc, format!("{:02X}:\t dec sp", opcode).to_string())),
            0xC1 => ret.push((current_pc, format!("{:02X}:\t pop bc", opcode).to_string())),
            0xD1 => ret.push((current_pc, format!("{:02X}:\t pop de", opcode).to_string())),
            0xE1 => ret.push((current_pc, format!("{:02X}:\t pop hl", opcode).to_string())),
            0xF1 => ret.push((current_pc, format!("{:02X}:\t pop af", opcode).to_string())),
            0xC5 => ret.push((current_pc, format!("{:02X}:\t push bc", opcode).to_string())),
            0xD5 => ret.push((current_pc, format!("{:02X}:\t push de", opcode).to_string())),
            0xE5 => ret.push((current_pc, format!("{:02X}:\t push hl", opcode).to_string())),
            0xF5 => ret.push((current_pc, format!("{:02X}:\t push af", opcode).to_string())),
            0xC3 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t jp ${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xE9 => ret.push((current_pc, format!("{:02X}:\t jp hl", opcode).to_string())),
            0xC2 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t jp nz,${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xD2 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t jp nc,${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xCA => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t jp z,${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xDA => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t jp c,${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x18 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t jp pc+({})", opcode, a1, *a1 as i8).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x20 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t jp nz,pc+({})", opcode, a1, *a1 as i8).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x30 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t jp nc,pc+({})", opcode, a1, *a1 as i8).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x28 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t jp z,pc+({})", opcode, a1, *a1 as i8).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0x38 => {
                if let Some(a1) = iter.next() {
                    ret.push((
                        current_pc,
                        format!("{:02X}{:02X}:\t jp c,pc+({})", opcode, a1, *a1 as i8).to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xCD => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t call ${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xC4 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t call nz,${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xCC => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t call nc,${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xD4 => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t call z,${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xDC => {
                if let (Some(a1), Some(a2)) = (iter.next(), iter.next()) {
                    ret.push((
                        current_pc,
                        format!(
                            "{:02X}{:02X}{:02X}:\t call c,${:02X}{:02X}",
                            opcode, a1, a2, a2, a1
                        )
                        .to_string(),
                    ))
                } else {
                    break;
                }
            }
            0xC9 => ret.push((current_pc, format!("{:02X}:\t ret", opcode).to_string())),
            0xC0 => ret.push((current_pc, format!("{:02X}:\t ret nz", opcode).to_string())),
            0xC8 => ret.push((current_pc, format!("{:02X}:\t ret z", opcode).to_string())),
            0xD0 => ret.push((current_pc, format!("{:02X}:\t ret nc", opcode).to_string())),
            0xD8 => ret.push((current_pc, format!("{:02X}:\t ret c", opcode).to_string())),
            0xD9 => ret.push((current_pc, format!("{:02X}:\t reti", opcode).to_string())),
            0xC7 => ret.push((current_pc, format!("{:02X}:\t rst 00", opcode).to_string())),
            0xCF => ret.push((current_pc, format!("{:02X}:\t rst 08", opcode).to_string())),
            0xD7 => ret.push((current_pc, format!("{:02X}:\t rst 10", opcode).to_string())),
            0xDF => ret.push((current_pc, format!("{:02X}:\t rst 18", opcode).to_string())),
            0xE7 => ret.push((current_pc, format!("{:02X}:\t rst 20", opcode).to_string())),
            0xEF => ret.push((current_pc, format!("{:02X}:\t rst 28", opcode).to_string())),
            0xF7 => ret.push((current_pc, format!("{:02X}:\t rst 30", opcode).to_string())),
            0xFF => ret.push((current_pc, format!("{:02X}:\t rst 38", opcode).to_string())),
            0x07 => ret.push((current_pc, format!("{:02X}:\t rlca", opcode).to_string())),
            0x17 => ret.push((current_pc, format!("{:02X}:\t rla", opcode).to_string())),
            0x0F => ret.push((current_pc, format!("{:02X}:\t rrca", opcode).to_string())),
            0x1F => ret.push((current_pc, format!("{:02X}:\t rra", opcode).to_string())),

            // CB Prefix
            0xCB => {
                if let Some(opcode) = iter.next() {
                    match opcode {
                        0x00 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rlc b", opcode).to_string()))
                        }
                        0x01 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rlc c", opcode).to_string()))
                        }
                        0x02 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rlc d", opcode).to_string()))
                        }
                        0x03 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rlc e", opcode).to_string()))
                        }
                        0x04 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rlc h", opcode).to_string()))
                        }
                        0x05 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rlc l", opcode).to_string()))
                        }
                        0x06 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t rlc (hl)", opcode).to_string(),
                        )),
                        0x07 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rlc a", opcode).to_string()))
                        }
                        0x08 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rrc b", opcode).to_string()))
                        }
                        0x09 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rrc c", opcode).to_string()))
                        }
                        0x0A => {
                            ret.push((current_pc, format!("CB{:02X}:\t rrc d", opcode).to_string()))
                        }
                        0x0B => {
                            ret.push((current_pc, format!("CB{:02X}:\t rrc e", opcode).to_string()))
                        }
                        0x0C => {
                            ret.push((current_pc, format!("CB{:02X}:\t rrc h", opcode).to_string()))
                        }
                        0x0D => {
                            ret.push((current_pc, format!("CB{:02X}:\t rrc l", opcode).to_string()))
                        }
                        0x0E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t rrc (hl)", opcode).to_string(),
                        )),
                        0x0F => {
                            ret.push((current_pc, format!("CB{:02X}:\t rrc a", opcode).to_string()))
                        }
                        0x10 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rl b", opcode).to_string()))
                        }
                        0x11 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rl c", opcode).to_string()))
                        }
                        0x12 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rl d", opcode).to_string()))
                        }
                        0x13 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rl e", opcode).to_string()))
                        }
                        0x14 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rl h", opcode).to_string()))
                        }
                        0x15 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rl l", opcode).to_string()))
                        }
                        0x16 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t rl (hl)", opcode).to_string(),
                        )),
                        0x17 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rl a", opcode).to_string()))
                        }
                        0x18 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rr b", opcode).to_string()))
                        }
                        0x19 => {
                            ret.push((current_pc, format!("CB{:02X}:\t rr c", opcode).to_string()))
                        }
                        0x1A => {
                            ret.push((current_pc, format!("CB{:02X}:\t rr d", opcode).to_string()))
                        }
                        0x1B => {
                            ret.push((current_pc, format!("CB{:02X}:\t rr e", opcode).to_string()))
                        }
                        0x1C => {
                            ret.push((current_pc, format!("CB{:02X}:\t rr h", opcode).to_string()))
                        }
                        0x1D => {
                            ret.push((current_pc, format!("CB{:02X}:\t rr l", opcode).to_string()))
                        }
                        0x1E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t rr (hl)", opcode).to_string(),
                        )),
                        0x1F => {
                            ret.push((current_pc, format!("CB{:02X}:\t rr a", opcode).to_string()))
                        }
                        0x20 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sla b", opcode).to_string()))
                        }
                        0x21 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sla c", opcode).to_string()))
                        }
                        0x22 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sla d", opcode).to_string()))
                        }
                        0x23 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sla e", opcode).to_string()))
                        }
                        0x24 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sla h", opcode).to_string()))
                        }
                        0x25 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sla l", opcode).to_string()))
                        }
                        0x26 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t sla (hl)", opcode).to_string(),
                        )),
                        0x27 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sla a", opcode).to_string()))
                        }
                        0x28 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sra b", opcode).to_string()))
                        }
                        0x29 => {
                            ret.push((current_pc, format!("CB{:02X}:\t sra c", opcode).to_string()))
                        }
                        0x2A => {
                            ret.push((current_pc, format!("CB{:02X}:\t sra d", opcode).to_string()))
                        }
                        0x2B => {
                            ret.push((current_pc, format!("CB{:02X}:\t sra e", opcode).to_string()))
                        }
                        0x2C => {
                            ret.push((current_pc, format!("CB{:02X}:\t sra h", opcode).to_string()))
                        }
                        0x2D => {
                            ret.push((current_pc, format!("CB{:02X}:\t sra l", opcode).to_string()))
                        }
                        0x2E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t sra (hl)", opcode).to_string(),
                        )),
                        0x2F => {
                            ret.push((current_pc, format!("CB{:02X}:\t sra a", opcode).to_string()))
                        }
                        0x30 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t swap b", opcode).to_string(),
                        )),
                        0x31 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t swap c", opcode).to_string(),
                        )),
                        0x32 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t swap d", opcode).to_string(),
                        )),
                        0x33 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t swap e", opcode).to_string(),
                        )),
                        0x34 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t swap h", opcode).to_string(),
                        )),
                        0x35 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t swap l", opcode).to_string(),
                        )),
                        0x36 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t swap (hl)", opcode).to_string(),
                        )),
                        0x37 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t swap a", opcode).to_string(),
                        )),
                        0x38 => {
                            ret.push((current_pc, format!("CB{:02X}:\t srl b", opcode).to_string()))
                        }
                        0x39 => {
                            ret.push((current_pc, format!("CB{:02X}:\t srl c", opcode).to_string()))
                        }
                        0x3A => {
                            ret.push((current_pc, format!("CB{:02X}:\t srl d", opcode).to_string()))
                        }
                        0x3B => {
                            ret.push((current_pc, format!("CB{:02X}:\t srl e", opcode).to_string()))
                        }
                        0x3C => {
                            ret.push((current_pc, format!("CB{:02X}:\t srl h", opcode).to_string()))
                        }
                        0x3D => {
                            ret.push((current_pc, format!("CB{:02X}:\t srl l", opcode).to_string()))
                        }
                        0x3E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t srl (hl)", opcode).to_string(),
                        )),
                        0x3F => {
                            ret.push((current_pc, format!("CB{:02X}:\t srl a", opcode).to_string()))
                        }
                        0x40 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 0,b", opcode).to_string(),
                        )),
                        0x41 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 0,c", opcode).to_string(),
                        )),
                        0x42 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 0,d", opcode).to_string(),
                        )),
                        0x43 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 0,e", opcode).to_string(),
                        )),
                        0x44 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 0,h", opcode).to_string(),
                        )),
                        0x45 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 0,l", opcode).to_string(),
                        )),
                        0x46 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 0,(hl)", opcode).to_string(),
                        )),
                        0x47 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 0,a", opcode).to_string(),
                        )),
                        0x48 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 1,b", opcode).to_string(),
                        )),
                        0x49 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 1,c", opcode).to_string(),
                        )),
                        0x4A => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 1,d", opcode).to_string(),
                        )),
                        0x4B => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 1,e", opcode).to_string(),
                        )),
                        0x4C => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 1,h", opcode).to_string(),
                        )),
                        0x4D => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 1,l", opcode).to_string(),
                        )),
                        0x4E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 1,(hl)", opcode).to_string(),
                        )),
                        0x4F => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 1,a", opcode).to_string(),
                        )),
                        0x50 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 2,b", opcode).to_string(),
                        )),
                        0x51 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 2,c", opcode).to_string(),
                        )),
                        0x52 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 2,d", opcode).to_string(),
                        )),
                        0x53 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 2,e", opcode).to_string(),
                        )),
                        0x54 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 2,h", opcode).to_string(),
                        )),
                        0x55 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 2,l", opcode).to_string(),
                        )),
                        0x56 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 2,(hl)", opcode).to_string(),
                        )),
                        0x57 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 2,a", opcode).to_string(),
                        )),
                        0x58 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 3,b", opcode).to_string(),
                        )),
                        0x59 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 3,c", opcode).to_string(),
                        )),
                        0x5A => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 3,d", opcode).to_string(),
                        )),
                        0x5B => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 3,e", opcode).to_string(),
                        )),
                        0x5C => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 3,h", opcode).to_string(),
                        )),
                        0x5D => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 3,l", opcode).to_string(),
                        )),
                        0x5E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 3,(hl)", opcode).to_string(),
                        )),
                        0x5F => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 3,a", opcode).to_string(),
                        )),
                        0x60 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 4,b", opcode).to_string(),
                        )),
                        0x61 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 4,c", opcode).to_string(),
                        )),
                        0x62 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 4,d", opcode).to_string(),
                        )),
                        0x63 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 4,e", opcode).to_string(),
                        )),
                        0x64 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 4,h", opcode).to_string(),
                        )),
                        0x65 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 4,l", opcode).to_string(),
                        )),
                        0x66 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 4,(hl)", opcode).to_string(),
                        )),
                        0x67 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 4,a", opcode).to_string(),
                        )),
                        0x68 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 5,b", opcode).to_string(),
                        )),
                        0x69 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 5,c", opcode).to_string(),
                        )),
                        0x6A => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 5,d", opcode).to_string(),
                        )),
                        0x6B => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 5,e", opcode).to_string(),
                        )),
                        0x6C => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 5,h", opcode).to_string(),
                        )),
                        0x6D => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 5,l", opcode).to_string(),
                        )),
                        0x6E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 5,(hl)", opcode).to_string(),
                        )),
                        0x6F => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 5,a", opcode).to_string(),
                        )),
                        0x70 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 6,b", opcode).to_string(),
                        )),
                        0x71 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 6,c", opcode).to_string(),
                        )),
                        0x72 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 6,d", opcode).to_string(),
                        )),
                        0x73 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 6,e", opcode).to_string(),
                        )),
                        0x74 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 6,h", opcode).to_string(),
                        )),
                        0x75 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 6,l", opcode).to_string(),
                        )),
                        0x76 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 6,(hl)", opcode).to_string(),
                        )),
                        0x77 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 6,a", opcode).to_string(),
                        )),
                        0x78 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 7,b", opcode).to_string(),
                        )),
                        0x79 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 7,c", opcode).to_string(),
                        )),
                        0x7A => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 7,d", opcode).to_string(),
                        )),
                        0x7B => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 7,e", opcode).to_string(),
                        )),
                        0x7C => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 7,h", opcode).to_string(),
                        )),
                        0x7D => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 7,l", opcode).to_string(),
                        )),
                        0x7E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 7,(hl)", opcode).to_string(),
                        )),
                        0x7F => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t bit 7,a", opcode).to_string(),
                        )),
                        0x80 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 0,b", opcode).to_string(),
                        )),
                        0x81 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 0,c", opcode).to_string(),
                        )),
                        0x82 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 0,d", opcode).to_string(),
                        )),
                        0x83 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 0,e", opcode).to_string(),
                        )),
                        0x84 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 0,h", opcode).to_string(),
                        )),
                        0x85 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 0,l", opcode).to_string(),
                        )),
                        0x86 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 0,(hl)", opcode).to_string(),
                        )),
                        0x87 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 0,a", opcode).to_string(),
                        )),
                        0x88 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 1,b", opcode).to_string(),
                        )),
                        0x89 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 1,c", opcode).to_string(),
                        )),
                        0x8A => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 1,d", opcode).to_string(),
                        )),
                        0x8B => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 1,e", opcode).to_string(),
                        )),
                        0x8C => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 1,h", opcode).to_string(),
                        )),
                        0x8D => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 1,l", opcode).to_string(),
                        )),
                        0x8E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 1,(hl)", opcode).to_string(),
                        )),
                        0x8F => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 1,a", opcode).to_string(),
                        )),
                        0x90 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 2,b", opcode).to_string(),
                        )),
                        0x91 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 2,c", opcode).to_string(),
                        )),
                        0x92 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 2,d", opcode).to_string(),
                        )),
                        0x93 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 2,e", opcode).to_string(),
                        )),
                        0x94 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 2,h", opcode).to_string(),
                        )),
                        0x95 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 2,l", opcode).to_string(),
                        )),
                        0x96 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 2,(hl)", opcode).to_string(),
                        )),
                        0x97 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 2,a", opcode).to_string(),
                        )),
                        0x98 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 3,b", opcode).to_string(),
                        )),
                        0x99 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 3,c", opcode).to_string(),
                        )),
                        0x9A => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 3,d", opcode).to_string(),
                        )),
                        0x9B => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 3,e", opcode).to_string(),
                        )),
                        0x9C => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 3,h", opcode).to_string(),
                        )),
                        0x9D => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 3,l", opcode).to_string(),
                        )),
                        0x9E => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 3,(hl)", opcode).to_string(),
                        )),
                        0x9F => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 3,a", opcode).to_string(),
                        )),
                        0xA0 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 4,b", opcode).to_string(),
                        )),
                        0xA1 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 4,c", opcode).to_string(),
                        )),
                        0xA2 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 4,d", opcode).to_string(),
                        )),
                        0xA3 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 4,e", opcode).to_string(),
                        )),
                        0xA4 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 4,h", opcode).to_string(),
                        )),
                        0xA5 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 4,l", opcode).to_string(),
                        )),
                        0xA6 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 4,(hl)", opcode).to_string(),
                        )),
                        0xA7 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 4,a", opcode).to_string(),
                        )),
                        0xA8 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 5,b", opcode).to_string(),
                        )),
                        0xA9 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 5,c", opcode).to_string(),
                        )),
                        0xAA => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 5,d", opcode).to_string(),
                        )),
                        0xAB => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 5,e", opcode).to_string(),
                        )),
                        0xAC => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 5,h", opcode).to_string(),
                        )),
                        0xAD => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 5,l", opcode).to_string(),
                        )),
                        0xAE => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 5,(hl)", opcode).to_string(),
                        )),
                        0xAF => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 5,a", opcode).to_string(),
                        )),
                        0xB0 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 6,b", opcode).to_string(),
                        )),
                        0xB1 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 6,c", opcode).to_string(),
                        )),
                        0xB2 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 6,d", opcode).to_string(),
                        )),
                        0xB3 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 6,e", opcode).to_string(),
                        )),
                        0xB4 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 6,h", opcode).to_string(),
                        )),
                        0xB5 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 6,l", opcode).to_string(),
                        )),
                        0xB6 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 6,(hl)", opcode).to_string(),
                        )),
                        0xB7 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 6,a", opcode).to_string(),
                        )),
                        0xB8 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 7,b", opcode).to_string(),
                        )),
                        0xB9 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 7,c", opcode).to_string(),
                        )),
                        0xBA => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 7,d", opcode).to_string(),
                        )),
                        0xBB => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 7,e", opcode).to_string(),
                        )),
                        0xBC => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 7,h", opcode).to_string(),
                        )),
                        0xBD => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 7,l", opcode).to_string(),
                        )),
                        0xBE => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 7,(hl)", opcode).to_string(),
                        )),
                        0xBF => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t res 7,a", opcode).to_string(),
                        )),
                        0xC0 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 0,b", opcode).to_string(),
                        )),
                        0xC1 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 0,c", opcode).to_string(),
                        )),
                        0xC2 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 0,d", opcode).to_string(),
                        )),
                        0xC3 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 0,e", opcode).to_string(),
                        )),
                        0xC4 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 0,h", opcode).to_string(),
                        )),
                        0xC5 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 0,l", opcode).to_string(),
                        )),
                        0xC6 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 0,(hl)", opcode).to_string(),
                        )),
                        0xC7 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 0,a", opcode).to_string(),
                        )),
                        0xC8 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 1,b", opcode).to_string(),
                        )),
                        0xC9 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 1,c", opcode).to_string(),
                        )),
                        0xCA => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 1,d", opcode).to_string(),
                        )),
                        0xCB => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 1,e", opcode).to_string(),
                        )),
                        0xCC => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 1,h", opcode).to_string(),
                        )),
                        0xCD => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 1,l", opcode).to_string(),
                        )),
                        0xCE => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 1,(hl)", opcode).to_string(),
                        )),
                        0xCF => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 1,a", opcode).to_string(),
                        )),
                        0xD0 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 2,b", opcode).to_string(),
                        )),
                        0xD1 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 2,c", opcode).to_string(),
                        )),
                        0xD2 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 2,d", opcode).to_string(),
                        )),
                        0xD3 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 2,e", opcode).to_string(),
                        )),
                        0xD4 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 2,h", opcode).to_string(),
                        )),
                        0xD5 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 2,l", opcode).to_string(),
                        )),
                        0xD6 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 2,(hl)", opcode).to_string(),
                        )),
                        0xD7 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 2,a", opcode).to_string(),
                        )),
                        0xD8 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 3,b", opcode).to_string(),
                        )),
                        0xD9 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 3,c", opcode).to_string(),
                        )),
                        0xDA => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 3,d", opcode).to_string(),
                        )),
                        0xDB => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 3,e", opcode).to_string(),
                        )),
                        0xDC => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 3,h", opcode).to_string(),
                        )),
                        0xDD => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 3,l", opcode).to_string(),
                        )),
                        0xDE => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 3,(hl)", opcode).to_string(),
                        )),
                        0xDF => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 3,a", opcode).to_string(),
                        )),
                        0xE0 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 4,b", opcode).to_string(),
                        )),
                        0xE1 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 4,c", opcode).to_string(),
                        )),
                        0xE2 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 4,d", opcode).to_string(),
                        )),
                        0xE3 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 4,e", opcode).to_string(),
                        )),
                        0xE4 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 4,h", opcode).to_string(),
                        )),
                        0xE5 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 4,l", opcode).to_string(),
                        )),
                        0xE6 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 4,(hl)", opcode).to_string(),
                        )),
                        0xE7 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 4,a", opcode).to_string(),
                        )),
                        0xE8 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 5,b", opcode).to_string(),
                        )),
                        0xE9 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 5,c", opcode).to_string(),
                        )),
                        0xEA => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 5,d", opcode).to_string(),
                        )),
                        0xEB => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 5,e", opcode).to_string(),
                        )),
                        0xEC => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 5,h", opcode).to_string(),
                        )),
                        0xED => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 5,l", opcode).to_string(),
                        )),
                        0xEE => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 5,(hl)", opcode).to_string(),
                        )),
                        0xEF => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 5,a", opcode).to_string(),
                        )),
                        0xF0 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 6,b", opcode).to_string(),
                        )),
                        0xF1 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 6,c", opcode).to_string(),
                        )),
                        0xF2 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 6,d", opcode).to_string(),
                        )),
                        0xF3 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 6,e", opcode).to_string(),
                        )),
                        0xF4 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 6,h", opcode).to_string(),
                        )),
                        0xF5 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 6,l", opcode).to_string(),
                        )),
                        0xF6 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 6,(hl)", opcode).to_string(),
                        )),
                        0xF7 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 6,a", opcode).to_string(),
                        )),
                        0xF8 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 7,b", opcode).to_string(),
                        )),
                        0xF9 => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 7,c", opcode).to_string(),
                        )),
                        0xFA => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 7,d", opcode).to_string(),
                        )),
                        0xFB => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 7,e", opcode).to_string(),
                        )),
                        0xFC => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 7,h", opcode).to_string(),
                        )),
                        0xFD => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 7,l", opcode).to_string(),
                        )),
                        0xFE => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 7,(hl)", opcode).to_string(),
                        )),
                        0xFF => ret.push((
                            current_pc,
                            format!("CB{:02X}:\t set 7,a", opcode).to_string(),
                        )),
                    };
                    current_pc += 1;
                }
            }
            _ => ret.push((current_pc, format!("{:02X}:\t ???", opcode).to_string())),
        };
        current_pc += OPCODE_SIZE[*opcode as usize] as u16;
    }
    ret
}

/// Returns a String representation of the
pub fn get_opcode(opcode: u8) -> String {
    OPCODE_STRINGS[opcode as usize].to_string()
}

const OPCODE_STRINGS: [&str; 256] = [
    "NOP",
    "LD BC,d16",
    "LD (BC),A",
    "INC BC",
    "INC B",
    "DEC B",
    "LD B,d8",
    "RLCA",
    "LD (a16),SP",
    "ADD HL,BC",
    "LD A,(BC)",
    "DEC BC",
    "INC C",
    "DEC C",
    "LD C,d8",
    "RRCA",
    "STOP 0",
    "LD DE,d16",
    "LD (DE),A",
    "INC DE",
    "INC D",
    "DEC D",
    "LD D,d8",
    "RLA",
    "JR r8",
    "ADD HL,DE",
    "LD A,(DE)",
    "DEC DE",
    "INC E",
    "DEC E",
    "LD E,d8",
    "RRA",
    "JR NZ,r8",
    "LD HL,d16",
    "LD (HL+),A",
    "INC HL",
    "INC H",
    "DEC H",
    "LD H,d8",
    "DAA",
    "JR Z,r8",
    "ADD HL,HL",
    "LD A,(HL+)",
    "DEC HL",
    "INC L",
    "DEC L",
    "LD L,d8",
    "CPL",
    "JR NC,r8",
    "LD SP,d16",
    "LD (HL-),A",
    "INC SP",
    "INC (HL)",
    "DEC (HL)",
    "LD (HL),d8",
    "SCF",
    "JR C,r8",
    "ADD HL,SP",
    "LD A,(HL-)",
    "DEC SP",
    "INC A",
    "DEC A",
    "LD A,d8",
    "CCF",
    "LD B,B",
    "LD B,C",
    "LD B,D",
    "LD B,E",
    "LD B,H",
    "LD B,L",
    "LD B,(HL)",
    "LD B,A",
    "LD C,B",
    "LD C,C",
    "LD C,D",
    "LD C,E",
    "LD C,H",
    "LD C,L",
    "LD C,(HL)",
    "LD C,A",
    "LD D,B",
    "LD D,C",
    "LD D,D",
    "LD D,E",
    "LD D,H",
    "LD D,L",
    "LD D,(HL)",
    "LD D,A",
    "LD E,B",
    "LD E,C",
    "LD E,D",
    "LD E,E",
    "LD E,H",
    "LD E,L",
    "LD E,(HL)",
    "LD E,A",
    "LD H,B",
    "LD H,C",
    "LD H,D",
    "LD H,E",
    "LD H,H",
    "LD H,L",
    "LD H,(HL)",
    "LD H,A",
    "LD L,B",
    "LD L,C",
    "LD L,D",
    "LD L,E",
    "LD L,H",
    "LD L,L",
    "LD L,(HL)",
    "LD L,A",
    "LD (HL),B",
    "LD (HL),C",
    "LD (HL),D",
    "LD (HL),E",
    "LD (HL),H",
    "LD (HL),L",
    "HALT",
    "LD (HL),A",
    "LD A,B",
    "LD A,C",
    "LD A,D",
    "LD A,E",
    "LD A,H",
    "LD A,L",
    "LD A,(HL)",
    "LD A,A",
    "ADD A,B",
    "ADD A,C",
    "ADD A,D",
    "ADD A,E",
    "ADD A,H",
    "ADD A,L",
    "ADD A,(HL)",
    "ADD A,A",
    "ADC A,B",
    "ADC A,C",
    "ADC A,D",
    "ADC A,E",
    "ADC A,H",
    "ADC A,L",
    "ADC A,(HL)",
    "ADC A,A",
    "SUB A,B",
    "SUB A,C",
    "SUB A,D",
    "SUB A,E",
    "SUB A,H",
    "SUB A,L",
    "SUB A,(HL)",
    "SUB A,A",
    "SBC A,B",
    "SBC A,C",
    "SBC A,D",
    "SBC A,E",
    "SBC A,H",
    "SBC A,L",
    "SBC A,(HL)",
    "SBC A,A",
    "AND B",
    "AND C",
    "AND D",
    "AND E",
    "AND H",
    "AND L",
    "AND (HL)",
    "AND A",
    "XOR B",
    "XOR C",
    "XOR D",
    "XOR E",
    "XOR H",
    "XOR L",
    "XOR (HL)",
    "XOR A",
    "OR B",
    "OR C",
    "OR D",
    "OR E",
    "OR H",
    "OR L",
    "OR (HL)",
    "OR A",
    "CP B",
    "CP C",
    "CP D",
    "CP E",
    "CP H",
    "CP L",
    "CP (HL)",
    "CP A",
    "RET NZ",
    "POP BC",
    "JP NZ,a16",
    "JP a16",
    "CALL NZ,a16",
    "PUSH BC",
    "ADD A,d8",
    "RST 00H",
    "RET Z",
    "RET",
    "JP Z,a16",
    "CB ",
    "CALL Z,a16",
    "CALL a16",
    "ADC A,d8",
    "RST 08H",
    "RET NC",
    "POP DE",
    "JP NC,a16",
    "NULL",
    "CALL NC,a16",
    "PUSH DE",
    "SUB d8",
    "RST 10H",
    "RET C",
    "RETI",
    "JP C,a16",
    "NULL",
    "CALL C,a16",
    "NULL",
    "SBC A,d8",
    "RST 18H",
    "LDH (a8),A",
    "POP HL",
    "LD (C),A",
    "NULL",
    "NULL",
    "PUSH HL",
    "AND d8",
    "RST 20H",
    "ADD SP,r8",
    "JP (HL)",
    "JP (a16),A",
    "NULL",
    "NULL",
    "NULL",
    "XOR d8",
    "RST 28H",
    "LDH A,(a8)",
    "POP AF",
    "LD A,(C)",
    "DI",
    "NULL",
    "PUSH AF",
    "OR d8",
    "RST 30H",
    "LD HL,SP+r8",
    "LD SP,HL",
    "JP A,(a16)",
    "EI",
    "NULL",
    "NULL",
    "CP d8",
    "RST 38H",
];

/// Tables of opcode sizes in bytes
/// Skipped when running rustfmt
#[rustfmt::skip]
const OPCODE_SIZE: [usize; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, // 0
    1, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 1
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 2
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 8
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 9
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // A
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // B
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, // C
    1, 1, 3, 1, 3, 1, 2, 1, 1, 1, 3, 1, 3, 1, 2, 1, // D
    2, 1, 1, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1, // E
    2, 1, 1, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1, // F
];

#[cfg(test)]
mod disassemble_tests {
    #[test]
    fn interrupt_requests() {}
}
