use crate::{
    execution::{ExecutableInstruction, ExecutionError, private::Sealed},
    instructions::{BranchInstruction, fields::LinkFlag},
};

impl ExecutableInstruction for BranchInstruction {
    fn execute_with(
        &self,
        emulator: &mut crate::Emulator,
    ) -> Result<(), ExecutionError> {
        // Step 1: Handle the "Link" operation for BL instructions.
        if self.l() == LinkFlag::Yes {
            let return_address = emulator.cpu.pc().wrapping_add(4);
            emulator.cpu.set_lr(return_address);
        }

        // Manually Sign-Extend the Offset ---
        // Without this, it will treat it as a positive number.
        // So an offset of -1 would be read as a huge number,
        // then it would just be shifted.

        // 1. Get the raw unsigned 24-bit value from the bitfield.
        //    For 0xEAFFFFBE, this will be the number 0xFFFFBE.
        let raw_offset: u32 = self.offset().into();

        // 2. Check the sign bit (bit 23) of the 24-bit value.
        let is_negative = (raw_offset & 0x00800000) != 0;

        // 3. Manually create the correct 32-bit signed integer.
        //    This is the crucial step that was missing.
        let signed_offset = if is_negative {
            // If it's negative, fill the upper 8 bits with 1s.
            (raw_offset | 0xFF000000) as i32
        } else {
            // Otherwise, it's just the positive value.
            raw_offset as i32
        };

        // The rest of the execution logic uses the CORRECT signed offset

        let pc = emulator.cpu.pc();
        let base_address = pc.wrapping_add(8);

        // Scale the *correctly signed* offset to a byte offset.
        // Every offset is actually multiplied by 4/shifted left by 2
        let byte_offset = signed_offset << 2;

        // Perform the calculation with the correct values.
        let target_address =
            (base_address as i64 + byte_offset as i64) as u32;
        // base_address.wrapping_add(byte_offset as _);

        emulator.cpu.set_pc(target_address);

        Ok(())
    }
}

impl Sealed for BranchInstruction {}
