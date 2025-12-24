use crate::{
    Emulator,
    execution::{ExecutableInstruction, ExecutionError, private::Sealed},
    instructions::BranchExchangeInstruction,
};

impl ExecutableInstruction for BranchExchangeInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError> {
        // Read the target address from the specified register (Rm)
        let rm = self.rm();
        let target_address = emulator.cpu.register(rm as u8);

        // Handle Thumb bit (Bit 0)
        // If bit 0 is 1, the processor should switch to Thumb state.
        // If bit 0 is 0, the address is aligned (ARM state).
        // For this emulator (Assuming ARMv7-A ARM state only for now):
        // We mask out the bottom bit to ensure 4-byte alignment.
        // If we want to implement Thumb later, we would check `target_address & 1`
        // and update `emulator.cpu.cpsr` T-flag accordingly.

        let aligned_address = target_address & !1;

        tracing::trace!(
            "BX: Branching to register {:?} = {:#X}",
            rm,
            aligned_address
        );

        // Update PC
        emulator.cpu.set_pc(aligned_address);

        Ok(())
    }
}

impl Sealed for BranchExchangeInstruction {}
