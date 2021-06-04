
mod fmt;
pub mod emitter;

pub mod lifter;
pub use crate::block::lifter::{ 
    BindOpLifter, MemoryOpLifter, ArithOpLifter, BranchOpLifter
};

use dynasmrt::{ ExecutableBuffer, AssemblyOffset };
use std::collections::HashMap;

use crate::ir::*;
use crate::regalloc::{ IntervalMap, StorageMap };
use crate::guest::ProgramCounter;
use crate::guest;

#[derive(Clone)]
pub struct LocalBindings {
    cur_varid: usize,
    var_map: HashMap<usize, Var>,
    const_map: HashMap<Constant, Var>,
}
impl LocalBindings {
    pub fn new() -> Self {
        LocalBindings { 
            cur_varid: 0, 
            var_map: HashMap::new(),
            const_map: HashMap::new(),
        }
    }
    pub fn next_id(&mut self) -> usize {
        let res = self.cur_varid;
        self.cur_varid += 1;
        res
    }
    pub fn get_constant(&mut self, c: Constant) -> Option<&Var> {
        self.const_map.get(&c)
    }

    pub fn remove_constant(&mut self, c: Constant) {
        self.const_map.remove(&c);
    }

    pub fn alloca_local(&mut self, width: usize) -> Var {
        let var = Var::new_local(self.next_id(), width);
        self.var_map.insert(var.id, var);
        var
    }
    pub fn alloca_constant(&mut self, c: Constant) -> Var {
        let var = Var::new_constant(self.next_id(), c.width, c.value);
        self.const_map.insert(c, var);
        self.var_map.insert(var.id, var);
        var
    }
    pub fn alloca_guestreg(&mut self, reg: guest::RegIdx) -> Var {
        let var = Var::new_guestreg(self.next_id(), reg);
        self.var_map.insert(var.id, var);
        var
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockLink {
    BranchAndLink(Var, Var),
    Branch(Var),
    BranchCond(guest::Cond, Var, Var),
}

pub struct BasicBlock {
    pub base_pc: ProgramCounter,

    /// The set of IR instructions in this block.
    pub data: Vec<Instruction>,
    /// The terminal element for this block.
    pub link: Option<BlockLink>,

    pub lb: LocalBindings,

    /// Live intervals for IR variables
    pub intervals: IntervalMap,
    /// Storage map for recompiled code
    pub storage: StorageMap,
    /// Recompiled code for this block
    pub code: ExecutableBuffer,
    /// Set of guest instructions in this block
    pub guest_ops: Vec<u32>,

    _pc: ProgramCounter,
}
impl BasicBlock {
    pub fn new(pc: ProgramCounter) -> Self { 
        BasicBlock { 
            base_pc: pc,
            data: Vec::new(), 
            lb: LocalBindings::new(), 
            link: None,

            intervals: IntervalMap::new(),
            storage: StorageMap::new(),
            code: ExecutableBuffer::new(0).unwrap(),

            guest_ops: Vec::new(),
            _pc: pc,
        }
    }

    fn last_opcd(&self) -> u32 { *self.guest_ops.last().unwrap() }
    fn push(&mut self, inst: Instruction) { 
        assert!(self.link.is_none());
        self.data.push(inst); 
    }

    pub fn read_fetch_pc(&self) -> u32 { self._pc.fetch() }
    pub fn read_exec_pc(&self) -> u32 { self._pc.exec() }
    pub fn increment_pc(&mut self) { self._pc.increment(); }

    // Return a pointer to the recompiled code for this block.
    pub fn entrypoint(&self) -> *const u8 { self.code.ptr(AssemblyOffset(0)) }

    // Terminate this block.
    pub fn terminate(&mut self, link: BlockLink) { 
        assert!(self.link.is_none());
        self.link = Some(link);
    }
}

impl BasicBlock {
    pub fn disas_guest(&self) {
        assert!(!self.guest_ops.is_empty());
        let buffer = unsafe { 
            std::slice::from_raw_parts(self.guest_ops.as_ptr() as *const u8,
                self.guest_ops.len() * std::mem::size_of::<u32>())
        };

        use yaxpeax_arm::armv7::*;
        use yaxpeax_arch::{ Decoder, LengthedInstruction };

        let dec = InstDecoder::armv5();
        let mut cur: u32 = 0;
        let mut pc: u32 = self.base_pc.fetch();

        println!("  // Guest disassembly");
        loop {
            match dec.decode(buffer[(cur as usize)..].iter().cloned()) {
                Ok(inst) => {
                    println!("  {:08x} {}", pc, inst);
                    cur += inst.len();
                    pc += 4;
                },
                Err(e) => { panic!("{:?}", e) },
            }
            if cur as usize >= buffer.len() { break; }
        }
    }

    pub fn disas_host(&self) {
        assert!(!self.code.is_empty());

        use yaxpeax_x86::long_mode::*;
        use yaxpeax_arch::{ Decoder, LengthedInstruction };

        let dec = InstDecoder::default();
        let mut cur: u64 = 0;

        println!("  // Host disassembly");
        loop {
            match dec.decode(self.code[(cur as usize)..].iter().cloned()) {
                Ok(inst) => {
                    println!("  {}", inst);
                    cur += inst.len();
                },
                Err(e) => { panic!("{:?}", e) },
            }
            if cur as usize >= self.code.len() { break; }
        }
    }

    pub fn disas_ir(&self) {
        assert!(!self.data.is_empty());
        println!("  // Intermediate Representation");
        for (idx, inst) in self.data.iter().enumerate() {
            println!("  {:08} {:08x} {}", idx, inst.guest_op, inst);
        }
        assert!(self.link.is_some());
        println!("  {:08x} Terminal {}", self.data.len(), self.link.unwrap());
    }
}


