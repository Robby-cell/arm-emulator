use std::ops::{Shl, Shr};

use crate::{
    Emulator, ExecutionError,
    instructions::{Operand2, fields::ShiftType},
};

mod branch;
mod data_processing;

#[cfg(test)]
mod tests;

pub trait ExecutableInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError>;
}

impl Operand2 {
    #[inline]
    fn eval(self, emulator: &mut Emulator) -> (u32, Option<bool>) {
        let op2 = self;

        // We need both the final value and a potential carry bit from the shifter.
        // An Option<bool> is perfect: Some(carry) for shifts, None for immediates.
        match op2 {
            Operand2::Immediate(imm) => {
                // For immediate values, the C flag is unaffected.
                (imm as u32, None)
            }
            Operand2::ShiftedRegisterOffset(sro) => {
                let register_value = emulator.cpu.register(sro.rm() as _);
                let shift_amount = sro.shift_amount();

                // A shift amount of 0 is a special case where the carry is also unaffected,
                // unless it's RRX (ROR by 0), which is handled separately in a full emulator.
                if shift_amount == 0 {
                    (register_value, None)
                } else {
                    match sro.shift_type() {
                        ShiftType::LSL => {
                            // Carry is the last bit shifted out, which was at position 32 - shift_amount
                            let carry = (register_value
                                >> (32 - shift_amount))
                                & 1
                                == 1;
                            let result = register_value.shl(shift_amount);
                            (result, Some(carry))
                        }
                        ShiftType::LSR => {
                            // Carry is the last bit shifted out, which was at position shift_amount - 1
                            let carry =
                                (register_value >> (shift_amount - 1)) & 1
                                    == 1;
                            let result = register_value.shr(shift_amount);
                            (result, Some(carry))
                        }
                        ShiftType::ASR => {
                            // Carry is calculated the same as LSR
                            let carry =
                                (register_value >> (shift_amount - 1)) & 1
                                    == 1;
                            let result = (register_value as i32)
                                .shr(shift_amount)
                                as u32;
                            (result, Some(carry))
                        }
                        ShiftType::ROR => {
                            // Carry is the last bit shifted out, which was at position shift_amount - 1
                            let carry =
                                (register_value >> (shift_amount - 1)) & 1
                                    == 1;
                            let result = register_value
                                .rotate_right(shift_amount as _);
                            (result, Some(carry))
                        }
                    }
                }
            }
        }
    }
}

trait N {
    fn n(self) -> bool;
}

trait Z {
    fn z(self) -> bool;
}

impl N for u32 {
    fn n(self) -> bool {
        (self >> 31) & 1 == 1
    }
}

impl Z for u32 {
    fn z(self) -> bool {
        self == 0
    }
}
