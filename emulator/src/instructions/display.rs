use std::fmt;

use super::{
    BlockDataTransferInstruction, BranchExchangeInstruction,
    BranchInstruction, BreakpointInstruction, DataProcessingInstruction,
    Instruction, MemoryAccessInstruction, MultiplyInstruction,
    MultiplyLongInstruction, SupervisorCallInstruction,
};

impl fmt::Display for DataProcessingInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for MemoryAccessInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for BranchInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for BranchExchangeInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for BlockDataTransferInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for SupervisorCallInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for MultiplyInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for MultiplyLongInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for BreakpointInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataProcessing(inst) => write!(f, "{inst}"),
            Self::MemoryAccess(inst) => write!(f, "{inst}"),
            Self::Branch(inst) => write!(f, "{inst}"),
            Self::BranchExchange(inst) => write!(f, "{inst}"),
            Self::BlockDataTransfer(inst) => write!(f, "{inst}"),
            Self::SupervisorCall(inst) => write!(f, "{inst}"),
            Self::Multiply(inst) => write!(f, "{inst}"),
            Self::MultiplyLong(inst) => write!(f, "{inst}"),
            Self::Breakpoint(inst) => write!(f, "{inst}"),
        }
    }
}
