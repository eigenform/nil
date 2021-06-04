
use crate::lift::arm::bits::*;
use crate::lift::alu::*;
use crate::ir::*;
use crate::block::*;
use crate::guest::Cond;

pub fn sub_imm(bb: &mut BasicBlock, op: DpImmBits) {
    assert_eq!(Cond::from(op.cond()), Cond::AL);
    let (imm, _) = barrel_shift(bb, ShiftArgs::Imm { imm12: op.imm12() });

    let rn = if op.rn() == 15 {
        bb.constant(32, bb.read_exec_pc() as usize)
    } else {
        bb.read_reg(op.rn())
    };

    let (res, c, v) = bb.sub32f(rn, imm);

    if op.rd() == 15 {
        unimplemented!();
    } else {
        if op.s() {
            let n = bb.is_negative(res);
            let z = bb.is_zero(res);
            bb.write_flag(FlagKind::Negative, n);
            bb.write_flag(FlagKind::Zero, z);
            bb.write_flag(FlagKind::Carry, c);
            bb.write_flag(FlagKind::Overflow, v);
        }
        bb.write_reg(op.rd(), res);
    }
}

pub fn mov_imm(bb: &mut BasicBlock, op: MovImmBits) {
    assert_eq!(Cond::from(op.cond()), Cond::AL);
    let (imm, c_out) = barrel_shift(bb, ShiftArgs::Imm { imm12: op.imm12() });

    if op.rd() == 15 {
        unimplemented!();
    } else {
        bb.write_reg(op.rd(), imm);
        if op.s() {
            let n = bb.is_negative(imm);
            let z = bb.is_zero(imm);
            bb.write_flag(FlagKind::Negative, n);
            bb.write_flag(FlagKind::Zero, z);
            bb.write_flag(FlagKind::Carry, c_out);
        }
    }
}

pub fn mov_reg(bb: &mut BasicBlock, op: MovRegBits) {
    assert_eq!(Cond::from(op.cond()), Cond::AL);
    let rm = if op.rm() == 15 {
        bb.constant(32, bb.read_exec_pc() as usize)
    } else {
        bb.read_reg(op.rm())
    };
    let (res, c) = barrel_shift(bb, ShiftArgs::Reg {
        rm, stype: op.stype(), imm5: op.imm5()
    });

    if op.rd() == 15 {
        unimplemented!();
    } else {
        bb.write_reg(op.rd(), res);
        if op.s() {
            let n = bb.is_negative(res);
            let z = bb.is_zero(res);
            bb.write_flag(FlagKind::Negative, n);
            bb.write_flag(FlagKind::Zero, z);
            bb.write_flag(FlagKind::Carry, c);
        }
    }
}

pub fn cmp_imm(bb: &mut BasicBlock, op: DpTestImmBits) {
    assert_eq!(Cond::from(op.cond()), Cond::AL);
    let rn = bb.read_reg(op.rn());
    let (imm, _) = barrel_shift(bb, ShiftArgs::Imm { imm12: op.imm12() });

    let (res, c, v) = bb.sub32f(rn, imm);
    let n = bb.is_negative(res);
    let z = bb.is_zero(res);
    bb.write_flag(FlagKind::Negative, n);
    bb.write_flag(FlagKind::Zero, z);
    bb.write_flag(FlagKind::Carry, c);
    bb.write_flag(FlagKind::Overflow, v);
}


