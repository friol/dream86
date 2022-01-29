
/* 

    our mythical 8086 cpu - dream86 2o22 

    TODO:
    - rewrite all the get/set flags functions as one
    - wrap around registers everywhere
    - unify lds, les, etc.
    - auxiliary flag in cmps and scas
    - overflow flag
    - test IDIV!!!
    
*/

use std::process;
use std::collections::HashMap;

use rand::Rng;

use crate::vga::vga;
use crate::machine::machine;
use crate::fddController::fddController;

//

#[derive(PartialEq)]
pub enum instructionType
{
    instrNone,
    instrPush,
    instrPushNoModRegRm,
    instrPop,
    instrPopNoModRegRm,
    instrPushf,
    instrPopf,
    instrRet,
    instrRetiw,
    instrClc,
    instrCld,
    instrStd,
    instrMov,
    instrMovNoModRegRm,
    instrAnd,
    instrAndNoModRegRm,
    instrCmp,
    instrCmpNoModRegRm,
    instrAdd,
    instrAddNoModRegRm,
    instrAdc,
    instrAdcNoModRegRm,
    instrInc,
    instrIncNoModRegRm,
    instrDec,
    instrDecNoModRegRm,
    instrXchg,
    instrXchgNoModRegRm,
    instrLods,
    instrMovs,
    instrStos,
    instrScas,
    instrCmps,
    instrJmpShort,
    instrJmpNear,
    instrJmpNp,
    instrInt,
    instrCallRel16,
    instrCallReg,
    instrCallFar,
    instrCallFarPtr,
    instrXlat,
    instrSub,
    instrSubNoModRegRm,
    instrSbb,
    instrIn,
    instrOr,
    instrOrNoModRegRm,
    instrXor,
    instrXorNoModRegRm,
    instrTest,
    instrTestNoModRegRm,
    instrShl, // to the left!
    instrShr, // to the right!!!
    instrCwd,
    instrNeg,
    instrNot,
    instrImul,
    instrMul,
    instrDiv,
    instrLea,
    instrOut,
    instrOutNoModRegRm,
    instrSbbNoModRegRm,
    instrNop,
    instrCbw,
    instrCmc,
    instrStc,
    instrCli,
    instrSti,
    instrLongJump,
    instrLds,
    instrLes,
    instrRor,
    instrRol,
    instrRcr,
    instrRcl,
    instrRetf,
    instrRetfiw,
    instrIret,
    instrJumpnw,
    instrJumpfw,
    instrPusha,
    instrPopa,
    instrLahf,
    instrSahf,
    instrAad,
    instrAam,
    instrSar,
    instrIdiv,
    instrDas,
    instrDaa,
    instrAaa,
    instrFninit,
    instrFnstsw,
}

pub struct decodedInstruction
{
    insType: instructionType,
    insLen: u8,
    instrSize: u8,
    operand1: String,
    operand2: String,
    displacement: i32,
    u8immediate: u8,
    u16immediate: u16,
    directAddr: u16,
    segOverride: String,
    repPrefix: String,
    debugDecode: String,
}    

pub struct x86cpu
{
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,
    pub si: u16,
    pub di: u16,
    pub bp: u16,
    pub sp: u16,
    pub ip: u16,
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub ss: u16,
    pub flags: u16,
    pub totInstructions: u64,
    decInstr: decodedInstruction,
    pub isIntPending: bool,
    pub intPendingNum: u8
}

//
//
//

impl x86cpu
{
    pub fn new() -> Self 
    {
        let decIn=decodedInstruction {
            insType: instructionType::instrNone,
            insLen: 0,
            instrSize: 16,
            operand1: String::from(""),
            operand2: String::from(""),
            displacement: 0,
            u8immediate: 0,
            u16immediate: 0,
            directAddr: 0,
            segOverride: String::from(""),
            repPrefix: String::from(""),
            debugDecode: String::from("Undecodable"),
        };

        x86cpu
        {
            ax: 0,
            bx: 0,
            cx: 0,
            dx: 0,
            si: 0,
            di: 0,
            bp: 0,
            sp: 0xFFFE,
            ip: 0x100, 
            cs: 0xf000,
            ds: 0xf000,
            es: 0xf000,
            ss: 0xf000,
            flags: 0,
            totInstructions: 0,
            decInstr: decIn,
            isIntPending: false,
            intPendingNum: 0,
        }
    }

    pub fn getRegisters(&self) -> HashMap<String,u16>
    {
        let retHashMap=HashMap::from(
            [
                (String::from("AX"),self.ax),
                (String::from("BX"),self.bx),
                (String::from("CX"),self.cx),
                (String::from("DX"),self.dx),
                (String::from("SI"),self.si),
                (String::from("DI"),self.di),
                (String::from("BP"),self.bp),
                (String::from("SP"),self.sp),
                (String::from("IP"),self.ip),
                (String::from("CS"),self.cs),
                (String::from("DS"),self.ds),
                (String::from("ES"),self.es),
                (String::from("SS"),self.ss),
            ]);

        return retHashMap;
    }

    /*
        MOD Field (determines how R/M operand is interpreted)
        00 Use R/M Table 1 for R/M operand
        01 Use R/M Table 2 with 8-bit signed displacement
        10 Use R/M Table 2 with 16-bit unsigned displacement
        11 Use REG table for R/M operand

        REG Field SegREG
            w=0 w=1   w=0 w=1
        000 AL AX 100 AH SP 000 ES
        001 CL CX 101 CH BP 001 CS
        010 DL DX 110 DH SI 010 SS
        011 BL BX 111 BH DI 011 DS

        R/M Table 1 (Mod = 00)
        000 [BX+SI] 010 [BP+SI] 100 [SI] 110 Direct Addr
        001 [BX+DI] 011 [BP+DI] 101 [DI] 111 [BX]

        R/M Table 2 (Mod = 01 or 10)
        000 [BX+SI+Disp] 101 [DI+Disp]
        001 [BX+DI+Disp] 011 [BP+DI+Disp] 110 [BP+Disp]
        010 [BP+SI+Disp] 100 [SI+Disp] 111 [BX+Disp]
    */

    // regtype: 0 register, 1 segreg
    fn debugDecodeAddressingModeByte(&self,b:u8,regType:u8,wbit:u8,coherency:&mut bool) -> Vec<String>
    {
        let mut retVec:Vec<String>=Vec::new();

        let rm:usize=(b&0x07).into();
        let modz=(b>>6)&0x03; // MOD Field (determines how R/M operand is interpreted)
        let reg:usize=((b>>3)&0x07).into();

        let segRegTable=vec!["ES","CS","SS","DS"]; 
        let reg16bTable=vec!["AX","CX","DX","BX","SP","BP","SI","DI"]; 
        let reg8bTable=vec!["AL","CL","DL","BL","AH","CH","DH","BH"]; 
        let rmTable1=vec!["[BX+SI]","[BX+DI]","[BP+SI]","[BP+DI]","[SI]","[DI]","Direct Addr","[BX]"]; 
        let rmTable2=vec!["[BX+SI+Disp]","[BX+DI+Disp]","[BP+SI+Disp]","[BP+DI+Disp]","[SI+Disp]","[DI+Disp]","[BP+Disp]","[BX+Disp]"]; 

        if modz==0
        {
            retVec.push(rmTable1[rm].to_string());
        }
        else if modz==1
        {
            // 01 Use R/M Table 2 with 8-bit signed displacement
            let mut s:String=rmTable2[rm].to_string();
            s.push_str(" with 8bit disp");
            retVec.push(s);
        }
        else if modz==2
        {
            // 10 Use R/M Table 2 with 16-bit unsigned displacement
            let mut s:String=rmTable2[rm].to_string();
            s.push_str(" with 16bit disp");
            retVec.push(s);
        }
        else if modz==3
        {
            if wbit==1 { retVec.push(reg16bTable[rm].to_string()); }
            else  { retVec.push(reg8bTable[rm].to_string()); }
        }

        if regType==1
        {
            if reg>3
            {
                *coherency=false;
                return retVec;
            }
            retVec.push(segRegTable[reg].to_string());
        }
        else
        {
            if wbit==1 { retVec.push(reg16bTable[reg].to_string()); }
            else  { retVec.push(reg8bTable[reg].to_string()); }
        }

        return retVec;
    }

    fn xchgRegs(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();

        let r0=self.getOperandValue(&operand1,pmachine,pvga);
        let r1=self.getOperandValue(&operand2,pmachine,pvga);

        self.moveToDestination(&r1,&operand1,pmachine,pvga);
        self.moveToDestination(&r0,&operand2,pmachine,pvga);
    }

