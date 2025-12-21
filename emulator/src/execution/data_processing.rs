use crate::{
    Emulator,
    execution::{
        ExecutableInstruction, ExecutionError, N, Z, private::Sealed,
    },
    instructions::{
        DataProcessingInstruction,
        fields::{Opcode, SetFlags},
    },
};

/// Calculates the V (signed overflow) flag for an addition.
/// Overflow occurs if two positive numbers make a negative, or two negatives make a positive.
#[inline(always)]
#[must_use]
fn v_flag_add(rn: u32, op2: u32, result: u32) -> bool {
    // A simplified way to express this is checking if the sign of the result
    // is different from the operands, but only when the operands had the same sign.
    ((rn ^ result) & (op2 ^ result) & 0x80000000) != 0
}

/// Calculates the V (signed overflow) flag for a subtraction (rn - op2).
/// Overflow occurs if a positive minus a negative gives a negative, or a negative minus a positive gives a positive.
#[inline(always)]
#[must_use]
fn v_flag_sub(rn: u32, op2: u32, result: u32) -> bool {
    // A simplified way to express this is checking if the operands had different signs,
    // and the result's sign differs from the first operand's.
    ((rn ^ op2) & (rn ^ result) & 0x80000000) != 0
}

impl DataProcessingInstruction {
    /// Execute a logical operation, `F`.
    /// This should be a binary operation, that returns the the result to be written to rd.
    /// The result of evaluating operand2 will be the result of the carry.
    /// This method is a helper for the logical operations; AND, ORR, EOR, BIC.
    ///
    /// Documention can be found on the [ARM website](https://developer.arm.com/documentation/dui0489/h/arm-and-thumb-instructions/and--orr--eor--bic--and-orn)
    fn execute_logical_op(
        &self,
        emulator: &mut Emulator,
        op: impl FnOnce(u32, u32) -> u32,
    ) -> Result<(), ExecutionError> {
        let rn_val = emulator.cpu.register(self.rn() as _);
        let (op2_val, carry) = self.op2().eval(emulator);

        let result = op(rn_val, op2_val);
        emulator.cpu.set_register(self.rd() as _, result);

        if self.s() == SetFlags::Yes {
            let cpu = &mut emulator.cpu;

            cpu.set_n(result.n());
            cpu.set_z(result.z());
            if let Some(carry) = carry {
                cpu.set_c(carry);
            }
            // V flag is unaffected for logical ops.
        }
        Ok(())
    }

    /// Execute an arithmetic operation.
    /// This method is a helper for the arithmetic operations; ADD, SUB, etc.
    ///
    /// Documentation can be found on the [ARM website](https://developer.arm.com/documentation/dui0489/h/arm-and-thumb-instructions/add--sub--rsb--adc--sbc--and-rsc)
    fn execute_arithmetic_op(
        &self,
        emulator: &mut Emulator,
        op: impl FnOnce(u32, u32) -> (u32, bool),
        v_flag_fn: impl FnOnce(u32, u32, u32) -> bool,
    ) -> Result<(), ExecutionError> {
        let rn_val = emulator.cpu[self.rn() as _];
        // For arithmetic, we only care about the value of op2, NOT the shifter carry.
        let (op2_val, _shifter_carry) = self.op2().eval(emulator);

        let (result, arithmetic_carry) = op(rn_val, op2_val); // This is the carry that matters
        emulator.cpu[self.rd() as _] = result;

        if self.s() == SetFlags::Yes {
            let cpu = &mut emulator.cpu;
            cpu.set_n(result.n());
            cpu.set_z(result.z());
            cpu.set_c(arithmetic_carry); // Use the carry from the operation
            cpu.set_v(v_flag_fn(rn_val, op2_val, result));
        }
        Ok(())
    }

    /// Generic executor for logical TEST operations (TST, TEQ).
    /// These only set flags (N, Z, C from shifter) and do NOT write to Rd.
    fn execute_test_op(
        &self,
        emulator: &mut Emulator,
        op: impl FnOnce(u32, u32) -> u32,
    ) -> Result<(), ExecutionError> {
        let rn_val = emulator.cpu[self.rn() as _];
        let (op2_val, carry) = self.op2().eval(emulator);

        let result = op(rn_val, op2_val);
        // No write to Rd!

        if self.s() == SetFlags::Yes {
            let cpu = &mut emulator.cpu;

            cpu.set_n(result.n());
            cpu.set_z(result.z());
            if let Some(carry) = carry {
                cpu.set_c(carry);
            }
        }
        Ok(())
    }

    /// Generic executor for arithmetic COMPARISON operations (CMP, CMN).
    /// These only set flags and do NOT write to Rd.
    fn execute_comparison_op(
        &self,
        emulator: &mut Emulator,
        op: impl FnOnce(u32, u32) -> (u32, bool),
        v_flag_fn: impl FnOnce(u32, u32, u32) -> bool,
    ) -> Result<(), ExecutionError> {
        let rn_val = emulator.cpu[self.rn() as _];
        let (op2_val, _) = self.op2().eval(emulator);

        let (result, carry) = op(rn_val, op2_val);
        // No write to Rd!

        if self.s() == SetFlags::Yes {
            // CMP/CMN are always flag-setting
            let cpu = &mut emulator.cpu;
            cpu.set_n(result.n());
            cpu.set_z(result.z());
            cpu.set_c(carry);
            cpu.set_v(v_flag_fn(rn_val, op2_val, result));
        }
        Ok(())
    }

