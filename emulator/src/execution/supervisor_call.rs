use crate::{
    Emulator,
    cpu::{
        ExitStatus,
        registers::{R0, R7},
    },
    execution::{ExecutableInstruction, ExecutionError, private::Sealed},
    instructions::SupervisorCallInstruction,
};

impl ExecutableInstruction for SupervisorCallInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError> {
        let syscall_number = emulator.cpu.register(R7);
        match syscall_number {
            1 => {
                let exit_code = emulator.cpu.register(R0) as i32;

                tracing::info!(
                    "SVC: Program exited with code {}",
                    exit_code
                );

                emulator.cpu.set_exit(ExitStatus { exit_code });

                Ok(())
            }

            _ => {
                tracing::warn!(
                    "Unimplemented syscall number: {}",
                    syscall_number
                );

                Ok(())
            }
        }
    }
}

impl Sealed for SupervisorCallInstruction {}
