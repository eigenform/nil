mod fmt;

use crate::guest;

/// Type used to identify a variable in the IR.
pub type VarId = usize;

/// Representing a particular kind of variable in the IR.
#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum VarKind { 
    /// Typical block-local variable.
    Local, 
    /// A constant value.
    Constant(usize), 
    /// The value of a guest register.
    GuestReg(guest::RegIdx) 
}

/// An abstract value/variable in the IR.
#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Var { 
    /// Unique identifier for this variable.
    pub id: VarId, 
    /// The width of this variable in bits.
    pub width: usize, 
    /// The kind of variable.
    pub kind: VarKind 
}
impl Var {
    fn new(id: usize, width: usize, kind: VarKind) -> Self {
        Var { id, width, kind }
    }
    pub fn new_local(id: usize, width: usize) -> Self {
        Var::new(id, width, VarKind::Local)
    }
    pub fn new_constant(id: usize, width: usize, val: usize) -> Self {
        Var::new(id, width, VarKind::Constant(val))
    }
    pub fn new_guestreg(id: usize, reg: guest::RegIdx) -> Self {
        Var::new(id, 32, VarKind::GuestReg(reg))
    }

}

/// A constant value in the IR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Constant { 
    /// The width of this value in bits.
    pub width: usize, 
    /// The value of this constant.
    pub value: usize 
}
impl Constant {
    pub fn new(width: usize, value: usize) -> Self {
        let value = value & ((1 << width) - 1);
        Constant { width, value }
    }
}

#[derive(Clone, Debug)]
pub enum FlagKind { Negative, Zero, Carry, Overflow }
pub struct Flag { 
    pub kind: FlagKind, 
    pub value: Option<bool> 
}

#[derive(Clone, Debug)]
pub enum MemoryOp { 
    Load32(Var), 
    Store32(Var, Var) 
}
#[derive(Clone, Debug)]
pub enum ArithOp { 
    Add32(Var, Var),
    Sub32(Var, Var),
    And32(Var, Var),
    Or32(Var, Var),
    Shl32(Var, Var),
    Shr32(Var, Var),
    Lsl32(Var, Var), 
    IsZero(Var),
    IsNegative(Var),
}
#[derive(Clone, Debug)]
pub enum BindOp { 
    Const(Constant),
    ReadGuestReg(guest::RegIdx), 
    WriteGuestReg(guest::RegIdx, Var),
    ReadFlag(FlagKind),
    WriteFlag(FlagKind, Var),
}
#[derive(Clone, Debug)]
pub enum BranchOp { 
    Branch(Var),
    BranchCond(guest::Cond, Var, Var),
}

#[derive(Clone)]
pub enum Operation {
    Memory(MemoryOp),
    Arith(ArithOp),
    Bind(BindOp),
}

#[derive(Clone)]
pub struct Instruction {
    pub guest_op: u32,
    pub lh: Option<Var>,
    pub lh_c: Option<Var>,
    pub lh_v: Option<Var>,
    pub rh: Operation,
}
impl Instruction {
    pub fn get_used_vars(&self) -> Vec<Var> {
        let mut vars = Vec::new();
        match self.rh {
            Operation::Bind(ref op) => match op {
                BindOp::WriteGuestReg(_, v) |
                BindOp::WriteFlag(_, v) => vars.push(*v),
                _ => {},
            },
            Operation::Memory(ref op) => match op {
                MemoryOp::Load32(v) => vars.push(*v),
                MemoryOp::Store32(a, v) => {
                    vars.push(*a);
                    vars.push(*v);
                },
            },
            Operation::Arith(ref op) => match op {
                ArithOp::Lsl32(x,y) |
                ArithOp::Add32(x,y) |
                ArithOp::Sub32(x,y) |
                ArithOp::And32(x,y) |
                ArithOp::Or32(x,y)  |
                ArithOp::Shl32(x,y) |
                ArithOp::Shr32(x,y) => {
                    vars.push(*x);
                    vars.push(*y);
                },
                ArithOp::IsNegative(x) |
                ArithOp::IsZero(x) => {
                    vars.push(*x);
                },
            },
        }
        vars
    }
}

