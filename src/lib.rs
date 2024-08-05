//! # `yaxpeax-avnera`, a decoder for the Avnera microcontroller instruction sets
//!
//! "Avnera" is not the documented name of this instruction set, so far as i know there is no
//! published name for the instruction set. instead, it is the name of what was once a [fabless
//! semiconductor
//! company](https://investors.skyworksinc.com/news-releases/news-release-details/skyworks-acquire-smart-interface-innovator-avnera-corporation)
//! seemingly specializing in ASICs for wireless audio purposes.
//!
//! a quick search online mostly reports "Avnera AV____" parts, like "AV6201", "AV6301", "AV6302",
//! "AV7201", or "AV7301". these seem to be device names reported when USB devices are partially
//! functioning and perhaps in a programming mode.
//!
//! regardless, the instruction set in these parts is entirely undocumented. the disassembler here
//! is a result of staring at a firmware dump and thinking really hard about what might be a
//! coherent interpretation for the bytes that seem like instructions.
//!
//! that reverse engineering and corresponding note taking is [here]. i am not the first to
//! look at this architecture, both whitequark and prehistorcman have also looked at this and come
//! to very similar conclusions:
//! * [Prehistoricman's notes and IDA plugin](https://github.com/Prehistoricman/AV7300)
//! * [whitequark's notes and binja plugin](https://github.com/whitequark/binja-avnera)
//!
//! ## usage
//!
//! the fastest way to decode an Avnera instruction is through
//! [`InstDecoder::decode_slice()`]:
//! ```
//! use yaxpeax_avnera::InstDecoder;
//!
//! let inst = InstDecoder::decode_slice(&[0xb9]).unwrap();
//!
//! assert_eq!("ret", inst.to_string());
//! ```
//!
//! opcodes and operands are available on the decoded instruction, as well as its length and
//! operand count:
//! ```
//! use yaxpeax_avnera::{InstDecoder, Operand};
//!
//! let inst = InstDecoder::decode_slice(&[0x28]).unwrap();
//!
//! assert_eq!("r0 ^= r0", inst.to_string());
//! assert_eq!(inst.operand_count(), 1);
//! assert_eq!(inst.len(), 1);
//! assert_eq!(inst.operand(0).unwrap(), Operand::Register { n: 0 });
//! ```
//!
//! additionally, `yaxpeax-avnera` implements `yaxpeax-arch` traits for generic use, such as
//! [`yaxpeax_arch::LengthedInstruction`]. [`yaxpeax_arch::Arch`] is implemented by the unit struct
//! [`Avnera`].
//!
//! ## `#![no_std]`
//!
//! `yaxpeax-avnera` should support `no_std` usage, but this is entirely untested.

#![no_std]

mod display;

use yaxpeax_arch::{AddressDiff, Arch, Decoder, LengthedInstruction, Reader, StandardDecodeError};

/// a trivial struct for [`yaxpeax_arch::Arch`] to be implemented on. it's only interesting for the
/// associated type parameters.
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Avnera;

impl Arch for Avnera {
    type Address = u16;
    type Word = u8;
    type Instruction = Instruction;
    type Decoder = InstDecoder;
    type DecodeError = StandardDecodeError;
    type Operand = Operand;
}

/// an `avnera` instruction.
///
/// `avnera` instructions are not publicly documented. the structure here has been inferred from
/// staring at binaries really hard.
///
/// it seems that instructions for Avnera processors specify at most one explicit register (or
/// register pair), and may also specify an immediate memory address. with implicit registers
/// included there seem to only be zero, one, or two operands to an instruction.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Instruction {
    opcode: Opcode,
    operands: [Operand; 2],
    operand_count: u8,
    length: u8,
}

impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {
            opcode: Opcode::Scf,
            operands: [Operand::Nothing, Operand::Nothing],
            operand_count: 0,
            length: 0,
        }
    }
}

impl Instruction {
    fn reset_operands(&mut self) {
        self.operands = [Operand::Nothing, Operand::Nothing];
        self.operand_count = 0;
    }

    pub fn len(&self) -> u8 {
        self.length
    }

    pub fn operand_count(&self) -> u8 {
        self.operand_count
    }

    pub fn operand(&self, idx: u8) -> Option<Operand> {
        self.operands.get(idx as usize).cloned()
    }
}

