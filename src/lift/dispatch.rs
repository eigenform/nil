
use crate::lift::decode::*;
use crate::ir::*;
use crate::block::*;

use crate::lift::arm;
//use crate::lift::thumb;

/// A function pointer to an ARM instruction implementation.
#[derive(Clone, Copy)]
pub struct ArmFn(pub fn(&mut BasicBlock, u32));

/// A function pointer to a Thumb instruction implementation.
#[derive(Clone, Copy)]
pub struct ThumbFn(pub fn(&mut BasicBlock, u16));

/// Handler for unimplemented ARM instructions.
pub fn arm_unimpl_instr(_bb: &mut BasicBlock, op: u32) {
    println!("Unimpl {:08x} {:?}", op, ArmInst::decode(op));
    panic!("Unimplemented ARM instruction");
}

/// Handler for unimplemented Thumb instructions.
pub fn thumb_unimpl_instr(_bb: &mut BasicBlock, _op: u16) {
    panic!("Unimplemented Thumb instruction");
}


/// Map each decoded instruction to an implementation of an ARM instruction.
impl ArmFn {
    pub const fn from_inst(inst: ArmInst) -> Self {

        macro_rules! afn { ($func:expr) => { unsafe {
            std::mem::transmute::<*const fn(), fn(&mut BasicBlock, u32)>
                ($func as *const fn())
        }}}

        use ArmInst::*;
        match inst {
            MsrImm          => ArmFn(afn!(arm_unimpl_instr)),
            MsrReg          => ArmFn(afn!(arm_unimpl_instr)),
            Mrs             => ArmFn(afn!(arm_unimpl_instr)),
            Umull           => ArmFn(afn!(arm_unimpl_instr)),
            Mul             => ArmFn(afn!(arm_unimpl_instr)),

            LdrImm          => ArmFn(afn!(arm::loadstore::ldr_imm)),
            LdrbImm         => ArmFn(afn!(arm_unimpl_instr)),
            LdrhImm         => ArmFn(afn!(arm_unimpl_instr)),
            SubImm          => ArmFn(afn!(arm::dataproc::sub_imm)),
            SubReg          => ArmFn(afn!(arm_unimpl_instr)),

            LdrReg          => ArmFn(afn!(arm_unimpl_instr)),
            StrReg          => ArmFn(afn!(arm_unimpl_instr)),

            Ldmib           => ArmFn(afn!(arm_unimpl_instr)),
            Ldm             => ArmFn(afn!(arm_unimpl_instr)),
            LdmRegUser      => ArmFn(afn!(arm_unimpl_instr)),

            StrImm          => ArmFn(afn!(arm::loadstore::str_imm)),
            StrbImm         => ArmFn(afn!(arm_unimpl_instr)),
            Stmdb           => ArmFn(afn!(arm::loadstore::stmdb)),
            Stm             => ArmFn(afn!(arm_unimpl_instr)),
            StmRegUser      => ArmFn(afn!(arm_unimpl_instr)),

            Mcr             => ArmFn(afn!(arm_unimpl_instr)),
            Mrc             => ArmFn(afn!(arm_unimpl_instr)),

            B               => ArmFn(afn!(arm::branch::b)),
            Bx              => ArmFn(afn!(arm_unimpl_instr)),
            BlImm           => ArmFn(afn!(arm::branch::bl_imm)),

            RsbImm          => ArmFn(afn!(arm_unimpl_instr)),
            RsbReg          => ArmFn(afn!(arm_unimpl_instr)),
            MovImm          => ArmFn(afn!(arm::dataproc::mov_imm)),
            MvnImm          => ArmFn(afn!(arm_unimpl_instr)),
            MvnReg          => ArmFn(afn!(arm_unimpl_instr)),
            MovReg          => ArmFn(afn!(arm::dataproc::mov_reg)),
            AddImm          => ArmFn(afn!(arm_unimpl_instr)),
            AddReg          => ArmFn(afn!(arm_unimpl_instr)),
            OrrImm          => ArmFn(afn!(arm_unimpl_instr)),
            OrrReg          => ArmFn(afn!(arm_unimpl_instr)),
            EorReg          => ArmFn(afn!(arm_unimpl_instr)),
            EorImm          => ArmFn(afn!(arm_unimpl_instr)),
            AndImm          => ArmFn(afn!(arm_unimpl_instr)),
            AndReg          => ArmFn(afn!(arm_unimpl_instr)),
            CmnImm          => ArmFn(afn!(arm_unimpl_instr)),
            CmpImm          => ArmFn(afn!(arm::dataproc::cmp_imm)),
            CmpReg          => ArmFn(afn!(arm_unimpl_instr)),
            TstReg          => ArmFn(afn!(arm_unimpl_instr)),
            TstImm          => ArmFn(afn!(arm_unimpl_instr)),
            BicImm          => ArmFn(afn!(arm_unimpl_instr)),
            BicReg          => ArmFn(afn!(arm_unimpl_instr)),
            Clz             => ArmFn(afn!(arm_unimpl_instr)),

            OrrRegShiftReg  => ArmFn(afn!(arm_unimpl_instr)),
            AndRegShiftReg  => ArmFn(afn!(arm_unimpl_instr)),
            _               => ArmFn(arm_unimpl_instr),
        }
    }
}


