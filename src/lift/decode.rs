
/// Enumerated type describing different kinds of ARM instruction encodings.
#[derive(Clone,Debug)]
pub enum ArmInst {
    AndRegShiftReg, AdcRegShiftReg, MovRegShiftReg, OrrRegShiftReg,
    EorRegShiftReg, RscRegShiftReg, MvnRegShiftReg, SbcRegShiftReg,
    AddRegShiftReg, BicRegShiftReg, RsbRegShiftReg, SubRegShiftReg,
    TeqRegShiftReg, CmnRegShiftReg, TstRegShiftReg, CmpRegShiftReg,

    SbcReg, OrrReg, BicReg, AddReg, RscReg, EorReg, MvnReg, AdcReg,
    SubReg, MovReg, AndReg, RsbReg, CmpReg, TstReg, CmnReg, TeqReg,
    MovImm, AddImm, AdcImm, RsbImm, OrrImm, BicImm, SubImm, MvnImm, 
    AndImm, RscImm, EorImm, SbcImm, CmnImm, CmpImm, TstImm, TeqImm,

    StrImm, StrhImm, StrdImm, StrbImm, StrReg, StrbReg, StrhReg, StrdReg, 
    LdrImm, LdrhImm, LdrdImm, LdrbImm, LdrsbImm, LdrshImm, 
    LdrReg, LdrbReg, LdrhReg, LdrdReg, LdrsbReg, LdrshReg, 

    Qdadd, Qsub, Qadd, Qdsub, Smull, Umlal, Smlal, Umull, Mul, Mla,
    Smulwb, Smlawb, Smlalbb, Smlabb, Smulbb,

    Ldrbt, Strbt, Ldrt, Strt, 
    MovImmAlt, LdrbtAlt, StrbtAlt, LdrtAlt, StrtAlt,
    Stm, Stmda, Ldmda, Ldmib, Ldmdb, Ldm, Stmdb, Stmib, 
    LdmRegUser, StmRegUser,
    MsrImm, MsrReg, Mrs, Mcrr, Mrrc, Mrc, Mcr, Stc,
    PldReg, PldImm, LdcImm, Clz, 
    B, BlImm, Bx, BlxReg, Bxj, 
    Svc, Bkpt, 
    Undefined,
}

