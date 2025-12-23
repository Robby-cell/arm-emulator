use crate::{
    Emulator,
    execution::{ExecutableInstruction, ExecutionError, private::Sealed},
    instructions::{
        BlockDataTransferInstruction,
        fields::{IndexFlag, LoadStoreFlag, UpDownFlag, WriteBackFlag},
    },
};

impl ExecutableInstruction for BlockDataTransferInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError> {
        let rn = self.rn();
        let base_address = emulator.cpu.register(rn as u8);
        let register_list = u16::from(self.register_list());

        let num_registers = register_list.count_ones();
        let total_bytes = num_registers * 4;

        // Calculate the starting address for the transfer.
        // ARM LDM/STM instructions always transfer the lowest register index
        // to the lowest memory address. We normalize all addressing modes
        // (IA, IB, DA, DB) to find that lowest start address.
        let start_address = match (self.u(), self.p()) {
            // Increment After (IA) - Default for POP
            (UpDownFlag::Add, IndexFlag::Post) => base_address,

            // Increment Before (IB)
            (UpDownFlag::Add, IndexFlag::Pre) => {
                base_address.wrapping_add(4)
            }

            // Decrement After (DA)
            (UpDownFlag::Sub, IndexFlag::Post) => {
                base_address.wrapping_sub(total_bytes).wrapping_add(4)
            }

            // Decrement Before (DB) - Default for PUSH
            (UpDownFlag::Sub, IndexFlag::Pre) => {
                base_address.wrapping_sub(total_bytes)
            }
        };

        let mut current_address = start_address;
        let is_load = self.l() == LoadStoreFlag::Load;

        // Iterate through registers R0-R15
        if is_load {
            for i in 0..16 {
                if (register_list >> i) & 1 == 1 {
                    let value = emulator.read32(current_address)?;
                    emulator.cpu.set_register(i, value);

                    current_address = current_address.wrapping_add(4);
                }
            }
        } else {
            for i in 0..16 {
                if (register_list >> i) & 1 == 1 {
                    let value = emulator.cpu.register(i);
                    emulator.write32(current_address, value)?;

                    current_address = current_address.wrapping_add(4);
                }
            }
        }
        // ^^^ == VVV
        // for i in 0..16 {
        //     // Check if the bit for register 'i' is set
        //     if (register_list >> i) & 1 == 1 {
        //         if is_load {
        //             let value = emulator.read32(current_address)?;
        //             emulator.cpu.set_register(i, value);
        //         } else {
        //             let value = emulator.cpu.register(i);
        //             emulator.write32(current_address, value)?;
        //         }
        //         current_address = current_address.wrapping_add(4);
        //     }
        // }

        // Handle Write-back (update base register)
        if self.w() == WriteBackFlag::Write {
            let new_base = match self.u() {
                UpDownFlag::Add => base_address.wrapping_add(total_bytes),
                UpDownFlag::Sub => base_address.wrapping_sub(total_bytes),
            };
            emulator.cpu.set_register(rn as u8, new_base);
        }

        Ok(())
    }
}

impl Sealed for BlockDataTransferInstruction {}
