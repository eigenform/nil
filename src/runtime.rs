
use dynasmrt::x64::{ Assembler, Rq };
use dynasmrt::{ dynasm, DynasmApi, ExecutableBuffer, AssemblyOffset };

use crate::block::BasicBlock;

/// Function pointer to a block of recompiled code.
#[repr(transparent)]
pub struct BlockFunc(pub extern "C" fn() -> usize);
impl BlockFunc {
    pub fn from_block(bb: &BasicBlock) -> Self {
        unsafe { std::mem::transmute(bb.entrypoint()) }
    }
    pub fn ptr(&self) -> usize { self.0 as usize }
}

/// Function pointer to a dispatcher block.
#[repr(transparent)]
pub struct DispatcherFunc(pub extern "C" fn(block_func: usize) -> RuntimeExitCode);
impl DispatcherFunc {
    pub fn ptr(&self) -> usize { self.0 as usize }
}

/// Trampoline into the runtime at the specified recompiled block.
#[no_mangle]
pub fn trampoline(ctx: &mut RuntimeContext, func: BlockFunc) -> RuntimeExitCode {
    RuntimeExitCode::from( (ctx.dispatcher.0)(func.ptr()) )
}

/// Runtime environment and interfaces for recompiled code.
///
/// NOTE: Ideally we reserve as few physical registers as possible for this.
#[repr(C)]
pub struct RuntimeContext {
    /// Function pointer to the dispatcher block.
    pub dispatcher: DispatcherFunc,

    /// Pointer to guest register state.
    pub register_ptr: usize,

    /// Pointer to the base of "fast memory."
    pub fastmem_ptr: usize,

    /// Pointer to the current program status register.
    pub cpsr_ptr: usize,
    pub cycles: usize,

    /// Actual storage for the dispatcher code
    _dispatcher: ExecutableBuffer,
}
impl RuntimeContext {
    /// The physical register reserved for the CPSR.
    pub const CTX_CPSR:     Rq = Rq::R13;
    /// The physical register reserved for the base of "fast memory."
    pub const CTX_FASTMEM:  Rq = Rq::R14;
    /// The physical register reserved for guest register state.
    pub const CTX_REG:      Rq = Rq::R15;

    /// The set of callee-save registers as-defined-by the SysV ABI.
    const CALLEE_SAVE_REGS: [Rq; 6] = [ 
        Rq::RBX, Rq::RBP, Rq::R12, Rq::R13, Rq::R14, Rq::R15
    ];
    /// The size (in bytes) of the callee-save registers.
    const CALLEE_SAVE_SIZE: usize = Self::CALLEE_SAVE_REGS
        .len() * std::mem::size_of::<usize>();

    /// The set of caller-save registers as-defined-by the SysV ABI.
    const CALLER_SAVE_REGS: [Rq; 7] = [ 
        Rq::RAX, Rq::RCX, Rq::RDX, Rq::R8, Rq::R9, Rq::R10, Rq::R11 
    ];
    /// The size (in bytes) of the caller-save registers.
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