/// Decoder implementation.
impl ArmInst {
    pub const fn decode(opcd: u32) -> Self {
        use ArmInst::*;
        match opcd & 0x0ff000f0 {
            0x01400050 => return Qdadd,
            0x01200050 => return Qsub,
            0x01000050 => return Qadd,
            0x01600050 => return Qdsub,
            0x01200010 => return Bx,
            0x01600010 => return Clz,
            0x01200020 => return Bxj,
            0x01200070 => return Bkpt,
            0x01200030 => return BlxReg,
            _ => {},
        }
        match opcd & 0x0fe000f0 {
            0x00c00090 => return Smull,
            0x00a00090 => return Umlal,
            0x00e00090 => return Smlal,
            0x00800090 => return Umull,
            0x00000090 => return Mul,
            0x00200090 => return Mla,
            _ => {},
        }
        match opcd & 0x0fb000f0 {
            0x01200000 => return MsrReg,
            0x01000000 => return Mrs,
            _ => {},
        }
        match opcd & 0x0ff000b0 {
            0x012000a0 => return Smulwb,
            0x01200080 => return Smlawb,
            _ => {},
        }
        match opcd & 0x0ff00090 {
            0x01400080 => return Smlalbb,
            0x01300010 => return TeqRegShiftReg,
            0x01700010 => return CmnRegShiftReg,
            0x01100010 => return TstRegShiftReg,
            0x01500010 => return CmpRegShiftReg,
            0x01000080 => return Smlabb,
            0x01600080 => return Smulbb,
            _ => {},
        }
        match opcd & 0x0e5000f0 {
            0x005000d0 => return LdrsbImm,
            0x000000f0 => return StrdReg,
            0x004000b0 => return StrhImm,
            0x001000d0 => return LdrsbReg,
            0x005000f0 => return LdrshImm,
            0x001000f0 => return LdrshReg,
            0x004000f0 => return StrdImm,
            0x005000b0 => return LdrhImm,
            0x000000d0 => return LdrdReg,
            0x001000b0 => return LdrhReg,
            0x004000d0 => return LdrdImm,
            0x000000b0 => return StrhReg,
            _ => {},
        }
        match opcd & 0x0fe00090 {
            0x00000010 => return AndRegShiftReg,
            0x00a00010 => return AdcRegShiftReg,
            0x01a00010 => return MovRegShiftReg,
            0x01800010 => return OrrRegShiftReg,
            0x00200010 => return EorRegShiftReg,
            0x00e00010 => return RscRegShiftReg,
            0x01e00010 => return MvnRegShiftReg,
            0x00c00010 => return SbcRegShiftReg,
            0x00800010 => return AddRegShiftReg,
            0x01c00010 => return BicRegShiftReg,
            0x00600010 => return RsbRegShiftReg,
            0x00400010 => return SubRegShiftReg,
            _ => {},
        }
        match opcd & 0x0ff00010 {
            0x01500000 => return CmpReg,
            0x01100000 => return TstReg,
            0x01700000 => return CmnReg,
            0x01300000 => return TeqReg,
            _ => {},
        }
        match opcd & 0x0ff00000 {
            0x03000000 => return MovImmAlt,
            0x03700000 => return CmnImm,
            0x0c400000 => return Mcrr,
            0x03500000 => return CmpImm,
            0x03100000 => return TstImm,
            0x0c500000 => return Mrrc,
            0x03300000 => return TeqImm,
            _ => {},
        }
        match opcd & 0x0f700010 {
            0x06700000 => return LdrbtAlt,
            0x06600000 => return StrbtAlt,
            0x06300000 => return LdrtAlt,
            0x06200000 => return StrtAlt,
            _ => {},
        }
        match opcd & 0x0fe00010 {
            0x00c00000 => return SbcReg,
            0x01800000 => return OrrReg,
            0x01c00000 => return BicReg,
            0x00800000 => return AddReg,
            0x00e00000 => return RscReg,
            0x00200000 => return EorReg,
            0x01e00000 => return MvnReg,
            0x00a00000 => return AdcReg,
            0x00400000 => return SubReg,
            0x01a00000 => return MovReg,
            0x00000000 => return AndReg,
            0x00600000 => return RsbReg,
            _ => {},
        }
        match opcd & 0x0fe00000 {
            0x03a00000 => return MovImm,
            0x02800000 => return AddImm,
            0x02a00000 => return AdcImm,
            0x02600000 => return RsbImm,
            0x03800000 => return OrrImm,
            0x03c00000 => return BicImm,
            0x02400000 => return SubImm,
            0x03e00000 => return MvnImm,
            0x02000000 => return AndImm,
            0x02e00000 => return RscImm,
            0x02200000 => return EorImm,
            0x02c00000 => return SbcImm,
            _ => {},
        }
        match opcd & 0x0f700000 {
            0x04700000 => return Ldrbt,
            0x04600000 => return Strbt,
            0x04300000 => return Ldrt,
            0x04200000 => return Strt,
            _ => {},
        }
        match opcd & 0x0fd00000 {
            0x08800000 => return Stm,
            0x08000000 => return Stmda,
            0x08100000 => return Ldmda,
            0x09900000 => return Ldmib,
            0x09100000 => return Ldmdb,
            0x08900000 => return Ldm,
            0x09000000 => return Stmdb,
            0x09800000 => return Stmib,
            _ => {},
        }
        match opcd & 0x0fb00000 {
            0x03200000 => return MsrImm,
            _ => {},
        }
        match opcd & 0x0e500010 {
            0x06100000 => return LdrReg,
            0x06400000 => return StrbReg,
            0x06500000 => return LdrbReg,
            0x06000000 => return StrReg,
            _ => {},
        }
        match opcd & 0x0f100010 {
            0x0e100010 => return Mrc,
            0x0e000010 => return Mcr,
            _ => {},
        }
        match opcd & 0x0e500000 {
            0x0c000000 => return Stc,
            0x08500000 => return LdmRegUser,
            0x0c100000 => return LdcImm,
            0x04000000 => return StrImm,
            0x04400000 => return StrbImm,
            0x04500000 => return LdrbImm,
            0x08400000 => return StmRegUser,
            0x04100000 => return LdrImm,
            _ => {},
        }
        match opcd & 0x0f000000 {
            0x0f000000 => return Svc,
            0x0a000000 => return B,
            0x0b000000 => return BlImm,
            _ => {},
        }
        Undefined
    }
}

