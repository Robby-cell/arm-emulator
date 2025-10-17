use crate::{
    Emulator, ExecutionError,
    execution::{ExecutableInstruction, N, Z},
    instructions::{
        DataProcessingInstruction,
        fields::{Opcode, SetFlags},
    },
};

impl DataProcessingInstruction {
    /// Execute a logical operation, `F`.
    /// This should be a binary operation, that returns the the result to be written to rd.
    /// The result of evaluating operand2 will be the result of the carry.
    /// This method is a helper for the logical operations; AND, ORR, EOR, BIC.
    ///
    /// Documention can be found on the [ARM website](https://developer.arm.com/documentation/dui0489/h/arm-and-thumb-instructions/and--orr--eor--bic--and-orn)
    fn execute_logical_op<F>(
        &self,
        emulator: &mut Emulator,
        op: F,
    ) -> Result<(), ExecutionError>
    where
        // The operation takes two u32s and returns a u32 result.
        F: FnOnce(u32, u32) -> u32,
    {
        let rn_val = emulator.cpu.register(self.rn() as _);
        let (op2_val, carry) = self.op2().eval(emulator);

        let result = op(rn_val, op2_val);
        emulator.cpu.set_register(self.rd() as _, result);

        if self.s() == SetFlags::Yes {
            let cpu = &mut emulator.cpu;

            cpu.set_n(result.n());
            cpu.set_z(result.z());
            cpu.set_c(carry.unwrap_or_else(|| cpu.c()));
            // V flag is unaffected for logical ops.
        }
        Ok(())
    }

    /// Execute an arithmetic operation.
    /// This method is a helper for the arithmetic operations; ADD, SUB, etc.
    ///
    /// Documentation can be found on the [ARM website](https://developer.arm.com/documentation/dui0489/h/arm-and-thumb-instructions/add--sub--rsb--adc--sbc--and-rsc)
    fn execute_arithmetic_op<F>(
        &self,
        emulator: &mut Emulator,
        op: F,
    ) -> Result<(), ExecutionError>
    where
        // The op must return the result and the unsigned carry (like overflowing_add).
        F: FnOnce(u32, u32) -> (u32, bool),
    {
        let rn_val = emulator.cpu.register(self.rn() as _);
        // For arithmetic, we only care about the value of op2, not the carry.
        let (op2_val, _) = self.op2().eval(emulator);

        let (result, carry) = op(rn_val, op2_val);
        emulator.cpu.set_register(self.rd() as _, result);

        if self.s() == SetFlags::Yes {
            let cpu = &mut emulator.cpu;

            cpu.set_n(result.n());
            cpu.set_z(result.z());
            cpu.set_c(carry);

            // V (signed overflow) is tricky. It occurs if the sign of both operands
            // is the same, but the sign of the result is different.
            let op1_sign = (rn_val >> 31) & 1;
            let op2_sign = (op2_val >> 31) & 1;
            let result_sign = (result >> 31) & 1;
            let overflow =
                (op1_sign == op2_sign) && (op1_sign != result_sign);
            cpu.set_v(overflow);
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
            // --- Arithmetic Opcodes ---
            Opcode::ADD => self.execute_arithmetic_op(emulator, |a, b| {
                a.overflowing_add(b)
            }),
            Opcode::SUB => self.execute_arithmetic_op(emulator, |a, b| {
                (a.overflowing_sub(b).0, a >= b)
            }),
            Opcode::RSB => self.execute_arithmetic_op(emulator, |a, b| {
                (b.overflowing_sub(a).0, b >= a)
            }), // Reverse Subtract

            // --- Logical Opcodes ---
            Opcode::AND => self.execute_logical_op(emulator, |a, b| a & b),
            Opcode::ORR => self.execute_logical_op(emulator, |a, b| a | b),
            Opcode::EOR => self.execute_logical_op(emulator, |a, b| a ^ b),
            Opcode::BIC => {
                self.execute_logical_op(emulator, |a, b| a & !b)
            } // Bit Clear

            // --- Other Opcodes ---
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
            Opcode::MVN => todo!(),
            Opcode::CMP => todo!(),
            Opcode::TST => todo!(),

            // -- Others --
            Opcode::ADC => todo!(),
            Opcode::SBC => todo!(),
            Opcode::RSC => todo!(),
            Opcode::TEQ => todo!(),
            Opcode::CMN => todo!(),
        }
    }
}
