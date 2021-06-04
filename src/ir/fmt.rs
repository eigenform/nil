//! Human-readable formatting for the intermediate representation.

use std::fmt;
use crate::ir::*;

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            VarKind::Constant(c) => write!(f, "#0x{:x}", c),
            VarKind::Local | 
            VarKind::GuestReg(_) => write!(f, "%{}", self.id),
        }
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#0x{:x}", self.value)
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value.is_some() {
            write!(f, "Flag({:?}, {}", self.kind, self.value.unwrap())
        } else {
            write!(f, "Flag({:?})", self.kind)
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.lh.is_none() {
            write!(f, "{}", self.rh)
        } else {
            let lh = self.lh.unwrap();
            match (self.lh_c, self.lh_v) {
                (None, None) => {
                    write!(f, "%{} := {}", lh.id, self.rh)
                },
                (Some(c), None) => {
                    write!(f, "%{}, c{}, _ := {}", lh.id, c, self.rh)
                },
                (None, Some(v)) => {
                    write!(f, "%{}, _, v{} := {}", lh.id, v, self.rh)
                },
                (Some(c), Some(v)) => {
                    write!(f, "%{}, c{}, v{} := {}", lh.id, c, v, self.rh)
                },
            }
        }
    }
}


impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Memory(op) => write!(f, "{}", op),
            Operation::Arith(op) => write!(f, "{}", op),
            Operation::Bind(op) => write!(f, "{}", op),
            //Operation::Branch(op) => write!(f, "{}", op),
        }
    }
}
impl fmt::Display for MemoryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryOp::Load32(addr) => write!(f, "[{}]", addr),
            MemoryOp::Store32(addr, val) => {
                write!(f, "Store32({}, {})", addr, val)
            }
        }
    }
}

impl fmt::Display for ArithOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithOp::Lsl32(x, y) => write!(f, "{} << {}", x, y),
            ArithOp::Add32(x, y) => write!(f, "{} + {}", x, y),
            ArithOp::Sub32(x, y) => write!(f, "{} - {}", x, y),
            ArithOp::And32(x, y) => write!(f, "{} & {}", x, y),
            ArithOp::Or32(x, y) => write!(f, "{} | {}", x, y),
            ArithOp::Shl32(x, y) => write!(f, "{} << {}", x, y),
            ArithOp::Shr32(x, y) => write!(f, "{} >> {}", x, y),
            ArithOp::IsZero(x) => write!(f, "IsZero({})", x),
            ArithOp::IsNegative(x) => write!(f, "IsNegative({})", x),
        }
    }
}

impl fmt::Display for BindOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindOp::Const(c) => write!(f, "{}", c),
            BindOp::ReadGuestReg(idx) => write!(f, "r{}", idx),
            BindOp::WriteGuestReg(idx, val) => {
                //write!(f, "WriteReg(r{}, {})", idx, val)
                write!(f, "r{} = {}", idx, val)
            }
            BindOp::ReadFlag(fl) => write!(f, "ReadFlag({:?})", fl),
            BindOp::WriteFlag(fl, v) => write!(f, "WriteFlag({:?}, {})", fl, v),
        }
    }
}

impl fmt::Display for BranchOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BranchOp::Branch(addr) => write!(f, "Branch({})", addr),
            BranchOp::BranchCond(cond, taddr, faddr) => {
                write!(f, "BranchCond({:?}, {}, {})", cond, taddr, faddr)
            },
        }
    }
}


