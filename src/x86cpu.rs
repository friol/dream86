
/* 

    our mythical 8086 cpu - dream86 2o22 

    TODO:
    - rewrite all the get/set flags functions as one
    - shorter & more compact code
    - find a solution for the "any register to any register" instructions
    - optimize. I think we are slow
    - wrap around registers
    - make dirojedc.com work. now works but backwards.
    
*/

use std::process;
use std::collections::HashMap;
use rand::Rng;

use crate::vga::vga;
use crate::machine::machine;

//

#[derive(PartialEq)]
pub enum instructionType
{
    instrNone,
    instrPush,
    instrPop,
    instrPushf,
    instrPopf,
    instrRet,
    instrClc,
    instrCld,
    instrMov,
    instrMovNoModRegRm,
    instrAnd,
    instrAndNoModRegRm,
    instrCmp,
    instrCmpNoModRegRm,
    instrAdd,
    instrAddNoModRegRm,
    instrAdc,
    instrInc,
    instrIncNoModRegRm,
    instrDec,
    instrDecNoModRegRm,
    instrXchg,
    instrLods,
    instrMovs,
    instrStos,
    instrScas,
    instrJmpShort,
    instrJmpNp,
    instrInt,
    instrCallRel16,
    instrCallReg,
    instrXlat,
    instrSub,
    instrSubNoModRegRm,
    instrSbb,
    instrIn,
    instrOr,
    instrXor,
    instrXorNoModRegRm,
    instrTest,
    instrTestNoModRegRm,
    instrShl, // to the left!
    instrShr, // to the right!!!
    instrCwd,
    instrNeg,
    instrImul,
    instrMul,
    instrDiv,
    instrLea,
}

