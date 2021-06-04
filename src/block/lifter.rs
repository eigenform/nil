
use crate::ir::*;
use crate::guest;
use crate::block::{ BasicBlock, BlockLink };

use crate::lift::lut::LUT;
use crate::lift::decode::ArmInst;

impl BasicBlock {
    pub fn lift(state: &guest::GuestState, mmu: &guest::GuestMmu) -> Self {
        // Make a new basic block
        let mut bb = BasicBlock::new(state.pc);
        loop {

            // Fetch the next instruction
            let opcd = mmu.read32( bb.read_fetch_pc() );
            bb.guest_ops.push(opcd);

            // Lift the instruction into the basic block
            LUT.arm.lookup(opcd).0(&mut bb, opcd);
            match bb.link {
                Some(link) => break,
                None => bb.increment_pc(),
            }
        }

        bb
    }
}


pub trait BindOpLifter {
    type Var;
    type Reg;
    type Flag;
    fn constant(&mut self, value: usize, width: usize) -> Self::Var;
    fn read_reg(&mut self, reg: Self::Reg) -> Self::Var;
    fn write_reg(&mut self, reg: Self::Reg, val: Self::Var);
    fn read_flag(&mut self, kind: Self::Flag) -> Self::Var;
    fn write_flag(&mut self, kind: Self::Flag, val: Self::Var);
}
impl BindOpLifter for BasicBlock {
    type Var = Var;
    type Reg = guest::RegIdx;
    type Flag = FlagKind;

    fn constant(&mut self, value: usize, width: usize) -> Var {
        let c = Constant::new(value, width);
        match self.lb.get_constant(c) {
            Some(var) => *var,
            None => {
                let v = self.lb.alloca_constant(c);
                self.push(Instruction::constant(self.last_opcd(), v, c));
                v
            },
        }
    }

    fn read_reg(&mut self, reg: guest::RegIdx) -> Var {
        let v = self.lb.alloca_guestreg(reg);
        self.push(Instruction::read_reg(self.last_opcd(), v, reg));
        v
    }

    fn write_reg(&mut self, reg: guest::RegIdx, val: Var) {
        self.push(Instruction::write_reg(self.last_opcd(), reg, val));
    }

    fn read_flag(&mut self, kind: FlagKind) -> Var {
        let v = self.lb.alloca_local(1);
        self.push(Instruction::read_flag(self.last_opcd(), v, kind));
        v
    }

    fn write_flag(&mut self, kind: FlagKind, val: Var) {
        self.push(Instruction::write_flag(self.last_opcd(), kind, val));
    }
}

pub trait MemoryOpLifter {
    type Var;
    fn load32(&mut self, addr: Self::Var) -> Self::Var;
    fn store32(&mut self, addr: Self::Var, val: Self::Var);
}
impl MemoryOpLifter for BasicBlock {
    type Var = Var;
    fn load32(&mut self, addr: Var) -> Var {
        let v = self.lb.alloca_local(32);
        self.push(Instruction::load32(self.last_opcd(), v, addr));
        v
    }
    fn store32(&mut self, addr: Var, val: Var) {
        self.push(Instruction::store32(self.last_opcd(), addr, val));
    }

}

pub trait ArithOpLifter {
    type Var;
    fn add32(&mut self, x: Self::Var, y: Self::Var) -> Self::Var;
    fn sub32(&mut self, x: Self::Var, y: Self::Var) -> Self::Var;
    fn sub32f(&mut self, x: Self::Var, y: Self::Var) 
        -> (Self::Var, Self::Var, Self::Var);
    fn lsl32f(&mut self, x: Self::Var, y: Self::Var) 
        -> (Self::Var, Self::Var, Self::Var);
    fn is_zero(&mut self, x: Self::Var) -> Self::Var;
    fn is_negative(&mut self, x: Self::Var) -> Self::Var;
}
impl ArithOpLifter for BasicBlock {
    type Var = Var;
    fn add32(&mut self, x: Var, y: Var) -> Var {
        let res = self.lb.alloca_local(32);
        self.push(Instruction::add32(self.last_opcd(), res, x, y));
        res
    }
    fn sub32(&mut self, x: Var, y: Var) -> Var {
        let res = self.lb.alloca_local(32);
        self.push(Instruction::sub32(self.last_opcd(), res, x, y));
        res
    }
    fn sub32f(&mut self, x: Var, y: Var) -> (Var, Var, Var) {
        let res = self.lb.alloca_local(32);
        let c = self.lb.alloca_local(1);
        let v = self.lb.alloca_local(1);
        self.push(Instruction::sub32f(self.last_opcd(), res, c, v, x, y));
        (res, c, v)
    }
    fn lsl32f(&mut self, x: Var, y: Var) -> (Var, Var, Var) {
        let res = self.lb.alloca_local(32);
        let c = self.lb.alloca_local(1);
        let v = self.lb.alloca_local(1);
        self.push(Instruction::lsl32f(self.last_opcd(), res, c, v, x, y));
        (res, c, v)
    }
    fn is_zero(&mut self, x: Var) -> Var {
        let res = self.lb.alloca_local(1);
        self.push(Instruction::is_zero(self.last_opcd(), res, x));
        res
    }
    fn is_negative(&mut self, x: Var) -> Var {
        let res = self.lb.alloca_local(1);
        self.push(Instruction::is_negative(self.last_opcd(), res, x));
        res
    }
}


pub trait BranchOpLifter {
    type Cond;
    type Var;
    fn branch(&mut self, x: Self::Var);
    fn branch_cond(&mut self, c: Self::Cond, t: Self::Var, f: Self::Var);
}
//impl BranchOpLifter for BasicBlock {
//    type Cond = guest::Cond;
//    type Var = Var;
//
//    fn branch(&mut self, x: Var) {
//        self.push(Instruction::branch(self.last_opcd(), x));
//    }
//    fn branch_cond(&mut self, c: guest::Cond, t: Var, f: Var) {
//        self.push(Instruction::branch_cond(self.last_opcd(), c, t, f));
//    }
//}


