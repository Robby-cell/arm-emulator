use crate::{Emulator, ExecutionError};

mod branch;
mod data_processing;

#[cfg(test)]
mod tests;

pub trait ExecutableInstruction {
    fn execute_with(
        &self,
        emulator: &mut Emulator,
    ) -> Result<(), ExecutionError>;
}
