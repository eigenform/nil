use crate::mem::*;

pub type RegIdx = u32;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ProgramCounter(pub u32);
impl ProgramCounter {
    pub fn exec(&self) -> u32 { self.0.wrapping_add(8) }
    pub fn fetch(&self) -> u32 { self.0 }
    pub fn increment(&mut self) { self.0 = self.0.wrapping_add(4); }
}

/// CPU operating mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CpuMode { 
    Usr = 0b10000, Fiq = 0b10001, Irq = 0b10010, Svc = 0b10011, 
    Abt = 0b10111, Und = 0b11011, Sys = 0b11111,
}
impl CpuMode {
    pub fn is_privileged(self) -> bool { self != CpuMode::Usr }
}
impl From<u32> for CpuMode {
    fn from(x: u32) -> Self {
        use CpuMode::*;
        match x {
            0b10000 => Usr, 0b10001 => Fiq, 0b10010 => Irq, 0b10011 => Svc,
            0b10111 => Abt, 0b11011 => Und, 0b11111 => Sys,
            _ => panic!("Invalid mode bits {:08x}", x),
        }
    }
}

/// Program status register.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Psr(pub u32);
impl Psr {
    fn set_bit(&mut self, idx: usize, val: bool) {
        self.0 = (self.0 & !(1 << idx)) | (val as u32) << idx
    }

    pub fn mode(&self) -> CpuMode { CpuMode::from(self.0 & 0x1f) }
    pub fn thumb(&self) -> bool { (self.0 & 0x0000_0020) != 0 }
    pub fn fiq_disable(&self) -> bool { (self.0 & 0x0000_0040) != 0 }
    pub fn irq_disable(&self) -> bool { (self.0 & 0x0000_0080) != 0 }

    pub fn q(&self) -> bool { (self.0 & 0x0800_0000) != 0 }
    pub fn v(&self) -> bool { (self.0 & 0x1000_0000) != 0 }
    pub fn c(&self) -> bool { (self.0 & 0x2000_0000) != 0 }
    pub fn z(&self) -> bool { (self.0 & 0x4000_0000) != 0 }
    pub fn n(&self) -> bool { (self.0 & 0x8000_0000) != 0 }

    pub fn set_mode(&mut self, mode: CpuMode) { 
        self.0 = (self.0 & !0x1f) | mode as u32 
    }
    pub fn set_thumb(&mut self, val: bool) { self.set_bit(5, val); }
    pub fn set_fiq_disable(&mut self, val: bool) { self.set_bit(6, val); }
    pub fn set_irq_disable(&mut self, val: bool) { self.set_bit(7, val); }

    pub fn set_q(&mut self, val: bool) { self.set_bit(27, val); }
    pub fn set_v(&mut self, val: bool) { self.set_bit(28, val); }
    pub fn set_c(&mut self, val: bool) { self.set_bit(29, val); }
    pub fn set_z(&mut self, val: bool) { self.set_bit(30, val); }
    pub fn set_n(&mut self, val: bool) { self.set_bit(31, val); }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cond {
    EQ = 0b0000, NE = 0b0001,
    CS = 0b0010, CC = 0b0011,
    MI = 0b0100, PL = 0b0101,
    VS = 0b0110, VC = 0b0111,
    HI = 0b1000, LS = 0b1001,
    GE = 0b1010, LT = 0b1011,
    GT = 0b1100, LE = 0b1101,
    AL = 0b1110,
}
impl From<u32> for Cond {
    fn from(x: u32) -> Self {
        use Cond::*;
        match x {
            0b0000 => EQ, 0b0001 => NE,
            0b0010 => CS, 0b0011 => CC,
            0b0100 => MI, 0b0101 => PL,
            0b0110 => VS, 0b0111 => VC,
            0b1000 => HI, 0b1001 => LS,
            0b1010 => GE, 0b1011 => LT,
            0b1100 => GT, 0b1101 => LE,
            0b1110 => AL,
            _ => panic!("Invalid condition bits {:08x}", x),
        }
    }
}

pub struct GuestMmu { 
    mem: MemRegion 
}
impl GuestMmu {
    pub fn new() -> Self {
        GuestMmu {
            mem: MemRegion::new("MEM", 0x0000_0000, 0x0010_0000),
        }
    }
    pub fn write_buf(&mut self, addr: u32, buf: &[u8]) {
        self.mem.write_buf(addr, buf);
    }
    pub fn read32(&self, addr: u32) -> u32 {
        self.mem.read32(addr as usize)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GuestState { 
    pub reg: [u32; 15],
    pub pc: ProgramCounter, 
    pub cpsr: Psr,
}
impl GuestState {
    pub fn new(pc: u32, cpsr: u32) -> Self {
        GuestState { 
            reg: [0; 15], 
            pc: ProgramCounter(pc), 
            cpsr: Psr(cpsr),
        }
    }

    pub fn dump(&self) {
        println!(" R0={:08x}  R1={:08x} R2={:08x}  R3={:08x}\n \
             R4={:08x}  R5={:08x} R6={:08x}  R7={:08x}\n \
             R8={:08x}  R9={:08x} R10={:08x} R11={:08x}\n\
            R12={:08x} R13={:08x} R14={:08x} R15={:08x}",
            self.reg[0], self.reg[1], self.reg[2], self.reg[3],
            self.reg[4], self.reg[5], self.reg[6], self.reg[7],
            self.reg[8], self.reg[9], self.reg[10], self.reg[11],
            self.reg[12], self.reg[13], self.reg[14], self.pc.fetch())
    }
}