    /// Generic executor for arithmetic operations WITH CARRY (ADC, SBC, RSC).
    fn execute_arithmetic_with_carry_op(
        &self,
        emulator: &mut Emulator,
        op: impl FnOnce(u32, u32, bool) -> (u32, bool),
        v_flag_fn: impl FnOnce(u32, u32, u32) -> bool,
    ) -> Result<(), ExecutionError> {
        let rn_val = emulator.cpu[self.rn() as _];
        let (op2_val, _) = self.op2().eval(emulator);
        let carry_in = emulator.cpu.c();

        let (result, carry_out) = op(rn_val, op2_val, carry_in);
        emulator.cpu[self.rd() as _] = result;

        if self.s() == SetFlags::Yes {
            let cpu = &mut emulator.cpu;
            cpu.set_n(result.n());
            cpu.set_z(result.z());
            cpu.set_c(carry_out);
            cpu.set_v(v_flag_fn(rn_val, op2_val, result));
        }
        Ok(())
    }
}

impl ExecutableInstruction for DataProcessingInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError> {
        match self.opcode() {
            // Arithmetic Opcodes
            Opcode::ADD => self.execute_arithmetic_op(
                emulator,
                |a, b| a.overflowing_add(b),
                v_flag_add,
            ),
            Opcode::SUB => self.execute_arithmetic_op(
                emulator,
                |a, b| {
                    let (result, borrow) = a.overflowing_sub(b);
                    (result, !borrow) // Invert the borrow flag to get the correct C flag
                },
                v_flag_sub,
            ),
            Opcode::RSB => self.execute_arithmetic_op(
                emulator,
                |a, b| {
                    let (result, borrow) = b.overflowing_sub(a);
                    (result, !borrow) // Invert the borrow flag
                },
                v_flag_sub,
            ),
            Opcode::ADC => self.execute_arithmetic_with_carry_op(
                emulator,
                |a, b, c| {
                    let (res1, carry1) = a.overflowing_add(b);
                    let (res2, carry2) = res1.overflowing_add(c as u32);
                    (res2, carry1 || carry2)
                },
                v_flag_add,
            ),
            Opcode::SBC => self.execute_arithmetic_with_carry_op(
                emulator,
                |a, b, c| {
                    let (res1, borrow1) = a.overflowing_sub(b);
                    // For SBC, the C flag is inverted (0=borrow, 1=no borrow). So we subtract (1-C).
                    let (res2, borrow2) =
                        res1.overflowing_sub((1 - c as u32) as u32);
                    (res2, !(borrow1 || borrow2)) // C is inverted for subtracts
                },
                v_flag_sub,
            ),
            Opcode::RSC => self.execute_arithmetic_with_carry_op(
                emulator,
                |a, b, c| {
                    let (res1, borrow1) = b.overflowing_sub(a);
                    let (res2, borrow2) =
                        res1.overflowing_sub((1 - c as u32) as u32);
                    (res2, !(borrow1 || borrow2))
                },
                v_flag_sub,
            ),

            // Logical Opcodes
            Opcode::AND => self.execute_logical_op(emulator, |a, b| a & b),
            Opcode::ORR => self.execute_logical_op(emulator, |a, b| a | b),
            Opcode::EOR => self.execute_logical_op(emulator, |a, b| a ^ b),
            Opcode::BIC => {
                self.execute_logical_op(emulator, |a, b| a & !b)
            } // Bit Clear

            // Single Operand / Special
            Opcode::MOV => {
                let (value, carry_out) = self.op2().eval(emulator);

                // Write the final value to the destination register.
                emulator.cpu.set_register(self.rd() as _, value);

                // If the 'S' bit is set, update the CPSR flags.
                if self.s() == SetFlags::Yes {
                    let cpsr = &mut emulator.cpu;

                    // 1. Set N (Negative) flag to bit 31 of the result.
                    cpsr.set_n(value.n());

                    // 2. Set Z (Zero) flag if the result is 0.
                    cpsr.set_z(value.z());

                    // 3. Set C (Carry) flag only if it was produced by a shift.
                    if let Some(carry) = carry_out {
                        cpsr.set_c(carry);
                    }

                    // 4. V (Overflow) flag is unaffected by MOV.
                }

                Ok(())
            }
            Opcode::MVN => {
                let (op2_val, carry) = self.op2().eval(emulator);
                let result = !op2_val;
                emulator.cpu[self.rd() as _] = result;
                if self.s() == SetFlags::Yes {
                    let cpu = &mut emulator.cpu;
                    cpu.set_n(result.n());
                    cpu.set_z(result.z());
                    if let Some(carry) = carry {
                        cpu.set_c(carry);
                    }
                }
                Ok(())
            }

            // Logical Test Opcodes (Non-Writing)
            Opcode::TST => self.execute_test_op(emulator, |a, b| a & b),
            Opcode::TEQ => self.execute_test_op(emulator, |a, b| a ^ b),

            // Arithmetic Comparison Opcodes (Non-Writing)
            Opcode::CMP => self.execute_comparison_op(
                emulator,
                |a, b| {
                    let (result, borrow) = a.overflowing_sub(b);
                    (result, !borrow) // Invert the borrow flag
                },
                v_flag_sub,
            ),
            Opcode::CMN => self.execute_comparison_op(
                emulator,
                |a, b| a.overflowing_add(b),
                v_flag_add,
            ),
        }
    }
}

impl Sealed for DataProcessingInstruction {}
