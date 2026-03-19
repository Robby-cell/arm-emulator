use crate::cpu::{Cpu, registers::*};

#[test]
fn correct_new_construction() {
    let cpu = Cpu::new();
    for register in cpu.registers {
        assert_eq!(register, 0);
    }
}

#[test]
fn correct_indexing() {
    let mut cpu = Cpu::new();

    cpu[R0] = 42;
    cpu[R1] = 58;
    cpu[R2] = 42 + 58;

    assert_eq!(cpu[R0], 42);
    assert_eq!(cpu[R1], 58);
    assert_eq!(cpu[R2], 42 + 58);

    for i in R3..=R15 {
        assert_eq!(cpu[i], 0);
    }
}
