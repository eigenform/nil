
use std::fmt;
use crate::block::*;

impl fmt::Display for BlockLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockLink::Branch(a) => write!(f, "Branch({})", a),
            BlockLink::BranchAndLink(addr, lr) => 
                write!(f, "BranchAndLink({}, {})", addr, lr),
            BlockLink::BranchCond(c, t_addr, f_addr) => 
                write!(f, "BranchCond({:?}, {}, {})", c, t_addr, f_addr),
        }
    }
}


