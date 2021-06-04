
use crate::lift::dispatch::*;
use crate::lift::decode::{ArmInst, ThumbInst};

/// The global lookup table for decoding ARM/Thumb instructions.
pub const LUT: DecoderLut = DecoderLut::new();

/// The ARMv5 lookup table.
pub struct ArmLut { pub data: [ArmFn; 0x1000] }
impl ArmLut {
    const LUT_SIZE: usize = 0x1000;
    pub fn lookup(&self, opcd: u32) -> ArmFn { self.data[Self::opcd_to_idx(opcd)] }
    const fn idx_to_opcd(idx: usize) -> u32 {
        (((idx & 0x0ff0) << 16) | ((idx & 0x000f) << 4)) as u32
    }
    const fn opcd_to_idx(opcd: u32) -> usize {
        (((opcd >> 16) & 0x0ff0) | ((opcd >> 4) & 0x000f)) as usize
    }
}
impl ArmLut {
    pub const fn create_lut(default_entry: ArmFn) -> Self {
        let mut lut = ArmLut {
            data: [default_entry; 0x1000],
        };

        let mut i = 0;
        while i < Self::LUT_SIZE {
            let opcd = ArmLut::idx_to_opcd(i);
            lut.data[i as usize] = ArmFn::from_inst(ArmInst::decode(opcd));
            i += 1;
        }
        lut
    }

}

/// The ARMv5T lookup table.
pub struct ThumbLut { pub data: [ThumbFn; 0x400] }
impl ThumbLut {
    const LUT_SIZE: usize = 0x400;
    pub fn lookup(&self, opcd: u16) -> ThumbFn { self.data[Self::opcd_to_idx(opcd)] }
    const fn idx_to_opcd(idx: usize) -> u16 { (idx << 6) as u16 }
    const fn opcd_to_idx(opcd: u16) -> usize { ((opcd & 0xffc0) >> 6) as usize }
}
impl ThumbLut {
    pub const fn create_lut(default_entry: ThumbFn) -> Self {
        let mut lut = ThumbLut {
            data: [default_entry; 0x400],
        };
        let mut i = 0;
        while i < Self::LUT_SIZE {
            let opcd = ThumbLut::idx_to_opcd(i);
            lut.data[i as usize] = ThumbFn::from_inst(ThumbInst::decode(opcd));
            i += 1;
        }
        lut
    }
}

/// Container for lookup tables
pub struct DecoderLut {
    /// Lookup table for ARM instructions.
    pub arm: ArmLut,
    /// Lookup table for Thumb instructions.
    pub thumb: ThumbLut,
}
impl DecoderLut {
    pub const fn new() -> Self {
        let arm = ArmLut::create_lut(ArmFn(arm_unimpl_instr));
        let thumb = ThumbLut::create_lut(ThumbFn(thumb_unimpl_instr));
        DecoderLut { arm, thumb }
    }
}