impl Instruction {
    pub fn constant(opcd: u32, v: Var, c: Constant) -> Self {
        Instruction { 
            lh: Some(v), lh_c: None, lh_v: None,
            rh: Operation::Bind(BindOp::Const(c)),
            guest_op: opcd,
        }
    }
    pub fn read_reg(opcd: u32, v: Var, reg: guest::RegIdx) -> Self {
        Instruction { 
            lh: Some(v), lh_c: None, lh_v: None,
            rh: Operation::Bind(BindOp::ReadGuestReg(reg)),
            guest_op: opcd,
        }
    }
    pub fn write_reg(opcd: u32, reg: guest::RegIdx, val: Var) -> Self {
        Instruction { 
            lh: None, lh_c: None, lh_v: None,
            rh: Operation::Bind(BindOp::WriteGuestReg(reg, val)),
            guest_op: opcd,
        }
    }
    pub fn read_flag(opcd: u32, v: Var, kind: FlagKind) -> Self {
        Instruction { 
            lh: Some(v), lh_c: None, lh_v: None,
            rh: Operation::Bind(BindOp::ReadFlag(kind)),
            guest_op: opcd,
        }
    }
    pub fn write_flag(opcd: u32, kind: FlagKind, val: Var) -> Self {
        Instruction { 
            lh: None, lh_c: None, lh_v: None,
            rh: Operation::Bind(BindOp::WriteFlag(kind, val)),
            guest_op: opcd,
        }
    }

    pub fn load32(opcd: u32, v: Var, addr: Var) -> Self {
        Instruction {
            lh: Some(v), lh_c: None, lh_v: None,
            rh: Operation::Memory(MemoryOp::Load32(addr)),
            guest_op: opcd,
        }
    }
    pub fn store32(opcd: u32, addr: Var, val: Var) -> Self {
        Instruction {
            lh: None, lh_c: None, lh_v: None,
            rh: Operation::Memory(MemoryOp::Store32(addr, val)),
            guest_op: opcd,
        }
    }

    pub fn add32(opcd: u32, dst: Var, x: Var, y: Var) -> Self {
        Instruction {
            lh: Some(dst), lh_c: None, lh_v: None,
            rh: Operation::Arith(ArithOp::Add32(x, y)),
            guest_op: opcd,
        }
    }
    pub fn sub32(opcd: u32, dst: Var, x: Var, y: Var) -> Self {
        Instruction {
            lh: Some(dst), lh_c: None, lh_v: None,
            rh: Operation::Arith(ArithOp::Sub32(x, y)),
            guest_op: opcd,
        }
    }
    pub fn sub32f(opcd: u32, dst: Var, c: Var, v: Var, x: Var, y: Var) -> Self {
        Instruction {
            lh: Some(dst), lh_c: Some(c), lh_v: Some(v),
            rh: Operation::Arith(ArithOp::Sub32(x, y)),
            guest_op: opcd,
        }
    }


    pub fn lsl32(opcd: u32, dst: Var, x: Var, y: Var) -> Self {
        Instruction {
            lh: Some(dst), lh_c: None, lh_v: None,
            rh: Operation::Arith(ArithOp::Lsl32(x, y)),
            guest_op: opcd,
        }
    }
    pub fn lsl32f(opcd: u32, dst: Var, c: Var, v: Var, x: Var, y: Var) -> Self {
        Instruction {
            lh: Some(dst), lh_c: Some(c), lh_v: Some(v),
            rh: Operation::Arith(ArithOp::Lsl32(x, y)),
            guest_op: opcd,
        }
    }

    pub fn is_zero(opcd: u32, dst: Var, x: Var) -> Self {
        Instruction {
            lh: Some(dst), lh_c: None, lh_v: None,
            rh: Operation::Arith(ArithOp::IsZero(x)),
            guest_op: opcd,
        }
    }
    pub fn is_negative(opcd: u32, dst: Var, x: Var) -> Self {
        Instruction {
            lh: Some(dst), lh_c: None, lh_v: None,
            rh: Operation::Arith(ArithOp::IsNegative(x)),
            guest_op: opcd,
        }
    }
}

