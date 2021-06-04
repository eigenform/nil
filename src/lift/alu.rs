//! Common ALU operations.

use crate::block::*;
use crate::ir::*;

pub enum ShiftType { Lsl = 0b00, Lsr = 0b01, Asr = 0b10, Ror = 0b11 }
impl From<u32> for ShiftType {
    fn from(x: u32) -> Self {
        match x {
            0b00 => ShiftType::Lsl, 0b01 => ShiftType::Lsr,
            0b10 => ShiftType::Asr, 0b11 => ShiftType::Ror,
            _ => unreachable!(),
        }
    }
}

pub enum ShiftArgs {
    /// Immediate shift
    Imm { imm12: u32 },
    /// Register shift by immediate
    Reg { rm: Var, stype: u32, imm5: u32 },
    /// Register shift by register
    Rsr { rm: Var, stype: u32, rs: u32 },
}

/// Perform some barrel-shifter operation.
///
/// Returns a tuple containing the output value and output carry flag.
pub fn barrel_shift(bb: &mut BasicBlock, args: ShiftArgs) -> (Var, Var) {
    match args {
        ShiftArgs::Imm { imm12 } => {
            rot_by_imm(bb, imm12)
        },
        ShiftArgs::Reg { rm, stype, imm5 } => {
            shift_by_imm(bb, rm, stype, imm5)
        },
        _ => unimplemented!(),
    }
}

pub fn rot_by_imm(bb: &mut BasicBlock, imm12: u32) -> (Var, Var) {
    let (simm, imm8) = ((imm12 & 0xf00) >> 8, imm12 & 0xff);
    let val = imm8.rotate_right(simm * 2);
    let res = bb.constant(32, val as usize);
    let c_out = if simm == 0 {
        bb.read_flag(FlagKind::Carry)
    } else {
        let carry_bool = (val & 0x8000_0000) != 0;
        bb.constant(1, carry_bool as usize)
    };
    (res, c_out)
}

pub fn shift_by_imm(bb: &mut BasicBlock, rm: Var, stype: u32, simm: u32) 
    -> (Var, Var) {
    match ShiftType::from(stype) {
        ShiftType::Lsl => do_lsl(bb, rm, simm),
        _ => unimplemented!(),
    }

}

pub fn do_lsl(bb: &mut BasicBlock, rm: Var, simm: u32) -> (Var, Var) {
    if simm == 0 {
        (rm, bb.read_flag(FlagKind::Carry))
    } else {
        let simm_val = bb.constant(32, simm as usize);
        let (res, c_out, _) = bb.lsl32f(rm, simm_val);
        (res, c_out)
    }
}