impl LengthedInstruction for Instruction {
    type Unit = AddressDiff<<Avnera as Arch>::Address>;
    fn min_size() -> Self::Unit {
        AddressDiff::from_const(1)
    }
    fn len(&self) -> Self::Unit {
        AddressDiff::from_const(self.length as u16)
    }
}

impl yaxpeax_arch::Instruction for Instruction {
    fn well_defined(&self) -> bool { true }
}

/// an operand for an `avnera` instruction. like the instructions themselves, these are not
/// documented in any way i could find. these operands are best guesses from staring at firmware
/// binaries really hard.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum Operand {
    /// no operand in this position.
    ///
    /// reaching this as a user of `yaxpeax_avnera` is almost certainly a bug.
    /// `Instruction::operand` will return `None` rather than `Operand::Nothing`.
    Nothing,
    /// a register, either `r0` (implicit) or one of `r0..r7`
    Register { n: u8 },
    /// a register pair. seems like theoretically this could be any `rN:rN+1`, especially uncertain
    /// what happens if `N` is 7. realistically programs seem to only pick even N.
    ///
    /// as an example: `RegisterPair { n: 4 }` describes the reigster pair `r4:r5`.
    RegisterPair { n: u8 },
    /// a memory access to a 16-bit address.
    MemAbs16 { addr: u16 },
    /// a memory access through a register pair.
    ///
    /// as an example: `MemRegIndirect { n: 6 }` describes the operand `[r6:r7]`
    MemRegIndirect { n: u8 },
    /// a memory access through a register pair, with an 8-bit offset.
    ///
    /// as an example: `MemRegIndirectOffset { n: 4, offs: 0x40 }` describes the operand `[r4:r5 + 0x40]`
    MemRegIndirectOffset { n: u8, offs: u8 },
    /// a relative branch by `rel`.
    ///
    ///  the actual address branched to for an instruction at address `A` would be `A + inst.len()
    ///  + rel as u16`.
    BranchRelI8 { rel: i8 },
    /// an 8-bit immediate.
    ///
    /// the meaning of this immediate is opcode-dependent, but usually a numeric operand for a
    /// bitwise or arithmetic operation.
    ImmU8 { imm: u8 },
    /// a 16-bit immediate.
    ///
    /// the meaning of this immediate is opcode-dependent, but is likely the absolute address for a
    /// `jmp` or `call` instruction.
    ImmU16 { imm: u16 },
}

/// an avnera instruction's operation.
///
/// instruction behavior is mostly unknown. the mnemonics here are best guesses from staring really
/// hard at firmwares.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
enum Opcode {
    /// add with carry from register N into register 0
    Adc,
    /// mov from reigster N to register 0
    MovRnR0,
    /// bitwise `or` from register N into register 0
    Or,
    /// bitwise `and` from register N into register 0
    And,
    /// bitwise `xor` from register N into register 0
    Xor,
    /// rotate register N left once through carry
    Rcl,
    /// rotate register N right once through carry
    Rcr,
    /// increment register N
    Inc,
    /// decrement register N
    Dec,
    /// increment register pair `rN:rN+1`
    IncW,
    /// subtract with carry from register N into register 0
    Sbc,
    /// add from register N into register 0
    Add,
    /// dunno what this is! one of the still-unknown opcodes in the range `[58..5f]`.
    Op5xHi,
    /// "set carry flag". not sure where the carry flag is, or what instructions read or write it,
    /// but it seems like this is one!
    Scf,
    /// "clear carry flag". not sure where the carry flag is, or what instructions read or write
    /// it, but it seems like this is one!
    Ccf,
    /// seems like "toggle bit `n` in register 0"
    Bit,
    /// dunno what this is! one of the still-unknown opcodes in the range `[68..6f]`.
    Op6xHi,
    /// mov from reigster 0 to register N
    MovR0Rn,
    /// compare registers N and 0, set flags with result
    Cmp,
    /// push register N. where the stack is and what the stack pointer is are unknown.
    Push,
    /// pop register N. where the stack is and what the stack pointer is are unknown.
    Pop,
    /// branch if `z` bit is clear
    Jnz,
    /// branch if `c` bit is clear
    Jnc,
    /// conditional branch (unknown condition, opcode in range `[90..97]`). the first operand is
    /// the bit pattern selecting a yet-unknown condition.
    JccLo,
    /// branch if `z` bit is set
    Jz,
    /// branch if `c` bit is set
    Jc,
    /// conditional branch (unknown condition, opcode in range `[98..9f]`). the first operand is
    /// the bit pattern selecting a yet-unknown condition.
    JccHi,
    /// return. where the stack is and what the stack pointer is are unknown, but it seems to do
    /// the thing.
    Ret,
    /// return from interrupt handler. there may be a separate interrupt enable bit that this
    /// resets, it's not clear.
    Iret,
    /// jump to absolute 16-bit address.
    Jmp,
    /// call to absolute 16-bit address. where the stack is and what the stack pointer is are
    /// unknown.
    Call,
    /// load immediate u8 into register N
    LoadImm8,
    /// load from absolute 16-bit immediate address into register N
    LoadAbs16,
    /// store from register N into absolute 16-bit immediate address
    StoreAbs16,
    /// load from `[rM:rM+1]` into register 0
    LoadRegPair,
    /// store from register 0 into `[rM:rM+1]`
    StoreRegPair,
    /// load from `[rM:rM+1 + C]` into register 0
    LoadRegPairC,
    /// store from register 0 into `[rM:rM+1 + C]`
    StoreRegPairC,
}


