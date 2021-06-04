use crate::lift::arm::bits::*;
use crate::ir::*;
use crate::block::*;
use crate::guest::Cond;

/// Compute an address (literal addressing mode).
pub fn amode_lit(pc: u32, imm: u32, p: bool, u: bool) -> u32 {
    match (p, u) {
        (true, true) => pc.wrapping_add(imm),
        (true, false) => pc.wrapping_sub(imm),
        _ => pc,
    }
}

pub fn amode(bb: &mut BasicBlock, 
    rn: Var, imm: Var, u: bool, p: bool, w: bool) -> (Var, Var) {
    let res = if u { bb.add32(rn, imm) } else { bb.sub32(rn, imm) };
    match (p, w) {
        (false, false) => (rn, res),
        (true, false) => (res, rn),
        (true, true) => (res, res),
        (false, true) => panic!("Unsupported addressing mode"),
    }
}

pub fn ldr_imm(bb: &mut BasicBlock, op: LsImmBits) {
    assert_eq!(Cond::from(op.cond()), Cond::AL);
    let res = if op.rn() == 15 {
        let addr_val = amode_lit(
            bb.read_exec_pc(), op.imm12(), op.p(), op.u()
        );
        let addr = bb.constant(32, addr_val as usize);
        bb.load32(addr)
    } else {
        let rn = bb.read_reg(op.rn());
        let imm = bb.constant(32, op.imm12() as usize);
        let (addr, wb_addr) = amode(bb, rn, imm, op.u(), op.p(), op.w());
        bb.write_reg(op.rn(), wb_addr);
        bb.load32(addr)
    };
        
    if op.rt() == 15 {
        panic!("");
    } else {
        bb.write_reg(op.rt(), res);
    }
}

pub fn str_imm(bb: &mut BasicBlock, op: LsImmBits) {
    assert_eq!(Cond::from(op.cond()), Cond::AL);
    let rt = bb.read_reg(op.rt());
    let rn = bb.read_reg(op.rn());
    let imm = bb.constant(32, op.imm12() as usize);

    let (addr, wb_addr) = amode(bb, rn, imm, op.u(), op.p(), op.w());
    bb.write_reg(op.rn(), wb_addr);
    bb.store32(addr, rt);
}

pub fn stmdb(bb: &mut BasicBlock, op: LsMultiBits) {
    assert_eq!(Cond::from(op.cond()), Cond::AL);
    assert_ne!(op.rn(), 15);

    let reglist = op.register_list();
    let num_regs = reglist.count_ones() as usize;
    let addr_off = bb.constant(32, num_regs * 4);
    let rn_val = bb.read_reg(op.rn());
    let base_addr = bb.sub32(rn_val, addr_off);
    let wb_addr = base_addr;

    stm_common(bb, reglist, op.rn(), base_addr, wb_addr, op.w());
}

pub fn stm_common(bb: &mut BasicBlock, 
    list: u32, rn: u32, base_addr: Var, wb_addr: Var, w: bool) {
    let mut addr = base_addr;
    let inc_val = bb.constant(32, 4);
    for reg_idx in 0..=14 {
        if (list & (1 << reg_idx)) != 0 {
            let reg_val = bb.read_reg(reg_idx);
            bb.store32(addr, reg_val);
            addr = bb.add32(addr, inc_val);
        }
    }

    if w {
        bb.write_reg(rn, wb_addr);
    }

    if (list & (1 << 15)) != 0 {
        let pc_val = bb.constant(32, bb.read_exec_pc() as usize);
        bb.store32(pc_val, addr);
    }
}