pub struct decodedInstruction
{
    insType: instructionType,
    insLen: u8,
    //numOperands: u8,
    instrSize: u8,
    operand1: String,
    operand2: String,
    displacement: i32,
    //displSize: u8,
    u8immediate: u8,
    u16immediate: u16,
    segOverride: String,
    repPrefix: String,
    debugDecode: String,
    //opcode: u16
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
    decInstr: decodedInstruction
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
            //numOperands: 0,
            instrSize: 16,
            operand1: String::from(""),
            operand2: String::from(""),
            displacement: 0,
            //displSize: 0,
            u8immediate: 0,
            u16immediate: 0,
            segOverride: String::from(""),
            repPrefix: String::from(""),
            debugDecode: String::from("Undecodable"),
            //opcode: 0
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
            ds: 0xf000, // right?
            es: 0,
            ss: 0xf000,
            flags: 0,
            totInstructions: 0,
            decInstr: decIn
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
    fn debugDecodeAddressingModeByte(&self,b:u8,regType:u8,wbit:u8) -> Vec<String>
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
            retVec.push(segRegTable[reg].to_string());
        }
        else
        {
            // TODO use registers table            
            if wbit==1 { retVec.push(reg16bTable[reg].to_string()); }
            else  { retVec.push(reg8bTable[reg].to_string()); }
        }

        return retVec;
    }

    fn xchgRegs(&mut self,r0:String,r1:String)
    {
        if (r0=="AX") && (r1=="CX") { let tmp=self.ax; self.ax=self.cx; self.cx=tmp; }
        else if (r0=="AX") && (r1=="BX") { let tmp=self.ax; self.ax=self.bx; self.bx=tmp; }
        else if (r0=="AX") && (r1=="DX") { let tmp=self.ax; self.ax=self.dx; self.dx=tmp; }
        else if (r0=="AX") && (r1=="DI") { let tmp=self.ax; self.ax=self.di; self.di=tmp; }
        else if (r0=="AX") && (r1=="SI") { let tmp=self.ax; self.ax=self.si; self.si=tmp; }
        else if (r0=="AX") && (r1=="SP") { let tmp=self.ax; self.ax=self.sp; self.sp=tmp; }
        else if (r0=="AX") && (r1=="BP") { let tmp=self.ax; self.ax=self.bp; self.bp=tmp; }
    }

    fn performLea(&mut self)
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
            else
            {
                self.abort("LEA");
            }

            self.moveToDestination(&dst,&operand2);
        }
        else
        {
            self.abort("Unimplemented LEA 8bit (does it exist?)");
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performShr(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags (S?)
        let operand1=self.decInstr.operand1.clone();
        let operand2=self.decInstr.operand2.clone();

        if self.decInstr.instrSize==16
        {
            let mut val2shift:u16=self.getOperandValue(&operand1,pmachine,pvga);

            let lastBit:bool=(val2shift&0x1)!=0;
            self.setCflag(lastBit);
            val2shift>>=1;
            self.moveToDestination(&val2shift,&operand2);

            self.doZflag(val2shift as u16);
            self.doPflag(val2shift as u16);
            self.doSflag(val2shift as u16,self.decInstr.instrSize);
        }
        else
        {
            let mut val2shift:u8=self.getOperandValue(&operand1,pmachine,pvga) as u8;

            let lastBit:bool=(val2shift&0x1)!=0;
            self.setCflag(lastBit);
            val2shift>>=1;
            self.moveToDestination(&(val2shift as u16),&operand2);

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
        let mut dst=self.getOperandValue(&operand1,pmachine,pvga); 
        let reg:u16=dst;
        dst<<=shiftAmount;
        self.moveToDestination(&dst,&operand1);

        let mut lastBit:bool=false;
        if self.decInstr.instrSize==16 { lastBit=(reg&0x8000)!=0; }
        else if self.decInstr.instrSize==8 { lastBit=(reg&0x80)!=0; }
        
        self.setCflag(lastBit);
        self.doZflag(dst);
        self.doPflag(dst);

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performNeg(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags
        let operand1=self.decInstr.operand1.clone();

        let mut inum=self.getOperandValue(&operand1,pmachine,pvga) as i16 as i32; 
        inum=0-inum;
        let dst:u16=inum as u16;
        self.moveToDestination(&dst,&operand1);

        self.doZflag(dst);
        self.doSflag(dst,self.decInstr.instrSize);
        self.doPflag(dst);

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
            //self.moveToDestination(&dst,&operand1);
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
            self.abort("Unhandled IMUL 8bit");
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
            self.abort("Unhandled MUL 8bit");
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performDiv(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO 
        let operand1=self.decInstr.operand1.clone();

        if self.decInstr.instrSize==16
        {
            //self.moveToDestination(&dst,&operand1);

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
            self.abort("Unhandled DIV 8bit");
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performInc(&mut self)
    {
        // TODO all the flags
        let operand1=self.decInstr.operand1.clone();
        let mut val2inc:u16=0;

        if operand1=="AX" { val2inc=self.ax; if val2inc==0xffff { val2inc=0; } else { val2inc+=1; } self.ax=val2inc; }
        else if operand1=="CX" { val2inc=self.cx; if val2inc==0xffff { val2inc=0; } else { val2inc+=1; } self.cx=val2inc; }
        else if operand1=="DX" { val2inc=self.dx; if val2inc==0xffff { val2inc=0; } else { val2inc+=1; } self.dx=val2inc; }
        else if operand1=="BX" { val2inc=self.bx; if val2inc==0xffff { val2inc=0; } else { val2inc+=1; } self.bx=val2inc; }
        else if operand1=="SP" { val2inc=self.sp; if val2inc==0xffff { val2inc=0; } else { val2inc+=1; } self.sp=val2inc; }
        else if operand1=="BP" { val2inc=self.bp; if val2inc==0xffff { val2inc=0; } else { val2inc+=1; } self.bp=val2inc; }
        else if operand1=="SI" { val2inc=self.si; if val2inc==0xffff { val2inc=0; } else { val2inc+=1; } self.si=val2inc; }
        else if operand1=="DI" { val2inc=self.di; if val2inc==0xffff { val2inc=0; } else { val2inc+=1; } self.di=val2inc; }
        else if operand1=="AL" { val2inc=self.ax&0xff; if val2inc==0xff { val2inc=0; } else { val2inc+=1; } self.ax=(self.ax&0xff00)|(val2inc&0xff); }
        else if operand1=="AH" { val2inc=self.ax>>8; if val2inc==0xff { val2inc=0; } else { val2inc+=1; } self.ax=(self.ax&0xff)|(val2inc<<8); }
        else
        {
            self.abort(&format!("Unhandled performInc {} at {:04x}",operand1,self.ip));
        }

        self.doZflag(val2inc); 
        self.doPflag(val2inc); 
        self.doSflag(val2inc,self.decInstr.instrSize); 

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performDec(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO all the flags
        let operand1=self.decInstr.operand1.clone();

        // TODO all the flags
        let mut val2dec:u16=0;

        if self.decInstr.operand1=="AX" { val2dec=self.ax; if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } self.ax=val2dec; }
        else if self.decInstr.operand1=="CX" { val2dec=self.cx; if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } self.cx=val2dec; }
        else if self.decInstr.operand1=="DX" { val2dec=self.dx; if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } self.dx=val2dec; }
        else if self.decInstr.operand1=="BX" { val2dec=self.bx; if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } self.bx=val2dec; }
        else if self.decInstr.operand1=="BH" { val2dec=self.bx>>8; if val2dec==0 { val2dec=0xff; } else { val2dec-=1; } self.bx=(val2dec<<8)|(self.bx&0xff); }
        else if self.decInstr.operand1=="BL" { val2dec=self.bx&0xff; if val2dec==0 { val2dec=0xff; } else { val2dec-=1; } self.bx=(val2dec)|(self.bx&0xff00); }
        else if self.decInstr.operand1=="SP" { val2dec=self.sp; if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } self.sp=val2dec; }
        else if self.decInstr.operand1=="BP" { val2dec=self.bp; if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } self.bp=val2dec; }
        else if self.decInstr.operand1=="SI" { val2dec=self.si; if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } self.si=val2dec; }
        else if self.decInstr.operand1=="DI" { val2dec=self.di; if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } self.di=val2dec; }
        else if self.decInstr.operand1=="Direct Addr" 
        { 
            if self.decInstr.instrSize==8 { val2dec=pmachine.readMemory(self.ds,self.decInstr.u16immediate,pvga) as u16;  }
            else if self.decInstr.instrSize==16 { val2dec=pmachine.readMemory16(self.ds,self.decInstr.u16immediate,pvga);  }

            if self.decInstr.instrSize==8 { if val2dec==0 { val2dec=0xff; } else { val2dec-=1; } }
            else if self.decInstr.instrSize==16 { if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } }

            if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,self.decInstr.u16immediate,val2dec as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,self.decInstr.u16immediate,val2dec,pvga); }
        }
        else if self.decInstr.operand1.contains("[BP+Disp]")
        {
            let mut bp32:i32=self.bp as i32;
            bp32+=self.decInstr.displacement;

            if self.decInstr.instrSize==8 { val2dec=pmachine.readMemory(self.ss,bp32 as u16,pvga) as u16;  }
            else if self.decInstr.instrSize==16 { val2dec=pmachine.readMemory16(self.ss,bp32 as u16,pvga);  }

            if self.decInstr.instrSize==8 { if val2dec==0 { val2dec=0xff; } else { val2dec-=1; } }
            else if self.decInstr.instrSize==16 { if val2dec==0 { val2dec=0xffff; } else { val2dec-=1; } }

            if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ss,bp32 as u16,val2dec as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ss,bp32 as u16,val2dec,pvga); }
        }
        else
        {
            self.abort(&format!("Unhandled performDec {} at {:04x}",operand1,self.ip));
        }

        self.doZflag(val2dec); 
        self.doPflag(val2dec); 
        self.doSflag(val2dec,self.decInstr.instrSize); 

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

        if (self.decInstr.repPrefix=="REPE") || (self.decInstr.repPrefix=="REPNE")
        {
            self.abort("Unhandled REP");
        }

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

        if self.decInstr.repPrefix!=""
        {
            self.abort("Unhandled rep prefix in SCAS");
        }

        if self.decInstr.instrSize==16
        {
            // TODO REVIEW
            let dataw=pmachine.readMemory16(readSeg,self.di,pvga);

            let result:u16=if dataw>self.ax { 0xffff-dataw+1 } else { self.ax-dataw };

            self.doZflag(result);
            self.doPflag(result);
            self.doSflag(result,self.decInstr.instrSize);
            self.doCflag(result,self.decInstr.instrSize);

            if self.getDflag() { self.di-=2; }
            else { self.di+=2; }
        }
        else { self.abort("Unhandled scasb"); }
        /*else if self.decInstr.instrSize==8
        {
            let datab=pmachine.readMemory(readSeg,self.si,pvga);
            self.ax=(self.ax&0xff00)|(datab as u16);

            if self.getDflag() { self.si-=1; }
            else { self.si+=1; }
        }*/

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performStos(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO REPE STOSB and REPNE
        if self.decInstr.instrSize==16
        {
            if self.decInstr.repPrefix=="REPE"
            {
                if self.cx!=0
                {
                    pmachine.writeMemory16(self.es,self.di,self.ax,pvga);
                    if self.getDflag() { self.di-=2; }
                    else { self.di+=2; }
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

    fn performJmpShort(&mut self)
    {
        let jumpAmt=self.decInstr.operand1.parse::<i8>().unwrap();
        let mut performJump:bool=false;

        if &self.decInstr.debugDecode[0..2]=="JB" { if self.getCflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JAE" { if !self.getCflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JE" { if self.getZflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JNE" { if !self.getZflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JBE" { if self.getZflag() || self.getCflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JA" { if (!self.getCflag()) && (!self.getZflag()) { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JS" { if self.getSflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JNS" { if !self.getSflag() { performJump=true; } }
        else if &self.decInstr.debugDecode[0..2]=="JG" 
        { 
            let of:u8=if self.getOflag() { 1 } else { 0 };
            let sf:u8=if self.getSflag() { 1 } else { 0 };
            let res:bool=if (of^sf)==1 { true } else { false };
            if (self.getZflag()) || (!res)
            { 
                performJump=true; 
            } 
        }
        else if &self.decInstr.debugDecode[0..3]=="JLE" { if self.getZflag() || (self.getOflag()!=self.getSflag()) { performJump=true; } }
        else if &self.decInstr.debugDecode[0..3]=="JMP" { if true { performJump=true; } }
        else if &self.decInstr.debugDecode[0..4]=="LOOP" { self.cx-=1; if self.cx!=0 { performJump=true; } }

        if performJump
        {
            self.ip=self.ip.wrapping_add((jumpAmt+2) as u16);
        }
        else
        {
            self.ip+=2;
        }
    }

    fn isRegister(&mut self,rname:&String) -> bool
    {
        if (rname=="AX") || (rname=="BX") || (rname=="CX") || (rname=="DX") || (rname=="ES") || (rname=="SS") || (rname=="DS") ||
           (rname=="CS") || (rname=="BP") || (rname=="IP") || (rname=="DI") || (rname=="SI") || (rname=="SP") ||
           (rname=="AL") || (rname=="BL") || (rname=="CL") || (rname=="DL") ||
           (rname=="AH") || (rname=="BH") || (rname=="CH") || (rname=="DH")
        {
            return true;
        }

        return false;
    }

    fn getOperandValue(&mut self,regname:&String,pmachine:&mut machine,pvga:&mut vga) -> u16
    {
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
                let data:u16=pmachine.readMemory16(self.ds,self.decInstr.u16immediate,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(self.ds,self.decInstr.u16immediate,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[DI]" 
        { 
            let data:u16=pmachine.readMemory16(self.ds,self.di,pvga);
            return data; 
        }
        else if regname=="[BX]" 
        { 
            let data:u16=pmachine.readMemory16(self.ds,self.bx,pvga);
            return data; 
        }
        else if regname=="[BX+DI]" 
        { 
            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(self.ds,self.bx+self.di,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(self.ds,self.bx+self.di,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[BX+Disp] with 8bit disp" 
        { 
            let mut bx32:i32=self.bx as i32;
            bx32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(self.ds,bx32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(self.ds,bx32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[DI+Disp] with 8bit disp" 
        { 
            let mut di32:i32=self.di as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(self.ds,di32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(self.ds,di32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[BP+Disp] with 8bit disp" 
        { 
            let mut bp32:i32=self.bp as i32;
            bp32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16
            {
                let data:u16=pmachine.readMemory16(self.ss,bp32 as u16,pvga);
                return data; 
            }
            else if self.decInstr.instrSize==8
            {
                let data:u16=pmachine.readMemory(self.ss,bp32 as u16,pvga) as u16;
                return data; 
            }
        }
        else if regname=="[SI]" 
        { 
            let data:u16=pmachine.readMemory16(self.ds,self.si,pvga);
            return data; 
        }
        else if regname=="ib" 
        { 
            return self.decInstr.u8immediate as u16; 
        }
        else if regname=="iw" 
        { 
            return self.decInstr.u16immediate as u16; 
        }
        else
        {
            self.abort(&format!("Unhandled getOperandValue {}",regname));
        }

        return 0;
    }

    fn moveToDestination(&mut self,srcVal:&u16,dstReg:&String)
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
        else
        {
            self.abort(&format!("Unhandled moveToDestination {} {}",dstReg,srcVal));
        }
    }

    fn doCmp(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO: other regs
        let mut val2compare:i32=0;
        if dstReg=="AX" { val2compare=self.ax as i32; }
        else if dstReg=="AL" { val2compare=(self.ax&0xff) as i32; }
        else if dstReg=="CL" { val2compare=(self.cx&0xff) as i32; }
        else if dstReg=="BL" { val2compare=(self.bx&0xff) as i32; }
        else if dstReg=="DL" { val2compare=(self.dx&0xff) as i32; }
        else if dstReg=="BH" { val2compare=(self.bx>>8) as i32; }
        else if dstReg=="CH" { val2compare=(self.cx>>8) as i32; }
        else if dstReg=="DX" { val2compare=self.dx as i32; }
        else if dstReg.contains("[DI+Disp]")
        {
            let mut di32:i32=self.di as i32;
            di32+=self.decInstr.displacement;
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ds,di32 as u16,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ds,di32 as u16,pvga) as i32; }
        }
        else if dstReg.contains("[SI+Disp]")
        {
            let mut si32:i32=self.si as i32;
            si32+=self.decInstr.displacement;
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ds,si32 as u16,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ds,si32 as u16,pvga) as i32; }
        }
        else if dstReg.contains("[BP+Disp]")
        {
            let mut bp32:i32=self.bp as i32;
            bp32+=self.decInstr.displacement;
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ss,bp32 as u16,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ss,bp32 as u16,pvga) as i32; }
        }
        else if dstReg.contains("[BX]")
        {
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ds,self.bx,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ds,self.bx,pvga) as i32; }
        }
        else if dstReg.contains("[DI]")
        {
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ds,self.di,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ds,self.di,pvga) as i32; }
        }
        else if dstReg=="ib"
        {
            val2compare=self.decInstr.u8immediate as i32;            
        }
        else
        {
            self.abort(&format!("Unhandled doCmp {} {} at {:04x}",dstReg,srcVal,self.ip));
        }

        let data:i32=*srcVal as i32; 

        let cmpval:i32=val2compare-data;

        /*if val2compare<data { self.setSflag(true); }
        else { self.setSflag(false); }*/

        if self.decInstr.instrSize==8 { self.doSflag((cmpval&0xff) as u16,8); }
        else if self.decInstr.instrSize==16 { self.doSflag((cmpval&0xffff) as u16,16); }

        if val2compare<data { self.setCflag(true); }
        else { self.setCflag(false); }

        self.doZflag(cmpval as u16);
        self.doPflag(cmpval as u16);
    }

    fn doTest(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        // TODO: other flags
        let mut val2compare:i32=0;
        if dstReg=="AX" { val2compare=self.ax as i32; }
        else if dstReg=="AL" { val2compare=(self.ax&0xff) as i32; }
        else if dstReg=="AH" { val2compare=(self.ax>>8) as i32; }
        else if dstReg=="DL" { val2compare=(self.dx&0xff) as i32; }
        else if dstReg=="BH" { val2compare=(self.bx>>8) as i32; }
        else if dstReg=="DX" { val2compare=self.dx as i32; }
        else if dstReg.contains("[DI+Disp]")
        {
            let mut di32:i32=self.di as i32;
            di32+=self.decInstr.displacement;
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ds,di32 as u16,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ds,di32 as u16,pvga) as i32; }
        }
        else if dstReg.contains("[SI+Disp]")
        {
            let mut si32:i32=self.si as i32;
            si32+=self.decInstr.displacement;
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ds,si32 as u16,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ds,si32 as u16,pvga) as i32; }
        }
        else if dstReg.contains("[BX]")
        {
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ds,self.bx,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ds,self.bx,pvga) as i32; }
        }
        else if dstReg.contains("[DI]")
        {
            if self.decInstr.instrSize==16 { val2compare=pmachine.readMemory16(self.ds,self.di,pvga) as i32; }
            else if self.decInstr.instrSize==8 { val2compare=pmachine.readMemory(self.ds,self.di,pvga) as i32; }
        }
        else if dstReg=="ib"
        {
            val2compare=self.decInstr.u8immediate as i32;            
        }
        else
        {
            self.abort(&format!("Unhandled doTest {} {} at {:04x}",dstReg,srcVal,self.ip));
        }

        let data:i32=*srcVal as i32; 
        let cmpval:i32=val2compare&data;
        self.doZflag(cmpval as u16);
        self.doPflag(cmpval as u16);
    }

    fn doAnd(&mut self,srcVal:&u16,dstReg:&String)
    {
        let mut lop:u16=0;
        let mut rop:u16=*srcVal;

        if dstReg=="AH" { lop=self.ax>>8; rop&=0xff; lop&=rop; self.ax=((lop as u16)<<8)|(self.ax&0xff); }
        else if dstReg=="AL" { lop=self.ax&0xff; rop&=0xff; lop&=rop; self.ax=(self.ax&0xff00)|(lop&0xff); }
        else if dstReg=="CL" { lop=self.cx&0xff; rop&=0xff; lop&=rop; self.cx=(self.cx&0xff00)|(lop&0xff); }
        else if dstReg=="AX" { lop=self.ax; lop&=rop; self.ax=lop; }
        else
        {
            self.abort(&format!("Unhandled doAnd {} {} at {:04x}",dstReg,srcVal,self.ip));
        }

        self.doZflag(lop as u16);
        self.doPflag(lop as u16);
    }

    fn doAdd(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut rez:u16=0;

        if dstReg=="AX" 
        { 
            let valtoadd:i32=*srcVal as i32;
            let mut ax32:i32=self.ax as i32;
            ax32+=valtoadd;
            self.ax=ax32 as u16;
            rez=self.ax;
        }
        else if dstReg=="DI" 
        { 
            let valtoadd:i32=*srcVal as i32;
            let mut di32:i32=self.di as i32;
            di32+=valtoadd;
            self.di=di32 as u16;
            rez=self.di;
        }
        else if dstReg.contains("[DI+")
        { 
            let valtoadd:i32=*srcVal as i32;

            let mut dstop:i32=0;
            let mut di32:i32=self.di as i32;
            di32+=self.decInstr.displacement;

            if self.decInstr.instrSize==16 { dstop=pmachine.readMemory16(self.ds,di32 as u16,pvga) as i32; }
            else if self.decInstr.instrSize==8 { dstop=pmachine.readMemory(self.ds,di32 as u16,pvga) as i32; }
            dstop+=valtoadd;
            if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,di32 as u16,dstop as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,di32 as u16,dstop as u16,pvga); }

            rez=dstop as u16;
        }
        else if dstReg=="AL" 
        { 
            let valtoadd:i32=(*srcVal as u8) as i32;
            let mut al32:i32=(self.ax&0xff) as i32;
            al32+=valtoadd;
            self.ax=(self.ax&0xff00)|((al32&0xff) as u16);
            rez=self.ax&0xff;
        }
        else if dstReg=="CL" 
        { 
            let valtoadd:i32=(*srcVal as u8) as i32;
            let mut cl32:i32=(self.cx&0xff) as i32;
            cl32+=valtoadd;
            self.cx=(self.cx&0xff00)|((cl32&0xff) as u16);
            rez=self.cx&0xff;
        }
        else if dstReg=="DL" 
        { 
            let valtoadd:i32=(*srcVal as u8) as i32;
            let mut dl32:i32=(self.dx&0xff) as i32;
            dl32+=valtoadd;
            self.dx=(self.dx&0xff00)|((dl32&0xff) as u16);
            rez=self.dx&0xff;
        }
        else
        {
            self.abort(&format!("Unhandled doAdd {} {}",dstReg,srcVal));
        }

        self.doZflag(rez);
        self.doPflag(rez);
        self.doSflag(rez,self.decInstr.instrSize);
    }

    fn doAdc(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut rezult:i32=0;
        let carry:i32=if self.getCflag() { 1 } else { 0 };

        if dstReg=="[BX]" 
        { 
            let mut op:i32=0;
            if self.decInstr.instrSize==16 { op=pmachine.readMemory16(self.ds,self.bx,pvga) as i32; }
            else if self.decInstr.instrSize==8 { op=pmachine.readMemory(self.ds,self.bx,pvga) as i32; }
            let op2:i32=*srcVal as i32;
            let res:i32=op+op2+carry;
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,self.bx,res as u16,pvga); rezult=res; }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,self.bx,(res&0xff) as u8,pvga); rezult=res&0xff; }
        }
        else if dstReg=="AH" 
        { 
            let op:i32=(self.ax>>8) as i32;
            let res:i32=op+op+carry;
            self.ax=(self.ax&0xff)|((res<<8) as u16);
            rezult=(self.ax>>8) as i32;
        }
        else
        {
            self.abort(&format!("Unhandled doAdc {} {}",dstReg,srcVal));
        }

        // TODO other flags
        self.doZflag(rezult as u16);
    }

    fn doOr(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut lop=self.getOperandValue(&dstReg,pmachine,pvga);
        lop|=*srcVal;
        self.moveToDestination(&lop,&dstReg);

        // TODO other flags
        self.doSflag(lop as u16,self.decInstr.instrSize);
        self.doZflag(lop);
        self.doPflag(lop);
    }

    fn doSbb(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let mut result:u16=0;
        let op1=*srcVal;        
        let lop=self.getOperandValue(&dstReg,pmachine,pvga);

        if op1>lop { self.moveToDestination(&(0xffff-op1+1),&dstReg); result=0xffff-op1+1; }
        else { self.moveToDestination(&(lop-op1),&dstReg); result=lop-op1; }
        if self.getCflag() { self.di-=1; }

        if self.decInstr.instrSize==16
        {
            if (result&0x8000)==0x8000
            {
                self.setCflag(true);
                self.setSflag(true);
            }        
        }
        else if self.decInstr.instrSize==8
        {
            if (result&0x80)==0x80
            {
                self.setCflag(true);
                self.setSflag(true);
            }        
        }

        self.doZflag(result);
        self.doPflag(result);
    }

    fn doXor(&mut self,srcVal:&u16,dstReg:&String,pmachine:&mut machine,pvga:&mut vga)
    {
        let op1=*srcVal;
        let mut op2=0;

        if dstReg=="AX" { op2=self.ax; op2^=op1;self.ax=op2; }
        else if dstReg=="AL" { op2=self.ax&0xff; op2^=op1; self.ax=(self.ax&0xff00)|op2; }
        else if dstReg=="DX" { op2=self.dx; op2^=op1;self.dx=op2; }
        else if dstReg=="DI" { op2=self.di; op2^=op1;self.di=op2; }
        else if dstReg=="Direct Addr" 
        { 
            // TODO wrong, check for size of read data?
            op2=pmachine.readMemory(self.ds,self.decInstr.u16immediate,pvga) as u16; 
            op2^=op1; 
            if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,self.decInstr.u16immediate,op2 as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,self.decInstr.u16immediate,op2,pvga); }
        }
        else if dstReg.contains("[SI+Disp]")
        {
            let mut si32:i32=self.si as i32;
            si32+=self.decInstr.displacement;

            if self.decInstr.instrSize==8 { op2=pmachine.readMemory(self.ds,si32 as u16,pvga) as u16;  }
            else if self.decInstr.instrSize==16 { op2=pmachine.readMemory16(self.ds,si32 as u16,pvga);  }
            op2^=op1; 
            if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,si32 as u16,op2 as u8,pvga); }
            else if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,si32 as u16,op2,pvga); }
        }
        else
        {
            self.abort(&format!("Unhandled doXor {} {} at {:04x}",dstReg,srcVal,self.ip));
        }

        // TODO other flags
        self.setCflag(false);
        self.setOflag(false);
        self.doZflag(op2);
        self.doSflag(op2,self.decInstr.instrSize);
    }

    fn doSub(&mut self,srcVal:&u16,dstReg:&String)
    {
        let mut result:u16=0;

        if dstReg=="AX" 
        { 
            if *srcVal>self.ax { self.ax=0xffff-(*srcVal)+1; }
            else { self.ax-=*srcVal; }
            result=self.ax;
        }
        else if dstReg=="AL" 
        { 
            let mut al=self.ax&0xff;
            if *srcVal>al { al=0xff-(*srcVal)+1; }
            else { al-=*srcVal; }
            result=al;
            self.ax=(self.ax&0xff00)|al;
        }
        else if dstReg=="BX" 
        { 
            if *srcVal>self.bx { self.bx=0xffff-(*srcVal)+1; }
            else { self.bx-=*srcVal; }
            result=self.bx;
        }
        else if dstReg=="DI" 
        { 
            if *srcVal>self.di { self.di=0xffff-(*srcVal)+1; }
            else { self.di-=*srcVal; }
            result=self.di;
        }
        else
        {
            self.abort(&format!("Unhandled doSub {} {}",dstReg,srcVal));
        }

        self.doZflag(result);
        self.doPflag(result);
        self.doSflag(result,self.decInstr.instrSize);
        self.doCflag(result,self.decInstr.instrSize);
    }

    // when things get hard
    fn performMove(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        if self.isRegister(&dstReg) 
        { 
            self.moveToDestination(&srcVal,&dstReg); 
        }
        else if dstReg=="Direct Addr"
        {
            let mut writeSeg:u16=self.ds;
            if self.decInstr.segOverride=="CS" { writeSeg=self.cs; }
            else if self.decInstr.segOverride=="SS" { writeSeg=self.ss; }
            else if self.decInstr.segOverride=="DS" { writeSeg=self.ds; }
            else if self.decInstr.segOverride=="ES" { writeSeg=self.es; }
    
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(writeSeg,self.decInstr.u16immediate,srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(writeSeg,self.decInstr.u16immediate,(srcVal&0xff) as u8,pvga); }
        }
        else if dstReg=="[BX]"
        {
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,self.bx,srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,self.bx,srcVal as u8,pvga); }
        }
        else if dstReg=="[DI]"
        {
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,self.di,srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,self.di,srcVal as u8,pvga); }
        }
        else if dstReg=="[SI]"
        {
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,self.si,srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,self.si,srcVal as u8,pvga); }
        }
        else if dstReg.contains("[SI+Disp]")
        {
            let mut si32:i32=self.si as i32;
            si32+=self.decInstr.displacement;
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,si32 as u16,srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,si32 as u16,srcVal as u8,pvga); }
        }
        else if dstReg.contains("[DI+Disp]")
        {
            let mut di32:i32=self.di as i32;
            di32+=self.decInstr.displacement;
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ds,di32 as u16,srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ds,di32 as u16,srcVal as u8,pvga); }
        }
        else if dstReg.contains("[BP+Disp]")
        {
            let mut bp32:i32=self.bp as i32;
            bp32+=self.decInstr.displacement;
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ss,bp32 as u16,srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ss,bp32 as u16,srcVal as u8,pvga); }
        }
        else if dstReg.contains("[BP+DI]")
        {
            let mut di32:i32=self.di as i32;
            di32+=self.bp as i32;
            if self.decInstr.instrSize==16 { pmachine.writeMemory16(self.ss,di32 as u16,srcVal,pvga); }
            else if self.decInstr.instrSize==8 { pmachine.writeMemory(self.ss,di32 as u16,srcVal as u8,pvga); }
        }
        else
        {
            self.abort(&format!("Unhandled performMove {} {}",dstReg,srcReg));
        }

        self.ip+=self.decInstr.insLen as u16;
    }

    fn performSub(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        let srcVal:u16=self.getOperandValue(&srcReg,pmachine,pvga); 
        self.doSub(&srcVal,&dstReg); 

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
        self.doAnd(&srcVal,&dstReg); 

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

    fn performIn(&mut self)
    {
        let srcReg=self.decInstr.operand1.clone();
        let dstReg=self.decInstr.operand2.clone();

        if dstReg=="AL"
        { 
            // TODO IN from timer
            if self.decInstr.u8immediate==0x40
            {
                let num:u16 = rand::thread_rng().gen_range(0..256);
                self.ax=(self.ax&0xff00)|num;    
            }
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
        else if it=="Push" { return instructionType::instrPush; }
        else if it=="Pushf" { return instructionType::instrPushf; }
        else if it=="Popf" { return instructionType::instrPopf; }
        else if it=="Ret" { return instructionType::instrRet; }
        else if it=="Clc" { return instructionType::instrClc; }
        else if it=="Cld" { return instructionType::instrCld; }
        else if it=="Inc" { return instructionType::instrInc; }
        else if it=="Dec" { return instructionType::instrDec; }
        else if it=="IncNMRR" { return instructionType::instrIncNoModRegRm; }
        else if it=="DecNMRR" { return instructionType::instrDecNoModRegRm; }
        else if it=="Xchg" { return instructionType::instrXchg; }
        else if it=="Lods" { return instructionType::instrLods; }
        else if it=="Movs" { return instructionType::instrMovs; }
        else if it=="Stos" { return instructionType::instrStos; }
        else if it=="Scas" { return instructionType::instrScas; }
        else if it=="JmpShort" { return instructionType::instrJmpShort; }
        else if it=="JmpNp" { return instructionType::instrJmpNp; }
        else if it=="Int" { return instructionType::instrInt; }
        else if it=="CallReg" { return instructionType::instrCallReg; }
        else if it=="CallRel16" { return instructionType::instrCallRel16; }
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
        else if it=="Xlat" { return instructionType::instrXlat; }
        else if it=="In" { return instructionType::instrIn; }
        else if it=="Or" { return instructionType::instrOr; }
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
        else if it=="Imul" { return instructionType::instrImul; }
        else if it=="Mul" { return instructionType::instrMul; }
        else if it=="Div" { return instructionType::instrDiv; }
        else if it=="Lea" { return instructionType::instrLea; }
        else { return instructionType::instrNone; }
    }

    fn getOpcodeStructure(&self,opcode:&u16,found:&mut bool) -> [&str;7]
    {
        // opcode info format:
        // dbg instruction, instruction bit size, number of arguments, arg1, arg2, instruction type, invert operands

        match opcode
        {
            // POP 16 bit reg
            0x07 => { return ["POP ES","16","1","ES","","Pop","0"]; }
            0x17 => { return ["POP SS","16","1","SS","","Pop","0"]; }
            0x1f => { return ["POP DS","16","1","DS","","Pop","0"]; }
            0x58 => { return ["POP AX","16","1","AX","","Pop","0"]; }
            0x59 => { return ["POP CX","16","1","CX","","Pop","0"]; }
            0x5a => { return ["POP DX","16","1","DX","","Pop","0"]; }
            0x5b => { return ["POP BX","16","1","BX","","Pop","0"]; }
            0x5c => { return ["POP SP","16","1","SP","","Pop","0"]; }
            0x5d => { return ["POP BP","16","1","BP","","Pop","0"]; }
            0x5e => { return ["POP SI","16","1","SI","","Pop","0"]; }
            0x5f => { return ["POP DI","16","1","DI","","Pop","0"]; }
            // PUSH 16 bit reg
            0x06 => { return ["PUSH ES","16","1","ES","","Push","0"]; }
            0x0e => { return ["PUSH CS","16","1","CS","","Push","0"]; }
            0x16 => { return ["PUSH SS","16","1","SS","","Push","0"]; }
            0x1e => { return ["PUSH DS","16","1","DS","","Push","0"]; }
            0x50 => { return ["PUSH AX","16","1","AX","","Push","0"]; }
            0x51 => { return ["PUSH CX","16","1","CX","","Push","0"]; }
            0x52 => { return ["PUSH DX","16","1","DX","","Push","0"]; }
            0x53 => { return ["PUSH BX","16","1","BX","","Push","0"]; }
            0x54 => { return ["PUSH SP","16","1","SP","","Push","0"]; }
            0x55 => { return ["PUSH BP","16","1","BP","","Push","0"]; }
            0x56 => { return ["PUSH SI","16","1","SI","","Push","0"]; }
            0x57 => { return ["PUSH DI","16","1","DI","","Push","0"]; }
            // POPF, PUSHF
            0x9c => { return ["PUSHF","16","0","","","Pushf","0"]; }
            0x9d => { return ["POPF","16","0","","","Popf","0"]; }
            // boring mono-opcode instructions
            0xc3 => { return ["RET","16","0","","","Ret","0"]; }
            0xf8 => { return ["CLC","16","0","","","Clc","0"]; }
            0xfc => { return ["CLD","16","0","","","Cld","0"]; }
            0x99 => { return ["CWD","16","0","","","Cwd","0"]; }
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
            // XCHG 16bit registers
            0x91 => { return ["XCHG AX,CX","16","1","AX","CX","Xchg","0"]; }
            0x92 => { return ["XCHG AX,DX","16","1","AX","DX","Xchg","0"]; }
            0x93 => { return ["XCHG AX,BX","16","1","AX","BX","Xchg","0"]; }
            0x94 => { return ["XCHG AX,SP","16","1","AX","SP","Xchg","0"]; }
            0x95 => { return ["XCHG AX,BP","16","1","AX","BP","Xchg","0"]; }
            0x96 => { return ["XCHG AX,SI","16","1","AX","SI","Xchg","0"]; }
            0x97 => { return ["XCHG AX,DI","16","1","AX","DI","Xchg","0"]; }
            // LODSB/W
            0xac => { return ["LODSB","8","0","","","Lods","0"]; }
            0xad => { return ["LODSW","16","0","","","Lods","0"]; }
            // MOVSB/W
            0xa4 => { return ["MOVSB","8","0","","","Movs","0"]; }
            0xa5 => { return ["MOVSW","16","0","","","Movs","0"]; }
            // STOSB/W
            0xaa => { return ["STOSB","8","0","","","Stos","0"]; }
            0xab => { return ["STOSW","16","0","","","Stos","0"]; }
            // SCASB/W
            0xaf => { return ["SCASW","16","0","","","Scas","0"]; }
            // Jump short
            0x72 => { return ["JB","8","1","r0","","JmpShort","0"]; }
            0x73 => { return ["JAE","8","1","r0","","JmpShort","0"]; }
            0x74 => { return ["JE","8","1","r0","","JmpShort","0"]; }
            0x75 => { return ["JNE","8","1","r0","","JmpShort","0"]; }
            0x76 => { return ["JBE","8","1","r0","","JmpShort","0"]; }
            0x77 => { return ["JA","8","1","r0","","JmpShort","0"]; }
            0x78 => { return ["JS","8","1","r0","","JmpShort","0"]; }
            0x79 => { return ["JNS","8","1","r0","","JmpShort","0"]; }
            0x7E => { return ["JLE","8","1","r0","","JmpShort","0"]; }
            0x7F => { return ["JG","8","1","r0","","JmpShort","0"]; }
            0xE2 => { return ["LOOP Short","8","1","r0","","JmpShort","0"]; }
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
            0x22 => { return ["AND","8","2","rb","rmb","And","1"]; }
            0x24 => { return ["AND","8","2","ib","AL","AndNMRR","0"]; }
            0x25 => { return ["AND","16","2","iw","AX","AndNMRR","0"]; }
            // OR instructions
            0x08 => { return ["OR","8","2","rmb","rb","Or","0"]; }
            0x09 => { return ["OR","16","2","rmw","rw","Or","0"]; }
            // XOR instructions
            0x31 => { return ["XOR","16","2","rmw","rw","Xor","1"]; }
            0x32 => { return ["XOR","8","2","rb","rmb","Xor","1"]; }
            0x34 => { return ["XOR","8","2","ib","AL","XorNMRR","1"]; }
            // XLAT
            0xd7 => { return ["XLAT","16","0","","","Xlat","0"]; }            
            // IN
            0xe4 => { return ["IN","8","2","ib","AL","In","0"]; }            
            0xe5 => { return ["IN","16","2","ib","AX","In","0"]; }            
            // JMP np
            0xe9 => { return ["JMPNP","16","1","iw","","JmpNp","0"]; }            
            // ADD
            0x00 => { return ["ADD","8","2","rb","rmb","Add","1"]; }
            0x01 => { return ["ADD","16","2","rmw","rw","Add","0"]; }
            0x02 => { return ["ADD","8","2","rb","rmb","Add","1"]; }
            0x03 => { return ["ADD","16","2","rmw","rw","Add","1"]; }
            0x04 => { return ["ADD","8","2","ib","AL","AddNMRR","0"]; }
            0x05 => { return ["ADD","16","2","iw","AX","AddNMRR","0"]; }
            // ADC
            0x10 => { return ["ADC","8","2","rb","rmb","Adc","0"]; }
            // CMP
            0x3b => { return ["CMP","16","2","rmw","rw","Cmp","1"]; }
            0x38 => { return ["CMP","8","2","rmb","rb","Cmp","0"]; }
            0x3c => { return ["CMP","8","2","ib","AL","CmpNMRR","0"]; }
            0x3d => { return ["CMP","16","2","iw","AX","CmpNMRR","0"]; } // the 3-D instruction
            // SUB
            0x29 => { return ["SUB","16","2","rmw","rw","Sub","0"]; }
            0x2c => { return ["SUB","8","2","ib","AL","SubNMRR","0"]; }
            0x2d => { return ["SUB","16","2","iw","AX","SubNMRR","0"]; } // the bidimensional instruction
            // TEST
            0x84 => { return ["TEST","8","2","rmb","rmb","Test","0"]; }
            0xa8 => { return ["TEST","8","2","ib","AL","TestNMRR","0"]; }
            // SHL
            0xd1 => { return ["SHL","16","1","rmw","1","Shl","1"]; }            
            // LEA 
            0x8d => { return ["LEA","16","2","rmw","rw","Lea","1"]; }            

            // Multi-instructions
            0x8000 => { return ["ADD","8","2","ib","rmb","Add","0"]; }
            /*0x8003 => { return ["SBB","8","2","ib","rmb","Sbb","0"]; }*/
            0x8004 => { return ["AND","8","2","ib","rmb","And","0"]; }
            0x8005 => { return ["SUB","8","2","ib","rmb","Sub","0"]; }
            0x8006 => { return ["XOR","8","2","ib","rmb","Xor","0"]; }
            0x8007 => { return ["CMP","8","2","ib","rmb","Cmp","0"]; }
            0x8105 => { return ["SUB","16","2","iw","rmw","Sub","0"]; }
            0x8300 => { return ["ADD","16","2","ib","rmw","Add","0"]; }
            0xd204 => { return ["SHL","8","2","rmb","CL","Shl","1"]; }
            0xd205 => { return ["SHR","8","2","CL","rmb","Shr","0"]; }            
            0xf703 => { return ["NEG","16","1","rmw","","Neg","0"]; }
            0xf704 => { return ["MUL","16","1","rmw","","Mul","1"]; }
            0xf705 => { return ["IMUL","16","1","rmw","","Imul","1"]; }
            0xf706 => { return ["DIV","16","1","rmw","","Div","1"]; }
            0xfe00 => { return ["INC","8","1","rmb","","Inc","1"]; }
            0xfe01 => { return ["DEC","8","1","rmb","","Dec","1"]; }
            0xff01 => { return ["DEC","16","1","rmw","","Dec","1"]; }
            0xff02 => { return ["CALL","16","1","rw","","CallReg","0"]; }

            _ => { *found=false; }
        }

        return ["","","","","","",""];
    }

    fn prepareInstructionParameters(&self,opcodeInfo:&[&str;7],cs:u16,ip:u16,instrLen:&mut u8,dbgStr:&mut String,instrWidth:&u8,
                                    u8op:&mut u8,u16op:&mut u16,
                                    opsrc:&mut String,opdst:&mut String,displ:&mut i32,displSize:&mut u8,iType:&instructionType,
                                    pmachine:&machine,pvga:&vga)
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
        else if *iType==instructionType::instrCallReg
        {
            // CALL reg
            *displ=0;
            *displSize=0;

            let dstIsSegreg:u8=if *opsrc=="sr".to_string() { 1 } else { 0 };
            let mut wbit:u8=0;
            if *instrWidth==16 { wbit=1; }

            let addressingModeByte=pmachine.readMemory(cs,ip+1,pvga);
            let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,dstIsSegreg,wbit);

            dbgStr.push_str(&format!(" {}",&moveVec[0]));
            *opsrc=moveVec[0].clone();
            *opdst="".to_string();
            *instrLen=2;
        }
        else if (*iType==instructionType::instrMov) || 
                (*iType==instructionType::instrAnd) ||
                (*iType==instructionType::instrOr) ||
                (*iType==instructionType::instrTest) ||
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
                (*iType==instructionType::instrImul) ||
                (*iType==instructionType::instrMul) ||
                (*iType==instructionType::instrDiv) ||
                (*iType==instructionType::instrLea) ||
                (*iType==instructionType::instrCmp)
        {
            // instructions with modregrm byte
            let mut totInstrLen:u8=2;
            let dstIsSegreg:u8=if *opsrc=="sr".to_string() { 1 } else { 0 };
            let mut wbit:u8=0;
            if *instrWidth==16 { wbit=1; }
            let mut operandAdder=0;
            let numOperands=opcodeInfo[2].parse::<u8>().unwrap();
            let invertOperands=opcodeInfo[6].parse::<u8>().unwrap();
            //let invertOperands=if (opcode&2)==2 { 1 } else { 0 };

            let addressingModeByte=pmachine.readMemory(cs,ip+1,pvga);
            let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,dstIsSegreg,wbit);

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
                *u16op=pmachine.readMemory16(cs,ip+2,pvga);
                totInstrLen+=2;
            }
            if *opsrc=="Direct Addr"
            {
                *u16op=pmachine.readMemory16(cs,ip+2,pvga);
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
                (*iType==instructionType::instrXorNoModRegRm) ||
                (*iType==instructionType::instrIncNoModRegRm) ||
                (*iType==instructionType::instrDecNoModRegRm) ||
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
                *u16op=pmachine.readMemory16(cs,ip+1,pvga);
                *instrLen+=2;
                realOpsrc=format!("[{:04x}]",*u16op);
            }

            if *opdst=="Direct Addr"
            {
                *u16op=pmachine.readMemory16(cs,ip+1,pvga);
                *instrLen+=2;
                realOpdst=format!("[{:04x}]",*u16op);
            }

            if numOperands==1 { dbgStr.push_str(&format!(" {}",realOpdst)); }
            else { dbgStr.push_str(&format!(" {},{}",realOpdst,realOpsrc)); } 
        }
        else if *iType==instructionType::instrIn
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

            dbgStr.push_str(&format!(" {},{}",realOpdst,realOpsrc));
        }
    }

    fn expandWideInstruction(&self,opcode:&u8,wideOpcode:&mut u16,pmachine:&machine,pvga:&vga,cs:&u16,ip:&u16)
    {
        if (*opcode==0xf7) || (*opcode==0xff) || (*opcode==0x80) || (*opcode==0xfe) || (*opcode==0xd2) ||
           (*opcode==0x81) || (*opcode==0x83)
        {
            let instrType=pmachine.readMemory(*cs,*ip,pvga);
            let reg:usize=((instrType>>3)&0x07).into();
            *wideOpcode=(reg as u16)|((*opcode as u16)<<8);
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

        let mut canDecode:bool=false;
        let mut soroAdder:u16=0;

        let mut segOverride:String=String::from("");
        let mut repOverride:String=String::from("");
        let mut opcode=pmachine.readMemory(cs,ip,pvga);

        // handle seg overrides
        if opcode==0x2e { segOverride="CS".to_string(); }
        else if opcode==0x36 { segOverride="SS".to_string(); }
        else if opcode==0x3e { segOverride="DS".to_string(); }
        else if opcode==0x26 { 
            segOverride="ES".to_string(); 
        }
        if segOverride!="" { opcode=pmachine.readMemory(cs,ip+1,pvga); soroAdder+=1; }

        // handle repetition prefix
        if opcode==0xf3 { repOverride="REPE".to_string(); }
        else if opcode==0xf2 { repOverride="REPNE".to_string(); }
        if repOverride!="" { opcode=pmachine.readMemory(cs,ip+1,pvga); soroAdder+=1; }

        let mut instrLen=1;
        /*if segOverride!="" { instrLen+=1; }
        if repOverride!="" { instrLen+=1; }*/

        // decode instruction
        let mut wasDecoded=true;
        let mut wideOpcode:u16=opcode as u16;
        self.expandWideInstruction(&opcode,&mut wideOpcode,pmachine,pvga,&cs,&(ip+1+(soroAdder as u16)));
        let opcodeInfo=self.getOpcodeStructure(&wideOpcode,&mut wasDecoded);
        if wasDecoded
        {
            canDecode=true;

            let mut dbgDec:String=String::from("");//opcodeInfo[0].to_string();
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
            let instrType=self.getInstructionType(&opcodeInfo[5].to_string());
            self.prepareInstructionParameters(&opcodeInfo,cs,ip+soroAdder,&mut instrLen,&mut dbgDec,
                                              &instrWidth,
                                              &mut u8op,&mut u16op,
                                              &mut operandSrc,&mut operandDst,
                                              &mut displacement,&mut displSize,
                                              &instrType,pmachine,pvga);
            instrLen+=soroAdder as u8;
            self.decInstr=decodedInstruction {
                insType: instrType,
                insLen: instrLen,
                //numOperands: opcodeInfo[2].parse::<u8>().unwrap(),
                instrSize: instrWidth,
                operand1: operandSrc,
                operand2: operandDst,
                displacement: displacement,
                //displSize: displSize,
                u8immediate: u8op,
                u16immediate: u16op,
                segOverride: segOverride,
                repPrefix: repOverride,
                debugDecode: dbgDec,
                //opcode: wideOpcode
            };
        }

        return canDecode;
    }

    pub fn exeCute(&mut self,pmachine:&mut machine,pvga:&mut vga)
    {
        if self.decInstr.insType==instructionType::instrPop
        {
            let popdval=pmachine.pop16(self.ss,self.sp);
            if self.decInstr.operand1=="ES" { self.es=popdval; }
            else if self.decInstr.operand1=="SS" { self.ss=popdval; }
            else if self.decInstr.operand1=="DS" { self.ds=popdval; }
            else if self.decInstr.operand1=="AX" { self.ax=popdval; }
            else if self.decInstr.operand1=="CX" { self.cx=popdval; }
            else if self.decInstr.operand1=="DX" { self.dx=popdval; }
            else if self.decInstr.operand1=="BX" { self.bx=popdval; }
            else if self.decInstr.operand1=="SP" { self.sp=popdval; }
            else if self.decInstr.operand1=="BP" { self.bp=popdval; }
            else if self.decInstr.operand1=="SI" { self.si=popdval; }
            else if self.decInstr.operand1=="DI" { self.di=popdval; }
            self.sp+=2;
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrPush
        {
            if self.decInstr.operand1=="AX" { pmachine.push16(self.ax,self.ss,self.sp); }
            else if self.decInstr.operand1=="BX" { pmachine.push16(self.bx,self.ss,self.sp); }
            else if self.decInstr.operand1=="CX" { pmachine.push16(self.cx,self.ss,self.sp); }
            else if self.decInstr.operand1=="DI" { pmachine.push16(self.di,self.ss,self.sp); }
            else if self.decInstr.operand1=="DX" { pmachine.push16(self.dx,self.ss,self.sp); }
            else if self.decInstr.operand1=="SI" { pmachine.push16(self.si,self.ss,self.sp); }
            else if self.decInstr.operand1=="BP" { pmachine.push16(self.bp,self.ss,self.sp); }
            else if self.decInstr.operand1=="DS" { pmachine.push16(self.ds,self.ss,self.sp); }
            else if self.decInstr.operand1=="ES" { pmachine.push16(self.es,self.ss,self.sp); }
            else if self.decInstr.operand1=="CS" { pmachine.push16(self.cs,self.ss,self.sp); }
            else if self.decInstr.operand1=="SS" { pmachine.push16(self.ss,self.ss,self.sp); }
            else { self.abort("unhandled push"); }
            self.sp-=2;
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrRet
        {
            // RET (near)
            let newip=pmachine.pop16(self.ss,self.sp);
            self.sp+=2;
            self.ip=newip;
        }
        else if self.decInstr.insType==instructionType::instrClc
        {
            // CLC
            self.setCflag(false);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrCld
        {
            // CLD
            self.setDflag(false);
            self.ip+=self.decInstr.insLen as u16;
        }
        else if self.decInstr.insType==instructionType::instrCwd
        {
            // CWD TODO really works this way?
            self.dx=if (self.ax&0x8000)==0x8000 { 1 } else { 0 };
            self.ip+=self.decInstr.insLen as u16;
        }
        else if (self.decInstr.insType==instructionType::instrInc) || (self.decInstr.insType==instructionType::instrIncNoModRegRm)
        {
            self.performInc();
        }
        else if (self.decInstr.insType==instructionType::instrDec) || (self.decInstr.insType==instructionType::instrDecNoModRegRm)
        {
            self.performDec(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrXchg
        {
            // XCHG reg16,reg16
            self.xchgRegs(self.decInstr.operand1.clone(),self.decInstr.operand2.clone());
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
        else if self.decInstr.insType==instructionType::instrXlat
        {
            // XLAT
            self.performXlat(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrJmpShort
        {
            // JE/JNE/etc.
            self.performJmpShort();
        }
        else if self.decInstr.insType==instructionType::instrInt
        {
            // INT nn
            let intNum=self.decInstr.operand1.parse::<u8>().unwrap();
            pmachine.handleINT(self,intNum,pvga);
            self.ip+=2;
        }
        else if self.decInstr.insType==instructionType::instrJmpNp
        {
            pmachine.push16(self.ip+3,self.ss,self.sp);
            self.sp-=2;
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
            // call register
            let addrRel:u16=self.getOperandValue(&self.decInstr.operand1.clone(),pmachine,pvga);
            pmachine.push16(self.ip+2,self.ss,self.sp);
            self.sp-=2;
            self.ip=addrRel;
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
        else if self.decInstr.insType==instructionType::instrOr
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
        else if self.decInstr.insType==instructionType::instrSbb
        {
            self.performSbb(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrLea
        {
            self.performLea();
        }
        else if (self.decInstr.insType==instructionType::instrXor) || (self.decInstr.insType==instructionType::instrXorNoModRegRm)
        {
            self.performXor(pmachine,pvga);
        }
        else if self.decInstr.insType==instructionType::instrAdc
        {
            self.performAdc(pmachine,pvga);
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
        else if self.decInstr.insType==instructionType::instrIn
        {
            self.performIn();
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
        else
        {
            self.abort(&format!("Cannot exeCute [{}].",self.decInstr.debugDecode));
        }
    }

    //
    //
    //

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

    fn setCflag(&mut self,val:bool)
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
        if val==0 { self.setZflag(true); }
        else { self.setZflag(false); }
    }

    fn doPflag(&mut self,val:u16)
    {
        let numOnes=(val&0xff).count_ones();
        if (numOnes%2)==0 { self.setPflag(true); }
        else { self.setPflag(false); }
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

        return retStr;
    }

    pub fn executeOne(&mut self,pmachine:&mut machine,pvga:&mut vga,debugFlag:bool,bytesRead:&mut u8,dbgCS:&u16,dbgIP:&u16,retErr:&mut String) -> String
    {
        /* decode&execute phases */
        let mut tmpcs:u16=self.cs;
        let mut tmpip:u16=self.ip;
        if debugFlag
        {
            tmpcs=*dbgCS;
            tmpip=*dbgIP;
        }
        let canDecode=self.dekode(pmachine,pvga,tmpcs,tmpip);
        if canDecode
        {
            let mut dbgAddress:String=self.prepareDbgInfo(tmpcs,tmpip,pmachine,pvga);
            dbgAddress.push_str(" ");
            dbgAddress.push_str(&self.decInstr.debugDecode);
            *bytesRead=self.decInstr.insLen;
            if debugFlag==false { self.exeCute(pmachine,pvga); self.totInstructions+=1; }
            return dbgAddress;
        }

        /* legacy handler */
        let mut dbgString:String=String::from("");

        let mut theCS=self.cs;
        let mut theIP=self.ip;
        if debugFlag
        {
            theCS=*dbgCS;
            theIP=*dbgIP;
        }

        *bytesRead=0;

        let opcode=pmachine.readMemory(theCS,theIP,pvga);
        let mut dbgAddress:String=format!("{:04x}:{:04x} ({:02x}) ",theCS,theIP,opcode);

        if !debugFlag
        {
            self.totInstructions+=1;
        }

        match opcode
        {
            0x81 =>
            {
                // the most complex instruction in the world
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if reg==0
                {
                    // ADD rmw,iw
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let data=pmachine.readMemory16(theCS,theIP+2,pvga);
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                    dbgString.push_str("ADD ");
                    dbgString.push_str(&moveVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:04x}",data));
                    *bytesRead=4;

                    if debugFlag==false
                    {
                        // TODO other regs + flags
                        let mut dest:i32=0;
                        let src:i32=data as i32;

                        if moveVec[0]=="DI" { dest=self.di as i32; self.di+=data; }
                        else if moveVec[0]=="SI" { dest=self.si as i32; self.si+=data; }
                        else { self.abort("add"); }

                        if (dest+src)>0xffff { self.setCflag(true); }
                        else { self.setCflag(false); }
                        self.doZflag((dest+src) as u16);
                        self.doPflag((dest+src) as u16);

                        self.ip+=4;
                    }
                }
                else if reg==7
                {
                    // CMP rmw,iw
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let data=pmachine.readMemory16(theCS,theIP+2,pvga) as i32;
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
    
                    dbgString.push_str("CMP ");
                    dbgString.push_str(&moveVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:04x}",data));
                    *bytesRead=4;
    
                    if debugFlag==false
                    {
                        // TODO overflow flag. DS:DI is correct?
                        let mut val2compare:i32=0;
                        if moveVec[0]=="SI" { val2compare=self.si as i32; }
                        let cmpval:i32=val2compare-data;
    
                        if val2compare<data { self.setSflag(true); }
                        else { self.setSflag(false); }
    
                        if val2compare<data { self.setCflag(true); }
                        else { self.setCflag(false); }
    
                        self.doZflag(cmpval as u16);
                        self.doPflag(cmpval as u16);
    
                        self.ip+=4;
                    }
                }
                else
                {
                    self.abort(&format!("unhandled 0x81 instruction reg={}",reg));
                }
            },
            0x83 =>
            {
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if reg==3
                {
                    // SBB rmw,ib
                    let rmw=pmachine.readMemory(theCS,theIP+1,pvga);
                    let ib:u16=pmachine.readMemory(theCS,theIP+2,pvga) as u16;
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(rmw,0,opcode&1);
                    dbgString=format!("SBB {},0x{:02x}",moveVec[0],ib);
                    *bytesRead=3;

                    if debugFlag==false
                    {
                        // TODO
                        let mut result:u16=0;
                        if moveVec[0]=="AX"
                        {
                            if ib>self.ax { self.ax=0xffff-ib+1; }
                            else { self.ax-=ib as u16; }
                            if self.getCflag() { self.ax-=1; }
                            result=self.ax;
                        }
                        else if moveVec[0]=="BX"
                        {
                            if ib>self.bx { self.bx=0xffff-ib+1; }
                            else { self.bx-=ib as u16; }
                            if self.getCflag() { self.bx-=1; }
                            result=self.bx;
                        }
                        else if moveVec[0]=="DI"
                        {
                            if ib>self.di { self.di=0xffff-ib+1; }
                            else { self.di-=ib as u16; }
                            if self.getCflag() { self.di-=1; }
                            result=self.di;
                        }
                        else
                        {
                            self.abort("Unhandled SBB");
                        }

                        self.doZflag(result);
                        self.doPflag(result);

                        if (result&0x8000)==0x8000
                        {
                            // right?
                            self.setCflag(true);
                            self.setSflag(true);
                        }

                        self.ip+=3;
                    }
                }
                else if reg==4
                {
                    // AND rmw,ib
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    let ib:u8=pmachine.readMemory(theCS,theIP+2,pvga);
    
                    dbgString.push_str("AND ");
                    dbgString.push_str(&moveVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:02x}",ib));
                    *bytesRead=3;
    
                    if debugFlag==false
                    {
                        // TODO flags, other regs
                        let mut lop:u16=0;
                        if moveVec[0]=="BP"
                        {
                            lop=self.bp;
                            let rop:u16=ib as u16;
                            lop&=rop;
                            self.bp=lop;
                        }
                        else
                        {
                            self.abort(&format!("and rmw,ib {} {}",moveVec[0],ib));
                        }
    
                        self.doZflag(lop as u16);
                        self.doPflag(lop as u16);
    
                        self.ip+=3;
                    }
    
                }
                else if reg==5
                {
                    // SUB rmw,ib
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let ib=pmachine.readMemory(theCS,theIP+2,pvga);
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                    dbgString.push_str("SUB ");
                    dbgString.push_str(&moveVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:04x}",ib));
                    *bytesRead=3;
    
                    if debugFlag==false
                    {
                        // TODO flags
                        let src:u16=ib as u16;
                        let mut result:u16=0;
                        if moveVec[0]=="BX" 
                        { 
                            if src>self.bx { self.bx=0xffff-src+1; }
                            else { self.bx-=src; }
                            result=self.bx;
                        }
                        else if moveVec[0]=="DI"
                        {
                            if src>self.di { self.di=0xffff-src+1; }
                            else { self.di-=src; }
                            result=self.di;
                        }

                        self.doZflag(result);
                        self.doPflag(result);

                        if (result&0x8000)==0x8000
                        {
                            self.setCflag(true);
                            self.setSflag(true);
                        }

                        self.ip+=3;
                    }
    
                }
                else
                {
                    if debugFlag==false
                    {
                        self.abort(&format!("unhandled 0x83 reg={}",reg));
                    }
                }
            },
            0xd0 =>
            {
                // f*cking multi instruction...
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if reg==4
                {
                    // SHL rmb,1
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    dbgString=format!("SHL {},1",regVec[0]);
                    *bytesRead=2;

                    if debugFlag==false
                    {
                        // TODO other regs, a,s flags
                        let mut val2shift:u16=0;
                        if regVec[0]=="AL" { val2shift=self.ax&0xff; }
                        else if regVec[0]=="BL" { val2shift=self.bx&0xff; }
                        else { self.abort("shl"); }

                        let lastBit:bool=(val2shift&0x80)!=0;
                        self.setCflag(lastBit);
                        if regVec[0]=="AL" { self.ax=(self.ax&0xff00)|((val2shift<<1)&0xff); }
                        else if regVec[0]=="BL" { self.bx=(self.bx&0xff00)|((val2shift<<1)&0xff); }

                        self.doZflag((val2shift<<1) as u16);
                        self.doPflag((val2shift<<1) as u16);
                        self.doSflag((val2shift<<1) as u16,16);

                        self.ip+=2;
                    }
                }
                else if reg==5
                {
                    // SHR rmb,1
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    dbgString=format!("SHR {},1",regVec[0]);
                    *bytesRead=2;

                    if debugFlag==false
                    {
                        // TODO other regs, a,s flags
                        let mut val2shift:u16=0;
                        if regVec[0]=="AL" { val2shift=self.ax&0xff; }
                        else if regVec[0]=="BL" { val2shift=self.bx&0xff; }
                        else { self.abort("shr"); }

                        let lastBit:bool=(val2shift&0x1)!=0;
                        self.setCflag(lastBit);
                        if regVec[0]=="AL" { self.ax=(self.ax&0xff00)|((val2shift>>1)&0xff); }
                        else if regVec[0]=="BL" { self.bx=(self.bx&0xff00)|((val2shift>>1)&0xff); }

                        self.doZflag((val2shift>>1) as u16);
                        self.doPflag((val2shift>>1) as u16);
                        self.doSflag((val2shift>>1) as u16,16);

                        self.ip+=2;
                    }
                }
                else
                {
                    self.abort("unhandl'd d0");
                }

            },
            0xf6 =>
            {
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if reg==0
                {
                    // TEST rmb,ib
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    let ib:u8=pmachine.readMemory(theCS,theIP+2,pvga);

                    dbgString.push_str("TEST ");
                    dbgString.push_str(&regVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:02x}",ib));
                    *bytesRead=3;
    
                    if debugFlag==false
                    {
                        let mut val2compare:i16=0;
                        let mut i16ib:i16=0;
                        if regVec[0]=="BL"
                        {
                            val2compare=(self.bx&0xff) as i16;
                            i16ib=ib as i16;
                        }
                        else
                        {
                            self.abort("Unhandled TEST rmb,ib");
                        }
    
                        let cmpval:i16=(val2compare&i16ib) as i16;
                        self.doZflag(cmpval as u16);
                        self.doPflag(cmpval as u16);
    
                        self.ip+=3;
                    }
                }
                else if reg==6
                {
                    // DIV rmb
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    dbgString=format!("DIV {}",regVec[0]);
                    *bytesRead=2;
    
                    if debugFlag==false
                    {
                        // TODO other regs, flags
                        if regVec[0]=="CL"
                        {
                            let val2divide:u32=self.ax as u32;
                            let modulo=val2divide%((self.cx&0xff) as u32);
                            let quotient=val2divide/((self.cx&0xff) as u32);
                            self.ax=((quotient as u16)&0xff)|((modulo as u16)<<8);

                            self.doZflag(quotient as u16);
                            //self.doPflag(quotient as u16); // todo check p flag
                        }
                        else
                        {
                            self.abort(&format!("Unhandled DIV {}",regVec[0]));
                        }

                        self.ip+=2;
                    }

                }
                else
                {
                    self.abort("0xf6");
                }
            },
            _ =>
            {
                *retErr=format!("x86cpu::Unhandled opcode 0x{:02x}",opcode);
                dbgString="UNKNOWN".to_string();
                *bytesRead=0;

                // abort only if executing
                if debugFlag==false
                {
                    self.abort(&format!("x86cpu::Unhandled opcode 0x{:02x} at {:04x}",opcode,self.ip));
                }
            }
        }

        dbgAddress.push_str(&dbgString);
        return dbgAddress;
    }
}