/// an avnera instruction decoder.
///
/// instruction decoding is best guess from staring really hard at firmwares. it's not clear if
/// there are minor or substantial changes in the instruction set from part to part. this has been
/// written purely from staring really hard at firmwares.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstDecoder { }

impl InstDecoder {
    /// decode a slice of bytes into an instruction (or error)
    ///
    /// this is just a higher-level interface to the [`InstDecoder`] impl of
    /// [`yaxpeax_arch::Decoder`].
    pub fn decode_slice(data: &[u8]) -> Result<Instruction, <Avnera as Arch>::DecodeError> {
        use yaxpeax_arch::U8Reader;

        InstDecoder::default()
            .decode(&mut U8Reader::new(data))
    }
}

impl Default for InstDecoder {
    fn default() -> Self {
        InstDecoder { }
    }
}

impl Decoder<Avnera> for InstDecoder {
    fn decode_into<T: Reader<<Avnera as Arch>::Address, <Avnera as Arch>::Word>>(&self, inst: &mut Instruction, words: &mut T) -> Result<(), <Avnera as Arch>::DecodeError> {
        inst.length = 0;
        inst.reset_operands();
        words.mark();
        let word = words.next()?;

        fn next_u16<T: Reader<<Avnera as Arch>::Address, <Avnera as Arch>::Word>>(words: &mut T) -> Result<u16, <Avnera as Arch>::DecodeError> {
            let i_lo = words.next()? as u16;
            let i_hi = words.next()? as u16;
            Ok(i_lo | (i_hi << 8))
        }

        use Opcode::*;

        let low_bits = word & 0b111;

        *inst = match word & 0xf8 {
            0x00 => {
                Instruction::new_1op(Inc, Operand::Register { n: low_bits })
            }
            0x08 => {
                Instruction::new_1op(Adc, Operand::Register { n: low_bits })
            }
            0x10 => {
                Instruction::new_1op(MovRnR0, Operand::Register { n: low_bits })
            }
            0x18 => {
                Instruction::new_1op(Or, Operand::Register { n: low_bits })
            }
            0x20 => {
                Instruction::new_1op(And, Operand::Register { n: low_bits })
            }
            0x28 => {
                Instruction::new_1op(Xor, Operand::Register { n: low_bits })
            }
            0x30 => {
                Instruction::new_1op(Rcl, Operand::Register { n: low_bits })
            }
            0x38 => {
                Instruction::new_1op(Rcr, Operand::Register { n: low_bits })
            }
            0x40 => {
                Instruction::new_1op(Dec, Operand::Register { n: low_bits })
            }
            0x48 => {
                Instruction::new_1op(Sbc, Operand::Register { n: low_bits })
            }
            0x50 => {
                Instruction::new_1op(Add, Operand::Register { n: low_bits })
            }
            0x58 => {
                if word == 0x59 {
                    Instruction::new_0op(Scf)
                } else {
                    Instruction::new_1op(Op5xHi, Operand::ImmU8 { imm: low_bits })
                }
            }
            0x60 => {
                Instruction::new_1op(Bit, Operand::ImmU8 { imm: low_bits })
            }
            0x68 => {
                if word == 0x69 {
                    Instruction::new_0op(Ccf)
                } else {
                    Instruction::new_1op(Op6xHi, Operand::ImmU8 { imm: low_bits })
                }
            }
            0x70 => {
                Instruction::new_1op(MovR0Rn, Operand::Register { n: low_bits })
            }
            0x78 => {
                Instruction::new_1op(Cmp, Operand::Register { n: low_bits })
            }
            0x80 => {
                Instruction::new_1op(Push, Operand::Register { n: low_bits })
            }
            0x88 => {
                Instruction::new_1op(Pop, Operand::Register { n: low_bits })
            },
            0x90 => {
                let op = Operand::BranchRelI8 { rel: words.next()? as i8 };
                match low_bits {
                    0 => { Instruction::new_1op(Jnz, op) },
                    1 => { Instruction::new_1op(Jnc, op) },
                    _ => { Instruction::new_2op(JccLo, [Operand::ImmU8 { imm: low_bits }, op]) },
                }
            },
            0x98 => {
                let op = Operand::BranchRelI8 { rel: words.next()? as i8 };
                match low_bits {
                    0 => { Instruction::new_1op(Jz, op) },
                    1 => { Instruction::new_1op(Jc, op) },
                    _ => { Instruction::new_2op(JccHi, [Operand::ImmU8 { imm: low_bits }, op]) },
                }
            },
            0xb8 => {
                if word == 0xb9 {
                    Instruction::new_0op(Ret)
                } else if word == 0xba {
                    Instruction::new_0op(Iret)
                } else if word == 0xbc {
                    Instruction::new_1op(
                        Jmp,
                        Operand::ImmU16 { imm: next_u16(words)? },
                    )
                } else if word == 0xbf {
                    Instruction::new_1op(
                        Call,
                        Operand::ImmU16 { imm: next_u16(words)? },
                    )
                } else {
                    return Err(StandardDecodeError::InvalidOpcode);
                }
            },
            0xc0 => {
                Instruction::new_1op(IncW, Operand::RegisterPair { n: low_bits })
            },
            0xc8 => {
                Instruction::new_2op(StoreAbs16,
                    [
                        Operand::Register { n: low_bits },
                        Operand::MemAbs16 { addr: next_u16(words)? },
                    ])
            },
            0xd0 => {
                Instruction::new_1op(
                    StoreRegPair,
                    Operand::MemRegIndirect { n: low_bits },
                )
            }
            0xd8 => {
                Instruction::new_1op(
                    StoreRegPairC,
                    Operand::MemRegIndirectOffset { n: low_bits, offs: words.next()? },
                )
            }
            0xe0 => {
                Instruction::new_2op(LoadImm8,
                    [
                        Operand::Register { n: low_bits },
                        Operand::ImmU8 { imm: words.next()? },
                    ])
            },
            0xe8 => {
                Instruction::new_2op(LoadAbs16,
                    [
                        Operand::Register { n: low_bits },
                        Operand::MemAbs16 { addr: next_u16(words)? },
                    ])
            },
            0xf0 => {
                Instruction::new_1op(
                    LoadRegPair,
                    Operand::MemRegIndirect { n: low_bits },
                )
            }
            0xf8 => {
                Instruction::new_1op(
                    LoadRegPairC,
                    Operand::MemRegIndirectOffset { n: low_bits, offs: words.next()? },
                )
            }
            _ => {
                return Err(StandardDecodeError::InvalidOpcode);
            }
        };

        inst.length = words.offset() as u8;
        Ok(())
    }
}

impl Instruction {
    fn new_0op(opcode: Opcode) -> Self {
        Self {
            opcode,
            operands: [Operand::Nothing, Operand::Nothing],
            operand_count: 0,
            length: 0,
        }
    }

    fn new_1op(opcode: Opcode, operand: Operand) -> Self {
        Self {
            opcode,
            operands: [operand, Operand::Nothing],
            operand_count: 1,
            length: 0,
        }
    }

    fn new_2op(opcode: Opcode, operands: [Operand; 2]) -> Self {
        Self {
            opcode,
            operands,
            operand_count: 2,
            length: 0,
        }
    }
}
