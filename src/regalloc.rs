//! Local (within a basic block) register allocation.
//!
//! # Liveness
//! Our IR is in "static single assignment"[^2] (SSA) form, where expressions 
//! with a result are always assigned to a unique storage location (an 
//! abstract variable). "Register allocation" is the process of mapping these
//! variables in the IR to actual storage locations on the target.
//! 
//! One advantage of an SSA representation is that, because all variables are
//! only defined once, it makes the lifetimes of values (within a basic block) 
//! easy to represent. Here, a [LiveInterval] represents the lifetime of a 
//! variable, which can be computed with the following rules:
//!
//! - A variable becomes live when it is *defined* in the left-hand side of
//!   an IR instruction
//! - A variable is no longer live after the last point where it is *used* in 
//!   the right-hand side of an IR instruction
//!
//! [^2]: See <https://en.wikipedia.org/wiki/Static_single_assignment_form>
//!   
//! # Allocator Behaviour
//! A "linear scan" register allocator[^1] colors a set of live intervals with
//! registers based on a simple rule: if the intervals associated with two 
//! variables overlap, they cannot be allocated to the same register.
//!
//! [^1]: See <https://dl.acm.org/doi/10.1145/330249.330250> and
//! <https://dl.acm.org/doi/10.5555/647478.727924>
//!
//!
//! # Available resources
//! The pool of physical registers available to an allocator might be limited 
//! by different factors, i.e.
//!
//! - Some target instructions may have fixed operands
//! - Some registers are reserved for interfaces with global state
//!
//! # Calling Conventions/Runtime Binary Interface
//!
//! | Register | Special Notes                                              |
//! | -------- | ---------------------------------------------------------- |
//! | `rax`    | Scratch register. |
//! | `rbx`    | Scratch register. |
//! | `rcx`    | Scratch register. |
//! | `rdx`    | Scratch register. |
//! | `rsi`    | Unused.           |
//! | `rdi`    | Unused.           |
//! | `r8`     | Scratch register. |
//! | `r9`     | Scratch register. |
//! | `r10`    | Scratch register. |
//! | `r11`    | Scratch register. |
//! | `r12`    | Unused.           |
//! | `r13`    | Unused.           |
//! | `r14`    | Unused.           |
//! | `rbp`    | Reserved. Base of the runtime stack frame. |
//! | `rsp`    | Reserved. Top of the runtime stack frame.  |
//! | `r15`    | Reserved. Pointer to registers in [crate::JitState]. |
//!

use std::fmt;
use std::collections::{BTreeMap, HashMap};
use crate::ir::*;
use crate::block::*;

/// Representing a physical register on the target machine.
#[derive(Clone, Copy, Debug)]
pub enum HostRegister { 
    RAX = 0x0, RCX = 0x1, RDX = 0x2, RBX = 0x3,
    RSP = 0x4, RBP = 0x5, RSI = 0x6, RDI = 0x7,
    R8  = 0x8, R9  = 0x9, R10 = 0xA, R11 = 0xB,
    R12 = 0xC, R13 = 0xD, R14 = 0xE, R15 = 0xF,
}
impl From<u8> for HostRegister {
    fn from(x: u8) -> Self {
        match x {
            0x0 => Self::RAX, 0x1 => Self::RCX, 0x2 => Self::RDX, 0x3 => Self::RBX,
            0x4 => Self::RSP, 0x5 => Self::RBP, 0x6 => Self::RSI, 0x7 => Self::RDI,
            0x8 => Self::R8, 0x9 => Self::R9, 0xa => Self::R10, 0xb => Self::R11,
            0xc => Self::R12, 0xd => Self::R13, 0xe => Self::R14, 0xf => Self::R15,
            _ => unreachable!(),
        }
    }
}

/// Abstract representation of a storage location on the target machine.
///
/// When basic blocks are lowered into the target ISA, each variable is
/// assigned a storage location by the register allocator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StorageLoc {
    /// A general-purpose register on the host machine.
    Gpr(u8),

    /// A constant value.
    Const(usize),
}
impl fmt::Display for StorageLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gpr(x) => write!(f, "{:?}", HostRegister::from(*x)),
            Self::Const(_) => Ok(()),
        }
    }
}

/// An interval (from definition to last-use) associated with some variable 
/// inside a basic block, describing the lifetime of the variable.
#[derive(Clone, Copy, Debug)]
pub struct LiveInterval(pub usize, pub usize);


/// A map from variable IDs to storage locations on the target machine.
///
/// When a basic block is being lowered into the target instruction set, this
/// map is used to translate from variables to actual storage locations.
pub struct StorageMap {
    /// The set of bindings from variables to storage locations.
    data: HashMap<Var, StorageLoc>,
}
impl StorageMap {
    pub fn new() -> Self { StorageMap { data: HashMap::new() } }

    /// Bind variable v to a storage location.
    pub fn bind(&mut self, v: Var, s: StorageLoc) { self.data.insert(v, s); }
    /// Get the storage location assigned to variable v.
    pub fn get(&self, v: &Var) -> Option<&StorageLoc> { self.data.get(v) }

