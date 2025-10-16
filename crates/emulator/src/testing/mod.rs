#[cfg(target_endian = "little")]
mod create_instruction {
    pub const fn big_endian_to_native(instr: u32) -> u32 {
        u32::from_be_bytes(instr.to_le_bytes())
    }

    pub const fn little_endian_to_native(instr: u32) -> u32 {
        u32::from_le_bytes(instr.to_le_bytes())
    }

    pub const fn to_bytes(value: u32) -> [u8; 4] {
        value.to_le_bytes()
    }
}

#[cfg(target_endian = "big")]
mod create_instruction {
    pub const fn big_endian_to_native(instr: u32) -> u32 {
        u32::from_be_bytes(instr.to_be_bytes())
    }

    pub const fn little_endian_to_native(instr: u32) -> u32 {
        u32::from_le_bytes(instr.to_be_bytes())
    }

    pub const fn to_bytes(value: u32) -> [u8; 4] {
        value.to_be_bytes()
    }
}

pub use create_instruction::{
    big_endian_to_native, little_endian_to_native, to_bytes,
};

#[test]
fn check_correctness_of_big_endian() {
    let n = 0x12345678;
    let r = to_bytes(big_endian_to_native(n));

    assert_eq!(r, [0x12, 0x34, 0x56, 0x78]);
}

#[test]
fn check_correctness_of_little_endian() {
    let n = 0x12345678;
    let r = to_bytes(little_endian_to_native(n));

    assert_eq!(r, [0x78, 0x56, 0x34, 0x12]);
}
