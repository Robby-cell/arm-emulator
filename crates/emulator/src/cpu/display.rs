use std::fmt;

use crate::cpu::{Cpu, CpuFlags};

impl fmt::Debug for CpuFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ctx = f.debug_struct("CpuFlags");

        ctx.field("N", &(*self & Cpu::N_FLAG != CpuFlags(0)));
        ctx.field("Z", &(*self & Cpu::Z_FLAG != CpuFlags(0)));
        ctx.field("C", &(*self & Cpu::C_FLAG != CpuFlags(0)));
        ctx.field("V", &(*self & Cpu::V_FLAG != CpuFlags(0)));

        ctx.finish()
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ctx = f.debug_struct("Cpu");
        let indexes = {
            let mut indexes: [usize; 16] = [0; _];
            for (i, v) in indexes.iter_mut().enumerate() {
                *v = i;
            }
            indexes
        };
        for (r, i) in self.registers.iter().zip(indexes.iter()) {
            ctx.field(format!("R{i}").as_str(), &r);
        }

        ctx.field("cpsr", &self.cpsr);

        ctx.finish()
    }
}
