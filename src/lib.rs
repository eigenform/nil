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

#[repr(C)]
pub struct Jit {
    pub state:  GuestState,
    pub mmu: GuestMmu,
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

        let mut ctx = RuntimeContext::new(
            unsafe { self.state.reg.as_ptr() as usize },
            mem::ARENA_BASE,
            unsafe { std::mem::transmute(&self.state.cpsr) },
        );

        loop {
            let pc = self.state.pc.fetch();

            let bb = match self.cache.get(&pc) {
                None => {
                    let mut new_block = BasicBlock::lift(&self.state, &self.mmu);
                    new_block.prune_dead_vars();
                    new_block.recompile();

                    println!("[*] Lifted new block {:08x}", pc);
                    new_block.disas_guest();
                    new_block.disas_ir();
                    new_block.disas_host();
                    new_block.storage.print();
                    new_block.intervals.print();
                    println!("");

                    self.cache.insert(pc, new_block);
                    self.cache.get(&pc).unwrap()
                },
                Some(block) => block,
            };

            let block_func = BlockFunc::from_block(&bb);
            println!("[*] Executing block {:08x}", pc);
            let res = runtime::trampoline(&mut ctx, block_func);
            match RuntimeExitCode::from(res) {
                RuntimeExitCode::NextBlock => {}, 
                RuntimeExitCode::Halt => break,
            }
        }
    }
}


