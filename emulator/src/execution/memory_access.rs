use crate::{
    Emulator, ExecutionError,
    execution::{ExecutableInstruction, private::Sealed},
    instructions::{
        MemoryAccessInstruction, ShiftedRegisterOffset,
        fields::{
            ByteWordFlag, IndexFlag, LoadStoreFlag, OffsetType, ShiftType,
            UpDownFlag, WriteBackFlag,
        },
    },
};

impl MemoryAccessInstruction {
    fn calculate_offset(&self, emulator: &Emulator) -> u32 {
        // Use a match statement for clarity and correctness.
        match self.i() {
            OffsetType::Immediate => {
                // When i=0, the `offset` field in your bitfield struct
                // directly represents the 12-bit immediate value.
                self.offset().into()
            }
            OffsetType::Register => {
                // When i=1, we must reinterpret the same 12 bits
                // according to the shifted register format.
                let shifted_offset_data =
                    ShiftedRegisterOffset::from(u16::from(self.offset()));

                let rm_val = emulator.cpu[shifted_offset_data.rm() as _];
                let shift_amount: u32 =
                    shifted_offset_data.shift_amount().into();

                if shift_amount == 0 {
                    return match shifted_offset_data.shift_type() {
                        ShiftType::LSL => rm_val,
                        ShiftType::LSR => 0,
                        ShiftType::ASR => (rm_val as i32 >> 31) as u32,
                        ShiftType::ROR => {
                            let old_c =
                                if emulator.cpu.c() { 1 << 31 } else { 0 };
                            (rm_val >> 1) | old_c
                        }
                    };
                }

                match shifted_offset_data.shift_type() {
                    ShiftType::LSL => rm_val << shift_amount,
                    ShiftType::LSR => rm_val >> shift_amount,
                    ShiftType::ASR => {
                        ((rm_val as i32) >> shift_amount) as u32
                    }
                    ShiftType::ROR => rm_val.rotate_right(shift_amount),
                }
            }
        }
    }
}

impl ExecutableInstruction for MemoryAccessInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError> {
        let offset = self.calculate_offset(emulator);
        let base_address = emulator.cpu[self.rn() as _];

        // Refactored address calculation to be more DRY.
        let new_rn_val = match self.u() {
            UpDownFlag::Add => {
                tracing::trace!("Up flag: Adding offset: {offset:#X}");
                base_address.wrapping_add(offset)
            }
            UpDownFlag::Sub => {
                tracing::trace!(
                    "Down flag: Subtracting offset: {offset:X}"
                );
                base_address.wrapping_sub(offset)
            }
        };

        let memory_address = match self.p() {
            IndexFlag::Pre => {
                tracing::trace!("Pre-indexed");
                new_rn_val
            }
            IndexFlag::Post => {
                tracing::trace!("Post-indexed");
                base_address
            }
        };
        tracing::trace!("Using address: {memory_address:#X}");

        // Load or Store data.
        match self.l() {
            LoadStoreFlag::Load => {
                tracing::trace!("Load flag.");
                let data = if self.b() == ByteWordFlag::Byte {
                    unimplemented!("LDRB not implemented");
                } else {
                    emulator.read32(memory_address)?
                };
                emulator.cpu[self.rd() as _] = data;
            }
            LoadStoreFlag::Store => {
                tracing::trace!("Store flag.");
                let data = emulator.cpu[self.rd() as _];
                if self.b() == ByteWordFlag::Byte {
                    unimplemented!("STRB not implemented");
                } else {
                    emulator.write32(memory_address, data)?;
                }
            }
        }

        // Simplified writeback condition.
        if self.p() == IndexFlag::Post || self.w() == WriteBackFlag::Write
        {
            tracing::trace!(
                "Updating RN ({:#X}) to {:#X}",
                emulator.cpu[self.rn() as _],
                new_rn_val
            );
            emulator.cpu[self.rn() as _] = new_rn_val;
        }

        Ok(())
    }
}

impl Sealed for MemoryAccessInstruction {}