impl ThumbFn {
    pub const fn from_inst(inst: ThumbInst) -> Self {

        macro_rules! tfn { ($func:expr) => { unsafe {
            std::mem::transmute::<*const fn(), fn(&mut BasicBlock, u16)>
                ($func as *const fn())
        }}}


        use ThumbInst::*;
        match inst {
            Push            => ThumbFn(tfn!(thumb_unimpl_instr)),
            Pop             => ThumbFn(tfn!(thumb_unimpl_instr)),
            Ldm             => ThumbFn(tfn!(thumb_unimpl_instr)),
            Stm             => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrLit          => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrbReg         => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrhReg         => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrsbReg        => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrshReg        => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrImm          => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrbImm         => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrhImm         => ThumbFn(tfn!(thumb_unimpl_instr)),
            LdrImmAlt       => ThumbFn(tfn!(thumb_unimpl_instr)),
            StrImmAlt       => ThumbFn(tfn!(thumb_unimpl_instr)),
            StrReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            StrbReg         => ThumbFn(tfn!(thumb_unimpl_instr)),
            StrhReg         => ThumbFn(tfn!(thumb_unimpl_instr)),
            StrImm          => ThumbFn(tfn!(thumb_unimpl_instr)),
            StrbImm         => ThumbFn(tfn!(thumb_unimpl_instr)),
            StrhImm         => ThumbFn(tfn!(thumb_unimpl_instr)),

            RsbImm          => ThumbFn(tfn!(thumb_unimpl_instr)),
            CmpImm          => ThumbFn(tfn!(thumb_unimpl_instr)),
            CmpReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            CmpRegAlt       => ThumbFn(tfn!(thumb_unimpl_instr)),
            MovReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            MovRegShiftReg  => ThumbFn(tfn!(thumb_unimpl_instr)),
            BicReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            TstReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            MvnReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            MovRegAlt       => ThumbFn(tfn!(thumb_unimpl_instr)),
            MovImm          => ThumbFn(tfn!(thumb_unimpl_instr)),
            AddRegAlt       => ThumbFn(tfn!(thumb_unimpl_instr)),
            AddReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            SubReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            AddImm          => ThumbFn(tfn!(thumb_unimpl_instr)),
            SubImm          => ThumbFn(tfn!(thumb_unimpl_instr)),
            AddImmAlt       => ThumbFn(tfn!(thumb_unimpl_instr)),
            SubImmAlt       => ThumbFn(tfn!(thumb_unimpl_instr)),
            AddSpImmAlt     => ThumbFn(tfn!(thumb_unimpl_instr)),
            AddSpImm        => ThumbFn(tfn!(thumb_unimpl_instr)),
            SubSpImm        => ThumbFn(tfn!(thumb_unimpl_instr)),
            AndReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            OrrReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            EorReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            SbcReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            AdcReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            Mul             => ThumbFn(tfn!(thumb_unimpl_instr)),

            BlPrefix        => ThumbFn(tfn!(thumb_unimpl_instr)),
            BlImmSuffix     => ThumbFn(tfn!(thumb_unimpl_instr)),
            BlxReg          => ThumbFn(tfn!(thumb_unimpl_instr)),
            Bx              => ThumbFn(tfn!(thumb_unimpl_instr)),
            B               => ThumbFn(tfn!(thumb_unimpl_instr)),
            BAlt            => ThumbFn(tfn!(thumb_unimpl_instr)),
            Svc             => ThumbFn(tfn!(thumb_unimpl_instr)),
            _               => ThumbFn(thumb_unimpl_instr),
        }
    }
}


