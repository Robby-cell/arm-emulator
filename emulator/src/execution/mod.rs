use thiserror::Error;

use crate::{
    Breakpoint, Emulator,
    cpu::Exception,
    instructions::{
        InstructionConversionError, Operand2, fields::ShiftType,
    },
    memory::MemoryAccessError,
};

mod block_data_transfer;
mod branch;
mod branch_exchange;
mod data_processing;
mod memory_access;
mod supervisor_call;

#[cfg(test)]
mod tests;

#[derive(Debug, Error, Clone)]
pub enum ExecutionError {
    #[error("breakpoint reached: {0}")]
    Breakpoint(#[from] Breakpoint),

    #[error("memory access error: {0}")]
    MemoryAccessError(#[from] MemoryAccessError),

    #[error("illegal instruction, could not decode instruction: {0}")]
    InstructionConversionError(#[from] InstructionConversionError),

    #[error("active exception: {0}")]
    Exception(#[from] Exception),
}

mod private {
    pub trait Sealed {}
}

pub trait ExecutableInstruction: private::Sealed {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError>;
}

impl Operand2 {
    #[inline]
    #[must_use]
    fn eval(self, emulator: &mut Emulator) -> (u32, Option<bool>) {
        let op2 = self;

        // We need both the final value and a potential carry bit from the shifter.
        // An Option<bool> is perfect: Some(carry) for shifts, None for immediates.
        let result = match op2 {
            Operand2::Immediate(imm) => {
                let value_8 = (imm & 0xFF) as u32;
                let rotate_4 = (imm >> 8) as u32;

                if rotate_4 == 0 {
                    // If rotate is 0, carry is unaffected (return None)
                    (value_8, None)
                } else {
                    let rotation = rotate_4 * 2;
                    let result = value_8.rotate_right(rotation);

                    // If rotated, the carry flag is set to the last bit rotated out.
                    // This corresponds to bit 31 of the result.
                    let carry = (result >> 31) & 1 == 1;
                    (result, Some(carry))
                }
            }
            Operand2::ShiftedRegisterOffset(sro) => {
                let rm_val = emulator.cpu[sro.rm() as _];
                let shift_amount = sro.shift_amount();

                match sro.shift_type() {
                    ShiftType::LSL => {
                        if shift_amount == 0 {
                            // LSL #0 does not change the value and does not affect the carry.
                            (rm_val, None)
                        } else {
                            let carry =
                                (rm_val >> (32 - shift_amount)) & 1 == 1;
                            (rm_val << shift_amount, Some(carry))
                        }
                    }
                    ShiftType::LSR => {
                        if shift_amount == 0 {
                            // An LSR with immediate 0 is treated as LSR #32.
                            // The result is 0, and the carry is bit 31 of the original value.
                            let carry = (rm_val >> 31) & 1 == 1;
                            (0, Some(carry))
                        } else {
                            let carry =
                                (rm_val >> (shift_amount - 1)) & 1 == 1;
                            (rm_val >> shift_amount, Some(carry))
                        }
                    }
                    ShiftType::ASR => {
                        if shift_amount == 0 {
                            // An ASR with immediate 0 is treated as ASR #32.
                            // The result is all copies of bit 31. Carry is bit 31.
                            let carry = (rm_val >> 31) & 1 == 1;
                            if carry {
                                // if bit 31 was 1
                                (0xFFFFFFFF, Some(carry))
                            } else {
                                (0, Some(carry))
                            }
                        } else {
                            let carry =
                                (rm_val >> (shift_amount - 1)) & 1 == 1;
                            (
                                ((rm_val as i32) >> shift_amount) as _,
                                Some(carry),
                            )
                        }
                    }
                    ShiftType::ROR => {
                        if shift_amount == 0 {
                            // ROR #0 is actually RRX (Rotate Right with Extend).
                            // The new C flag is bit 0 of the value.
                            // The new value has the old C flag in bit 31.
                            let carry_out = (rm_val & 1) == 1;
                            let old_c_val =
                                if emulator.cpu.c() { 1 << 31 } else { 0 };
                            ((rm_val >> 1) | old_c_val, Some(carry_out))
                        } else {
                            let carry =
                                (rm_val >> (shift_amount - 1)) & 1 == 1;
                            (
                                rm_val.rotate_right(shift_amount as _),
                                Some(carry),
                            )
                        }
                    }
                }
            }
        };
        tracing::trace!("Evaluation of Operand2: {result:?}");
        result
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
