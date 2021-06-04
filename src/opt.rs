
use std::collections::HashMap;
use crate::block::BasicBlock;
use crate::ir::*;
use crate::regalloc::{ HostRegister, IntervalMap };
use crate::regalloc;

impl BasicBlock {
    pub fn prune_dead_vars(&mut self) {
        loop {
            self.intervals = IntervalMap::from_block(self);
            let deadlist = self.intervals.get_dead_vars();

            // Exit when there are no more dead variables to remove
            if deadlist.is_empty() {
                break;
            }

            // Iterate over each dead variable
            for dvar in deadlist.iter() {
                // If this variable is a constant, we can unbind it
                if let VarKind::Constant(c) = dvar.kind {
                    let c = Constant::new(dvar.width, c);
                    self.lb.remove_constant(c);
                }
                // Unused variables bound to flags can be removed
                for inst in self.data.iter_mut() {
                    match inst.lh_c {
                        Some(c) => if c == *dvar { inst.lh_c = None; }
                        None => {},
                    }
                    match inst.lh_v {
                        Some(v) => if v == *dvar { inst.lh_v = None; }
                        None => {},
                    }
                }
            }

            self.data.retain(|inst| {
                // Keep instructions that bind results to flags
                if inst.lh_c.is_some() || inst.lh_v.is_some() { return true; }

                deadlist.iter().all(|&dvar|
                    match inst.lh {
                        // Remove instructions that bind to dead variables
                        Some(var) => dvar != var,
                        // Keep instructions that don't bind a result
                        None => true,
                    }
                )
            });
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GuestRegInterval(pub usize, pub usize, pub Var);
pub struct GuestRegGraph {
    pub data: HashMap<u32, Vec<GuestRegInterval>>,
    _version_map: [u32; 15],
}
impl GuestRegGraph {
    pub fn new() -> Self {
        GuestRegGraph { data: HashMap::new(), _version_map: [0; 15] }
    }
    pub fn print(&self) {
        for reg in self.data.iter() { println!("{:?}", reg); }
    }
}

impl GuestRegGraph {
    fn read(&mut self, idx: usize, r: u32, v: Var) { 
        match self.data.get_mut(&r) {
            Some(vec) => {
                let latest = vec.last_mut().unwrap();
                latest.1 = idx;
                latest.2 = v;
            },

            // The first ref to this register is a read
            None => {
                let mut list = Vec::new();
                list.push(GuestRegInterval(0, idx, v));
                self.data.insert(r, list);
            },
        }
    }

    fn write(&mut self, idx: usize, r: u32, v: Var) { 
        match self.data.get_mut(&r) {

            Some(vec) => {
                let latest = vec.last_mut().unwrap();
                vec.push(GuestRegInterval(idx, idx, v))
            },

            // The first ref to this register is a write
            None => {
                let mut list = Vec::new();
                list.push(GuestRegInterval(idx, idx, v));
                self.data.insert(r, list);
            },
        }
    }

    // Build some map of the intervals of *guest registers* in this block.
    pub fn build(&mut self, data: &Vec<Instruction>) {
        for (idx, inst) in data.iter().enumerate() {
            if let Operation::Bind(ref op) = inst.rh {
                match op {
                    BindOp::ReadGuestReg(r) => {
                        let lh = inst.lh.unwrap();
                        self.read(idx, *r, lh);
                    },

                    BindOp::WriteGuestReg(r, val) => {
                        self.write(idx, *r, *val);
                    },
                    _ => {},
                }
            }
        }

    }
}

