#![feature(const_fn_transmute, const_fn_fn_ptr_basics, const_mut_refs, asm)]

pub mod mem;
pub mod runtime;

pub mod lift;
pub mod opt;
pub mod regalloc;

pub mod ir;
pub mod block;
pub mod guest;

use std::collections::HashMap;

use crate::runtime::{ RuntimeContext, RuntimeExitCode, BlockFunc };
use crate::guest::{ GuestState, GuestMmu, Psr };
use crate::block::BasicBlock;

/// Top-level emulator state.
#[repr(C)]
pub struct Jit {
    /// The register state associated with this guest machine.
    pub state:  GuestState,
    /// The virtual MMU associated with this guest machine.
    pub mmu: GuestMmu,
    /// A cache of previously-visited basic blocks.
    pub cache: HashMap<u32, BasicBlock>,
}

impl Jit {
    pub fn new() -> Self {
        Jit { 
            state: GuestState::new(0x0000_0000, 0x0000_0000), 
            mmu: GuestMmu::new(),
            cache: HashMap::new(),
        }
    }

    pub fn run(&mut self) {

        // Instantiate the runtime context/dispatcher
        let mut ctx = RuntimeContext::new(
            unsafe { self.state.reg.as_ptr() as usize },
            mem::ARENA_BASE,
            unsafe { std::mem::transmute(&self.state.cpsr) },
        );

        loop {
            let pc = self.state.pc.fetch();
            let bb = match self.cache.get(&pc) {
                // Lift, compile, and cache a block if we haven't seen it
                None => {
                    let mut new_block = BasicBlock::lift(&self.state, &self.mmu);
                    println!("[*] Lifted new block {:08x}", pc);
                    new_block.disas_guest();

                    new_block.prune_dead_vars();
                    new_block.disas_ir();

                    new_block.recompile();
                    new_block.disas_host();
                    new_block.storage.print();
                    new_block.intervals.print();
                    println!("");

                    self.cache.insert(pc, new_block);
                    self.cache.get(&pc).unwrap()
                },
                // Otherwise, retrieve the block from the cache
                Some(block) => block,
            };

            // Enter the dispatcher at the current block
            println!("[*] Executing block {:08x}", pc);
            let res = runtime::trampoline(&mut ctx, BlockFunc::from_block(&bb));
            match RuntimeExitCode::from(res) {
                RuntimeExitCode::NextBlock => {}, 
                RuntimeExitCode::Halt => break,
            }
        }

    }
}


