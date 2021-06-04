
use crate::guest::Cond;
use crate::lift::arm::bits::*;
use crate::block::*;

pub fn sign_extend(x: u32, bits: i32) -> i32 {
    if ((x as i32 >> (bits - 1)) & 1) != 0 { 
        x as i32 | !0 << bits 
    } else { 
        x as i32 
    }
}

pub fn b(bb: &mut BasicBlock, op: BranchBits) {
    let offset = sign_extend(op.imm24(), 24) * 4;
    let target_val = (bb.read_exec_pc() as i32).wrapping_add(offset) as u32;
    let target = bb.constant(32, target_val as usize);

    let cond = Cond::from(op.cond());
    if cond == Cond::AL {
        bb.terminate(BlockLink::Branch(target));
    } else {
        let target_false = bb.constant(32, 
            bb.read_fetch_pc()
                .wrapping_add(4) as usize
        );
        bb.terminate(BlockLink::BranchCond(cond, target, target_false));
    }
}

pub fn bl_imm(bb: &mut BasicBlock, op: BranchBits) {
    assert_eq!(Cond::from(op.cond()), Cond::AL);
    let offset = sign_extend(op.imm24(), 24) * 4;

    let lr_val = bb.read_fetch_pc().wrapping_add(4);
    let new_lr = bb.constant(32, lr_val as usize);

    let target_val = (bb.read_exec_pc() as i32).wrapping_add(offset) as u32;
    let target = bb.constant(32, target_val as usize);

    //bb.write_reg(14, new_lr);
    bb.terminate(BlockLink::BranchAndLink(target, new_lr));
}
