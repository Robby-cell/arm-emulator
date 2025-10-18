use crate::memory::{
    big_endian_to_native, little_endian_to_native, u32_to_native_bytes,
};

#[test]
fn check_correctness_of_big_endian() {
    let n = 0x12345678;
    let r = u32_to_native_bytes(big_endian_to_native(n));

    assert_eq!(r, [0x12, 0x34, 0x56, 0x78]);
}

#[test]
fn check_correctness_of_little_endian() {
    let n = 0x12345678;
    let r = u32_to_native_bytes(little_endian_to_native(n));

    assert_eq!(r, [0x78, 0x56, 0x34, 0x12]);
}