    fn performLea(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();

        if self.decInstr.instrSize==16
        {
            let mut dst:u16=0;
            if operand1.contains("[BX+Disp] with 8bit disp")
            {
                let theAddr:i32=(self.bx as i32)+(self.decInstr.displacement as i32);
                dst=theAddr as u16;
            }
            else if operand1=="[DI]"
            {
                dst=self.di;
            }
            else if operand1.contains("[DI+Disp]")
            {
                let theAddr:i32=(self.di as i32)+(self.decInstr.displacement as i32);
                dst=theAddr as u16;
            }
            else if operand1.contains("[SI+Disp]")
            {
                let theAddr:i32=(self.si as i32)+(self.decInstr.displacement as i32);
                dst=theAddr as u16;
            }
            else if operand1.contains("[BX+Disp]")
            {
                let theAddr:i32=(self.bx as i32)+(self.decInstr.displacement as i32);
                dst=theAddr as u16;
            }
            else if operand1.contains("[BP+Disp]")
            {
                let theAddr:i32=(self.bp as i32)+(self.decInstr.displacement as i32);
                dst=theAddr as u16;
            }
            else if operand1.contains("[BP+DI+Disp]")
            {
                let theAddr:i32=(self.bp as i32)+(self.di as i32)+(self.decInstr.displacement as i32);
                dst=theAddr as u16;
            }
            else if operand1=="[BX+DI]" 
            { 
                let mut offs32:i32=self.bx as i32;
                offs32+=self.di as i32;
                offs32&=0xffff;
                dst=offs32 as u16;
            }
            else if operand1=="[BP+DI]" 
            { 
                let mut offs32:i32=self.bp as i32;
                offs32+=self.di as i32;
                offs32&=0xffff;
                dst=offs32 as u16;
            }
            else if operand1.contains("Direct Addr")
            {
                dst=self.decInstr.directAddr;
            }
            else
            {
                self.abort(&format!("unimplemented LEA {}",operand1));
            }

            self.moveToDestination(&dst,&operand2,pmachine,pvga);
        }
        else
        {
            self.abort("Unimplemented LEA 8bit (does it exist?)");
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performRcr(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags (S?)
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();
        let shiftAmount:u16=self.getOperandValue(&operand2,pmachine,pvga);

        if shiftAmount==0 { self.ip+=self.decInstr.insLen as u16; return; }

        if self.decInstr.instrSize==16
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga);

            for _idx in 0..shiftAmount
            {
                let cval=if self.getCflag() { 1 } else { 0 };
                let lastBit=val2shift&0x01;
                if lastBit==1 { self.setCflag(true); }                
                else { self.setCflag(false); }
                val2shift>>=1;
                val2shift=(val2shift&0x7fff)|(cval<<15);
            }

            self.moveToDestination(&(val2shift as u16),&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }
        else
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga);

            for _idx in 0..shiftAmount
            {
                let cval=if self.getCflag() { 1 } else { 0 };
                let lastBit=val2shift&0x01;
                if lastBit==1 { self.setCflag(true); }                
                else { self.setCflag(false); }
                val2shift>>=1;
                val2shift=(val2shift&0x7f)|(cval<<7);
            }

            self.moveToDestination(&(val2shift as u16),&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performRcl(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags (S?)
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();
        let shiftAmount:u16=self.getOperandValue(&operand2,pmachine,pvga);

        if shiftAmount==0 { self.ip+=self.decInstr.insLen as u16; return; }

        if self.decInstr.instrSize==16
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga);

            for _idx in 0..shiftAmount
            {
                let cval=if self.getCflag() { 1 } else { 0 };
                let lastBit=val2shift&0x8000;
                if lastBit==1 { self.setCflag(true); }                
                else { self.setCflag(false); }
                val2shift<<=1;
                val2shift=(val2shift&0xfffe)|cval;
            }

            self.moveToDestination(&(val2shift as u16),&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }
        else
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga);

            for _idx in 0..shiftAmount
            {
                let cval=if self.getCflag() { 1 } else { 0 };
                let lastBit=val2shift&0x80;
                if lastBit==1 { self.setCflag(true); }                
                else { self.setCflag(false); }
                val2shift<<=1;
                val2shift=(val2shift&0xfe)|cval;
            }

            self.moveToDestination(&(val2shift as u16),&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performRor(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags (S?)
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();
        let shiftAmount:u16=self.getOperandValue(&operand2,pmachine,pvga);

        if shiftAmount==0 { self.ip+=self.decInstr.insLen as u16; return; }

        if self.decInstr.instrSize==16
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga);

            for _idx in 0..shiftAmount
            {
                let lastBit=val2shift&0x01;
                if lastBit==1 { self.setCflag(true); }                
                else { self.setCflag(false); }
                val2shift>>=1;
                val2shift=(val2shift&0x7fff)|(lastBit<<15);
            }

            self.moveToDestination(&val2shift,&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }
        else
        {
            let mut val2shift:u8=self.getOperandValue(&operand1,pmachine,pvga) as u8;

            for _idx in 0..shiftAmount
            {
                let lastBit=val2shift&0x01;
                if lastBit==1 { self.setCflag(true); }                
                else { self.setCflag(false); }
                val2shift>>=1;
                val2shift=(val2shift&0x7f)|(lastBit<<7);
            }

            self.moveToDestination(&(val2shift as u16),&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performRol(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags (S?)
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();
        let shiftAmount:u16=self.getOperandValue(&operand2,pmachine,pvga);

        if shiftAmount==0 { self.ip+=self.decInstr.insLen as u16; return; }

        if self.decInstr.instrSize==16
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga) as u16;

            for _idx in 0..shiftAmount
            {
                let lastBit=val2shift&0x8000;
                if lastBit==1 { self.setCflag(true); }                
                else { self.setCflag(false); }
                val2shift<<=1;
                val2shift&=0xffff;
                val2shift=(val2shift&0xfffe)|lastBit;
            }

            self.moveToDestination(&val2shift,&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }
        else
        {
            let mut val2shift:u8=self.getOperandValue(&operand1,pmachine,pvga) as u8;

            for _idx in 0..shiftAmount
            {
                let lastBit=val2shift&0x80;
                if lastBit==1 { self.setCflag(true); }                
                else { self.setCflag(false); }
                val2shift<<=1;
                val2shift&=0xff;
                val2shift=(val2shift&0xfe)|lastBit;
            }

            self.moveToDestination(&(val2shift as u16),&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performSar(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags (S?)
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();
        let shiftAmount:u16=self.getOperandValue(&operand2,pmachine,pvga);

        if shiftAmount==0 { self.ip+=self.decInstr.insLen as u16; return; }

        if self.decInstr.instrSize==16
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga);

            if shiftAmount>15 
            { 
                val2shift=0; 
                self.setCflag(false);
            }
            else 
            { 
                let lastBit:bool=(val2shift&(1<<(shiftAmount-1)))!=0;
                self.setCflag(lastBit);
                let sign=val2shift&0x8000;
                val2shift>>=shiftAmount; 
                val2shift|=sign;
            }

            self.moveToDestination(&val2shift,&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }
        else
        {
            let mut val2shift:u8=self.getOperandValue(&operand1,pmachine,pvga) as u8;

            if shiftAmount>7 
            { 
                val2shift=0; 
                self.setCflag(false);
            }
            else 
            { 
                let lastBit:bool=(val2shift&(1<<(shiftAmount-1)))!=0;
                self.setCflag(lastBit);
                let sign=val2shift&0x80;
                val2shift>>=shiftAmount; 
                val2shift|=sign;
            }
            self.moveToDestination(&(val2shift as u16),&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performShr(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags (S?)
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();
        let shiftAmount:u16=self.getOperandValue(&operand2,pmachine,pvga);

        if shiftAmount==0 { self.ip+=self.decInstr.insLen as u16; return; }

        if self.decInstr.instrSize==16
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga);

            if shiftAmount>15 
            { 
                val2shift=0; 
                self.setCflag(false);
            }
            else 
            { 
                let lastBit:bool=(val2shift&(1<<(shiftAmount-1)))!=0;
                self.setCflag(lastBit);
                val2shift>>=shiftAmount; 
            }

            self.moveToDestination(&val2shift,&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }
        else
        {
            let mut val2shift:u8=self.getOperandValue(&operand1,pmachine,pvga) as u8;

            if shiftAmount>7 
            { 
                val2shift=0; 
                self.setCflag(false);
            }
            else 
            { 
                let lastBit:bool=(val2shift&(1<<(shiftAmount-1)))!=0;
                self.setCflag(lastBit);
                val2shift>>=shiftAmount; 
            }
            self.moveToDestination(&(val2shift as u16),&operand1,pmachine,pvga);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performShl(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags (S?)
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();

        let shiftAmount=self.getOperandValue(&operand2,pmachine,pvga); 

        if shiftAmount==0 { self.ip+=self.decInstr.insLen as u16; return; }

        let mut dst:u32=self.getOperandValue(&operand1,pmachine,pvga) as u32; 
        let reg:u16=dst as u16;
        dst<<=shiftAmount;

        if self.decInstr.instrSize==8 { dst&=0xff; }
        else if self.decInstr.instrSize==16 { dst&=0xffff; }

        self.moveToDestination(&(dst as u16),&operand1,pmachine,pvga);

        let mut lastBit:bool=false;
        if self.decInstr.instrSize==16 
        { 
            lastBit=(reg&(1<<(16-shiftAmount)))!=0; 
        }
        else if self.decInstr.instrSize==8 
        { 
            lastBit=(reg&(1<<(8-shiftAmount)))!=0; 
        }
        
        self.setCflag(lastBit);
        self.doZflag(dst as u16);
        self.doPflag(dst as u16);

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performNeg(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let operand1=self.decInstr.operand1.clone();

        if self.decInstr.instrSize==16 
        { 
            let inum=self.getOperandValue(&operand1,pmachine,pvga) as i16; 
            let dst:u16=(-inum) as u16;
            self.moveToDestination(&dst,&operand1,pmachine,pvga);

            self.setOflag(dst==0x8000);
            self.setCflag(dst != 0);
            self.setAflag(dst & 0xf != 0);

            self.doZflag(dst);
            self.doSflag(dst,self.decInstr.instrSize);
            self.doPflag(dst);
        }
        else if self.decInstr.instrSize==8 
        { 
            let inum=self.getOperandValue(&operand1,pmachine,pvga) as i8; 
            let dst:u8=(-inum) as u8;
            self.moveToDestination(&(dst as u16),&operand1,pmachine,pvga);

            self.setOflag(dst==0x80);
            self.setCflag(dst != 0);
            self.setAflag(dst & 0xf != 0);

            self.doZflag(dst as u16);
            self.doSflag(dst as u16,self.decInstr.instrSize);
            self.doPflag(dst as u16);
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performNot(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let operand1=self.decInstr.operand1.clone();

        let mut inum=self.getOperandValue(&operand1,pmachine,pvga) as i16 as i32; 
        inum=!inum;
        let dst:u16=inum as u16;
        self.moveToDestination(&dst,&operand1,pmachine,pvga);

        self.ip+=self.decInstr.insLen as u16;
    }

    // signed multiply
    fn performImul(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO 
        let operand1=self.decInstr.operand1.clone();

        if self.decInstr.instrSize==16
        {
            let mul64:i64=self.getOperandValue(&operand1,pmachine,pvga) as i16 as i64;
            let ax64:i64=self.ax as i16 as i64;

            let result=ax64*mul64;
            let dst:u16=(result&0xffff) as u16;
            //self.moveToDestination(&dst,&operand1,pmachine,pvga);
            self.ax=dst;
            self.dx=((result>>16)&0xffff) as u16;

            // TODO right?
            if (self.dx&0x8000)==0x8000 { self.setCflag(true); }
            else { self.setCflag(false); }
            if (self.dx&0x8000)==0x8000 { self.setSflag(true); }
            else { self.setSflag(false); }
            self.doZflag(dst);
            //self.doPflag(quotient as u16); // todo check p flag
        }
        else
        {
            let mul16:i16=self.getOperandValue(&operand1,pmachine,pvga) as i16;
            let al:i16=(self.ax&0xff) as i8 as i16;

            self.ax=(mul16*al) as u16;

            if (self.ax&0x8000)==0x8000 { self.setCflag(true); }
            else { self.setCflag(false); }
            if (self.ax&0x8000)==0x8000 { self.setSflag(true); }
            else { self.setSflag(false); }
            self.doZflag(self.ax);
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    // unsigned multiply
    fn performMul(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO 
        let operand1=self.decInstr.operand1.clone();

        if self.decInstr.instrSize==16
        {
            let mul64:i64=self.getOperandValue(&operand1,pmachine,pvga) as i64;
            let ax64:i64=self.ax as i64;

            let result=ax64*mul64;
            let dst:u16=(result&0xffff) as u16;
            self.ax=dst;
            self.dx=((result>>16)&0xffff) as u16;

            // TODO right?
            if (self.dx&0x8000)==0x8000 { self.setCflag(true); }
            else { self.setCflag(false); }
            if (self.dx&0x8000)==0x8000 { self.setSflag(true); }
            else { self.setSflag(false); }
            self.doZflag(dst);
            //self.doPflag(quotient as u16); // todo check p flag
        }
        else
        {
            let m1:u16=self.getOperandValue(&operand1,pmachine,pvga) as u16;
            let m2:u16=self.ax&0xff;

            let result=m1*m2;
            self.ax=result;

            // TODO right?
            if (self.ax&0x8000)==0x8000 { self.setCflag(true); }
            else { self.setCflag(false); }
            if (self.ax&0x8000)==0x8000 { self.setSflag(true); }
            else { self.setSflag(false); }
            self.doZflag(result);

            //self.abort(&format!("Unhandled MUL 8bit {}",operand1));
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performDiv(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO 
        let operand1=self.decInstr.operand1.clone();

        if self.decInstr.instrSize==16
        {
            let dx32:u32=self.dx as u32;
            let ax32:u32=self.ax as u32;
            let dv32:u32=self.getOperandValue(&operand1,pmachine,pvga) as u32;
            let val2divide:u32=ax32|(dx32<<16);
            let modulo=val2divide%dv32;
            let quotient=val2divide/dv32;
            self.dx=modulo as u16;
            self.ax=quotient as u16;

            self.doZflag(quotient as u16);
            //self.doPflag(quotient as u16); // todo check p flag
        }
        else
        {
            let dv32:u32=self.getOperandValue(&operand1,pmachine,pvga) as u32;
            let val2divide:u32=self.ax as u32;
            let modulo=val2divide%dv32;
            let quotient=val2divide/dv32;
            self.ax=((quotient as u16)&0xff)|((modulo as u16)<<8);

            self.doZflag(quotient as u16);
            //self.doPflag(quotient as u16); // todo check p flag
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performIdiv(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO 
        let operand1=self.decInstr.operand1.clone();

        if self.decInstr.instrSize==16
        {
            // Gaaaahh TODO recheck
            let dx32:i32=self.dx as i32;
            let ax32:i32=self.ax as i32;
            let dv32:i32=self.getOperandValue(&operand1,pmachine,pvga) as i16 as i32;
            let val2divide:i32=ax32|(dx32<<16);
            let modulo=val2divide%dv32;
            let quotient=val2divide/dv32;
            self.dx=modulo as u16;
            self.ax=quotient as u16;

            self.doZflag(quotient as u16);
            //self.doPflag(quotient as u16); // todo check p flag
        }
        else
        {
            let dv32:i32=self.getOperandValue(&operand1,pmachine,pvga) as i8 as i32;
            let val2divide:i32=self.ax as i16 as i32;
            let modulo=val2divide%dv32;
            let quotient=val2divide/dv32;
            self.ax=((quotient as u16)&0xff)|((modulo as u16)<<8);

            self.doZflag(quotient as u16);
            //self.doPflag(quotient as u16); // todo check p flag
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performInc(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let operand1=self.decInstr.operand1.clone();
        let mut val2inc:u16=self.getOperandValue(&operand1,pmachine,pvga);
        val2inc=val2inc.wrapping_add(1);
        self.moveToDestination(&val2inc,&operand1,pmachine,pvga);

        if self.decInstr.instrSize==8
        {
            self.setOflag(val2inc == 0x80);
            self.setAflag(val2inc&0xf==0x0);
        }
        else
        {
            self.setOflag(val2inc == 0x8000);
            self.setAflag(val2inc & 0x000f == 0x0);
        }

        self.doZflag(val2inc); 
        self.doPflag(val2inc); 
        self.doSflag(val2inc,self.decInstr.instrSize); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performDec(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let operand1=self.decInstr.operand1.clone();

        let mut val2dec:u16=self.getOperandValue(&operand1,pmachine,pvga);
        val2dec=val2dec.wrapping_sub(1);
        self.moveToDestination(&val2dec,&operand1,pmachine,pvga);

        if self.decInstr.instrSize==8
        {
            self.setOflag(val2dec == 0x7f);
            self.setAflag(val2dec&0xf==0x0f);
        }
        else
        {
            self.setOflag(val2dec == 0x7fff);
            self.setAflag(val2dec & 0x000f == 0x0f);
        }

        self.doZflag(val2dec); 
        self.doPflag(val2dec); 
        self.doSflag(val2dec,self.decInstr.instrSize); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performLds(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut readSeg:u16=self.ds;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();

        let mut destAddr=0;
        let mut destSeg=0;

        if operand1=="Direct Addr"
        {
            destAddr=pmachine.readMemory16(readSeg,self.decInstr.directAddr,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.decInstr.directAddr+2,pvga);
        }
        else if operand1=="[BX]"
        {
            destAddr=pmachine.readMemory16(readSeg,self.bx,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.bx+2,pvga);
        }
        else if operand1=="[SI]"
        {
            destAddr=pmachine.readMemory16(readSeg,self.si,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.si+2,pvga);
        }
        else if operand1.contains("[SI+Disp]")
        {
            destAddr=pmachine.readMemory16(readSeg,self.si+self.decInstr.displacement as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.si+2+self.decInstr.displacement as u16,pvga);
        }
        else if operand1.contains("[DI+Disp]")
        {
            destAddr=pmachine.readMemory16(readSeg,self.di+self.decInstr.displacement as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.di+2+self.decInstr.displacement as u16,pvga);
        }
        else if operand1.contains("[BP+Disp]")
        {
            let mut readSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

            destAddr=pmachine.readMemory16(readSeg,((self.bp as i32)+self.decInstr.displacement) as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,(((self.bp+2) as i32)+self.decInstr.displacement) as u16,pvga);
        }
        else if operand1.contains("[BX+Disp]")
        {
            destAddr=pmachine.readMemory16(readSeg,((self.bx as i32)+self.decInstr.displacement) as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,(((self.bx+2) as i32)+self.decInstr.displacement) as u16,pvga);
        }
        else if operand1=="[DI]"
        {
            destAddr=pmachine.readMemory16(readSeg,self.di,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.di+2,pvga);
        }
        else
        {
            self.abort(&format!("Unhandled LDS at {:04x}",self.ip));
        }

        self.moveToDestination(&destAddr,&operand2,pmachine,pvga);
        self.moveToDestination(&destSeg,&"DS".to_string(),pmachine,pvga);

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performLes(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // todo: what happens if is les xx,[bp+n] ?
        let mut readSeg:u16=self.ds;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();

        let mut destAddr=0;
        let mut destSeg=0;

        if operand1=="Direct Addr"
        {
            destAddr=pmachine.readMemory16(readSeg,self.decInstr.directAddr,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.decInstr.directAddr+2,pvga);
        }
        else if operand1=="[BX]"
        {
            destAddr=pmachine.readMemory16(readSeg,self.bx,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.bx+2,pvga);
        }
        else if operand1=="[DI]"
        {
            destAddr=pmachine.readMemory16(readSeg,self.di,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.di+2,pvga);
        }
        else if operand1=="[SI]"
        {
            destAddr=pmachine.readMemory16(readSeg,self.si,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.si+2,pvga);
        }
        else if operand1.contains("[BX+Disp]")
        {
            destAddr=pmachine.readMemory16(readSeg,self.bx+self.decInstr.displacement as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.bx+2+self.decInstr.displacement as u16,pvga);
        }
        else if operand1.contains("[DI+Disp]")
        {
            destAddr=pmachine.readMemory16(readSeg,self.di+self.decInstr.displacement as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.di+2+self.decInstr.displacement as u16,pvga);
        }
        else if operand1.contains("[SI+Disp]")
        {
            destAddr=pmachine.readMemory16(readSeg,self.si+self.decInstr.displacement as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.si+2+self.decInstr.displacement as u16,pvga);
        }
        else if operand1.contains("[BX+SI+Disp]")
        {
            destAddr=pmachine.readMemory16(readSeg,self.bx+self.si+self.decInstr.displacement as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,self.bx+self.si+2+self.decInstr.displacement as u16,pvga);
        }
        else if operand1.contains("[BP+Disp]")
        {
            let mut readSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; }
    
            destAddr=pmachine.readMemory16(readSeg,((self.bp as i32)+self.decInstr.displacement) as u16,pvga);
            destSeg=pmachine.readMemory16(readSeg,(((self.bp+2) as i32)+self.decInstr.displacement) as u16,pvga);
        }
        else
        {
            self.abort(&format!("Unhandled LES {} at {:04x}:{:04x}",operand1,self.cs,self.ip));
        }

        self.moveToDestination(&destAddr,&operand2,pmachine,pvga);
        self.moveToDestination(&destSeg,&"ES".to_string(),pmachine,pvga);

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performLods(&mut self,pmachine:&machine,pvga:&vga)
    {
        // LODSB/LODSW
        let mut readSeg:u16=self.ds;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        if self.decInstr.repPrefix!="".to_string()
        {
            self.abort("Unhandled rep prefix lods");
        }

        if self.decInstr.instrSize==16
        {
            let dataw=pmachine.readMemory16(readSeg,self.si,pvga);
            self.ax=dataw;

            if self.getDflag() { self.si-=2; }
            else { self.si+=2; }
        }
        else if self.decInstr.instrSize==8
        {
            let datab=pmachine.readMemory(readSeg,self.si,pvga);
            self.ax=(self.ax&0xff00)|(datab as u16);

            if self.getDflag() { self.si-=1; }
            else { self.si+=1; }
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performMovs(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let inc:u16=if self.decInstr.instrSize==16 { 2 } else { 1 };
        let mut readSeg:u16=self.ds;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        if self.decInstr.repPrefix=="REPNE"
        {
            self.abort("Unhandled REPNE movs");
        }

        if self.decInstr.repPrefix=="REPE"
        {
            if self.cx!=0
            {
                if self.decInstr.instrSize==16
                {
                    let dataw=pmachine.readMemory16(readSeg,self.si,pvga);
                    pmachine.writeMemory16(self.es,self.di,dataw,pvga);
                    if self.getDflag() 
                    { 
                        self.si=self.si.wrapping_sub(inc);
                        self.di=self.di.wrapping_sub(inc);
                    }
                    else 
                    { 
                        self.si=self.si.wrapping_add(inc);
                        self.di=self.di.wrapping_add(inc);
                    }
                }
                else if self.decInstr.instrSize==8
                {
                    let datab=pmachine.readMemory(readSeg,self.si,pvga);
                    pmachine.writeMemory(self.es,self.di,datab,pvga);
                    if self.getDflag() 
                    { 
                        self.si=self.si.wrapping_sub(inc);
                        self.di=self.di.wrapping_sub(inc);
                    }
                    else 
                    { 
                        self.si=self.si.wrapping_add(inc);
                        self.di=self.di.wrapping_add(inc);
                    }
                }
                self.cx-=1;
            }
            else
            {
                self.ip+=self.decInstr.insLen as u16;
            }
        }
        else
        {
            if self.decInstr.instrSize==16
            {
                let dataw=pmachine.readMemory16(readSeg,self.si,pvga);
                pmachine.writeMemory16(self.es,self.di,dataw,pvga);
                if self.getDflag() { self.si-=inc; self.di-=inc; }
                else { self.si+=inc; self.di+=inc; }
            }
            else if self.decInstr.instrSize==8
            {
                let datab=pmachine.readMemory(readSeg,self.si,pvga);
                pmachine.writeMemory(self.es,self.di,datab,pvga);
                if self.getDflag() { self.si-=inc; self.di-=inc; }
                else { self.si+=inc; self.di+=inc; }
            }

            self.ip+=self.decInstr.insLen as u16;
        }
    }

    fn performStos(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        if self.decInstr.repPrefix=="REPNE"
        {
            self.abort("Unhandled REPNE stos");
        }

        if self.decInstr.instrSize==16
        {
            if self.decInstr.repPrefix=="REPE"
            {
                if self.cx!=0
                {
                    pmachine.writeMemory16(self.es,self.di,self.ax,pvga);
                    if self.getDflag() { self.di-=2; }
                    else { self.di=self.di.wrapping_add(2); }
                    self.cx-=1;
                }
                else
                {
                    self.ip+=2;
                }
            }
            else
            {
                pmachine.writeMemory16(self.es,self.di,self.ax,pvga);
                if self.getDflag() { self.di-=2; }
                else 
                { 
                    let mut di32:i32=self.di as i32;
                    di32+=2; 
                    self.di=(di32&0xffff) as u16;
                }
                self.ip+=1;
            }
        }
        else if self.decInstr.instrSize==8
        {
            if self.decInstr.repPrefix=="REPE"
            {
                if self.cx!=0
                {
                    pmachine.writeMemory(self.es,self.di,(self.ax&0xff) as u8,pvga);
                    if self.getDflag() { self.di-=1; }
                    else { self.di+=1; }
                    self.cx-=1;
                }
                else
                {
                    self.ip+=2;
                }
            }
            else
            {
                pmachine.writeMemory(self.es,self.di,(self.ax&0xff) as u8,pvga);
                if self.getDflag() { self.di-=1; }
                else { self.di+=1; }
                self.ip+=1;
            }
        }
    }

    // the instruction that scas'd everything
    fn performScas(&mut self,pmachine:&machine,pvga:&vga)
    {
        let mut readSeg:u16=self.es;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        if self.decInstr.segOverride!=""
        {
            self.abort("Unhandled seg override in SCAS");
        }

        if self.decInstr.instrSize==16
        {
            if self.decInstr.repPrefix=="REPNE"
            {
                if self.cx!=0
                {
                    let dataw:i32=pmachine.readMemory16(readSeg,self.di,pvga) as i32;

                    let axi32=self.ax as i32;
                    let result:i32=axi32-dataw;
    
                    self.doZflag(result as u16);
                    self.doPflag(result as u16);
                    self.doSflag(result as u16,self.decInstr.instrSize);
                    self.doCflag(result as u16,self.decInstr.instrSize);
            
                    if self.getDflag() { self.di-=2; }
                    else { self.di+=2; }

                    self.cx-=1;
                    if result==0 { self.ip+=self.decInstr.insLen as u16; return; }
                }
                else
                {
                    self.ip+=self.decInstr.insLen as u16;
                }
            }
            else if self.decInstr.repPrefix=="REPE"
            {
                if self.cx!=0
                {
                    let dataw:i32=pmachine.readMemory16(readSeg,self.di,pvga) as i32;

                    let axi32=self.ax as i32;
                    let result:i32=axi32-dataw;
    
                    self.doZflag(result as u16);
                    self.doPflag(result as u16);
                    self.doSflag(result as u16,self.decInstr.instrSize);
                    self.doCflag(result as u16,self.decInstr.instrSize);
            
                    if self.getDflag() { self.di-=2; }
                    else { self.di+=2; }

                    self.cx-=1;
                    if result!=0 { self.ip+=self.decInstr.insLen as u16; return; }
                }
                else
                {
                    self.ip+=self.decInstr.insLen as u16;
                }
            }
            else if self.decInstr.repPrefix==""
            {
                let dataw:i32=pmachine.readMemory16(readSeg,self.di,pvga) as i32;

                let axi32=self.ax as i32;
                let result:i32=axi32-dataw;

                self.doZflag(result as u16);
                self.doPflag(result as u16);
                self.doSflag(result as u16,self.decInstr.instrSize);
                self.doCflag(result as u16,self.decInstr.instrSize);

                if self.getDflag() { self.di-=2; }
                else { self.di+=2; }

                self.ip+=self.decInstr.insLen as u16;
            }
        }
        else if self.decInstr.instrSize==8
        {
            if self.decInstr.repPrefix=="REPNE"
            {
                if self.cx!=0
                {
                    let datab:i16=pmachine.readMemory(readSeg,self.di,pvga) as i16;

                    let axi16=(self.ax&0xff) as i16;
                    let mut result:i16=axi16-datab;
                    result&=0xff;
        
                    self.doZflag(result as u16);
                    self.doPflag(result as u16);
                    self.doSflag(result as u16,self.decInstr.instrSize);
                    self.doCflag(result as u16,self.decInstr.instrSize);
        
                    if self.getDflag() { self.di-=1; }
                    else { self.di+=1; }

                    self.cx-=1;
                    if result==0 { self.ip+=self.decInstr.insLen as u16; return; }
                }
                else
                {
                    self.ip+=self.decInstr.insLen as u16;
                }
            }
            else if self.decInstr.repPrefix=="REPE"
            {
                if self.cx!=0
                {
                    let datab:i16=pmachine.readMemory(readSeg,self.di,pvga) as i16;

                    let axi16=(self.ax&0xff) as i16;
                    let mut result:i16=axi16-datab;
                    result&=0xff;
        
                    self.doZflag(result as u16);
                    self.doPflag(result as u16);
                    self.doSflag(result as u16,self.decInstr.instrSize);
                    self.doCflag(result as u16,self.decInstr.instrSize);
        
                    if self.getDflag() { self.di-=1; }
                    else { self.di+=1; }

                    self.cx-=1;
                    if result!=0 { self.ip+=self.decInstr.insLen as u16; return; }
                }
                else
                {
                    self.ip+=self.decInstr.insLen as u16;
                }
            }
            else
            {
                let datab:i16=pmachine.readMemory(readSeg,self.di,pvga) as i16;

                let axi16=(self.ax&0xff) as i16;
                let mut result:i16=axi16-datab;
                result&=0xff;
    
                self.doZflag(result as u16);
                self.doPflag(result as u16);
                self.doSflag(result as u16,self.decInstr.instrSize);
                self.doCflag(result as u16,self.decInstr.instrSize);
    
                if self.getDflag() { self.di-=1; }
                else { self.di+=1; }

                self.ip+=self.decInstr.insLen as u16;
            }
        }
    }

    fn performCmps(&mut self,pmachine:&machine,pvga:&vga)
    {
        let mut readSeg:u16=self.ds;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        if self.decInstr.repPrefix=="".to_string()
        {
            if self.decInstr.instrSize==16
            {
                let data:i32=pmachine.readMemory16(self.es,self.di,pvga) as i32;
                let val2compare:i32=pmachine.readMemory16(readSeg,self.si,pvga) as i32;

                let cmpval:i32=val2compare-data;
        
                if val2compare<data { self.setCflag(true); }
                else { self.setCflag(false); }
        
                self.doSflag((cmpval&0xffff) as u16,16);
                self.doZflag(cmpval as u16);
                self.doPflag(cmpval as u16);

                if self.getDflag() { 
                    self.di=self.di.wrapping_sub(2);
                    self.si=self.si.wrapping_sub(2);
                }
                else { 
                    self.di=self.di.wrapping_add(2);
                    self.si=self.si.wrapping_add(2);
                }

                self.ip+=self.decInstr.insLen as u16;
            }
            else
            {
                let data:i32=pmachine.readMemory(self.es,self.di,pvga) as i32;
                let val2compare:i32=pmachine.readMemory(readSeg,self.si,pvga) as i32;

                let cmpval:i32=val2compare-data;
        
                if val2compare<data { self.setCflag(true); }
                else { self.setCflag(false); }
        
                self.doSflag((cmpval&0xff) as u16,8);
                self.doZflag(cmpval as u16);
                self.doPflag(cmpval as u16);

                if self.getDflag() { 
                    self.di=self.di.wrapping_sub(1);
                    self.si=self.si.wrapping_sub(1);
                }
                else { 
                    self.di=self.di.wrapping_add(1);
                    self.si=self.si.wrapping_add(1);
                }
                
                self.ip+=self.decInstr.insLen as u16;
                //self.abort(&format!("Unhandled cmps without rep prefix bits {}",self.decInstr.instrSize));
            }

            return;
        }

        if self.decInstr.instrSize==16
        {
            if self.decInstr.repPrefix=="REPE"
            {
                if self.cx!=0
                {
                    let data:i32=pmachine.readMemory16(self.es,self.di,pvga) as i32;
                    let val2compare:i32=pmachine.readMemory16(readSeg,self.si,pvga) as i32;

                    let cmpval:i32=val2compare-data;
            
                    if val2compare<data { self.setCflag(true); }
                    else { self.setCflag(false); }
            
                    self.doSflag((cmpval&0xffff) as u16,16);
                    self.doZflag(cmpval as u16);
                    self.doPflag(cmpval as u16);

                    if self.getDflag() { 
                        self.di=self.di.wrapping_sub(2);
                        self.si=self.si.wrapping_sub(2);
                    }
                    else { 
                        self.di=self.di.wrapping_add(2);
                        self.si=self.si.wrapping_add(2);
                    }

                     self.cx-=1;
                     if cmpval!=0 { self.ip+=self.decInstr.insLen as u16; return; }
                }
                else
                {
                    self.ip+=self.decInstr.insLen as u16;
                }
            }
            else
            {
                self.abort("Unhandled REP prefix");
            }
        }
        else
        {
            if self.decInstr.repPrefix=="REPE"
            {
                if self.cx!=0
                {
                    let data:i32=pmachine.readMemory(self.es,self.di,pvga) as i32;
                    let val2compare:i32=pmachine.readMemory(readSeg,self.si,pvga) as i32;

                    let cmpval:i32=val2compare-data;
            
                    if val2compare<data { self.setCflag(true); }
                    else { self.setCflag(false); }
            
                    self.doSflag((cmpval&0xff) as u16,8);
                    self.doZflag(cmpval as u16);
                    self.doPflag(cmpval as u16);

                    if self.getDflag() { 
                        self.di=self.di.wrapping_sub(1);
                        self.si=self.si.wrapping_sub(1);
                    }
                    else { 
                        self.di=self.di.wrapping_add(1);
                        self.si=self.si.wrapping_add(1);
                    }

                     self.cx-=1;
                     if cmpval!=0 { self.ip+=2; return; }
                }
                else
                {
                    self.ip+=2;
                }
            }
            else
            {
                self.abort("Unhandled REP prefix");
            }
        }
    }

    fn performXlat(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut readSeg:u16=self.ds;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        let al=self.ax&0xff;
        let tableval:u16=pmachine.readMemory(readSeg,self.bx+al,pvga) as u16;
        self.ax=(self.ax&0xff00)|tableval;

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performJmp(&mut self)
    {
        let mut jumpAmt=0;
        
        if (self.decInstr.instrSize==8) && (jumpAmt==0)
        {
            jumpAmt=self.decInstr.operand1.parse::<i8>().unwrap() as i16;
        }
        else
        {
            jumpAmt=self.decInstr.operand1.parse::<i16>().unwrap() as i16;
        }

        let mut performJump:bool=false;

        if &self.decInstr.debugDecode[0..3]=="JBE" 
        { 
            if self.getZflag() || self.getCflag() 
            { 
                performJump=true; 
            } 
        }
        else if &self.decInstr.debugDecode[0..2]=="JB" { if self.getCflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JO" { if self.getOflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JAE" { if !self.getCflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JE" { if self.getZflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JNE" { if !self.getZflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JA" { if (!self.getCflag()) && (!self.getZflag()) { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JS" { if self.getSflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JNP" { if !self.getPflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JP" { if self.getPflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JNS" { if !self.getSflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JGE" 
        { 
            let of:u8=if self.getOflag() { 1 } else { 0 };
            let sf:u8=if self.getSflag() { 1 } else { 0 };
            let res:bool=if (of^sf)==1 { true } else { false };
            if !res { performJump=true; } 
        }
        else if &self.decInstr.debugDecode[0..2]=="JG" 
        { 
            let of:u8=if self.getOflag() { 1 } else { 0 };
            let sf:u8=if self.getSflag() { 1 } else { 0 };
            let res:bool=if (of^sf)==1 { true } else { false };
            if (self.getZflag()==false) && (!res)
            { 
                performJump=true; 
            } 
        }
        else if &self.decInstr.debugDecode[0..3]=="JLE" { if self.getZflag() || (self.getOflag()!=self.getSflag()) { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JL" 
        { 
            let of:u8=if self.getOflag() { 1 } else { 0 };
            let sf:u8=if self.getSflag() { 1 } else { 0 };
            let res:bool=if (of^sf)==1 { true } else { false };
            if res { performJump=true; } 
        }
        else if &self.decInstr.debugDecode[0..3]=="JMP" { if true { performJump=true; } }
        else if &self.decInstr.debugDecode[0..6]=="LOOPNE" 
        { 
            self.cx=self.cx.wrapping_sub(1); 
            if (self.cx!=0) && (self.getZflag()==false) { performJump=true; } 
        }
        else if &self.decInstr.debugDecode[0..6]=="LOOPE" 
        { 
            self.cx=self.cx.wrapping_sub(1); 
            if (self.cx!=0) && (self.getZflag()==true) { performJump=true; } 
        }
        else if &self.decInstr.debugDecode[0..4]=="LOOP" 
        { 
            self.cx=self.cx.wrapping_sub(1); 
            if self.cx!=0 { performJump=true; } 
        }
        else if &self.decInstr.debugDecode[0..4]=="JCXZ" { if self.cx==0 { performJump=true; } }
        else
        {
            self.abort(&format!("Unhandled jump instr {}",self.decInstr.debugDecode));
        }

        if performJump
        {
            self.ip=self.ip.wrapping_add((jumpAmt+2) as u16);
        }
        else
        {
            self.ip+=self.decInstr.insLen as u16;
        }
    }

    fn performCallFar(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut readSeg:u16=self.ds;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        let mut adr=0;
        let mut seg=0;
        let op1=self.decInstr.operand1.clone();

        if op1=="Direct Addr"
        {
            let offset16=self.decInstr.directAddr;
            adr=pmachine.readMemory16(readSeg,offset16,pvga);
            seg=pmachine.readMemory16(readSeg,offset16+2,pvga);
        }
        else if op1.contains("[BX+Disp]")
        {
            let offset16=self.bx+self.decInstr.displacement as u16;
            adr=pmachine.readMemory16(readSeg,offset16,pvga);
            seg=pmachine.readMemory16(readSeg,offset16+2,pvga);
        }
        else if op1.contains("[DI+Disp]")
        {
            let offset16=self.di+self.decInstr.displacement as u16;
            adr=pmachine.readMemory16(readSeg,offset16,pvga);
            seg=pmachine.readMemory16(readSeg,offset16+2,pvga);
        }
        else if op1.contains("[BP+Disp]")
        {
            let mut readSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; }
    
            let offset16=self.bp+self.decInstr.displacement as u16;
            adr=pmachine.readMemory16(readSeg,offset16,pvga);
            seg=pmachine.readMemory16(readSeg,offset16+2,pvga);
        }
        else if op1.contains("[BX+DI]")
        {
            let offset16=self.bx+self.di;
            adr=pmachine.readMemory16(readSeg,offset16,pvga);
            seg=pmachine.readMemory16(readSeg,offset16+2,pvga);
        }
        else if op1=="[DI]"
        {
            let offset16=self.di;
            adr=pmachine.readMemory16(readSeg,offset16,pvga);
            seg=pmachine.readMemory16(readSeg,offset16+2,pvga);
        }
        else
        {
            self.abort(&format!("Call far {} at {:04x}:{:04x}",op1,self.cs,self.ip));
        }

        //self.abort(&format!("Call far {} at {:04x}:{:04x}",op1,self.cs,self.ip));

        pmachine.push16(self.cs,self.ss,self.sp);
        self.sp-=2;
        pmachine.push16(self.ip+self.decInstr.insLen as u16,self.ss,self.sp);
        self.sp-=2;

        self.cs=seg;
        self.ip=adr;
    }

    //

    fn getOperandValue(&mut self,regname:&String,pmachine:&mut machine,pvga:&mut vga) -> u16
    {
        let mut readSeg:u16=self.ds;
        if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
        else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
        else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
        else if self.decInstr.segOverride=="ES" { readSeg=self.es; }

        if regname=="AX" { return self.ax; }
        else if regname=="BX" { return self.bx; }
        else if regname=="CX" { return self.cx; }
        else if regname=="DX" { return self.dx; }
        else if regname=="ES" { return self.es; }
        else if regname=="SS" { return self.ss; }
        else if regname=="DS" { return self.ds; }
        else if regname=="CS" { return self.cs; }
        else if regname=="BP" { return self.bp; }
        else if regname=="IP" { return self.ip; }
        else if regname=="DI" { return self.di; }
        else if regname=="SI" { return self.si; }
        else if regname=="SP" { return self.sp; }
        else if regname=="AH" { return (self.ax&0xff00)>>8; }
        else if regname=="BH" { return (self.bx&0xff00)>>8; }
        else if regname=="CH" { return (self.cx&0xff00)>>8; }
        else if regname=="DH" { return (self.dx&0xff00)>>8; }
        else if regname=="AL" { return self.ax&0xff; }
        else if regname=="BL" { return self.bx&0xff; }
        else if regname=="CL" { return self.cx&0xff; }
        else if regname=="DL" { return self.dx&0xff; }
        else if regname=="1" { return 1; }
        else if regname=="Direct Addr" 
        { 
            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,self.decInstr.directAddr,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,self.decInstr.directAddr,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[DI]" 
        { 
            let mut data:u16=pmachine.readMemory16(readSeg,self.di,pvga);
            if self.decInstr.instrSize==8 { data=pmachine.readMemory(readSeg,self.di,pvga) as u16; }
            return data; 
        }
        else if regname=="[SI]" 
        { 
            let mut data:u16=pmachine.readMemory16(readSeg,self.si,pvga);
            if self.decInstr.instrSize==8 { data=pmachine.readMemory(readSeg,self.si,pvga) as u16; }
            return data; 
        }
        else if regname=="[BX]" 
        { 
            let mut data:u16=pmachine.readMemory16(readSeg,self.bx,pvga);
            if self.decInstr.instrSize==8 { data=pmachine.readMemory(readSeg,self.bx,pvga) as u16; }
            return data; 
        }
        else if regname=="[BX+DI]" 
        { 
            let mut offs32:i32=self.bx as i32;
            offs32+=self.di as i32;
            offs32&=0xffff;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,offs32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,offs32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[BX+SI]" 
        { 
            let mut offs32:i32=self.bx as i32;
            offs32+=self.si as i32;
            offs32&=0xffff;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,offs32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,offs32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname.contains("[BX+Disp] with")
        { 
            let mut bx32:i32=self.bx as i32;
            bx32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,bx32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,bx32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname.contains("[BX+SI+Disp]")
        { 
            let mut bx32:i32=self.bx as i32;
            bx32+=self.si as i32;
            bx32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,bx32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,bx32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname.contains("[BX+DI+Disp]")
        { 
            let mut bx32:i32=self.bx as i32;
            bx32+=self.di as i32;
            bx32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,bx32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,bx32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname.contains("[DI+Disp]")
        { 
            let mut di32:i32=self.di as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,di32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,di32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname.contains("[SI+Disp]")
        { 
            let mut di32:i32=self.si as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,di32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,di32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname.contains("[BP+Disp]")
        { 
            let mut readSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; }
    
            let mut bp32:i32=self.bp as i32;
            bp32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,bp32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,bp32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname.contains("[BP+SI+Disp]")
        { 
            let mut readSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; }
    
            let mut bp32:i32=self.bp as i32;
            bp32+=self.si as i32;
            bp32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,bp32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,bp32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[BP+DI]" 
        { 
            let mut readSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; }
    
            let mut bp32:i32=self.bp as i32;
            bp32+=self.di as i32;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,bp32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,bp32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname.contains("[BP+DI+Disp]")
        { 
            let mut readSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; }
    
            let mut bp32:i32=self.bp as i32;
            bp32+=self.di as i32;
            bp32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,bp32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,bp32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[BP+SI]" 
        { 
            let mut readSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; }
    
            let mut bp32:i32=self.bp as i32;
            bp32+=self.si as i32;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(readSeg,bp32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(readSeg,bp32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname=="ib" 
        { 
            return self.decInstr.u8immediate as u16; 
        }
        else if (regname=="iw") || (regname=="eb")
        { 
            return self.decInstr.u16immediate as u16; 
        }
        else
        {
            self.abort(&format!("Unhandled getOperandValue {} at {:04x}",regname,self.ip));
        }

        return 0;
    }

    fn moveToDestination(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        if dstReg=="AX" { self.ax=*srcVal; }
        else if dstReg=="BX" { self.bx=*srcVal; }
        else if dstReg=="CX" { self.cx=*srcVal; }
        else if dstReg=="DX" { self.dx=*srcVal; }
        else if dstReg=="ES" { self.es=*srcVal; }
        else if dstReg=="SS" { self.ss=*srcVal; }
        else if dstReg=="DS" { self.ds=*srcVal; }
        else if dstReg=="CS" { self.cs=*srcVal; }
        else if dstReg=="BP" { self.bp=*srcVal; }
        else if dstReg=="IP" { self.ip=*srcVal; }
        else if dstReg=="DI" { self.di=*srcVal; }
        else if dstReg=="SI" { self.si=*srcVal; }
        else if dstReg=="SP" { self.sp=*srcVal; }
        else if dstReg=="AL" { self.ax=(self.ax&0xff00)|((*srcVal)&0xff); }
        else if dstReg=="AH" { self.ax=(self.ax&0xff)|((*srcVal)<<8); }
        else if dstReg=="BL" { self.bx=(self.bx&0xff00)|((*srcVal)&0xff); }
        else if dstReg=="BH" { self.bx=(self.bx&0xff)|((*srcVal)<<8); }
        else if dstReg=="CL" { self.cx=(self.cx&0xff00)|((*srcVal)&0xff); }
        else if dstReg=="CH" { self.cx=(self.cx&0xff)|(((*srcVal)&0xff)<<8); }
        else if dstReg=="DL" { self.dx=(self.dx&0xff00)|((*srcVal)&0xff); }
        else if dstReg=="DH" { self.dx=(self.dx&0xff)|((*srcVal)<<8); }
        else if dstReg=="Direct Addr"
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }
    
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,self.decInstr.directAddr,*srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,self.decInstr.directAddr,(*srcVal&0xff) as u8,pvga); }
        }
        else if dstReg=="[SI]"
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,self.si,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,self.si,*srcVal as u16,pvga); }
        }
        else if dstReg=="[DI]"
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,self.di,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,self.di,*srcVal as u16,pvga); }
        }
        else if dstReg=="[BX]"
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,self.bx,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,self.bx,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[DI+Disp]")
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.di as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BX+DI+Disp]")
        {
            if self.decInstr.segOverride!=""
            {
                self.abort("unhandled seg override");
            }
            let mut di32:i32=self.di as i32;
            di32+=self.decInstr.displacement;
            di32+=self.bx as i32;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BX+DI]")
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.di as i32;
            di32+=self.bx as i32;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BX+SI]")
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.si as i32;
            di32+=self.bx as i32;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[SI+Disp]")
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.si as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BP+Disp]")
        {
            let mut writeSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.bp as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BP+SI+Disp]")
        {
            let mut writeSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.si as i32;
            di32+=self.decInstr.displacement;
            di32+=self.bp as i32;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BX+Disp]")
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.bx as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BX+SI+Disp]")
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.bx as i32;
            di32+=self.si as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BP+DI]")
        {
            let mut writeSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.bp as i32;
            di32+=self.di as i32;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BP+SI]")
        {
            let mut writeSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.bp as i32;
            di32+=self.si as i32;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else if dstReg.contains("[BP+DI+Disp]")
        {
            let mut writeSeg:u16=self.ss;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }

            let mut di32:i32=self.bp as i32;
            di32+=self.di as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,di32 as u16,*srcVal as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,di32 as u16,*srcVal as u16,pvga); }
        }
        else
        {
            self.abort(&format!("Unhandled moveToDestination {} {} at {:04x}",dstReg,srcVal,self.ip));
        }
    }

    fn doCmp(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let val2compare:i32=self.getOperandValue(&dstReg,pmachine,pvga) as i32;

        let data:i32=*srcVal as i32; 
        let cmpval:i32=val2compare-data;

        if self.decInstr.instrSize==8 
        { 
            self.doSflag((cmpval&0xff) as u16,8); 

            let diff_sign = ((val2compare ^ data) & 0x80) == 0x80;
            self.setOflag(diff_sign && (((cmpval ^ data) & 0x80) != 0x80));
            let (ms, ma) = (cmpval & 0xf, val2compare & 0xf);
            self.setAflag(ms > ma);
        }
        else if self.decInstr.instrSize==16 
        { 
            self.doSflag((cmpval&0xffff) as u16,16); 

            let diff_sign = ((val2compare ^ data) & 0x8000) == 0x8000;
            self.setOflag(diff_sign && (((cmpval ^ data) & 0x8000) != 0x8000));
            let (ms, ma) = (cmpval & 0xf, val2compare & 0xf);
            self.setAflag(ms > ma);
        }

        if val2compare<data { self.setCflag(true); }
        else { self.setCflag(false); }

        self.doSflag(cmpval as u16,self.decInstr.instrSize);
        self.doZflag(cmpval as u16);
        self.doPflag(cmpval as u16);
    }

    fn doTest(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let val2compare:i32=self.getOperandValue(dstReg,pmachine,pvga) as i32;
        let data:i32=*srcVal as i32; 
        let cmpval:i32=val2compare&data;
        self.doZflag(cmpval as u16);
        self.doPflag(cmpval as u16);
        self.doSflag(cmpval as u16,self.decInstr.instrSize);
        self.setCflag(false);
    }

    fn doAnd(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let rop:u16=*srcVal;
        let mut lop:u16=self.getOperandValue(dstReg,pmachine,pvga);
        lop&=rop;
        self.moveToDestination(&lop,&dstReg,pmachine,pvga);

        self.doZflag(lop as u16);
        self.doPflag(lop as u16);
        self.doSflag(lop,self.decInstr.instrSize);
        self.setCflag(false);
    }

    fn doAdd(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO oa flags
        let valtoadd:i32=*srcVal as i32;
        let mut ax32:i32=self.getOperandValue(&dstReg,pmachine,pvga) as i32;

        let result:i32=ax32+valtoadd;

        if self.decInstr.instrSize==8 { if ((result&0xff)<ax32)||((result&0xff)<valtoadd) { self.setCflag(true); } else { self.setCflag(false); } }
        else if self.decInstr.instrSize==16 { if ((result&0xffff)<ax32)||((result&0xffff)<valtoadd) { self.setCflag(true); } else { self.setCflag(false); } }

        self.doOAflagsAdd(ax32 as u16,valtoadd as u16,(ax32+valtoadd) as u16);
        
        ax32+=valtoadd;

        self.moveToDestination(&(ax32 as u16),&dstReg,pmachine,pvga); 
        let rez:u16=ax32 as u16;

        self.doZflag(rez);
        self.doPflag(rez);
        self.doSflag(rez,self.decInstr.instrSize);
    }

    fn doAdc(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let carry:i32=if self.getCflag() { 1 } else { 0 };

        let op=self.getOperandValue(&dstReg,pmachine,pvga) as i32;
        let op2:i32=*srcVal as i32;
        let mut res:i32=op+op2+carry;
        self.moveToDestination(&(res as u16),&dstReg,pmachine,pvga);

        if self.decInstr.instrSize==8 { res&=0xff; }
        else { res&=0xffff; }

        self.doOAflagsAdd(op as u16,op2 as u16,res as u16);

        // TODO oca flags
        self.doZflag(res as u16);
        self.doSflag(res as u16,self.decInstr.instrSize);
        self.doPflag(res as u16);
    }

    fn doPush(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 

        if self.decInstr.instrSize==16
        {
            pmachine.push16(srcVal,self.ss,self.sp);
            self.sp-=2;
        }
        else if self.decInstr.instrSize==8
        {
            self.abort("8 bit push does not exist (does it?)");
        }
    }

    fn doPop(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let popdval=pmachine.pop16(self.ss,self.sp);
        let dstReg=self.decInstr.operand1.clone();

        if self.decInstr.instrSize==16
        {
            self.moveToDestination(&popdval,&dstReg,pmachine,pvga);
            self.sp+=2;
        }
        else if self.decInstr.instrSize==8
        {
            self.abort("8 bit pop does not exist (or not?)");
        }
    }

    fn doOr(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut lop=self.getOperandValue(&dstReg,pmachine,pvga);
        lop|=*srcVal;
        self.moveToDestination(&lop,&dstReg,pmachine,pvga);

        self.doSflag(lop as u16,self.decInstr.instrSize);
        self.doZflag(lop);
        self.doPflag(lop);
        self.setCflag(false);
        self.setOflag(false);
    }

    fn doSbb(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let op1=*srcVal;        
        let lop=self.getOperandValue(&dstReg,pmachine,pvga);

        let mut cf=0;
        if self.getCflag() { cf=1; }

        if self.decInstr.instrSize==16
        {
            let mut result:u16=lop.wrapping_sub(op1);
            result=result.wrapping_sub(cf);

            let diff_sign = ((lop ^ op1) & 0x8000) == 0x8000;
            self.setOflag(diff_sign && (((result ^ op1) & 0x8000) != 0x8000));
            let (ms, ma) = (result & 0xf, lop & 0xf);
            self.setAflag(ms > ma);

            self.moveToDestination(&result,&dstReg,pmachine,pvga); 
            if (result&0x8000)==0x8000
            {
                self.setCflag(true);
            }        
            else
            {
                self.setCflag(false);
            }
            self.doSflag(result,self.decInstr.instrSize);
            self.doZflag(result);
            self.doPflag(result);
        }
        else if self.decInstr.instrSize==8
        {
            let op18:u8=(op1&0xff) as u8;
            let lop8:u8=(lop&0xff) as u8;
            let mut result8:u8=lop8.wrapping_sub(op18);
            result8=result8.wrapping_sub(cf as u8);

            let diff_sign = ((lop8 ^ op18) & 0x80) == 0x80;
            self.setOflag(diff_sign && (((result8 ^ op18) & 0x80) != 0x80));
            let (ms, ma) = (result8 & 0xf, lop8 & 0xf);
            self.setAflag(ms > ma);

            self.moveToDestination(&(result8 as u16),&dstReg,pmachine,pvga); 
            if (result8&0x80)==0x80
            {
                self.setCflag(true);
            }        
            else
            {
                self.setCflag(false);
            }

            self.doSflag(result8 as u16,self.decInstr.instrSize);
            self.doZflag(result8 as u16);
            self.doPflag(result8 as u16);
        }
    }

    fn doXor(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let op1=*srcVal;
        let mut op2=self.getOperandValue(&dstReg,pmachine,pvga);
        op2^=op1;
        self.moveToDestination(&op2,&dstReg,pmachine,pvga); 

        self.setCflag(false);
        self.setOflag(false);
        self.doZflag(op2);
        self.doPflag(op2);
        self.doSflag(op2,self.decInstr.instrSize);
    }

    fn doSub(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut result:u16=0;

        let val2compare=self.getOperandValue(&dstReg,pmachine,pvga);
        if *srcVal>val2compare
        {
            if (self.decInstr.instrSize==8) && (result==0)
            {
                let isrc:i16=*srcVal as i16;
                let iv2c:i16=val2compare as i16;
                result=(iv2c-isrc) as u8 as u16;
            }
            else
            {
                let isrc:i32=*srcVal as i32;
                let iv2c:i32=val2compare as i32;
                result=(iv2c-isrc) as u16;
            }
            self.setCflag(true);
        }
        else
        {
            result=val2compare-(*srcVal);
            self.setCflag(false);
        }
        self.moveToDestination(&result,&dstReg,pmachine,pvga);

        if self.decInstr.instrSize==8
        {
            let diff_sign = ((val2compare ^ *srcVal) & 0x80) == 0x80;
            self.setOflag(diff_sign && (((result ^ *srcVal) & 0x80) != 0x80));
            let (ms, ma) = (result & 0xf, val2compare & 0xf);
            self.setAflag(ms > ma);
        }
        else
        {
            let diff_sign = ((val2compare ^ *srcVal) & 0x8000) == 0x8000;
            self.setOflag(diff_sign && (((result ^ *srcVal) & 0x8000) != 0x8000));
            let (ms, ma) = (result & 0xf, val2compare & 0xf);
            self.setAflag(ms > ma);
        }

        self.doZflag(result);
        self.doPflag(result);
        self.doSflag(result,self.decInstr.instrSize);
    }

    fn performMove(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.moveToDestination(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performSub(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doSub(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performSbb(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doSbb(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performAnd(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doAnd(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performCompare(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doCmp(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performTest(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doTest(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performIn(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        if dstReg=="AL"
        { 
            pmachine.handleIn(self,pvga,self.decInstr.u8immediate,self.decInstr.u16immediate,self.decInstr.instrSize);
        }
        else if dstReg=="AX"
        { 
            // TODO IN from timer
            if self.decInstr.u8immediate==0x40
            {
                let num:u16 = rand::thread_rng().gen_range(0..256);
                self.ax=num;
            }
        }
        else
        {
            self.abort(&format!("Unhandled IN [{}] [{}]",dstReg,srcReg));
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performOut(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        pmachine.handleOut(self,pvga,self.decInstr.u8immediate,self.decInstr.u16immediate,self.decInstr.instrSize);

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performAdd(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doAdd(&srcVal,&dstReg,pmachine,pvga);

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performAdc(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doAdc(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performOr(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doOr(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performXor(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 

        self.doXor(&srcVal,&dstReg,pmachine,pvga); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn getInstructionType(&self,it:&String) -> instructionType
    {
        if it=="Pop" { return instructionType::instrPop; }
        else if it=="PopNMRR" { return instructionType::instrPopNoModRegRm; }
        else if it=="Push" { return instructionType::instrPush; }
        else if it=="PushNMRR" { return instructionType::instrPushNoModRegRm; }
        else if it=="Pushf" { return instructionType::instrPushf; }
        else if it=="Popf" { return instructionType::instrPopf; }
        else if it=="Ret" { return instructionType::instrRet; }
        else if it=="Clc" { return instructionType::instrClc; }
        else if it=="Cld" { return instructionType::instrCld; }
        else if it=="Std" { return instructionType::instrStd; }
        else if it=="Inc" { return instructionType::instrInc; }
        else if it=="Dec" { return instructionType::instrDec; }
        else if it=="IncNMRR" { return instructionType::instrIncNoModRegRm; }
        else if it=="DecNMRR" { return instructionType::instrDecNoModRegRm; }
        else if it=="Xchg" { return instructionType::instrXchg; }
        else if it=="XchgNMRR" { return instructionType::instrXchgNoModRegRm; }
        else if it=="Lods" { return instructionType::instrLods; }
        else if it=="Movs" { return instructionType::instrMovs; }
        else if it=="Stos" { return instructionType::instrStos; }
        else if it=="Scas" { return instructionType::instrScas; }
        else if it=="Cmps" { return instructionType::instrCmps; }
        else if it=="JmpShort" { return instructionType::instrJmpShort; }
        else if it=="JmpNear" { return instructionType::instrJmpNear; }
        else if it=="JmpNp" { return instructionType::instrJmpNp; }
        else if it=="Int" { return instructionType::instrInt; }
        else if it=="CallReg" { return instructionType::instrCallReg; }
        else if it=="CallRel16" { return instructionType::instrCallRel16; }
        else if it=="CallFar" { return instructionType::instrCallFar; }
        else if it=="CallFarPtr" { return instructionType::instrCallFarPtr; }
        else if it=="Mov" { return instructionType::instrMov; }
        else if it=="MovNMRR" { return instructionType::instrMovNoModRegRm; }
        else if it=="Sub" { return instructionType::instrSub; }
        else if it=="SubNMRR" { return instructionType::instrSubNoModRegRm; }
        else if it=="Sbb" { return instructionType::instrSbb; }
        else if it=="And" { return instructionType::instrAnd; }
        else if it=="AndNMRR" { return instructionType::instrAndNoModRegRm; }
        else if it=="Add" { return instructionType::instrAdd; }
        else if it=="AddNMRR" { return instructionType::instrAddNoModRegRm; }
        else if it=="Adc" { return instructionType::instrAdc; }
        else if it=="AdcNMRR" { return instructionType::instrAdcNoModRegRm; }
        else if it=="Xlat" { return instructionType::instrXlat; }
        else if it=="In" { return instructionType::instrIn; }
        else if it=="Or" { return instructionType::instrOr; }
        else if it=="OrNMRR" { return instructionType::instrOrNoModRegRm; }
        else if it=="Xor" { return instructionType::instrXor; }
        else if it=="XorNMRR" { return instructionType::instrXorNoModRegRm; }
        else if it=="Cmp" { return instructionType::instrCmp; }
        else if it=="Test" { return instructionType::instrTest; }
        else if it=="TestNMRR" { return instructionType::instrTestNoModRegRm; }
        else if it=="CmpNMRR" { return instructionType::instrCmpNoModRegRm; }
        else if it=="Shl" { return instructionType::instrShl; }
        else if it=="Shr" { return instructionType::instrShr; }
        else if it=="Cwd" { return instructionType::instrCwd; }
        else if it=="Neg" { return instructionType::instrNeg; }
        else if it=="Not" { return instructionType::instrNot; }
        else if it=="Imul" { return instructionType::instrImul; }
        else if it=="Mul" { return instructionType::instrMul; }
        else if it=="Div" { return instructionType::instrDiv; }
        else if it=="Lea" { return instructionType::instrLea; }
        else if it=="Out" { return instructionType::instrOut; }
        else if it=="OutNMRR" { return instructionType::instrOutNoModRegRm; }
        else if it=="Cbw" { return instructionType::instrCbw; }
        else if it=="Cmc" { return instructionType::instrCmc; }
        else if it=="Stc" { return instructionType::instrStc; }
        else if it=="Cli" { return instructionType::instrCli; }
        else if it=="Sti" { return instructionType::instrSti; }
        else if it=="Ljmp" { return instructionType::instrLongJump; }
        else if it=="Nop" { return instructionType::instrNop; }
        else if it=="Lds" { return instructionType::instrLds; }
        else if it=="Les" { return instructionType::instrLes; }
        else if it=="Ror" { return instructionType::instrRor; }
        else if it=="Rol" { return instructionType::instrRol; }
        else if it=="Rcr" { return instructionType::instrRcr; }
        else if it=="Rcl" { return instructionType::instrRcl; }
        else if it=="Pusha" { return instructionType::instrPusha; }
        else if it=="Popa" { return instructionType::instrPopa; }
        else if it=="Retf" { return instructionType::instrRetf; }
        else if it=="Retfiw" { return instructionType::instrRetfiw; }
        else if it=="Iret" { return instructionType::instrIret; }
        else if it=="Jumpnw" { return instructionType::instrJumpnw; }
        else if it=="Jumpfw" { return instructionType::instrJumpfw; }
        else if it=="Lahf" { return instructionType::instrLahf; }
        else if it=="Sahf" { return instructionType::instrSahf; }
        else if it=="Aad" { return instructionType::instrAad; }
        else if it=="Aam" { return instructionType::instrAam; }
        else if it=="Sar" { return instructionType::instrSar; }
        else if it=="Retiw" { return instructionType::instrRetiw; }
        else if it=="Idiv" { return instructionType::instrIdiv; }
        else if it=="Das" { return instructionType::instrDas; }
        else if it=="Daa" { return instructionType::instrDaa; }
        else if it=="Aaa" { return instructionType::instrAaa; }
        else if it=="Fninit" { return instructionType::instrFninit; }
        else if it=="Fnstsw" { return instructionType::instrFnstsw; }
        else if it=="SbbNMRR" { return instructionType::instrSbbNoModRegRm; }
        else { return instructionType::instrNone; }
    }

    fn getOpcodeStructure(&self,opcode:&u16,found:&mut bool) -> [&str;7]
    {
        // opcode info format:
        // dbg instruction, instruction bit size, number of arguments, arg1, arg2, instruction type, invert operands

        match opcode
        {
            // POP 16 bit reg
            0x07 => { return ["POP ES","16","1","ES","","PopNMRR","0"]; }
            0x17 => { return ["POP SS","16","1","SS","","PopNMRR","0"]; }
            0x1f => { return ["POP DS","16","1","DS","","PopNMRR","0"]; }
            0x58 => { return ["POP AX","16","1","AX","","PopNMRR","0"]; }
            0x59 => { return ["POP CX","16","1","CX","","PopNMRR","0"]; }
            0x5a => { return ["POP DX","16","1","DX","","PopNMRR","0"]; }
            0x5b => { return ["POP BX","16","1","BX","","PopNMRR","0"]; }
            0x5c => { return ["POP SP","16","1","SP","","PopNMRR","0"]; }
            0x5d => { return ["POP BP","16","1","BP","","PopNMRR","0"]; }
            0x5e => { return ["POP SI","16","1","SI","","PopNMRR","0"]; }
            0x5f => { return ["POP DI","16","1","DI","","PopNMRR","0"]; }
            0x8f => { return ["POP","16","1","rmw","","Pop","1"]; }
            // PUSH 16 bit reg
            0x06 => { return ["PUSH ES","16","1","ES","","PushNMRR","0"]; }
            0x0e => { return ["PUSH CS","16","1","CS","","PushNMRR","0"]; }
            0x16 => { return ["PUSH SS","16","1","SS","","PushNMRR","0"]; }
            0x1e => { return ["PUSH DS","16","1","DS","","PushNMRR","0"]; }
            0x50 => { return ["PUSH AX","16","1","AX","","PushNMRR","0"]; }
            0x51 => { return ["PUSH CX","16","1","CX","","PushNMRR","0"]; }
            0x52 => { return ["PUSH DX","16","1","DX","","PushNMRR","0"]; }
            0x53 => { return ["PUSH BX","16","1","BX","","PushNMRR","0"]; }
            0x54 => { return ["PUSH SP","16","1","SP","","PushNMRR","0"]; }
            0x55 => { return ["PUSH BP","16","1","BP","","PushNMRR","0"]; }
            0x56 => { return ["PUSH SI","16","1","SI","","PushNMRR","0"]; }
            0x57 => { return ["PUSH DI","16","1","DI","","PushNMRR","0"]; }
            // PUSHA/POPA
            0x60 => { return ["PUSHA","16","0","","","Pusha","0"]; }
            0x61 => { return ["POPA","16","0","","","Popa","0"]; }
            // POPF, PUSHF
            0x9c => { return ["PUSHF","16","0","","","Pushf","0"]; }
            0x9d => { return ["POPF","16","0","","","Popf","0"]; }
            // boring mono-opcode instructions
            0xc3 => { return ["RET","16","0","","","Ret","0"]; }
            0xf8 => { return ["CLC","16","0","","","Clc","0"]; }
            0xfc => { return ["CLD","16","0","","","Cld","0"]; }
            0x98 => { return ["CBW","16","0","","","Cbw","0"]; }
            0x99 => { return ["CWD","16","0","","","Cwd","0"]; }
            0xf5 => { return ["CMC","16","0","","","Cmc","0"]; }
            0xf9 => { return ["STC","16","0","","","Stc","0"]; }
            0xfa => { return ["CLI","16","0","","","Cli","0"]; }
            0xfb => { return ["STI","16","0","","","Sti","0"]; }
            0xfd => { return ["STD","16","0","","","Std","0"]; }
            // INC 16bit reg
            0x40 => { return ["INC AX","16","1","AX","","IncNMRR","0"]; }
            0x41 => { return ["INC CX","16","1","CX","","IncNMRR","0"]; }
            0x42 => { return ["INC DX","16","1","DX","","IncNMRR","0"]; }
            0x43 => { return ["INC BX","16","1","BX","","IncNMRR","0"]; }
            0x44 => { return ["INC SP","16","1","SP","","IncNMRR","0"]; }
            0x45 => { return ["INC BP","16","1","BP","","IncNMRR","0"]; }
            0x46 => { return ["INC SI","16","1","SI","","IncNMRR","0"]; }
            0x47 => { return ["INC DI","16","1","DI","","IncNMRR","0"]; }
            // DEC 16bit reg
            0x48 => { return ["DEC AX","16","1","AX","","DecNMRR","0"]; }
            0x49 => { return ["DEC CX","16","1","CX","","DecNMRR","0"]; }
            0x4a => { return ["DEC DX","16","1","DX","","DecNMRR","0"]; }
            0x4b => { return ["DEC BX","16","1","BX","","DecNMRR","0"]; }
            0x4c => { return ["DEC SP","16","1","SP","","DecNMRR","0"]; }
            0x4d => { return ["DEC BP","16","1","BP","","DecNMRR","0"]; }
            0x4e => { return ["DEC SI","16","1","SI","","DecNMRR","0"]; }
            0x4f => { return ["DEC DI","16","1","DI","","DecNMRR","0"]; }
            // XCHG 
            0x86 => { return ["XCHG","8","2","rb","rmb","Xchg","0"]; }
            0x87 => { return ["XCHG","16","2","rw","rmw","Xchg","0"]; }
            0x91 => { return ["XCHG AX,CX","16","2","AX","CX","XchgNMRR","0"]; }
            0x92 => { return ["XCHG AX,DX","16","2","AX","DX","XchgNMRR","0"]; }
            0x93 => { return ["XCHG AX,BX","16","2","AX","BX","XchgNMRR","0"]; }
            0x94 => { return ["XCHG AX,SP","16","2","AX","SP","XchgNMRR","0"]; }
            0x95 => { return ["XCHG AX,BP","16","2","AX","BP","XchgNMRR","0"]; }
            0x96 => { return ["XCHG AX,SI","16","2","AX","SI","XchgNMRR","0"]; }
            0x97 => { return ["XCHG AX,DI","16","2","AX","DI","XchgNMRR","0"]; }
            // LODSB/W
            0xac => { return ["LODSB","8","0","","","Lods","0"]; }
            0xad => { return ["LODSW","16","0","","","Lods","0"]; }
            // MOVSB/W
            0xa4 => { return ["MOVSB","8","0","","","Movs","0"]; }
            0xa5 => { return ["MOVSW","16","0","","","Movs","0"]; }
            // CMPS
            0xa6 => { return ["CMPSB","8","0","","","Cmps","0"]; }
            0xa7 => { return ["CMPSW","16","0","","","Cmps","0"]; }
            // STOSB/W
            0xaa => { return ["STOSB","8","0","","","Stos","0"]; }
            0xab => { return ["STOSW","16","0","","","Stos","0"]; }
            // SCASB/W
            0xae => { return ["SCASB","8","0","","","Scas","0"]; }
            0xaf => { return ["SCASW","16","0","","","Scas","0"]; }
            // Jump short
            0x70 => { return ["JO","8","1","r0","","JmpShort","0"]; }
            0x72 => { return ["JB","8","1","r0","","JmpShort","0"]; }
            0x73 => { return ["JAE","8","1","r0","","JmpShort","0"]; }
            0x74 => { return ["JE","8","1","r0","","JmpShort","0"]; }
            0x75 => { return ["JNE","8","1","r0","","JmpShort","0"]; }
            0x76 => { return ["JBE","8","1","r0","","JmpShort","0"]; }
            0x77 => { return ["JA","8","1","r0","","JmpShort","0"]; }
            0x78 => { return ["JS","8","1","r0","","JmpShort","0"]; }
            0x79 => { return ["JNS","8","1","r0","","JmpShort","0"]; }
            0x7A => { return ["JP","8","1","r0","","JmpShort","0"]; }
            0x7B => { return ["JNP","8","1","r0","","JmpShort","0"]; }
            0x7C => { return ["JL","8","1","r0","","JmpShort","0"]; }
            0x7D => { return ["JGE","8","1","r0","","JmpShort","0"]; }
            0x7E => { return ["JLE","8","1","r0","","JmpShort","0"]; }
            0x7F => { return ["JG","8","1","r0","","JmpShort","0"]; }
            0xE0 => { return ["LOOPNE","8","1","r0","","JmpShort","0"]; }
            0xE1 => { return ["LOOPE","8","1","r0","","JmpShort","0"]; }
            0xE2 => { return ["LOOP Short","8","1","r0","","JmpShort","0"]; }
            0xE3 => { return ["JCXZ","8","1","r0","","JmpShort","0"]; }
            0xEB => { return ["JMP Short","8","1","r0","","JmpShort","0"]; }
            // INT nn
            0xcd => { return ["INT","8","1","intnum","","Int","0"]; }
            // CALL 16bit relative offset
            0xe8 => { return ["CALL","16","1","r0","","CallRel16","0"]; }
            // MOV instructions (with modregrm byte)
            0x88 => { return ["MOV","8","2","rb","rmb","Mov","0"]; }
            0x8a => { return ["MOV","8","2","rb","rmb","Mov","1"]; }
            0x89 => { return ["MOV","16","2","rmw","rw","Mov","0"]; }
            0x8e => { return ["MOV","16","2","sr","rmw","Mov","1"]; }
            0x8b => { return ["MOV","16","2","rw","rmw","Mov","1"]; }
            0x8c => { return ["MOV","16","2","rmw","sr","Mov","0"]; }
            0xc6 => { return ["MOV","8","2","ib","rmb","Mov","0"]; }
            0xc7 => { return ["MOV","16","2","iw","rmw","Mov","0"]; }
            // MOV instructions (without modregrm byte)
            0xa1 => { return ["MOV","16","2","Direct Addr","AX","MovNMRR","0"]; }
            0xa2 => { return ["MOV","8","2","AL","Direct Addr","MovNMRR","0"]; }
            0xb0 => { return ["MOV","8","2","ib","AL","MovNMRR","0"]; }
            0xb1 => { return ["MOV","8","2","ib","CL","MovNMRR","0"]; }
            0xb2 => { return ["MOV","8","2","ib","DL","MovNMRR","0"]; }
            0xb3 => { return ["MOV","8","2","ib","BL","MovNMRR","0"]; }
            0xb4 => { return ["MOV","8","2","ib","AH","MovNMRR","0"]; }
            0xb5 => { return ["MOV","8","2","ib","CH","MovNMRR","0"]; }
            0xb6 => { return ["MOV","8","2","ib","DH","MovNMRR","0"]; }
            0xb7 => { return ["MOV","8","2","ib","BH","MovNMRR","0"]; }
            0xb8 => { return ["MOV","16","2","iw","AX","MovNMRR","0"]; }
            0xb9 => { return ["MOV","16","2","iw","CX","MovNMRR","0"]; }
            0xba => { return ["MOV","16","2","iw","DX","MovNMRR","0"]; }
            0xbb => { return ["MOV","16","2","iw","BX","MovNMRR","0"]; }
            0xbc => { return ["MOV","16","2","iw","SP","MovNMRR","0"]; }
            0xbd => { return ["MOV","16","2","iw","BP","MovNMRR","0"]; }
            0xbe => { return ["MOV","16","2","iw","SI","MovNMRR","0"]; }
            0xbf => { return ["MOV","16","2","iw","DI","MovNMRR","0"]; }
            0xa0 => { return ["MOV","8","2","Direct Addr","AL","MovNMRR","0"]; }
            0xa3 => { return ["MOV","16","2","AX","Direct Addr","MovNMRR","0"]; }
            // AND instructions
            0x20 => { return ["AND","8","2","rmb","rb","And","0"]; }
            0x21 => { return ["AND","16","2","rw","rmw","And","0"]; }
            0x22 => { return ["AND","8","2","rb","rmb","And","1"]; }
            0x23 => { return ["AND","16","2","rw","rmw","And","1"]; }
            0x24 => { return ["AND","8","2","ib","AL","AndNMRR","0"]; }
            0x25 => { return ["AND","16","2","iw","AX","AndNMRR","0"]; }
            // OR instructions
            0x08 => { return ["OR","8","2","rmb","rb","Or","0"]; }
            0x09 => { return ["OR","16","2","rmw","rw","Or","0"]; }
            0x0A => { return ["OR","8","2","rmb","rb","Or","1"]; }
            0x0B => { return ["OR","16","2","rmw","rw","Or","1"]; }
            0x0C => { return ["OR","8","2","ib","AL","OrNMRR","0"]; }
            0x0D => { return ["OR","16","2","iw","AX","OrNMRR","0"]; }
            // XOR instructions
            0x30 => { return ["XOR","8","2","rb","rmb","Xor","0"]; }
            0x31 => { return ["XOR","16","2","rmw","rw","Xor","0"]; }
            0x32 => { return ["XOR","8","2","rb","rmb","Xor","1"]; }
            0x33 => { return ["XOR","16","2","rw","rmw","Xor","1"]; } // right or inverted?
            0x34 => { return ["XOR","8","2","ib","AL","XorNMRR","1"]; }
            0x35 => { return ["XOR","16","2","iw","AX","XorNMRR","1"]; }
            // XLAT
            0xd7 => { return ["XLAT","16","0","","","Xlat","0"]; }            
            // IN
            0xe4 => { return ["IN","8","2","ib","AL","In","0"]; }            
            0xec => { return ["IN","16","2","DX","AL","In","0"]; }            
            0xe5 => { return ["IN","16","2","ib","AX","In","0"]; }            
            // JMP np
            0xe9 => { return ["JMPNP","16","1","iw","","JmpNp","0"]; }            
            // ADD
            0x00 => { return ["ADD","8","2","rb","rmb","Add","0"]; }
            0x01 => { return ["ADD","16","2","rmw","rw","Add","0"]; }
            0x02 => { return ["ADD","8","2","rb","rmb","Add","1"]; }
            0x03 => { return ["ADD","16","2","rmw","rw","Add","1"]; }
            0x04 => { return ["ADD","8","2","ib","AL","AddNMRR","0"]; }
            0x05 => { return ["ADD","16","2","iw","AX","AddNMRR","0"]; }
            // ADC
            0x10 => { return ["ADC","8","2","rb","rmb","Adc","0"]; }
            0x11 => { return ["ADC","16","2","rw","rmw","Adc","0"]; }
            0x12 => { return ["ADC","8","2","rb","rmb","Adc","1"]; }
            0x13 => { return ["ADC","16","2","rmw","rw","Adc","1"]; }
            0x14 => { return ["ADC","8","2","ib","AL","AdcNMRR","0"]; }
            0x15 => { return ["ADC","16","2","iw","AX","AdcNMRR","0"]; }
            // CMP
            0x38 => { return ["CMP","8","2","rmb","rb","Cmp","0"]; }
            0x39 => { return ["CMP","16","2","rmw","rw","Cmp","0"]; }
            0x3a => { return ["CMP","8","2","rmb","rb","Cmp","1"]; }
            0x3b => { return ["CMP","16","2","rmw","rw","Cmp","1"]; }
            0x3c => { return ["CMP","8","2","ib","AL","CmpNMRR","0"]; }
            0x3d => { return ["CMP","16","2","iw","AX","CmpNMRR","0"]; } // the 3-D instruction
            // SUB
            0x28 => { return ["SUB","8","2","rmb","rb","Sub","0"]; }
            0x29 => { return ["SUB","16","2","rmw","rw","Sub","0"]; }
            0x2a => { return ["SUB","8","2","rb","rmb","Sub","1"]; }
            0x2b => { return ["SUB","16","2","rw","rmw","Sub","1"]; }
            0x2c => { return ["SUB","8","2","ib","AL","SubNMRR","0"]; }
            0x2d => { return ["SUB","16","2","iw","AX","SubNMRR","0"]; } // the bidimensional instruction
            // TEST
            0x84 => { return ["TEST","8","2","rmb","rmb","Test","0"]; }
            0x85 => { return ["TEST","16","2","rmw","rmw","Test","0"]; }
            0xa8 => { return ["TEST","8","2","ib","AL","TestNMRR","0"]; }
            0xa9 => { return ["TEST","16","2","iw","AX","TestNMRR","0"]; }
            // LEA 
            0x8d => { return ["LEA","16","2","rmw","rw","Lea","1"]; }            
            // OUT 
            0xe6 => { return ["OUT","8","2","ib","AL","OutNMRR","0"]; }            
            0xee => { return ["OUT","8","2","AL","DX","OutNMRR","0"]; }            
            0xef => { return ["OUT","16","2","AX","DX","OutNMRR","0"]; }            
            // NOP 
            0x90 => { return ["NOP","8","0","","","Nop","0"]; } // best instruction evah
            // SBB
            0x18 => { return ["SBB","8","2","rmb","rb","Sbb","0"]; }
            0x19 => { return ["SBB","16","2","rmw","rw","Sbb","0"]; }
            0x1a => { return ["SBB","8","2","rb","rmb","Sbb","1"]; }
            0x1b => { return ["SBB","16","2","rw","rmw","Sbb","1"]; }
            0x1c => { return ["SBB","8","2","ib","AL","SbbNMRR","1"]; }
            0x1d => { return ["SBB","16","2","iw","AX","SbbNMRR","1"]; }
            // LONG JUMP
            0xea => { return ["LJMP","16","2","iw","iw","Ljmp","0"]; }
            // LES
            0xc4 => { return ["LES","16","2","rw","rmw","Les","1"]; }
            // LDS
            0xc5 => { return ["LDS","16","2","rw","rmw","Lds","1"]; }
            // another PUSH
            0x68 => { return ["PUSH","16","1","iw","","PushNMRR","0"]; }
            // another PUSH TODO
            0x6a => { return ["PUSH","16","1","eb","","PushNMRR","0"]; }
            // RETiw
            0xc2 => { return ["RETIW","16","1","iw","","Retiw","0"]; }
            // RETF iw
            0xca => { return ["RETFIW","16","1","iw","","Retfiw","0"]; }
            // RETF
            0xcb => { return ["RETF","16","0","","","Retf","0"]; }
            // CALL FAR PTR
            0x9a => { return ["CALL FAR PTR","16","1","","","CallFarPtr","0"]; }
            // IRET
            0xcf => { return ["IRET","16","0","","","Iret","0"]; }
            // (don't) LAHF
            0x9f => { return ["LAHF","8","0","","","Lahf","0"]; }
            // SAHF
            0x9e => { return ["SAHF","8","0","","","Sahf","0"]; }
            // AAM
            0xd4 => { return ["AAM","8","0","","","Aam","0"]; }
            // AAD
            0xd5 => { return ["AAD","16","0","","","Aad","0"]; }
            // DAA
            0x27 => { return ["DAA","8","0","","","Daa","0"]; }
            // DAS
            0x2f => { return ["DAS","8","0","","","Das","0"]; }
            // AAA
            0x37 => { return ["AAA","8","0","","","Aaa","0"]; }

            // Multi-byte instructions
            0x8000 => { return ["ADD","8","2","ib","rmb","Add","0"]; }
            0x8001 => { return ["OR","8","2","ib","rmb","Or","0"]; }
            0x8002 => { return ["ADC","8","2","ib","rmb","Adc","0"]; }
            0x8003 => { return ["SBB","8","2","ib","rmb","Sbb","0"]; }
            0x8004 => { return ["AND","8","2","ib","rmb","And","0"]; }
            0x8005 => { return ["SUB","8","2","ib","rmb","Sub","0"]; }
            0x8006 => { return ["XOR","8","2","ib","rmb","Xor","0"]; }
            0x8007 => { return ["CMP","8","2","ib","rmb","Cmp","0"]; }
            
            0x8100 => { return ["ADD","16","2","iw","rmw","Add","0"]; }
            0x8101 => { return ["OR" ,"16","2","iw","rmw","Or" ,"0"]; }
            0x8102 => { return ["ADC","16","2","iw","rmw","Adc","0"]; }
            0x8103 => { return ["SBB","16","2","iw","rmw","Sbb","0"]; }
            0x8104 => { return ["AND","16","2","iw","rmw","And","0"]; }
            0x8105 => { return ["SUB","16","2","iw","rmw","Sub","0"]; }
            0x8106 => { return ["XOR","16","2","iw","rmw","Xor","0"]; }
            0x8107 => { return ["CMP","16","2","iw","rmw","Cmp","0"]; }

            0x8207 => { return ["CMP","16","2","ib","rmw","Cmp","0"]; }

            0x8300 => { return ["ADD","16","2","eb","rmw","Add","0"]; }
            0x8301 => { return ["OR" ,"16","2","eb","rmw","Or" ,"0"]; }
            0x8302 => { return ["ADC","16","2","eb","rmw","Adc","0"]; }
            0x8303 => { return ["SBB","16","2","eb","rmw","Sbb","0"]; }
            0x8304 => { return ["AND","16","2","eb","rmw","And","0"]; }
            0x8305 => { return ["SUB","16","2","eb","rmw","Sub","0"]; }
            0x8306 => { return ["XOR","16","2","eb","rmw","Xor","0"]; }
            0x8307 => { return ["CMP","16","2","eb","rmw","Cmp","0"]; }

            0xc004 => { return ["SHL","8","2","rmb","ib","Shl","1"]; }            // 186
            0xc005 => { return ["SHR","8","2","rmb","ib","Shl","1"]; }            // 186
            
            0xc104 => { return ["SHL","16","2","rmw","ib","Shl","1"]; }            // 186
            0xc105 => { return ["SHR","16","2","rmw","ib","Shr","1"]; }            // 186

            0xd000 => { return ["ROL","8","2","rmb","1","Rol","1"]; }
            0xd002 => { return ["RCL","8","2","rmb","1","Rcl","1"]; }
            0xd001 => { return ["ROR","8","2","rmb","1","Ror","1"]; }
            0xd003 => { return ["RCR","8","2","rmb","1","Rcr","1"]; }
            0xd004 => { return ["SHL","8","2","rmb","1","Shl","1"]; }
            0xd005 => { return ["SHR","8","2","rmb","1","Shr","1"]; }

            0xd100 => { return ["ROL","16","2","rmw","1","Rol","1"]; }            
            0xd101 => { return ["ROR","16","2","rmw","1","Ror","1"]; }            
            0xd102 => { return ["RCL","16","2","rmw","1","Rcl","1"]; }            
            0xd103 => { return ["RCR","16","2","rmw","1","Rcr","1"]; }            
            0xd104 => { return ["SHL","16","1","rmw","1","Shl","1"]; }            
            0xd105 => { return ["SHR","16","1","rmw","1","Shr","1"]; }            
            0xd107 => { return ["SAR","16","1","rmw","1","Sar","1"]; }            

            0xd200 => { return ["ROL","8","2","rmb","CL","Rol","1"]; }
            0xd201 => { return ["ROR","8","2","rmb","CL","Ror","1"]; }
            0xd204 => { return ["SHL","8","2","rmb","CL","Shl","1"]; }
            0xd205 => { return ["SHR","8","2","rmb","CL","Shr","1"]; }            
            0xd207 => { return ["SAR","8","2","rmb","CL","Sar","1"]; }            

            0xd300 => { return ["ROL","16","2","rmw","CL","Rol","1"]; }            
            0xd301 => { return ["ROR","16","2","rmw","CL","Ror","1"]; }            
            0xd304 => { return ["SHL","16","2","rmw","CL","Shl","1"]; }            
            0xd305 => { return ["SHR","16","2","rmw","CL","Shr","1"]; }            
            0xd307 => { return ["SAR","16","2","rmw","CL","Sar","1"]; }            

            0xf600 => { return ["TEST","8","2","ib","rmb","Test","0"]; }
            0xf602 => { return ["NOT","8","1","rmb","","Not","1"]; }
            0xf603 => { return ["NEG","8","1","rmb","","Neg","1"]; }
            0xf604 => { return ["MUL","8","1","rmb","","Mul","1"]; }
            0xf605 => { return ["IMUL","8","1","rmb","","Imul","1"]; }
            0xf606 => { return ["DIV","8","1","rmb","","Div","1"]; }
            0xf607 => { return ["IDIV","8","1","rmb","","Idiv","1"]; }

            0xf700 => { return ["TEST","16","2","iw","rmw","Test","0"]; }
            0xf702 => { return ["NOT","16","1","rmw","","Not","1"]; }
            0xf703 => { return ["NEG","16","1","rmw","","Neg","1"]; }
            0xf704 => { return ["MUL","16","1","rmw","","Mul","1"]; }
            0xf705 => { return ["IMUL","16","1","rmw","","Imul","1"]; }
            0xf706 => { return ["DIV","16","1","rmw","","Div","1"]; }
            0xf707 => { return ["IDIV","16","1","rmw","","Idiv","1"]; }

            0xfe00 => { return ["INC","8","1","rmb","","Inc","1"]; }
            0xfe01 => { return ["DEC","8","1","rmb","","Dec","1"]; }

            0xff00 => { return ["INC","16","1","rmw","","Inc","1"]; }
            0xff01 => { return ["DEC","16","1","rmw","","Dec","1"]; }
            0xff02 => { return ["CALL","16","1","rw","","CallReg","1"]; }
            0xff03 => { return ["CALL FAR","16","1","rw","","CallFar","1"]; }
            0xff04 => { return ["JMP NEAR WORD","16","1","rmw","","Jumpnw","1"]; }
            0xff05 => { return ["JMP FAR WORD","16","1","rmw","","Jumpfw","1"]; }
            0xff06 => { return ["PUSH","16","1","rmw","","Push","1"]; }

            0xdbe3 => { return ["FNINIT","8","0","","","Fninit","1"]; }
            0xdd3e => { return ["FNSTSW","8","0","","","Fnstsw","1"]; }

            0x0f82 => { return ["JB","16","0","","","JmpNear","0"]; }

            _ => { *found=false; }
        }

        return ["","","","","","",""];
    }

    fn prepareInstructionParameters(&self,opcodeInfo:&[&str;7],cs:u16,ip:u16,instrLen:&mut u8,dbgStr:&mut String,instrWidth:&u8,
                                    u8op:&mut u8,u16op:&mut u16,daddr:&mut u16,
                                    opsrc:&mut String,opdst:&mut String,displ:&mut i32,displSize:&mut u8,iType:&instructionType,
                                    pmachine:&machine,pvga:&vga) -> bool
    {
        if *iType==instructionType::instrJmpShort
        {
            // JMP<condition> i8 displacement
            *displ=0;
            *displSize=0;

            let jumpAmt=pmachine.readMemory(cs,ip+1,pvga);
            dbgStr.push_str(&format!(" {}",jumpAmt));
            let delta:i8=jumpAmt as i8;
            *opsrc=delta.to_string();
            *opdst="".to_string();
            *instrLen=2;
        }
        else if *iType==instructionType::instrJmpNear
        {
            // JMP<condition> i16 displacement
            *displ=0;
            *displSize=0;

            let jumpAmt=pmachine.readMemory16(cs,ip+2,pvga);
            dbgStr.push_str(&format!(" {}",jumpAmt));
            let delta:i16=jumpAmt as i16;
            *opsrc=delta.to_string();
            *opdst="".to_string();
            *instrLen=4;
        }
        else if *iType==instructionType::instrInt
        {
            // INT nn
            *displ=0;
            *displSize=0;

            let intNum=pmachine.readMemory(cs,ip+1,pvga);
            dbgStr.push_str(&format!(" 0x{:02x}",intNum));
            *opsrc=intNum.to_string();
            *opdst="".to_string();
            *instrLen=2;
        }
        else if *iType==instructionType::instrCallRel16
        {
            // CALL relative
            *displ=0;
            *displSize=0;

            let offset16=pmachine.readMemory16(cs,ip+1,pvga);
            dbgStr.push_str(&format!(" 0x{:04x}",offset16));
            *opsrc=offset16.to_string();
            *opdst="".to_string();
            *instrLen=3;
        }
        else if *iType==instructionType::instrJmpNp
        {
            *displ=0;
            *displSize=0;

            let offset16=pmachine.readMemory16(cs,ip+1,pvga);
            dbgStr.push_str(&format!(" 0x{:04x}",offset16));
            *u16op=offset16;
            *instrLen=3;
        }
        else if (*iType==instructionType::instrMov) || 
                (*iType==instructionType::instrAnd) ||
                (*iType==instructionType::instrOr) ||
                (*iType==instructionType::instrTest) ||
                (*iType==instructionType::instrXchg) ||
                (*iType==instructionType::instrXor) ||
                (*iType==instructionType::instrInc) ||
                (*iType==instructionType::instrDec) ||
                (*iType==instructionType::instrShl) ||
                (*iType==instructionType::instrShr) ||
                (*iType==instructionType::instrAdd) ||
                (*iType==instructionType::instrAdc) ||
                (*iType==instructionType::instrSub) ||
                (*iType==instructionType::instrSbb) ||
                (*iType==instructionType::instrNeg) ||
                (*iType==instructionType::instrNot) ||
                (*iType==instructionType::instrImul) ||
                (*iType==instructionType::instrMul) ||
                (*iType==instructionType::instrDiv) ||
                (*iType==instructionType::instrIdiv) ||
                (*iType==instructionType::instrPush) ||
                (*iType==instructionType::instrPop) ||
                (*iType==instructionType::instrLea) ||
                (*iType==instructionType::instrLds) ||
                (*iType==instructionType::instrLes) ||
                (*iType==instructionType::instrRor) ||
                (*iType==instructionType::instrRol) ||
                (*iType==instructionType::instrRcr) ||
                (*iType==instructionType::instrRcl) ||
                (*iType==instructionType::instrSar) ||
                (*iType==instructionType::instrCallReg) ||
                (*iType==instructionType::instrCallFar) ||
                (*iType==instructionType::instrJumpnw) ||
                (*iType==instructionType::instrJumpfw) ||
                (*iType==instructionType::instrCmp)
        {
            // instructions with modregrm byte
            let mut totInstrLen:u8=2;
            
            let mut dstIsSegreg:u8=0;
            if ((*opsrc)=="sr".to_string()) || ((*opdst)=="sr".to_string()) 
            {
                dstIsSegreg=1;
            }

            let mut wbit:u8=0;
            if (*instrWidth)==16 { wbit=1; }
            let mut operandAdder=0;
            let numOperands=opcodeInfo[2].parse::<u8>().unwrap();
            let invertOperands=opcodeInfo[6].parse::<u8>().unwrap();
            //let invertOperands=if (opcode&2)==2 { 1 } else { 0 };

            let mut ddambCoherency:bool=true;
            let addressingModeByte=pmachine.readMemory(cs,ip+1,pvga);
            let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,dstIsSegreg,wbit,&mut ddambCoherency);
            if ddambCoherency==false
            {
                return false;
            }

            if moveVec[0].contains("with 8bit disp") || moveVec[1].contains("with 8bit disp") { operandAdder=1; }
            if moveVec[0].contains("with 16bit disp") || moveVec[1].contains("with 16bit disp") { operandAdder=2; }
            if (moveVec[0]=="Direct Addr") || (moveVec[1]=="Direct Addr") { operandAdder=2; }

            let mut op0=0 as usize;
            let mut op1=1 as usize;
            if invertOperands==1
            {
                op0=1;
                op1=0;
            }

            if (*opdst=="sr") || (*opdst=="rb")  || (*opdst=="rw") || (*opdst=="rmb") || (*opdst=="rmw")
            {
                *opdst=moveVec[op0].clone();
            }
            if (*opsrc=="sr") || (*opsrc=="rb")  || (*opsrc=="rw") || (*opsrc=="rmb") || (*opsrc=="rmw")
            {
                *opsrc=moveVec[op1].clone();
            }

            if *opdst=="ib"
            {
                let ib:u8=pmachine.readMemory(cs,ip+2+operandAdder,pvga);
                *u8op=ib;
                totInstrLen+=1;
            }
            else if *opsrc=="ib"
            {
                let ib:u8=pmachine.readMemory(cs,ip+2+operandAdder,pvga);
                *u8op=ib;
                totInstrLen+=1;
            }

            if *opdst=="eb"
            {
                let ib:u16=pmachine.readMemory(cs,ip+2+operandAdder,pvga) as u16;
                let sign=ib&0x80;
                let sign_extended=if sign!=0 {0xff} else {0};
                *u16op=(sign_extended<<8)|(ib&0xff);
                totInstrLen+=1;
            }
            else if *opsrc=="eb"
            {
                let ib:u16=pmachine.readMemory(cs,ip+2+operandAdder,pvga) as u16;
                let sign=ib&0x80;
                let sign_extended=if sign!=0 {0xff} else {0};
                *u16op=(sign_extended<<8)|(ib&0xff);
                totInstrLen+=1;
            }

            if *opdst=="iw"
            {
                let iw:u16=pmachine.readMemory16(cs,ip+2+operandAdder,pvga);
                *u16op=iw;
                totInstrLen+=2;
            }
            else if *opsrc=="iw"
            {
                let iw:u16=pmachine.readMemory16(cs,ip+2+operandAdder,pvga);
                *u16op=iw;
                totInstrLen+=2;
            }

            // TODO handle other types of src/dest regs

            // TODO handle move from/to memory
            if *opdst=="Direct Addr"
            {
                *daddr=pmachine.readMemory16(cs,ip+2,pvga);
                totInstrLen+=2;
            }
            if *opsrc=="Direct Addr"
            {
                *daddr=pmachine.readMemory16(cs,ip+2,pvga);
                totInstrLen+=2;
            }

            // handle displacements
            if (*opdst).contains("with 8bit disp") || (*opsrc).contains("with 8bit disp")
            {
                *displSize=8;
                *displ=pmachine.readMemory(cs,ip+2,pvga) as i8 as i32;
                totInstrLen+=1;
            }
            else if (*opdst).contains("with 16bit disp") || (*opsrc).contains("with 16bit disp")
            {
                *displSize=16;
                *displ=pmachine.readMemory16(cs,ip+2,pvga) as i16 as i32;
                totInstrLen+=2;
            }
            else
            {
                *displ=0;
                *displSize=0;
            }

            // debug string
            let mut finalOpsrc=(*opsrc).clone();
            let mut finalOpdst=(*opdst).clone();
           
            if finalOpsrc.contains("with") { finalOpsrc=finalOpsrc.replace("Disp",&format!("{}",*displ)); }
            if finalOpdst.contains("with") { finalOpdst=finalOpdst.replace("Disp",&format!("{}",*displ)); }
            if finalOpsrc=="iw" { finalOpsrc=finalOpsrc.replace("iw",&format!("{:04x}",*u16op)); }
            if finalOpdst=="iw" { finalOpdst=finalOpdst.replace("iw",&format!("{:04x}",*u16op)); }
            if finalOpsrc=="eb" { finalOpsrc=finalOpsrc.replace("eb",&format!("{:04x}",*u16op)); }
            if finalOpdst=="eb" { finalOpdst=finalOpdst.replace("eb",&format!("{:04x}",*u16op)); }
            if finalOpsrc=="ib" { finalOpsrc=finalOpsrc.replace("ib",&format!("{:02x}",*u8op)); }
            if finalOpdst=="ib" { finalOpdst=finalOpdst.replace("ib",&format!("{:02x}",*u8op)); }
            if finalOpsrc=="Direct Addr" { finalOpsrc=finalOpsrc.replace("Direct Addr",&format!("[{:04x}]",*daddr)); }
            if finalOpdst=="Direct Addr" { finalOpdst=finalOpdst.replace("Direct Addr",&format!("[{:04x}]",*daddr)); }

            if numOperands==1 { dbgStr.push_str(&format!(" {}",finalOpsrc)); }
            else { dbgStr.push_str(&format!(" {},{}",finalOpdst,finalOpsrc)); } 

            // ilen
            *instrLen=totInstrLen;
        }
        else if (*iType==instructionType::instrMovNoModRegRm) ||
                (*iType==instructionType::instrAndNoModRegRm) ||
                (*iType==instructionType::instrCmpNoModRegRm) ||
                (*iType==instructionType::instrTestNoModRegRm) ||
                (*iType==instructionType::instrAddNoModRegRm) ||
                (*iType==instructionType::instrAdcNoModRegRm) ||
                (*iType==instructionType::instrOrNoModRegRm) ||
                (*iType==instructionType::instrXorNoModRegRm) ||
                (*iType==instructionType::instrIncNoModRegRm) ||
                (*iType==instructionType::instrDecNoModRegRm) ||
                (*iType==instructionType::instrOut) ||
                (*iType==instructionType::instrPushNoModRegRm) ||
                (*iType==instructionType::instrSbbNoModRegRm) ||
                (*iType==instructionType::instrLongJump) ||
                (*iType==instructionType::instrRetfiw) ||
                (*iType==instructionType::instrRetiw) ||
                (*iType==instructionType::instrSubNoModRegRm)
        {
            *displ=0;
            *displSize=0;

            let numOperands=opcodeInfo[2].parse::<u8>().unwrap();

            let mut realOpdst:String=String::from("");
            realOpdst.push_str(&*opdst);
            let mut realOpsrc:String=String::from("");
            realOpsrc.push_str(&*opsrc);

            if *opsrc=="iw"
            {
                let iw:u16=pmachine.readMemory16(cs,ip+1,pvga);
                *u16op=iw;
                *instrLen+=2;
                realOpsrc=format!("0x{:04x}",iw);
            }
            else if *opsrc=="ib"
            {
                let ib:u8=pmachine.readMemory(cs,ip+1,pvga);
                *u8op=ib;
                *instrLen+=1;
                realOpsrc=format!("0x{:02x}",ib);
            }
            else if *opsrc=="Direct Addr"
            {
                *daddr=pmachine.readMemory16(cs,ip+1,pvga);
                *instrLen+=2;
                realOpsrc=format!("[{:04x}]",*daddr);
            }

            if *opdst=="Direct Addr"
            {
                *daddr=pmachine.readMemory16(cs,ip+1,pvga);
                *instrLen+=2;
                realOpdst=format!("[{:04x}]",*daddr);
            }
            else if *opdst=="iw"
            {
                let rip=ip+(*instrLen as u16);
                let iw:u16=pmachine.readMemory16(cs,rip,pvga);
                *instrLen+=2;
                realOpdst=format!("0x{:04x}",iw);
            }

            if *opsrc=="eb"
            {
                let ib:u16=pmachine.readMemory(cs,ip+1,pvga) as u16;
                let sign=ib&0x80;
                let sign_extended=if sign!=0 {0xff} else {0};
                *u16op=(sign_extended<<8)|(ib&0xff);
                *instrLen+=1;
                realOpdst=format!(" [{:04x}]",*u16op);
            }

            if numOperands==1 { dbgStr.push_str(&format!(" {}",realOpdst)); }
            else { dbgStr.push_str(&format!(" {},{}",realOpdst,realOpsrc)); } 
        }
        else if (*iType==instructionType::instrIn) || (*iType==instructionType::instrOutNoModRegRm)
        {
            *displ=0;
            *displSize=0;

            let mut realOpdst:String=String::from("");
            realOpdst.push_str(&*opdst);
            let mut realOpsrc:String=String::from("");
            realOpsrc.push_str(&*opsrc);

            if *opsrc=="ib"
            {
                let ib:u8=pmachine.readMemory(cs,ip+1,pvga);
                *u8op=ib;
                *instrLen+=1;
                realOpsrc=format!("0x{:02x}",ib);
            }
            else if (*opsrc)=="DX"
            {
                *u16op=self.dx;
            }

            if (*opdst)=="DX"
            {
                *u16op=self.dx;
            }

            dbgStr.push_str(&format!(" {},{}",realOpdst,realOpsrc));
        }

        return true;
    }

    fn expandWideInstruction(&self,opcode:&u8,wideOpcode:&mut u16,pmachine:&machine,pvga:&vga,cs:&u16,ip:&u16)
    {
        if (*opcode==0xf7) || (*opcode==0xff) || (*opcode==0x80) || (*opcode==0xfe) || (*opcode==0xd2) ||
           (*opcode==0xc1) || (*opcode==0x81) || (*opcode==0x83) || (*opcode==0xd0) || (*opcode==0xf6) || 
           (*opcode==0xd1) || (*opcode==0xd3) || (*opcode==0xc0) || (*opcode==0x82)
        {
            let instrType=pmachine.readMemory(*cs,*ip,pvga);
            let reg:usize=((instrType>>3)&0x07).into();
            *wideOpcode=(reg as u16)|((*opcode as u16)<<8);
        }
        else if (*opcode==0xdb) || (*opcode==0xdd) || (*opcode==0x0f)
        {
            let secondByte=pmachine.readMemory(*cs,*ip,pvga);
            *wideOpcode=(secondByte as u16)|((*opcode as u16)<<8);
        }
    }

    pub fn dekode(&mut self,pmachine:&machine,pvga:&vga,cs:u16,ip:u16) -> bool
    {
        //
        // decode an 8086 instruction (also get debugging info)
        // get:
        // - instruction lenght in bytes (this is useful to eventually increment IP)
        // - number of operands
        // - instruction/operands size (8, 16 bit)
        // - op1, ...
        // - eventual segment override
        // - eventual rep prefix
        // - instruction decode for debug (like "MOV AX,BX")
        // - instruction type (move, dec, inc, etc.)
        // - eventual displacement (for example [SI+Displ])
        // - displacement size (8-16 bit)
        //

        let mut instrLen=1;
        let mut canDecode:bool=false;
        let mut soroAdder:u16=0;

        let mut segOverride:String=String::from("");
        let mut repOverride:String=String::from("");
        let mut opcode=pmachine.readMemory(cs,ip,pvga);

        // handle repetition prefix
        if opcode==0xf3 { repOverride="REPE".to_string(); }
        else if opcode==0xf2 { repOverride="REPNE".to_string(); }
        if repOverride!="" { opcode=pmachine.readMemory(cs,ip+1,pvga); soroAdder+=1; }

        // handle seg overrides
        if opcode==0x2e { segOverride="CS".to_string(); }
        else if opcode==0x36 { segOverride="SS".to_string(); }
        else if opcode==0x3e { segOverride="DS".to_string(); }
        else if opcode==0x26 { segOverride="ES".to_string(); }
        if segOverride!="" { opcode=pmachine.readMemory(cs,ip+1+soroAdder,pvga); soroAdder+=1; }

        // decode instruction
        let mut wasDecoded=true;
        let mut wideOpcode:u16=opcode as u16;
        self.expandWideInstruction(&opcode,&mut wideOpcode,pmachine,pvga,&cs,&(ip+1+(soroAdder as u16)));
        let opcodeInfo=self.getOpcodeStructure(&wideOpcode,&mut wasDecoded);
        if wasDecoded
        {
            canDecode=true;

            if wideOpcode==0xf603
            {
                //self.abort(&format!("Neg! at {:04x}:{:04x}",self.cs,self.ip));
            }

            let mut dbgDec:String=String::from("");
            if segOverride!="" { dbgDec.push_str(&format!("{} ",segOverride)); } 
            if repOverride!="" { dbgDec.push_str(&format!("{} ",repOverride)); }
            dbgDec.push_str(&opcodeInfo[0].to_string());

            let mut operandSrc:String=String::from(opcodeInfo[3].to_string());
            let mut operandDst:String=String::from(opcodeInfo[4].to_string());
            let instrWidth:u8=opcodeInfo[1].parse::<u8>().unwrap();
            let mut displacement:i32=0;
            let mut displSize:u8=0;
            let mut u8op:u8=0;
            let mut u16op:u16=0;
            let mut daddr:u16=0;
            let instrType=self.getInstructionType(&opcodeInfo[5].to_string());
            let pipRet=self.prepareInstructionParameters(&opcodeInfo,cs,ip+soroAdder,&mut instrLen,&mut dbgDec,
                                              &instrWidth,
                                              &mut u8op,&mut u16op,&mut daddr,
                                              &mut operandSrc,&mut operandDst,
                                              &mut displacement,&mut displSize,
                                              &instrType,pmachine,pvga);
            if pipRet==false
            {
                // inconsistent situation (when debugging) with memory and opcodes, bailout
                return false;
            }

            instrLen+=soroAdder as u8;
            self.decInstr=decodedInstruction {
                insType: instrType,
                insLen: instrLen,
                instrSize: instrWidth,
                operand1: operandSrc,
                operand2: operandDst,
                displacement: displacement,
                u8immediate: u8op,
                u16immediate: u16op,
                directAddr: daddr,
                segOverride: segOverride,
                repPrefix: repOverride,
                debugDecode: dbgDec,
            };
        }

        return canDecode;
    }

    pub fn exeCute(&mut self,pmachine:&mut machine,pvga:&mut vga,pdisk:&fddController)
    {
        if (self.decInstr.insType==instructionType::instrPopNoModRegRm) || (self.decInstr.insType==instructionType::instrPop)
        {
            self.doPop(pmachine,pvga);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if (self.decInstr.insType==instructionType::instrPush) || (self.decInstr.insType==instructionType::instrPushNoModRegRm)
        {
            self.doPush(pmachine,pvga);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrRet
        {
            // RET (near)
            let newip=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            self.ip=newip;
        }
        else if self.decInstr.insType==instructionType::instrRetiw
        {
            // RET with iw
            let newip=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            self.sp+=self.decInstr.u16immediate;
            self.ip=newip;
        }
        else if self.decInstr.insType==instructionType::instrRetf
        {
            // RET (far)
            let newip=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            let newcs=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            self.ip=newip;
            self.cs=newcs;
        }
        else if self.decInstr.insType==instructionType::instrRetfiw
        {
            // RET (far) iw
            let newip=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            let newcs=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            self.ip=newip;
            self.cs=newcs;

            self.sp+=self.decInstr.u16immediate;
        }
        else if self.decInstr.insType==instructionType::instrIret
        {
            // return from interrupt
            let newip=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            let newcs=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            self.flags=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;

            self.ip=newip;
            self.cs=newcs;
        }
        else if self.decInstr.insType==instructionType::instrClc
        {
            self.setCflag(false);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrStc
        {
            self.setCflag(true);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrStd
        {
            self.setDflag(true);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrCli
        {
            self.setIflag(false);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrSti
        {
            self.setIflag(true);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrCld
        {
            self.setDflag(false);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrCwd
        {
            let sign=self.ax&0x8000;
            let sign_extended=if sign!=0 {0xffff} else {0};
            self.dx=sign_extended;

            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrCbw
        {
            let sign=self.ax&0x80;
            let sign_extended=if sign!=0 {0xff} else {0};
            self.ax=(sign_extended<<8)|(self.ax&0xff);

            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrCmc
        {
            // CMC
            self.setCflag(!self.getCflag());
            self.ip+=self.decInstr.insLen as u16;
        }
        else if (self.decInstr.insType==instructionType::instrInc) || (self.decInstr.insType==instructionType::instrIncNoModRegRm)
        {
            self.performInc(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrDec) || (self.decInstr.insType==instructionType::instrDecNoModRegRm)
        {
            self.performDec(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrXchg) || (self.decInstr.insType==instructionType::instrXchgNoModRegRm)
        {
            self.xchgRegs(pmachine,pvga);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrPushf
        {
            // PUSHF
            pmachine.push16(self.flags,self.ss,self.sp);
            self.sp-=2;
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrPopf
        {
            // POPF
            self.flags=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrLods
        {
            // LODSB/LODSW
            self.performLods(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrMovs
        {
            // MOVSB/MOVSW
            self.performMovs(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrStos
        {
            // STOSB/STOSW
            self.performStos(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrScas
        {
            // SCASB/SCASW
            self.performScas(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrCmps
        {
            // CMPS
            self.performCmps(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrXlat
        {
            // XLAT
            self.performXlat(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrJmpShort
        {
            // JE/JNE/etc.
            self.performJmp();
        }
        else if self.decInstr.insType==instructionType::instrJmpNear
        {
            self.performJmp();
        }
        else if self.decInstr.insType==instructionType::instrLongJump
        {
            let newip=pmachine.readMemory16(self.cs,self.ip+1,pvga);
            let newcs=pmachine.readMemory16(self.cs,self.ip+3,pvga);
            self.ip=newip;
            self.cs=newcs;
        }
        else if self.decInstr.insType==instructionType::instrInt
        {
            // INT nn
            let intNum=self.decInstr.operand1.parse::<u8>().unwrap();

            //if (intNum==0x1c) || (intNum==0x21) || (intNum==0x2a) || (intNum==0x2f) || (intNum==0x20) || (intNum==0x28) || (intNum==0x3f) || (intNum==131)
            if intNum>=0x1c
            {
                let newip=pmachine.readMemory16(0x0,(intNum as u16)*4,pvga);
                let newcs=pmachine.readMemory16(0x0,((intNum as u16)*4)+2,pvga);

                pmachine.push16(self.flags,self.ss,self.sp);
                self.sp-=2;
                self.setTflag(false);
                self.setIflag(false);

                pmachine.push16(self.cs,self.ss,self.sp);
                self.sp-=2;
                pmachine.push16(self.ip+2,self.ss,self.sp);
                self.sp-=2;
               
                self.cs=newcs;
                self.ip=newip;

                //self.abort(&format!("INT 21h,{:02x} towards {:04x}:{:04x}",self.ax>>8,newcs,newip));
            }
            else
            {
                let goOn:bool=pmachine.handleINT(intNum,self,pvga,pdisk);
                if goOn { self.ip+=2; }
            }
        }
        else if self.decInstr.insType==instructionType::instrJmpNp
        {
            self.ip=self.ip.wrapping_add((self.decInstr.u16immediate+3) as u16);
        }
        else if self.decInstr.insType==instructionType::instrCallRel16
        {
            // call with 16 bit relative offset
            let offset16=self.decInstr.operand1.parse::<u16>().unwrap();
            pmachine.push16(self.ip+3,self.ss,self.sp);
            self.sp-=2;
            self.ip=self.ip.wrapping_add((offset16+3) as u16);
        }
        else if self.decInstr.insType==instructionType::instrCallReg
        {
            // call procedure in reg
            let addrRel:u16=self.getOperandValue(&self.decInstr.operand1.clone(),pmachine,pvga);
            pmachine.push16(self.ip+self.decInstr.insLen as u16,self.ss,self.sp);
            self.sp-=2;
            self.ip=addrRel;
        }
        else if self.decInstr.insType==instructionType::instrJumpnw
        {
            // Jump to near word
            let addrRel:u16=self.getOperandValue(&self.decInstr.operand1.clone(),pmachine,pvga);
            self.ip=addrRel;
        }
        else if self.decInstr.insType==instructionType::instrJumpfw
        {
            // Jump to far word
            let mut soadder=0;
            let mut offs=0;
            let newip:u16;
            let newcs:u16;

            let mut readSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { readSeg=self.cs; soadder=1; }
            else if self.decInstr.segOverride=="SS" { readSeg=self.ss; soadder=1; }
            else if self.decInstr.segOverride=="DS" { readSeg=self.ds; soadder=1; }
            else if self.decInstr.segOverride=="ES" { readSeg=self.es; soadder=1; }

            if self.decInstr.operand1=="[DI]"
            {
                offs=self.di;
            }
            else if self.decInstr.operand1=="[BX]"
            {
                offs=self.bx;
            }
            else if self.decInstr.operand1.contains("[DI+Disp]")
            {
                offs=self.di+self.decInstr.displacement as u16;
            }
            else if self.decInstr.operand1=="Direct Addr"
            {
                offs=pmachine.readMemory16(readSeg,self.ip+2+soadder,pvga);
            }
            else
            {
                self.abort(&format!("Unhandled Jumpfw {}",self.decInstr.operand1));
            }

            newip=pmachine.readMemory16(readSeg,offs,pvga);
            newcs=pmachine.readMemory16(readSeg,offs+2,pvga);
        
            self.ip=newip;
            self.cs=newcs;
        }
        else if self.decInstr.insType==instructionType::instrCallFarPtr
        {
            if self.decInstr.segOverride!="".to_string()
            {
                self.abort("call far ptr with seg override");
            }

            let newip=pmachine.readMemory16(self.cs,self.ip+1,pvga);
            let newcs=pmachine.readMemory16(self.cs,self.ip+3,pvga);

            pmachine.push16(self.cs,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.ip+5,self.ss,self.sp);
            self.sp-=2;

            self.ip=newip;
            self.cs=newcs;
        }
        else if self.decInstr.insType==instructionType::instrCallFar
        {
            self.performCallFar(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrMov) || (self.decInstr.insType==instructionType::instrMovNoModRegRm)
        {
            self.performMove(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrAnd) || (self.decInstr.insType==instructionType::instrAndNoModRegRm)
        {
            self.performAnd(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrAdd) || (self.decInstr.insType==instructionType::instrAddNoModRegRm)
        {
            self.performAdd(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrOr) || (self.decInstr.insType==instructionType::instrOrNoModRegRm)
        {
            self.performOr(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrShl
        {
            self.performShl(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrShr
        {
            self.performShr(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrSbb) || (self.decInstr.insType==instructionType::instrSbbNoModRegRm)
        {
            self.performSbb(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrLea
        {
            self.performLea(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrLds
        {
            self.performLds(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrLes
        {
            self.performLes(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrRor
        {
            self.performRor(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrRol
        {
            self.performRol(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrSar
        {
            self.performSar(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrRcr
        {
            self.performRcr(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrRcl
        {
            self.performRcl(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrXor) || (self.decInstr.insType==instructionType::instrXorNoModRegRm)
        {
            self.performXor(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrAdc) || (self.decInstr.insType==instructionType::instrAdcNoModRegRm)
        {
            self.performAdc(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrNot
        {
            self.performNot(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrNeg
        {
            self.performNeg(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrImul
        {
            self.performImul(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrMul
        {
            self.performMul(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrDiv
        {
            self.performDiv(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrIdiv
        {
            self.performIdiv(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrIn
        {
            self.performIn(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrOut) || (self.decInstr.insType==instructionType::instrOutNoModRegRm)
        {
            self.performOut(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrCmp) || (self.decInstr.insType==instructionType::instrCmpNoModRegRm)
        {
            self.performCompare(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrTest) || (self.decInstr.insType==instructionType::instrTestNoModRegRm)
        {
            self.performTest(pmachine,pvga);
        }
        else if (self.decInstr.insType==instructionType::instrSub) || (self.decInstr.insType==instructionType::instrSubNoModRegRm)
        {
            self.performSub(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrPusha
        {
            pmachine.push16(self.ax,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.cx,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.dx,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.bx,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.sp,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.bp,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.si,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.di,self.ss,self.sp);
            self.sp-=2;

            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrPopa
        {
            self.di=pmachine.pop16(self.ss,self.sp); self.sp+=2;
            self.si=pmachine.pop16(self.ss,self.sp); self.sp+=2;
            self.bp=pmachine.pop16(self.ss,self.sp); self.sp+=2;
            let _discarded=pmachine.pop16(self.ss,self.sp); self.sp+=2;
            self.bx=pmachine.pop16(self.ss,self.sp); self.sp+=2;
            self.dx=pmachine.pop16(self.ss,self.sp); self.sp+=2;
            self.cx=pmachine.pop16(self.ss,self.sp); self.sp+=2;
            self.ax=pmachine.pop16(self.ss,self.sp); self.sp+=2;

            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrLahf
        {
            self.ax=(self.ax&0xff)|((self.flags&0xff)<<8);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrSahf
        {
            let mut ah=self.ax>>8;
            ah&=0b11010101;
            ah|=2;
            self.flags=(self.flags&0xff00)|ah;
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrAad
        {
            let mut ah=self.ax>>8;
            ah*=10;
            let mut al=self.ax&0xff;
            al+=ah;
            al&=0xff;
            self.ax=al;

            self.doZflag(self.ax);
            self.doPflag(self.ax);
            self.doSflag(self.ax,16);
            
            self.ip+=2;
        }
        else if self.decInstr.insType==instructionType::instrAam
        {
            // wtf does this do
            let mut al=self.ax&0xff;
            let remainder=al%0x0a;
            al/=0x0a;
            self.ax=remainder|(al<<8);

            self.doZflag(self.ax&0xff);
            self.doPflag(self.ax&0xff);
            self.doSflag(self.ax&0xff,8);

            self.ip+=2;
        }
        else if self.decInstr.insType==instructionType::instrDas
        {
            let origValue:u8=(self.ax&0xff) as u8;
            let origC = self.getCflag();

            self.setCflag(false);

            if ((self.ax&0x0F)>9) || (self.getAflag()) 
            {
                let mut al=self.ax&0xff;
                al=al.wrapping_sub(6);
                self.ax=(self.ax&0xff00)|al;
                self.setCflag(origC | ((self.ax & 0x80) == 0));
                self.setAflag(true);
            } 
            else 
            {
                self.setAflag(false);
            }
            
            if (origValue > 0x99) || (origC) 
            {
                let mut al=self.ax&0xff;
                al=al.wrapping_sub(0x60);
                self.ax=(self.ax&0xff00)|al;
                self.setCflag(true);
            }

            self.doZflag(self.ax&0xff);
            self.doPflag(self.ax&0xff);
            self.doSflag(self.ax&0xff,8);

            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrDaa
        {
            let val = self.ax&0xff;
            let mut lo = val & 0xf;
            let mut hi = val >> 4;
            let mut carry = false;
            if self.getAflag()
            {
                lo -= 10;
                hi += 1;
            }
            if lo > 9
            {
                lo -= 10;
                hi += 1;
            }
            if hi > 9 || self.getCflag()
            {
                hi -= 10;
                carry = true;
            }

            let result = lo | (hi<<4);
            self.ax=(self.ax&0xff00)|result;
            self.setCflag(carry);
            self.setAflag(false);

            self.doZflag(result);
            self.doPflag(result);
            self.doSflag(result,8);

            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrAaa
        {
            let mut al=self.ax&0xff;
            let mut ah=self.ax>>8;
            if ((al & 0x0F) > 9) || (self.getAflag()) 
            {
                al += 6;
                ah += 1;
                self.setCflag(true);
                self.setAflag(true);
            } 
            else 
            {
                self.setCflag(false);
                self.setAflag(false);
            }
            al &= 0x0F;
            self.ax=al|(ah<<8);

            self.doZflag(self.ax&0xff);
            self.doPflag(self.ax&0xff);
            self.doSflag(self.ax&0xff,8);

            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrNop
        {
            // do absolutely nothing
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrFninit
        {
            // TODO
            self.ip+=2;
        }
        else if self.decInstr.insType==instructionType::instrFnstsw
        {
            // TODO
            self.ip+=4;
        }
        else
        {
            self.abort(&format!("Cannot exeCute [{}].",self.decInstr.debugDecode));
        }
    }

    //
    //
    //

    fn getAflag(&self) -> bool
    {
        return (self.flags&(1<<4))==(1<<4);
    }

    fn setAflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1<<4;
        }
        else
        {
            self.flags&=!(1<<4);
        }
    }

    fn doOAflagsAdd(&mut self,a:u16,b:u16,rez:u16)
    {
        if self.decInstr.instrSize==8
        {
            let a8=a as u8;
            let b8=b as u8;
            let rez8=rez as u8;
            let same_sign = ((a8 ^ b8) & 0x80) != 0x80;
            let overflow = same_sign && (((rez8 ^ a8) & 0x80) == 0x80);
            let (ms, ma, mb) = (rez8 & 0xf, a8 & 0xf, b8 & 0xf);
            let half_carry = ms < ma || ms < mb;
            self.setAflag(half_carry);
            self.setOflag(overflow);
        }
        else if self.decInstr.instrSize==16
        {
            let same_sign = ((a ^ b) & 0x8000) != 0x8000;
            let overflow = same_sign && (((rez ^ a) & 0x8000) == 0x8000);
            let (ms, ma, mb) = (rez & 0x000f, a & 0x000f, b & 0x000f);
            let half_carry = ms < ma || ms < mb;        
            self.setAflag(half_carry);
            self.setOflag(overflow);
        }
    }

    fn getIflag(&self) -> bool
    {
        return (self.flags&(1<<9))==(1<<9);
    }

    fn setIflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1<<9;
        }
        else
        {
            self.flags&=!(1<<9);
        }
    }

    fn setTflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1<<8;
        }
        else
        {
            self.flags&=!(1<<8);
        }
    }

    fn setPflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1<<2;
        }
        else
        {
            self.flags&=!(1<<2);
        }
    }

    fn getPflag(&self) -> bool
    {
        return (self.flags&(1<<2))==(1<<2);
    }

    fn getOflag(&self) -> bool
    {
        return (self.flags&1<<11)==(1<<11);
    }

    fn setOflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1<<11;
        }
        else
        {
            self.flags&=!(1<<11);
        }
    }

    fn getDflag(&self) -> bool
    {
        return (self.flags&1<<10)==(1<<10);
    }

    fn setDflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1<<10;
        }
        else
        {
            self.flags&=!(1<<10);
        }
    }

    fn getCflag(&self) -> bool
    {
        return (self.flags&1)==(1<<0);
    }

    pub fn setCflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1;
        }
        else
        {
            self.flags&=!(1<<0);
        }
    }

    fn getZflag(&self) -> bool
    {
        return (self.flags&(1<<6))==(1<<6);
    }

    pub fn setZflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1<<6;
        }
        else
        {
            self.flags&=!(1<<6);
        }
    }

    fn getSflag(&self) -> bool
    {
        return (self.flags&(1<<7))==(1<<7);
    }

    fn setSflag(&mut self,val:bool)
    {
        if val
        {
            self.flags|=1<<7;
        }
        else
        {
            self.flags&=!(1<<7);
        }
    }

    fn doZflag(&mut self,val:u16)
    {
        if self.decInstr.instrSize==8
        {
            if (val&0xff)==0 { self.setZflag(true); }
            else { self.setZflag(false); }
        }
        else
        {
            if val==0 { self.setZflag(true); }
            else { self.setZflag(false); }
        }
    }

    fn doPflag(&mut self,val:u16)
    {
        let mut ret = val as u8;
        ret ^= ret >> 4;
        ret ^= ret >> 2;
        ret ^= ret >> 1;
        self.setPflag((ret & 1) == 0);
    }

    fn doSflag(&mut self,val:u16,bits:u8)
    {
        if bits==8
        {
            if (val&0x80)==0x80 { self.setSflag(true); }
            else { self.setSflag(false); }
        }
        else if bits==16
        {
            if (val&0x8000)==0x8000 { self.setSflag(true); }
            else { self.setSflag(false); }
        }
    }

    fn doCflag(&mut self,val:u16,bits:u8)
    {
        if bits==8
        {
            if (val&0x80)==0x80 { self.setCflag(true); }
            else { self.setCflag(false); }
        }
        else if bits==16
        {
            if (val&0x8000)==0x8000 { self.setCflag(true); }
            else { self.setCflag(false); }
        }
    }

    //
    //
    //

    fn abort(&self,s:&str)
    {
        println!("bailing out due to {}...",s);
        process::exit(0x0100);
    }

    fn prepareDbgInfo(&self,cs:u16,ip:u16,pmachine:&mut machine,pvga:&mut vga) -> String
    {
        let mut retStr:String=format!("{:04x}:{:04x} ",cs,ip);
        for idx in 0..self.decInstr.insLen
        {
            let ibyte:u8=pmachine.readMemory(cs,ip+idx as u16,pvga);
            retStr.push_str(&format!("({:02x})",ibyte));
        }

        if self.decInstr.insLen<7
        {
            for _idx in 0..7-self.decInstr.insLen
            {
                retStr.push_str("    ");
            }
        }

        return retStr;
    }

    pub fn triggerHwIrq(&mut self,intNum: u8)
    {
        self.isIntPending=true;
        self.intPendingNum=intNum;        
    }

    pub fn executeOne(&mut self,pmachine:&mut machine,pvga:&mut vga,pdisk:&fddController,debugFlag:bool,bytesRead:&mut u8,dbgCS:&u16,dbgIP:&u16) -> String
    {
        /* decode&execute phases */
        let mut tmpcs:u16=self.cs;
        let mut tmpip:u16=self.ip;
        if debugFlag
        {
            tmpcs=*dbgCS;
            tmpip=*dbgIP;
        }

        /* handle eventual hw irqs */
        if (!debugFlag) && (self.isIntPending) && (self.getIflag()==true)
        {
            self.isIntPending=false;

            let newip=pmachine.readMemory16(0x0,(self.intPendingNum as u16)*4,pvga);
            let newcs=pmachine.readMemory16(0x0,((self.intPendingNum as u16)*4)+2,pvga);

            pmachine.push16(self.flags,self.ss,self.sp);
            self.sp-=2;
            self.setTflag(false);
            self.setIflag(false);
            pmachine.push16(self.cs,self.ss,self.sp);
            self.sp-=2;
            pmachine.push16(self.ip,self.ss,self.sp);
            self.sp-=2;
            self.cs=newcs;
            self.ip=newip;
            tmpcs=self.cs;
            tmpip=self.ip;
        }

        let canDecode=self.dekode(pmachine,pvga,tmpcs,tmpip);
        if canDecode
        {
            let mut dbgAddress:String=self.prepareDbgInfo(tmpcs,tmpip,pmachine,pvga);
            dbgAddress.push_str(" ");
            dbgAddress.push_str(&self.decInstr.debugDecode);
            *bytesRead=self.decInstr.insLen;
            if debugFlag==false { self.exeCute(pmachine,pvga,pdisk); self.totInstructions+=1; }
            return dbgAddress;
        }
        else
        {
            // abort only if executing
            let opcode=pmachine.readMemory(tmpcs,tmpip,pvga);
            if debugFlag==false
            {
                self.abort(&format!("x86cpu::Unhandled opcode {:02x} at {:04x}:{:04x} Next bytes {:02x}{:02x}{:02x}",opcode,self.cs,self.ip,
                pmachine.readMemory(self.cs,self.ip+1,pvga),
                pmachine.readMemory(self.cs,self.ip+2,pvga),
                pmachine.readMemory(self.cs,self.ip+3,pvga)
                )
            );
            }
            return format!("UNHANDLED ({:02x})",opcode);
        }
    }
}