    pub fn print(&self) { 
        for e in self.data.iter() { 
            match e.0.kind {
                VarKind::Local | 
                VarKind::GuestReg(_) => println!("  {}\t {}", e.0, e.1),
                _ => {},
            }
        } 
    }
}

/// A pool of physical registers available to an allocator.
pub struct RegisterPool {
    data: Vec<HostRegister>,
}
impl RegisterPool {
    /// Create a new pool of physical registers.
    pub fn new() -> Self {
        use HostRegister::*;
        RegisterPool { 
            data: vec![R11, R10, R9, R8, RBX, RDX, RCX, RAX],
        }
    }

    /// Remove (use) a register from the pool.
    pub fn take(&mut self) -> HostRegister { 
        self.data.remove(self.data.len() - 1) 
    }

    /// Restore (free) a register back to the pool.
    pub fn put_back(&mut self, r: HostRegister) { 
        self.data.push(r); 
    }

    /// Returns true if the pool of available registers is empty.
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
}

/// A map from variables to live intervals.
///
/// This uses [LiveInterval] to represent the lifetimes of variables within a 
/// basic block.
pub struct IntervalMap {
    data: BTreeMap<Var, LiveInterval>
}
impl IntervalMap {
    pub fn new() -> Self { IntervalMap { data: BTreeMap::new() } }

    /// Iterate through the instructions in a basic block and compute the map 
    /// of live intervals for all associated variables within the block.
    pub fn from_block(bb: &BasicBlock) -> Self {
        let mut map = IntervalMap::new();
        for (pos, inst) in bb.data.iter().enumerate() {

            // Variables are defined on the left-hand side
            if inst.lh.is_some() { 
                map.define_var(inst.lh.unwrap(), pos); 
            }
            if inst.lh_c.is_some() { 
                map.define_var(inst.lh_c.unwrap(), pos); 
            }
            if inst.lh_v.is_some() { 
                map.define_var(inst.lh_v.unwrap(), pos); 
            }

            // Variables are used on the right hand side
            let used_vars = inst.get_used_vars();
            for var in used_vars.iter() {
                map.use_var(*var, pos); 
            }
        }

        assert!(bb.link.is_some());
        match bb.link.unwrap() {
            BlockLink::Branch(addr) => 
                map.use_var(addr, bb.data.len()),
            BlockLink::BranchAndLink(addr, lr) => {
                map.use_var(addr, bb.data.len());
                map.use_var(lr, bb.data.len());
            },
            BlockLink::BranchCond(_, t, f) => {
                map.use_var(t, bb.data.len());
                map.use_var(f, bb.data.len());
            },
            _ => panic!(""),
        }
        map
    }


    pub fn clear(&mut self) { self.data.clear(); }

    pub fn print(&self) { 
        for e in self.data.iter() { 
            println!("  {}\t {:?}", e.0, e.1) } 
    }

    /// Insert a new entry for the provided variable.
    pub fn define_var(&mut self, v: Var, def_idx: usize) {
        self.data.insert(v, LiveInterval(def_idx, 0));
    }

    /// Update the entry for the provided variable.
    pub fn use_var(&mut self, v: Var, use_idx: usize) {
        self.data.get_mut(&v).unwrap().1 = use_idx;
    }
    /// Return a list of variables in the map which are currently unused.
    pub fn get_dead_vars(&self) -> Vec<Var> {
        self.data.iter().filter(|(_, v)| v.1 == 0).map(|(&k, _)| k).collect()
    }
}

/// Representing an allocated register and the associated live value.
///
/// A list of these structures are used during register allocation, in order
/// to keep track of the set of currently-live bindings between variables
/// and physical registers on the target.
pub struct ActiveEntry { 
    /// The live variable.
    pub var: Var, 
    /// The interval associated with this variable.
    pub interval: LiveInterval,
    /// The register allocated for this variable.
    pub reg: HostRegister, 
}


/// Given an [IntervalMap] for some basic block, color all variables and
/// return a map from variables to storage locations.
///
/// NOTE: Spilling values is currently unimplemented.
///
/// Simple "linear-scan" allocator.
/// Not much more to be done here until we can test it on actual code.
pub fn allocate_registers(intervals: &IntervalMap) -> StorageMap {
    let mut active: Vec<ActiveEntry> = Vec::new();
    let mut pool = RegisterPool::new();
    let mut storage  = StorageMap::new();

    // Iterate over all bound values/live intervals in this block
    for (var, interval) in intervals.data.iter() {

        // If this is a constant, just add it to the storage map
        if let VarKind::Constant(c) = var.kind {
            storage.bind(*var, StorageLoc::Const(c));
            continue;
        }

        // Expire any values which are not still live
        active.retain(|active_var| {
            if active_var.interval.1 <= interval.0 {
                pool.put_back(active_var.reg);
                false
            } else { true }
        });

        // If there are no available registers, spill this value.
        // Otherwise, allocate a register and mark the variable as active.
        if pool.is_empty() {
            panic!("Spilling is unimplemented");
        } else {
            let reg = pool.take();
            storage.bind(*var, StorageLoc::Gpr(reg as u8));
            active.push(ActiveEntry {
                interval: *interval,
                var: *var,
                reg
            });
        }
    }

    storage
}



