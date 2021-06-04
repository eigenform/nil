
use dynasmrt::x64::{ Assembler, Rq };
use dynasmrt::{ dynasm, DynasmApi, ExecutableBuffer, AssemblyOffset };

use crate::Jit;
use crate::block::BasicBlock;

#[repr(transparent)]
pub struct BlockFunc(pub extern "C" fn() -> usize);
impl BlockFunc {
    pub fn from_block(bb: &BasicBlock) -> Self {
        unsafe { std::mem::transmute(bb.entrypoint()) }
    }
    pub fn ptr(&self) -> usize { self.0 as usize }
}

#[repr(transparent)]
pub struct DispatcherFunc(pub extern "C" fn(block_func: usize) -> RuntimeExitCode);
impl DispatcherFunc {
    pub fn ptr(&self) -> usize { self.0 as usize }
}

#[no_mangle]
pub fn trampoline(ctx: &mut RuntimeContext, func: BlockFunc) -> RuntimeExitCode {
    RuntimeExitCode::from( (ctx.dispatcher.0)(func.ptr()) )
}

#[repr(C)]
pub struct RuntimeContext {
    pub dispatcher: DispatcherFunc,
    pub _dispatcher: ExecutableBuffer,

    pub register_ptr: usize,
    pub fastmem_ptr: usize,
    pub cpsr_ptr: usize,
    pub cycles: usize,
}
impl RuntimeContext {
    pub const CTX_CPSR:     Rq = Rq::R13;
    pub const CTX_FASTMEM:  Rq = Rq::R14;
    pub const CTX_REG:      Rq = Rq::R15;

    const CALLEE_SAVE_REGS: [Rq; 6] = [ 
        Rq::RBX, Rq::RBP, Rq::R12, Rq::R13, Rq::R14, Rq::R15
    ];
    const CALLEE_SAVE_SIZE: usize = Self::CALLEE_SAVE_REGS
        .len() * std::mem::size_of::<usize>();

    const CALLER_SAVE_REGS: [Rq; 7] = [ 
        Rq::RAX, Rq::RCX, Rq::RDX, Rq::R8, Rq::R9, Rq::R10, Rq::R11 
    ];
    const CALLER_SAVE_SIZE: usize = Self::CALLER_SAVE_REGS
        .len() * std::mem::size_of::<usize>();

}

impl RuntimeContext {
    pub fn new(register_ptr: usize, fastmem_ptr: usize, cpsr_ptr: usize) -> Self {
        let mut asm = Assembler::new().unwrap();

        dynasm!(asm
            ; .arch x64
            ; push  rbx
            ; push  rbp
            ; push  r12
            ; push  r13
            ; push  r14
            ; push  r15
            ; sub   rsp, Self::CALLEE_SAVE_SIZE as _
        );
        dynasm!(asm
            ; mov   Rq(Self::CTX_REG as u8), QWORD register_ptr as _
            ; mov   Rq(Self::CTX_FASTMEM as u8), QWORD fastmem_ptr as _
            ; mov   Rq(Self::CTX_CPSR as u8), QWORD cpsr_ptr as _
        );
        dynasm!(asm
            ; call  rsi
        );
        dynasm!(asm
            ; add   rsp, Self::CALLEE_SAVE_SIZE as _
            ; pop   r15
            ; pop   r14
            ; pop   r13
            ; pop   r12
            ; pop   rbp
            ; pop   rbx
            ; ret
        );

        let buf = asm.finalize().unwrap();
        println!("[*] dispatcher @ {:016?}", buf.ptr(AssemblyOffset(0)));
        RuntimeContext {
            dispatcher: unsafe { 
                std::mem::transmute(buf.ptr(AssemblyOffset(0))) 
            },
            _dispatcher: buf,
            register_ptr, fastmem_ptr, cpsr_ptr,
            cycles: 0
        }
    }
}

#[repr(usize)]
pub enum RuntimeExitCode { NextBlock, Halt }
impl From<usize> for RuntimeExitCode {
    fn from(x: usize) -> Self {
        match x {
            0 => RuntimeExitCode::NextBlock,
            1 => RuntimeExitCode::Halt,
            _ => panic!("Unhandled block return code {}", x),
        }
    }
}