#[derive(Clone, Debug)]
pub enum ThumbInst {
    SbcReg, CmpReg, OrrReg, BicReg, TstReg, EorReg, MvnReg, CmnReg, AdcReg,
    AndReg, MovReg, SubReg, AddReg, CmpRegAlt, AddRegAlt, MovRegAlt,
    MovRegShiftReg,

    RsbImm, AddImm, MovImm, SubImm, CmpImm, AddSpImm, SubSpImm,
    AddSpImmAlt, AddImmAlt, SubImmAlt, 

    StrbReg, LdrhReg, LdrbReg, StrReg, StrhReg, LdrReg, LdrsbReg, LdrshReg,

    StrhImm, StrImm, StrbImm, StrImmAlt, LdrhImm, LdrbImm, LdrImm, LdrImmAlt, 
    LdrLit, Stm, Ldm,

    Pop, Push, Mul,
    B, Bx, BlxReg, Svc, Bkpt, BAlt,

    Undefined,

    // These are exceptional (added by hand) until I decide sort how these
    // are decoded
    BlPrefix, BlImmSuffix, BlxImmSuffix,
}


impl ThumbInst {
    pub const fn decode(opcd: u16) -> ThumbInst {
        use ThumbInst::*;
        match opcd & 0xffc0 {
            0x4240 => return RsbImm,
            0x4180 => return SbcReg,
            0x4280 => return CmpReg,
            0x4300 => return OrrReg,
            0x4380 => return BicReg,
            0x4200 => return TstReg,
            0x4040 => return EorReg,
            0x43c0 => return MvnReg,
            0x42c0 => return CmnReg,
            0x4140 => return AdcReg,
            0x4340 => return Mul,
            0x4000 => return AndReg,
            _ => {},
        }
        match opcd & 0xff80 {
            0xb000 => return AddSpImmAlt,
            0xb080 => return SubSpImm,
            0x4700 => return Bx,
            0x4780 => return BlxReg,
            _ => {},
        }
        match opcd & 0xff00 {
            0xdf00 => return Svc,
            0x4500 => return CmpRegAlt,
            0x4400 => return AddRegAlt,
            0x4600 => return MovReg,
            0xbe00 => return Bkpt,
            _ => {},
        }
        match opcd & 0xfe00 {
            0x1c00 => return AddImm,
            0x5800 => return LdrReg,
            0x5600 => return LdrsbReg,
            0x4000 => return MovRegShiftReg,
            0x5e00 => return LdrshReg,
            0x1e00 => return SubImm,
            0xbc00 => return Pop,
            0x1800 => return AddReg,
            0x5400 => return StrbReg,
            0x1a00 => return SubReg,
            0xb400 => return Push,
            0x5a00 => return LdrhReg,
            0x5c00 => return LdrbReg,
            0x5000 => return StrReg,
            0x5200 => return StrhReg,
            _ => {},
        }
        match opcd & 0xf800 {
            // Exceptional (added by hand)
            0xf000 => return BlPrefix,
            0xf800 => return BlImmSuffix,
            0xe800 => return BlxImmSuffix,

            0xe000 => return BAlt,
            0x2000 => return MovImm,
            0x3000 => return AddImmAlt,
            0xa800 => return AddSpImm,
            0x8000 => return StrhImm,
            0xc000 => return Stm,
            0x3800 => return SubImmAlt,
            0x2800 => return CmpImm,
            0x6000 => return StrImm,
            0x9000 => return StrImmAlt,
            0x7000 => return StrbImm,
            0x8800 => return LdrhImm,
            0x7800 => return LdrbImm,
            0xc800 => return Ldm,
            0x6800 => return LdrImm,
            0x9800 => return LdrImmAlt,
            0x4800 => return LdrLit,
            _ => {},
        }
        match opcd & 0xf000 {
            0xd000 => return B,
            _ => {},
        }
        match opcd & 0xe000 {
            0x0000 => return MovRegAlt,
            _ => {},
        }
        Undefined
    }
}


