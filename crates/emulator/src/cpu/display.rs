use std::fmt;

use super::Cpu;

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

        ctx.field("N", &(self.cpsr & Cpu::N_FLAG != 0));
        ctx.field("Z", &(self.cpsr & Cpu::Z_FLAG != 0));
        ctx.field("C", &(self.cpsr & Cpu::C_FLAG != 0));
        ctx.field("V", &(self.cpsr & Cpu::V_FLAG != 0));

        ctx.finish()
    }
}
