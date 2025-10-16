use std::ops::{Shl, Shr};

use crate::{
    Emulator, ExecutionError,
    execution::ExecutableInstruction,
    instructions::{
        DataProcessingInstruction, Operand2,
        fields::{Opcode, SetFlags, ShiftType},
    },
};

impl ExecutableInstruction for DataProcessingInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError> {
        match self.opcode() {
            Opcode::AND => todo!(),
            Opcode::EOR => todo!(),
            Opcode::SUB => todo!(),
            Opcode::RSB => todo!(),
            Opcode::ADD => todo!(),
            Opcode::ADC => todo!(),
            Opcode::SBC => todo!(),
            Opcode::RSC => todo!(),
            Opcode::TST => todo!(),
            Opcode::TEQ => todo!(),
            Opcode::CMP => todo!(),
            Opcode::CMN => todo!(),
            Opcode::ORR => todo!(),
            Opcode::MOV => {
                let op2 = self.op2();

                // We need both the final value and a potential carry bit from the shifter.
                // An Option<bool> is perfect: Some(carry) for shifts, None for immediates.
                let (value, carry_out): (u32, Option<bool>) = match op2 {
                    Operand2::Immediate(imm) => {
                        // For immediate values, the C flag is unaffected.
                        (imm as u32, None)
                    }
                    Operand2::ShiftedRegisterOffset(sro) => {
                        let register_value = emulator.cpu[sro.rm() as _];
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
                                    let result =
                                        register_value.shl(shift_amount);
                                    (result, Some(carry))
                                }
                                ShiftType::LSR => {
                                    // Carry is the last bit shifted out, which was at position shift_amount - 1
                                    let carry = (register_value
                                        >> (shift_amount - 1))
                                        & 1
                                        == 1;
                                    let result =
                                        register_value.shr(shift_amount);
                                    (result, Some(carry))
                                }
                                ShiftType::ASR => {
                                    // Carry is calculated the same as LSR
                                    let carry = (register_value
                                        >> (shift_amount - 1))
                                        & 1
                                        == 1;
                                    let result = (register_value as i32)
                                        .shr(shift_amount)
                                        as u32;
                                    (result, Some(carry))
                                }
                                ShiftType::ROR => {
                                    // Carry is the last bit shifted out, which was at position shift_amount - 1
                                    let carry = (register_value
                                        >> (shift_amount - 1))
                                        & 1
                                        == 1;
                                    let result = register_value
                                        .rotate_right(shift_amount as _);
                                    (result, Some(carry))
                                }
                            }
                        }
                    }
                };

                // Write the final value to the destination register.
                emulator.cpu[self.rd() as _] = value;

                // If the 'S' bit is set, update the CPSR flags.
                if self.s() == SetFlags::Yes {
                    let cpsr = &mut emulator.cpu;

                    // 1. Set N (Negative) flag to bit 31 of the result.
                    cpsr.set_n((value >> 31) & 1 == 1);

                    // 2. Set Z (Zero) flag if the result is 0.
                    cpsr.set_z(value == 0);

                    // 3. Set C (Carry) flag only if it was produced by a shift.
                    if let Some(carry) = carry_out {
                        cpsr.set_c(carry);
                    }

                    // 4. V (Overflow) flag is unaffected by MOV.
                }
            }
            Opcode::BIC => todo!(),
            Opcode::MVN => todo!(),
        }

        Ok(())
    }
}
