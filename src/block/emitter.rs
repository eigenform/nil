
use dynasmrt::x64::{ Assembler, Rq };
use dynasmrt::{ dynasm, DynasmApi, ExecutableBuffer };

use crate::block::{ BasicBlock, BlockLink };
use crate::ir::*;
use crate::regalloc;
use crate::regalloc::{ HostRegister, IntervalMap, StorageMap, StorageLoc };
use crate::runtime::RuntimeContext;

macro_rules! emit { 
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
            ; .arch  x64
            $($t)*
        )
    }
}

impl BasicBlock {
    pub fn recompile(&mut self) {
        use StorageLoc::*;

        let mut asm = Assembler::new().unwrap();
        self.intervals = IntervalMap::from_block(self);
        self.storage = regalloc::allocate_registers(&self.intervals);

        for inst in self.data.iter_mut() {
            match inst.rh {
                Operation::Bind(ref op) => match op {
                    BindOp::Const(_) => {},
                    BindOp::ReadGuestReg(idx) => {
                        let off = (idx * 4) as i32;
                        let lh = self.storage.get(&inst.lh.unwrap()).unwrap();
                        match lh {
                            Gpr(r) => emit!(asm 
                                ; mov Rd(r), DWORD [Rq(RuntimeContext::CTX_REG as u8) + off]
                            ),
                            _ => panic!("read_reg unimpl {:?}", lh), 
                        }
                    },
                    BindOp::WriteGuestReg(idx, v) => {
                        let off = (idx * 4) as i32;
                        let val = self.storage.get(v).unwrap();
                        match val {
                            Gpr(r) => emit!(asm
                                ; mov DWORD [Rq(RuntimeContext::CTX_REG as u8) + off], Rd(r)
                            ),
                            Const(c) => emit!(asm
                                ; mov DWORD [Rq(RuntimeContext::CTX_REG as u8) + off], *c as _
                            ),
                        }
                    },
                    _ => panic!("emitter doesn't implement {:?}", op),
                },

                Operation::Memory(ref op) => match op {
                    MemoryOp::Store32(addr, val) => {
                        let addr = self.storage.get(addr).unwrap();
                        let val = self.storage.get(val).unwrap();
                        match (addr, val) {
                            (Gpr(a), Gpr(v)) => emit!(asm
                                ; mov   DWORD [Rq(RuntimeContext::CTX_FASTMEM as u8) + Rq(a)], Rd(v)
                            ),
                            (Gpr(a), Const(v)) => emit!(asm
                                ; mov   DWORD [Rq(RuntimeContext::CTX_FASTMEM as u8) + Rq(a)], *v as _
                            ),
                            _ => panic!("store32 unimpl {:?} {:?}", addr, val),
                        }
                    },
                    MemoryOp::Load32(addr) => {
                        let lh = self.storage.get(&inst.lh.unwrap()).unwrap();
                        let addr = self.storage.get(addr).unwrap();
                        match (lh, addr) {
                            (Gpr(dst), Gpr(addr)) => emit!(asm
                                ; mov   Rd(dst), DWORD [Rq(RuntimeContext::CTX_FASTMEM as u8) + Rq(addr)]
                            ),
                            _ => panic!("load32 unimpl {:?} {:?}", lh, addr),
                        }
                    },
                    _ => panic!("emitter doesn't implement {:?}", op),
                },

                Operation::Arith(ref op) => match op {
                    ArithOp::Sub32(x, y) => {
                        let lh = self.storage.get(&inst.lh.unwrap()).unwrap();
                        let x = self.storage.get(x).unwrap();
                        let y = self.storage.get(y).unwrap();
                        match (lh, x, y) {
                            (Gpr(d), Gpr(x), Const(y)) => {
                                if d == x { 
                                    emit!(asm; sub Rd(x), *y as _); 
                                } 
                                else { 
                                    emit!(asm
                                    ; mov Rq(d), Rq(x)
                                    ; sub Rd(d), *y as _);
                                }
                            },
                            _ => panic!("unimpl sub32 {:?} {:?}", x, y),
                        }

                        // How to deal with flags?
                        //if inst.lh_c.is_some() {
                        //    let c_var = inst.lh_c.unwrap();
                        //    let c_loc = self.storage.get(&c_var).unwrap();
                        //    match c_loc {
                        //        Gpr(c) => { emit!(asm; setc Rb(c)); },
                        //        _ => panic!("lh_c is a constant?"),
                        //    }
                        //}
                        //if inst.lh_v.is_some() {
                        //    let v_var = inst.lh_v.unwrap();
                        //    let v_loc = self.storage.get(&v_var).unwrap();
                        //    match v_loc {
                        //        Gpr(v) => { emit!(asm; seto Rb(v)); },
                        //        _ => panic!("lh_v is a constant?"),
                        //    }
                        //}

                    },

                    ArithOp::Add32(x, y) => {
                        let lh = self.storage.get(&inst.lh.unwrap()).unwrap();
                        let x = self.storage.get(x).unwrap();
                        let y = self.storage.get(y).unwrap();
                        match (lh, x, y) {
                            (Gpr(d), Gpr(x), Const(y)) => {
                                if d == x { 
                                    emit!(asm; add Rd(x), *y as _); 
                                } else {
                                    emit!(asm
                                        ; mov Rq(d), Rq(x)
                                        ; add Rd(d), *y as _
                                    );
                                }
                            },
                            _ => panic!("unimpl add32 {:?} {:?}", x, y),
                        }
                        if inst.lh_c.is_some() { unimplemented!(""); }
                        if inst.lh_v.is_some() { unimplemented!(""); }
                    },
                    ArithOp::IsNegative(x) => {
                        let lh = self.storage.get(&inst.lh.unwrap()).unwrap();
                        let x = self.storage.get(x).unwrap();
                        match (lh, x) {
                            _ => panic!("unimpl is_negative {:?}", x),
                        }
                    },

                    _ => panic!("emitter doesn't implement {:?}", op),
                },
            }
        }

        if let Some(link) = self.link {
            match link {
                BlockLink::Branch(ref addr) => {
                    let addr = self.storage.get(addr).unwrap();
                    match addr {
                        // NOTE: Is the layout of GuestState stable enough for this?
                        Const(c) => emit!(asm
                            ; mov   DWORD [Rq(RuntimeContext::CTX_REG as u8) + 0x3c], *c as _
                            ; mov   rax, 0x0
                            ; ret
                        ),
                        _ => panic!("unimpl branch to {:?}", addr),
                    }
                },

                BlockLink::BranchAndLink(ref addr, ref new_lr) => {
                    let addr = self.storage.get(addr).unwrap();
                    let new_lr = self.storage.get(new_lr).unwrap();
                    match (addr, new_lr) {
                        // NOTE: Is the layout of GuestState stable enough for this?
                        (Const(d), Const(l)) => emit!(asm
                            ; mov   DWORD [Rq(RuntimeContext::CTX_REG as u8) + 0x38], *l as _
                            ; mov   DWORD [Rq(RuntimeContext::CTX_REG as u8) + 0x3c], *d as _
                            ; mov   rax, 0x0
                            ; ret
                        ),
                        _ => panic!("unimpl branch_link to {:?}", addr),
                    }
                },
                _ => panic!("emitter terminal {:?} unimpl", link),
            }
        } else {
            panic!("Block has no terminal element");
        }

        asm.commit().unwrap();
        self.code = asm.finalize().unwrap();
    }
}




