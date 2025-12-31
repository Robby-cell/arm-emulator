use crate::{
    Emulator,
    execution::{
        ExecutableInstruction, ExecutionError, N, Z, private::Sealed,
    },
    instructions::{
        MultiplyInstruction, MultiplyLongInstruction,
        fields::{AccumulateFlag, SetFlags, SignedFlag},
    },
};

impl ExecutableInstruction for MultiplyInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError> {
        let rm = emulator.cpu[self.rm() as _];
        let rs = emulator.cpu[self.rs() as _];

        // 1. Calculate Base Product (Rm * Rs)
        let product = rm.wrapping_mul(rs);

        // 2. Handle Accumulate (MLA: + Rn)
        let result = if self.a() == AccumulateFlag::Accumulate {
            let rn = emulator.cpu[self.rn() as _];
            product.wrapping_add(rn)
        } else {
            product
        };

        // 3. Write Result
        emulator.cpu[self.rd() as _] = result;

        // 4. Set Flags (N, Z only. C/V usually unaffected/undefined in v7)
        if self.s() == SetFlags::Yes {
            emulator.cpu.set_n(result.n());
            emulator.cpu.set_z(result.z());
        }

        Ok(())
    }
}

impl ExecutableInstruction for MultiplyLongInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError> {
        let rm_val = emulator.cpu[self.rm() as _];
        let rs_val = emulator.cpu[self.rs() as _];

        // 1. Perform 64-bit Multiplication
        let result_64: u64 = match self.u() {
            SignedFlag::Signed => {
                let rm_i = rm_val as i32 as i64;
                let rs_i = rs_val as i32 as i64;
                (rm_i * rs_i) as u64 // Cast back to bits
            }
            SignedFlag::Unsigned => (rm_val as u64) * (rs_val as u64),
        };

        // 2. Handle Accumulate (UMLAL/SMLAL: + Existing 64-bit value)
        let final_result = if self.a() == AccumulateFlag::Accumulate {
            let rd_lo = emulator.cpu[self.rd_lo() as _] as u64;
            let rd_hi = emulator.cpu[self.rd_hi() as _] as u64;
            let current_val = (rd_hi << 32) | rd_lo;
            result_64.wrapping_add(current_val)
        } else {
            result_64
        };

        // 3. Write Result (Split back to 32-bit regs)
        emulator.cpu[self.rd_lo() as _] =
            (final_result & 0xFFFFFFFF) as u32;
        emulator.cpu[self.rd_hi() as _] = (final_result >> 32) as u32;

        // 4. Set Flags (N, Z)
        if self.s() == SetFlags::Yes {
            // N is bit 63 of result
            emulator.cpu.set_n((final_result >> 63) == 1);
            emulator.cpu.set_z(final_result == 0);
        }

        Ok(())
    }
}

impl Sealed for MultiplyInstruction {}
impl Sealed for MultiplyLongInstruction {}
