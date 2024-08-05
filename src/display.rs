use core::fmt;

use crate::{Instruction, Opcode, Operand};

impl fmt::Debug for crate::Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <crate::Operand as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for crate::Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::Operand::*;
        match self {
            Nothing => { Ok(()) },
            Register { n } => {
                write!(f, "r{}", n)
            }
            RegisterPair { n } => {
                write!(f, "r{}:r{}", n, n + 1)
            }
            MemAbs16 { addr } => {
                write!(f, "[0x{:04x}]", addr)
            }
            MemRegIndirect { n } => {
                write!(f, "[r{}:r{}]", n, n + 1)
            }
            MemRegIndirectOffset { n, offs } => {
                write!(f, "[r{}:r{} + 0x{:x}]", n, n + 1, offs)
            }
            BranchRelI8 { rel } => {
                if rel < &0 {
                    write!(f, "$-0x{:x}", rel)
                } else {
                    write!(f, "$+0x{:x}", rel)
                }
            }
            ImmU8 { imm } => {
                write!(f, "0x{:02x}", imm)
            }
            ImmU16 { imm } => {
                write!(f, "0x{:04x}", imm)
            }
        }
    }
}

impl fmt::Debug for crate::Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <crate::Opcode as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for crate::Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::Opcode::*;
        match self {
            Adc => { f.write_str("adc") },
            MovRnR0 => { f.write_str("movrnr0") },
            Or => { f.write_str("or") },
            And => { f.write_str("and") },
            Xor => { f.write_str("xor") },
            Rcl => { f.write_str("rcl") },
            Rcr => { f.write_str("rcr") },
            Inc => { f.write_str("inc") },
            IncW => { f.write_str("incw") },
            Dec => { f.write_str("dec") },
            Sbc => { f.write_str("sbc") },
            Add => { f.write_str("add") },
            Op5xHi => { f.write_str("op5xhi") },
            Scf => { f.write_str("scf") },
            Ccf => { f.write_str("ccf") },
            Bit => { f.write_str("bit") },
            Op6xHi => { f.write_str("op6xhi") },
            MovR0Rn => { f.write_str("movr0rn") },
            Cmp => { f.write_str("cmp") },
            Push => { f.write_str("push") },
            Pop => { f.write_str("pop") },
            Jz => { f.write_str("jz") },
            Jc => { f.write_str("jc") },
            JccLo => { f.write_str("jcclo") },
            Jnz => { f.write_str("jnz") },
            Jnc => { f.write_str("jnc") },
            JccHi => { f.write_str("jcchi") },
            Ret => { f.write_str("ret") },
            Iret => { f.write_str("iret") },
            Jmp => { f.write_str("jmp") },
            Call => { f.write_str("call") },
            LoadImm8 => { f.write_str("loadimm8") },
            LoadAbs16 => { f.write_str("loadabs16") },
            StoreAbs16 => { f.write_str("storeabs16") },
            LoadRegPair => { f.write_str("loadregpair") },
            StoreRegPair => { f.write_str("storeregpair") },
            LoadRegPairC => { f.write_str("loadregpairc") },
            StoreRegPairC => { f.write_str("storeregpairc") },
        }
    }
}


impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.opcode {
            Opcode::Ret => {
                f.write_str("ret")
            },
            Opcode::Iret => {
                f.write_str("iret")
            },
            Opcode::Jnz => {
                write!(f, "jnz {}", self.operands[0])
            },
            Opcode::Jnc => {
                write!(f, "jnc {}", self.operands[0])
            },
            Opcode::Jz => {
                write!(f, "jz {}", self.operands[0])
            },
            Opcode::Jc => {
                write!(f, "jc {}", self.operands[0])
            },
            Opcode::JccLo => {
                if let Operand::ImmU8 { imm } = self.operands[0] {
                    write!(f, "jcc.lo.{:x} {}", imm, self.operands[1])
                } else {
                    unreachable!()
                }
            }
            Opcode::JccHi => {
                if let Operand::ImmU8 { imm } = self.operands[0] {
                    write!(f, "jcc.hi.{:x} {}", imm, self.operands[1])
                } else {
                    unreachable!()
                }
            }
            Opcode::Adc => {
                write!(f, "adc r0, {}", self.operands[0])
            },
            Opcode::MovRnR0 => {
                write!(f, "r0 <- {}", self.operands[0])
            },
            Opcode::Or => {
                write!(f, "r0 |= {}", self.operands[0])
            },
            Opcode::And => {
                write!(f, "r0 &= {}", self.operands[0])
            },
            Opcode::Xor => {
                write!(f, "r0 ^= {}", self.operands[0])
            },
            Opcode::Rcl => {
                write!(f, "rcl {}", self.operands[0])
            },
            Opcode::Rcr => {
                write!(f, "rcr {}", self.operands[0])
            },
            Opcode::Inc => {
                write!(f, "inc {}", self.operands[0])
            },
            Opcode::IncW => {
                write!(f, "incw {}", self.operands[0])
            },
            Opcode::Dec => {
                write!(f, "dec {}", self.operands[0])
            },
            Opcode::Sbc => {
                write!(f, "sbc r0, {}", self.operands[0])
            },
            Opcode::Add => {
                write!(f, "r0 += {}", self.operands[0])
            },
            Opcode::Op5xHi => {
                write!(f, "op5xhi {}", self.operands[0])
            },
            Opcode::Scf => {
                write!(f, "scf")
            },
            Opcode::Ccf => {
                write!(f, "ccf")
            },
            Opcode::Bit => {
                write!(f, "bit r0, {}", self.operands[0])
            },
            Opcode::Op6xHi => {
                write!(f, "op6xhi {}", self.operands[0])
            },
            Opcode::MovR0Rn => {
                write!(f, "{} <- r0", self.operands[0])
            },
            Opcode::Cmp => {
                write!(f, "cmp r0, {}", self.operands[0])
            },
            Opcode::Push => {
                write!(f, "push {}", self.operands[0])
            },
            Opcode::Pop => {
                write!(f, "pop {}", self.operands[0])
            },
            Opcode::Jmp => {
                write!(f, "jmp {}", self.operands[0])
            },
            Opcode::Call => {
                write!(f, "call {}", self.operands[0])
            },
            Opcode::LoadImm8 => {
                write!(f, "{} <- {}", self.operands[0], self.operands[1])
            }
            Opcode::LoadAbs16 => {
                write!(f, "{} <- {}", self.operands[0], self.operands[1])
            }
            Opcode::StoreAbs16 => {
                write!(f, "{} <- {}", self.operands[1], self.operands[0])
            }
            Opcode::LoadRegPair => {
                write!(f, "r0 <- {}", self.operands[0])
            }
            Opcode::StoreRegPair => {
                write!(f, "{} <- r0", self.operands[0])
            }
            Opcode::LoadRegPairC => {
                write!(f, "r0 <- {}", self.operands[0])
            }
            Opcode::StoreRegPairC => {
                write!(f, "{} <- r0", self.operands[0])
            }
        }
    }
}
