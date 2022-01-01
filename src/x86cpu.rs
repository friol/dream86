
/* 
    8086 cpu - dream86 2o21 

    TODO:
    - rewrite all the get/set flags functions as one
    - fucking shorter & more compact code
    - find a solution for the "any register to any register" fucking instructions
    - remove that fucking warnings
    - optimize. I think we are slow
    
*/

use std::process;
use std::collections::HashMap;

use crate::vga::vga;
use crate::machine::machine;

//

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
    pub totInstructions: u64
}

#[derive(PartialEq)]
pub enum instructionType
{
    instrPop,
    instrMov,
    instrInc,
    instrDec
}

pub struct decodedInstruction
{
    insLen: u8,
    numOperands: u8,
    instrSize: u8,
    operand1: String,
    operand2: String,
    segOverride: String,
    repPrefix: String,
    debugDecode: String
}    

impl x86cpu
{
    pub fn new() -> Self 
    {
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
            ds: 0,
            es: 0,
            ss: 0xf000,
            flags: 0,
            totInstructions: 0
        }
    }

    pub fn getRegisters(&self) -> HashMap<String,u16>
    {
        let mut retHashMap=HashMap::from(
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

    fn xchgRegs(&mut self,xchgstr:String)
    {
        // TODO all regs combinations
        //(0x91,"AX,CX"),(0x92,"AX,DX"),(0x93,"AX,BX"),(0x94,"AX,SP"),(0x95,"AX,BP"),(0x96,"AX,SI"),(0x97,"AX,DI")

        let mut rgs=xchgstr.split(",");
        let r0=rgs.next().unwrap();
        let r1=rgs.next().unwrap();

        if (r0=="AX") && (r1=="CX") { let tmp=self.ax; self.ax=self.cx; self.cx=tmp; }
        else if (r0=="AX") && (r1=="BX") { let tmp=self.ax; self.ax=self.bx; self.bx=tmp; }
        else if (r0=="AX") && (r1=="DX") { let tmp=self.ax; self.ax=self.dx; self.dx=tmp; }
        else if (r0=="AX") && (r1=="DI") { let tmp=self.ax; self.ax=self.di; self.di=tmp; }
        else { self.abort("xchg not supported"); }
    }

    fn dekode(&self,pmachine:&machine,pvga:&vga,cs:u16,ip:u16) -> bool
    {
        //
        // decode an 8086 instruction
        // get:
        // - instruction lenght in bytes (this is useful to increment IP)
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

        let mut segOverride:String=String::from("");
        let mut repOverride:String=String::from("");
        let mut opcode=pmachine.readMemory(cs,ip,pvga);

        // handle seg overrides
        if opcode==0x2e { segOverride="CS".to_string(); }
        else if opcode==0x36 { segOverride="SS".to_string(); }
        else if opcode==0x3e { segOverride="DS".to_string(); }
        else if opcode==0x26 { segOverride="ES".to_string(); }
        if segOverride!="" { opcode=pmachine.readMemory(cs,ip+1,pvga); }

        // handle repetition prefix
        if opcode==0xf3 { repOverride="REPE".to_string(); }
        else if opcode==0xf2 { repOverride="REPNE".to_string(); }
        if repOverride!="" { opcode=pmachine.readMemory(cs,ip+1,pvga); }

        // one byte opcodes

        let oneByteOpcodes=HashMap::from(
            [
                (0x07,["POP ES","ES"]),
                (0x17,["POP SS","SS"]),
                (0x1f,["POP DS","DS"]),
                (0x58,["POP AX","AX"]),
                (0x59,["POP CX","CX"]),
                (0x5a,["POP DX","DX"]),
                (0x5b,["POP BX","BX"]),
                (0x5c,["POP SP","SP"]),
                (0x5d,["POP BP","BP"]),
                (0x5e,["POP SI","SI"]),
                (0x5f,["POP DI","DI"])
            ]
        );

        if oneByteOpcodes.contains_key(&opcode)
        {

        }


        return canDecode; // couldn't decode the instruction
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
            self.flags|=(1<<6);
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
            self.flags|=(1<<7);
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

    fn abort(&self,s:&str)
    {
        println!("bailing out due to {}...",s);
        process::exit(0x0100);
    }

    pub fn executeOne(&mut self,pmachine:&mut machine,pvga:&mut vga,debugFlag:bool,bytesRead:&mut u8,dbgCS:&u16,dbgIP:&u16,retErr:&mut String) -> String
    {
        let mut dbgString:String=String::from("");
        let mut dbgAddress:String=String::from("");
        let mut segOverride:String=String::from("");
        let mut repOverride:String=String::from("");

        let mut theCS=self.cs;
        let mut theIP=self.ip;
        if debugFlag
        {
            theCS=*dbgCS;
            theIP=*dbgIP;
        }

        *bytesRead=0;

        let mut opcode=pmachine.readMemory(theCS,theIP,pvga);
        dbgAddress=format!("{:04x}:{:04x} ({:02x}) ",theCS,theIP,opcode);

        if !debugFlag
        {
            self.totInstructions+=1;
        }

        // handle seg overrides
        if opcode==0x2e
        {
            segOverride="CS".to_string();
            *bytesRead=1;
            opcode=pmachine.readMemory(theCS,theIP+1,pvga);
        }

        // handle repetition 
        if opcode==0xf3
        {
            repOverride="REPE".to_string();
            *bytesRead=1;
            opcode=pmachine.readMemory(theCS,theIP+1,pvga);
        }

        match opcode
        {
            0x01 =>
            {
                // ADD rmw,rw
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                dbgString.push_str("ADD ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags, other regs
                    if (moveVec[0]=="DI") && (moveVec[1]=="BX")
                    {
                        let mut di32:i32=self.di as i32;
                        let mut bx32:i32=self.bx as i32;
                        di32+=bx32;
                        self.di=di32 as u16;
                        self.doZflag(self.di);
                        self.doPflag(self.di);
                        self.doSflag(self.di,16);
                    }
                    else
                    {
                        self.abort(&format!("unhandled registers in add rmw,rw {} {}",moveVec[0],moveVec[1]));
                    }
                    self.ip+=2;
                }
            },
            0x10 =>
            {
                // ADC rmb,rb
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("ADC ");
                dbgString.push_str(&moveVec[1]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[0]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags
                    if (moveVec[0]=="AH") && (moveVec[1]=="AH")
                    {
                        // private static final int[] MASK   = new int[] { 0xff, 0xffff };
                        // final int carry = (flags & CF) == CF ? 1 : 0;
                        // final int res = dst + src + carry & MASK[w];                                                

                        let carry=if self.getCflag() { 1 } else { 0 };
                        let op=(self.ax>>8);
                        let res:u16=op+op+carry;
                        self.ax=(self.ax&0xff)|(res<<8);

                        self.doZflag(self.ax>>8);
                    }
                    else
                    {
                        self.abort("adc");
                    }
                    self.ip+=2;
                }
            },
            0x04 =>
            {
                // ADD     AL,ib
                let ib=pmachine.readMemory(theCS,theIP+1,pvga) as i16;
                dbgString.push_str("ADD AL,");
                dbgString.push_str(&format!("{:02x}",ib));
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags
                    let mut al:i16=(self.ax&0xff) as i16;
                    al+=ib;
                    self.ax=(self.ax&0xff00)|(al&0xff) as u16;
                    self.ip+=2;
                }
            },
            0x05 =>
            {
                // ADD     AX,iw
                let data=pmachine.readMemory16(theCS,theIP+1,pvga);

                dbgString.push_str(&format!("ADD AX,0x{:04x}",data));
                *bytesRead=3;

                if debugFlag==false
                {
                    let mut dest:i32=0;
                    let mut src:i32=data as i32;

                    dest=self.ax as i32; 
                    self.ax+=data; 

                    if (dest+src)>0xffff { self.setCflag(true); }
                    else { self.setCflag(false); }
                    self.doZflag((dest+src) as u16);
                    self.doPflag((dest+src) as u16);

                    self.ip+=3;
                }
            },
            0x07 | 0x17 | 0x1f | 0x58 | 0x59 | 0x5a | 0x5b | 0x5c | 0x5d | 0x5e | 0x5f =>
            {
                // POP register
                let regHashMap = HashMap::from(
                    [
                        (0x07,"ES"),(0x17,"SS"),(0x1f,"DS"),
                        (0x58,"AX"),(0x59,"CX"),(0x5a,"DX"),(0x5b,"BX"),
                        (0x5c,"SP"),(0x5d,"BP"),(0x5e,"SI"),(0x5f,"DI")
                    ]
                );

                dbgString=format!("POP {}",regHashMap[&opcode]);
                *bytesRead=1;

                if debugFlag==false
                {
                    let popdval=pmachine.pop16(self.ss,self.sp);
                    if regHashMap[&opcode]=="ES" { self.es=popdval; }
                    if regHashMap[&opcode]=="SS" { self.ss=popdval; }
                    if regHashMap[&opcode]=="DS" { self.ds=popdval; }
                    if regHashMap[&opcode]=="AX" { self.ax=popdval; }
                    if regHashMap[&opcode]=="CX" { self.cx=popdval; }
                    if regHashMap[&opcode]=="DX" { self.dx=popdval; }
                    if regHashMap[&opcode]=="BX" { self.bx=popdval; }
                    if regHashMap[&opcode]=="SP" { self.sp=popdval; }
                    if regHashMap[&opcode]=="BP" { self.bp=popdval; }
                    if regHashMap[&opcode]=="SI" { self.si=popdval; }
                    if regHashMap[&opcode]=="DI" { self.di=popdval; }
                    self.sp+=2;
                    self.ip+=1;
                }
            },
            0x06 | 0x0e | 0x16 | 0x1e | 0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x57 =>
            {
                // PUSH 16 bit register
                let regHashMap = HashMap::from(
                    [
                        (0x06,"ES"),(0x0e,"CS"),(0x16,"SS"),(0x1e,"DS"),
                        (0x50,"AX"),(0x51,"CX"),(0x52,"DX"),(0x53,"BX"),
                        (0x54,"SP"),(0x55,"BP"),(0x56,"SI"),(0x57,"DI")
                    ]
                );

                dbgString=format!("PUSH {}",regHashMap[&opcode]);
                *bytesRead=1;

                if debugFlag==false
                {
                    // TODO all the regs
                    let mut val:u16=0;
                    if regHashMap[&opcode]=="AX" { pmachine.push16(self.ax,self.ss,self.sp); }
                    else if regHashMap[&opcode]=="BX" { pmachine.push16(self.bx,self.ss,self.sp); }
                    else if regHashMap[&opcode]=="CX" { pmachine.push16(self.cx,self.ss,self.sp); }
                    else if regHashMap[&opcode]=="DI" { pmachine.push16(self.di,self.ss,self.sp); }
                    else if regHashMap[&opcode]=="DX" { pmachine.push16(self.dx,self.ss,self.sp); }
                    else if regHashMap[&opcode]=="SI" { pmachine.push16(self.si,self.ss,self.sp); }
                    else if regHashMap[&opcode]=="BP" { pmachine.push16(self.bp,self.ss,self.sp); }
                    else { self.abort("unhandled push"); }
                    self.sp-=2;

                    self.ip+=1;
                }
            },
            0x08 =>
            {
                // OR rmb,rb
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("OR ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags, other regs
                    if (moveVec[0]=="AH") && (moveVec[1]=="AL")
                    {
                        let mut lop=(self.ax&0xff00)>>8;
                        let mut rop=self.ax&0xff;
                        lop|=rop;
                        self.ax=(lop<<8)|(self.ax&0xff);

                        self.doSflag(lop as u16,8);
                    }
                    else if (moveVec[0]=="AL") && (moveVec[1]=="AL")
                    {
                        let mut lop=self.ax&0xff;
                        let mut rop=lop;
                        lop|=rop;
                        self.ax=(self.ax&0xff00)|(lop);

                        self.doSflag(lop as u16,8);
                    }
                    else
                    {
                        self.abort(&format!("or rmb,rb {} {}",moveVec[0],moveVec[1]));
                    }

                    self.ip+=2;
                }
            },
            0x09 =>
            {
                // OR rmw,rw
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("OR ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags, other regs
                    if (moveVec[0]=="AX") && (moveVec[1]=="AX")
                    {
                        let mut lop=self.ax;
                        let mut rop=self.ax;
                        lop|=rop;
                        self.ax=lop;

                        self.doSflag(lop as u16,8);
                    }
                    else
                    {
                        self.abort(&format!("or rmw,rw {} {}",moveVec[0],moveVec[1]));
                    }

                    self.ip+=2;
                }



            },        
            0x20 =>
            {
                // AND rmb,rb
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("AND ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags, other regs
                    let mut lop:u8=0;
                    let mut rop:u8=0;
                    if (moveVec[0]=="AH") && (moveVec[1]=="BL")
                    {
                        lop=((self.ax&0xff00)>>8) as u8;
                        rop=(self.bx&0xff) as u8;
                        lop&=rop;
                        self.ax=((lop as u16)<<8)|(self.ax&0xff);
                    }
                    else
                    {
                        self.abort(&format!("and rmb,rb {} {}",moveVec[0],moveVec[1]));
                    }

                    self.doZflag(lop as u16);
                    self.doPflag(lop as u16);

                    self.ip+=2;
                }
            },
            0x25 =>
            {
                // AND AX,iw
                let iw=pmachine.readMemory16(theCS,theIP+1,pvga);
                dbgString=format!("*AND AX,0x{:04x}",iw);
                *bytesRead=3;

                if debugFlag==false
                {
                    // TODO
                    self.abort("unhandled hand");
                    self.ip+=3;
                }
            },
            0x29 =>
            {
                // SUB rmw,rw
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let data=pmachine.readMemory16(theCS,theIP+2,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                dbgString.push_str("SUB ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags
                    let mut src:u16=0;
                    if moveVec[1]=="BX" { src=self.bx; }
                    if moveVec[0]=="DI" { self.di=self.di-src; }

                    self.ip+=2;
                }
            },
            0x2c =>
            {
                // SUB AL,ib
                let ib=pmachine.readMemory(theCS,theIP+1,pvga) as i16;
                dbgString.push_str("SUB AL,");
                dbgString.push_str(&format!("{:02x}",ib));
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags
                    let mut al:i16=(self.ax&0xff) as i16;
                    al-=ib;
                    self.ax=(self.ax&0xff00)|(al&0xff) as u16;
                    self.ip+=2;
                }
            },
            0x31 =>
            {
                // XOR rmw,rw
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("XOR ");
                dbgString.push_str(&moveVec[1]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[0]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO review, other registers
                    let mut op1:u16=0;                    
                    let mut op2:u16=0;                    

                    if moveVec[0]=="AX" { op1=self.ax; }
                    if moveVec[1]=="AX" { op2=self.ax; op2^=op1; self.ax=op2; }
                    if moveVec[0]=="DX" { op1=self.dx; }
                    if moveVec[1]=="DX" { op2=self.dx; op2^=op1; self.dx=op2; }

                    self.setCflag(false);
                    self.setOflag(false);
                    self.doZflag(op2);
                    self.doSflag(op2,16);
                    self.ip+=2;
                }
            },
            0x32 =>
            {
                // XOR rb,rmb
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("XOR ");
                dbgString.push_str(&moveVec[1]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[0]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags, other regs
                    let mut op1:u16=0;                    
                    let mut op2:u16=0;                    
                    if moveVec[0]=="[DI]" 
                    { 
                        op1=pmachine.readMemory(self.ds,self.di,pvga) as u16; 
                        dbgString.push_str(&format!(" [DI]={:02x}",op1));
                    }
                    if moveVec[1]=="AL" { op2=(self.ax&0xff) as u16; }
                    
                    if moveVec[1]=="AL" { op2^=op1; self.ax=(self.ax&0xff00)|op2; }

                    self.setCflag(false);
                    self.setOflag(false);
                    self.doSflag(op2,8);
                    self.doZflag(op2);

                    self.ip+=2;
                }
            },
            0x38 =>
            {
                // CMP rmb,rb
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let mut moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                let mut displType:u8=0;
                if moveVec[0].contains("16") { displType=2; }
                else { displType=1; }

                let mut displacement:i16=0;
                if displType==1 { displacement=pmachine.readMemory(theCS,theIP+2,pvga) as i8 as i16; }
                else { displacement=pmachine.readMemory16(theCS,theIP+2,pvga) as i16; }

                let sdispl:String=format!("{}",displacement);
                moveVec[0]=moveVec[0].replace("Disp",&sdispl);

                dbgString.push_str("CMP ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);

                *bytesRead=2+displType;

                if debugFlag==false
                {
                    // TODO
                    let mut val2compare:i32=0;
                    let mut data:i32=0;
                    let mut diPlusDisp=self.di as i32;
                    diPlusDisp=diPlusDisp+(displacement as i32);
                    
                    if moveVec[0].contains("[DI+") 
                    { 
                        val2compare=pmachine.readMemory(self.ds,diPlusDisp as u16,pvga) as i32; 
                    }
                    if moveVec[1]=="CH" { data=((self.cx&0xff00)>>8) as i32; }
                    //*retErr=format!("DiPlusDisp is {:04x} data is {} val2compare is {}",diPlusDisp,data,val2compare);

                    let cmpval:i32=(val2compare-data);

                    if val2compare<data { self.setSflag(true); }
                    else { self.setSflag(false); }

                    if val2compare<data { self.setCflag(true); }
                    else { self.setCflag(false); }

                    self.doZflag(cmpval as u16);
                    self.doPflag(cmpval as u16);

                    self.ip+=2+(displType as u16);
                }
            },
            0x3b =>
            {
                // CMP rw,rmw
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let mut data=pmachine.readMemory16(theCS,theIP+2,pvga) as i32;
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("CMP ");
                dbgString.push_str(&moveVec[1]);
                dbgString.push_str(",");
                if moveVec[0]=="Direct Addr" { dbgString.push_str(&format!("[{:04x}]",data)); }
                else { dbgString.push_str(&format!("0x{:04x}",data)); }
                *bytesRead=4;

                if debugFlag==false
                {
                    // TODO overflow flag
                    let mut val2compare:i32=0;

                    if moveVec[0]=="Direct Addr" { data=pmachine.readMemory16(self.ds,data as u16,pvga) as i32; }
                    if moveVec[1]=="DX" { val2compare=self.dx as i32; }

                    let cmpval:i32=(val2compare-data);

                    if val2compare<data { self.setSflag(true); }
                    else { self.setSflag(false); }

                    if val2compare<data { self.setCflag(true); }
                    else { self.setCflag(false); }

                    self.doZflag(cmpval as u16);
                    self.doPflag(cmpval as u16);

                    self.ip+=4;
                }
            },
            0x3c =>
            {
                // CMP AL,ib
                let ib=pmachine.readMemory(theCS,theIP+1,pvga);

                dbgString.push_str("CMP AL");
                dbgString.push_str(",");
                dbgString.push_str(&format!("0x{:02x}",ib));
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO overflow flag
                    let mut val2compare:i16=0;
                    let i16ib=ib as i16;
                    val2compare=(self.ax&0xff) as i16;
                    let cmpval:i16=(val2compare-i16ib) as i16;

                    if val2compare<i16ib { self.setSflag(true); }
                    else { self.setSflag(false); }

                    if val2compare<i16ib { self.setCflag(true); }
                    else { self.setCflag(false); }

                    self.doZflag(cmpval as u16);
                    self.doPflag(cmpval as u16);

                    self.ip+=2;
                }
            },
            0x3d =>
            {
                // the three-dimensional opcode
                // CMP     AX,iw
                let data=pmachine.readMemory16(theCS,theIP+1,pvga) as i32;

                dbgString.push_str("CMP AX,");
                dbgString.push_str(&format!("0x{:04x}",data));
                *bytesRead=3;

                if debugFlag==false
                {
                    // TODO overflow flag. DS:DI is correct?
                    let mut val2compare:i32=0;
                    val2compare=self.ax as i32;
                    let cmpval:i32=(val2compare-data);

                    if val2compare<data { self.setSflag(true); }
                    else { self.setSflag(false); }

                    if val2compare<data { self.setCflag(true); }
                    else { self.setCflag(false); }

                    self.doZflag(cmpval as u16);
                    self.doPflag(cmpval as u16);

                    self.ip+=3;
                }
            },
            0x40 | 0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x46 | 0x47 =>
            {
                // INC reg16
                let regHashMap = HashMap::from(
                    [
                        (0x40,"AX"),(0x41,"CX"),(0x42,"DX"),(0x43,"BX"),(0x44,"SP"),(0x45,"BP"),(0x46,"SI"),(0x47,"DI")
                    ]
                );

                dbgString=format!("INC {}",regHashMap[&opcode]);
                *bytesRead=1;

                if debugFlag==false
                {
                    // TODO flags
                    if opcode==0x40 { self.ax+=1; self.doZflag(self.ax); self.doPflag(self.ax); self.doSflag(self.ax,16); }
                    else if opcode==0x41 { self.cx+=1; self.doZflag(self.cx); self.doPflag(self.cx); self.doSflag(self.cx,16); }
                    else if opcode==0x42 { self.dx+=1; self.doZflag(self.dx); self.doPflag(self.dx); self.doSflag(self.dx,16); }
                    else if opcode==0x43 { self.bx+=1; self.doZflag(self.bx); self.doPflag(self.bx); self.doSflag(self.bx,16); }
                    else if opcode==0x44 { self.sp+=1; self.doZflag(self.sp); self.doPflag(self.sp); self.doSflag(self.sp,16); }
                    else if opcode==0x45 { self.bp+=1; self.doZflag(self.bp); self.doPflag(self.bp); self.doSflag(self.bp,16); }
                    else if opcode==0x46 { self.si+=1; self.doZflag(self.si); self.doPflag(self.si); self.doSflag(self.si,16); }
                    else if opcode==0x47 { self.di+=1; self.doZflag(self.di); self.doPflag(self.di); self.doSflag(self.di,16); }
                    self.ip+=1;
                }
            },
            0x48 | 0x49 | 0x4a | 0x4b | 0x4c | 0x4d | 0x4e | 0x4f =>
            {
                // DEC reg16
                let regHashMap = HashMap::from(
                    [
                        (0x48,"AX"),(0x49,"CX"),(0x4a,"DX"),(0x4b,"BX"),(0x4c,"SP"),(0x4d,"BP"),(0x4e,"SI"),(0x4f,"DI")
                    ]
                );

                dbgString=format!("DEC {}",regHashMap[&opcode]);
                *bytesRead=1;

                if opcode==0x48 { self.ax-=1; self.doZflag(self.ax); self.doPflag(self.ax); self.doSflag(self.ax,16); }
                else if opcode==0x49 { self.cx-=1; self.doZflag(self.cx); self.doPflag(self.cx); self.doSflag(self.cx,16); }
                else if opcode==0x4a { self.dx-=1; self.doZflag(self.dx); self.doPflag(self.dx); self.doSflag(self.dx,16); }
                else if opcode==0x4b { self.bx-=1; self.doZflag(self.bx); self.doPflag(self.bx); self.doSflag(self.bx,16); }
                else if opcode==0x4c { self.sp-=1; self.doZflag(self.sp); self.doPflag(self.sp); self.doSflag(self.sp,16); }
                else if opcode==0x4d { self.bp-=1; self.doZflag(self.bp); self.doPflag(self.bp); self.doSflag(self.bp,16); }
                else if opcode==0x4e { self.si-=1; self.doZflag(self.si); self.doPflag(self.si); self.doSflag(self.si,16); }
                else if opcode==0x4f { self.di-=1; self.doZflag(self.di); self.doPflag(self.di); self.doSflag(self.di,16); }
                self.ip+=1;
            },
            0x6f =>
            {
                // OUTSW (186)
                dbgString.push_str("OUTSW");
                *bytesRead=1;

                if debugFlag==false
                {
                    // TODO
                    self.abort("outsw");
                    self.ip+=1;
                }
            },
            0x72 =>
            {
                // JB short
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("JB 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    if self.getCflag()
                    {
                        let delta:i8=jumpAmt as i8;
                        self.ip=self.ip.wrapping_add((delta+2) as u16);
                    }
                    else
                    {
                        self.ip+=2;
                    }
                }
            },
            0x73 =>
            {
                // JAE short
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("JAE 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    if !self.getCflag()
                    {
                        let delta:i8=jumpAmt as i8;
                        self.ip=self.ip.wrapping_add((delta+2) as u16);
                    }
                    else
                    {
                        self.ip+=2;
                    }
                }
            },
            0x74 =>
            {
                // JE short
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("JE 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    if self.getZflag()
                    {
                        let delta:i8=jumpAmt as i8;
                        self.ip=self.ip.wrapping_add((delta+2) as u16);
                    }
                    else
                    {
                        self.ip+=2;
                    }
                }
            },
            0x75 =>
            {
                // JNE short
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("JNE 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    if self.getZflag()==false
                    {
                        let delta:i8=jumpAmt as i8;
                        self.ip=self.ip.wrapping_add((delta+2) as u16);
                    }
                    else
                    {
                        self.ip+=2;
                    }
                }
            },
            0x76 =>
            {
                // JBE short
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("JBE 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    if self.getZflag() || self.getCflag()
                    {
                        let delta:i8=jumpAmt as i8;
                        self.ip=self.ip.wrapping_add((delta+2) as u16);
                    }
                    else
                    {
                        self.ip+=2;
                    }
                }
            },
            0x77 =>
            {
                // JA short TODO
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("*JA 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    let delta:i8=jumpAmt as i8;
                    self.ip=self.ip.wrapping_add((delta+2) as u16);
                    self.abort("JA short todo");
                }
            },
            0x78 =>
            {
                // JS short 
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("JS 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    if self.getSflag()
                    {
                        let delta:i8=jumpAmt as i8;
                        self.ip=self.ip.wrapping_add((delta+2) as u16);
                    }
                    else
                    {
                        self.ip+=2;
                    }
                }
            },
            0x7F =>
            {
                // JG short TODO
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("*JG 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    let delta:i8=jumpAmt as i8;
                    self.ip=self.ip.wrapping_add((delta+2) as u16);
                    self.abort("jg short todo");
                }
            },
            0x80 =>
            {
                // oh no, another multi-instruction opcode 
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if (reg==4)
                {
                    // AND rmb,ib
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let ib=pmachine.readMemory(theCS,theIP+2,pvga);
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                    dbgString.push_str("AND ");
                    dbgString.push_str(&moveVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:02x}",ib));
                    *bytesRead=3;

                    if debugFlag==false
                    {
                        // TODO flags, other regs
                        let mut lop=0;
                        if moveVec[0]=="AH"
                        {
                            lop=(self.ax&0xff00)>>8;
                            lop&=ib as u16;
                            self.ax=(lop<<8)|(self.ax&0xff);
                        }
                        else
                        {
                            self.abort(&format!("unhandled NAD rmb,ib {}",moveVec[0]));
                        }

                        self.doZflag(lop as u16);
                        self.doPflag(lop as u16);
    
                        self.ip+=3;
                    }
                }
                else if (reg==6)
                {
                    // XOR rmb,ib
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
    
                    let mut offset=0;
                    let mut ib=0;
                    let mut displacement:i32=0;
                    if moveVec[0].contains("[SI+Disp] with 8bit")
                    {
                        displacement=pmachine.readMemory(theCS,theIP+2,pvga) as i8 as i32;
                        ib=pmachine.readMemory(theCS,theIP+3,pvga);
                        *bytesRead=4;
                    }
                    else
                    {
                        offset=pmachine.readMemory16(theCS,theIP+2,pvga);
                        ib=pmachine.readMemory(theCS,theIP+4,pvga);
                        *bytesRead=5;
                    }
    
                    dbgString.push_str("XOR byte ");
                    dbgString.push_str(&format!("{} [{:04x}]",moveVec[0],offset));
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:02x}",ib));
    
                    if debugFlag==false
                    {
                        // TODO flags, other regs
                        let mut op1:u8=0;                    
                        if moveVec[0].contains("[SI+Disp] with 8bit")
                        {
                            let mut i32si=self.si as i32;
                            i32si+=displacement;
                            op1=pmachine.readMemory(self.ds,i32si as u16,pvga); 
                            op1^=ib;
                            pmachine.writeMemory(self.ds,i32si as u16,op1,pvga);
                            self.ip+=4;
                        }
                        else
                        {    
                            op1=pmachine.readMemory(self.ds,offset,pvga); 
                            op1^=ib;
                            pmachine.writeMemory(self.ds,offset,op1,pvga);
                            self.ip+=5;
                        }
    
                        self.setCflag(false);
                        self.setOflag(false);
                        self.doSflag(op1 as u16,8);
                        self.doZflag(op1 as u16);
                    }
                }
                else if (reg==7)
                {
                    // CMP rmb,ib
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                    let mut displ:i16=0;
                    let mut ib:u8=0;

                    if moveVec[0].contains("[SI+Disp] with 8bit") 
                    { 
                        displ=pmachine.readMemory(theCS,theIP+2,pvga) as i8 as i16; 
                        ib=pmachine.readMemory(theCS,theIP+3,pvga);
                        *bytesRead=4;
                    }
                    else if moveVec[0].contains("[DI+Disp] with 16bit") 
                    { 
                        displ=pmachine.readMemory16(theCS,theIP+2,pvga) as i8 as i16; 
                        ib=pmachine.readMemory(theCS,theIP+4,pvga);
                        *bytesRead=5;
                    }
                    else
                    {
                        ib=pmachine.readMemory(theCS,theIP+2,pvga);
                        *bytesRead=3;
                    }

                    dbgString.push_str("CMP ");
                    dbgString.push_str(&moveVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:02x}",ib));

                    if debugFlag==false
                    {
                        // TODO overflow flag
                        let mut val2compare:i16=0;
                        let i16ib=ib as i16;
                        if moveVec[0]=="BH" { val2compare=((self.bx&0xff00)>>8) as i16; }
                        if moveVec[0]=="[DI]" { val2compare=pmachine.readMemory(self.ds,self.di,pvga) as i16; }
                        if moveVec[0].contains("[SI+Disp] with 8bit") 
                        { 
                            let i16si=self.si as i16;
                            val2compare=pmachine.readMemory(self.ds,(i16si+displ) as u16,pvga) as i16; 
                        }
                        if moveVec[0].contains("[DI+Disp] with 16bit") 
                        { 
                            let i16di=self.di as i16;
                            val2compare=pmachine.readMemory(self.ds,(i16di+displ) as u16,pvga) as i16; 
                        }
                        let cmpval:i16=(val2compare-i16ib) as i16;

                        if val2compare<i16ib { self.setSflag(true); }
                        else { self.setSflag(false); }

                        if val2compare<i16ib { self.setCflag(true); }
                        else { self.setCflag(false); }

                        self.doZflag(cmpval as u16);
                        self.doPflag(cmpval as u16);

                        self.ip+=3;
                        if moveVec[0].contains("[SI+Disp] with 8bit") { self.ip+=1; }
                        if moveVec[0].contains("[DI+Disp] with 16bit") { self.ip+=2; }
                    }
                }

            },
            0x81 =>
            {
                // the most complex instruction in the world
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if (reg==0)
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
                        let mut src:i32=data as i32;

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
                else if (reg==7)
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
                        let cmpval:i32=(val2compare-data);
    
                        if val2compare<data { self.setSflag(true); }
                        else { self.setSflag(false); }
    
                        if val2compare<data { self.setCflag(true); }
                        else { self.setCflag(false); }
    
                        self.doZflag(cmpval as u16);
                        self.doPflag(cmpval as u16);
    
                        self.ip+=4;
                    }
                }
                else if reg==5
                {
                    // SUB rmw,iw
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let data=pmachine.readMemory16(theCS,theIP+2,pvga);
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    dbgString.push_str("SUB ");
                    dbgString.push_str(&moveVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("{:04x}",data));
                    *bytesRead=4;
    
                    if debugFlag==false
                    {
                        // TODO flags
                        let mut src:u16=0;
                        if moveVec[1]=="BX" { src=self.bx; }
                        if moveVec[0]=="DI" { self.di=self.di-src; }
                        self.abort(&format!("bailing out {} {}",moveVec[0],data));
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

                if reg==0
                {
                    // ADD     rmw,ib
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let ib=pmachine.readMemory(theCS,theIP+2,pvga);
                    let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                    dbgString.push_str("ADD ");
                    dbgString.push_str(&moveVec[0]);
                    dbgString.push_str(",");
                    dbgString.push_str(&format!("0x{:04x}",ib));
                    *bytesRead=3;

                    if debugFlag==false
                    {
                        // TODO other regs + flags
                        let mut dest:i32=0;
                        let mut src:i32=ib as i32;

                        if moveVec[0]=="AX" { dest=self.ax as i32; self.ax+=ib as u16; }
                        else if moveVec[0]=="DI" { dest=self.di as i32; self.di+=ib as u16; }
                        else { self.abort(&format!("add 0x83 {}",moveVec[0])); }

                        if (dest+src)>0xffff { self.setCflag(true); }
                        else { self.setCflag(false); }
                        self.doZflag((dest+src) as u16);
                        self.doPflag((dest+src) as u16);

                        self.ip+=3;
                    }
                }
                else if reg==3
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
                        let mut rop:u16=0;
                        if (moveVec[0]=="BP")
                        {
                            lop=self.bp;
                            rop=ib as u16;
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
                        let mut src:u16=ib as u16;
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
                    //self.abort(&format!("unhandled 0x83 reg={}",reg));
                }
            },
            0x84 =>
            {
                // TEST rmb,rmb
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("TEST ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO recheck
                    let mut val2compare:i16=0;
                    let mut i16ib:i16=0;
                    if (moveVec[0]=="BH") && (moveVec[1]=="BH")
                    {
                        val2compare=(self.bx>>8) as i16;
                        i16ib=val2compare;
                    }
                    else if (moveVec[0]=="AH") && (moveVec[1]=="AL")
                    {
                        val2compare=(self.ax>>8) as i16;
                        i16ib=(self.ax&0xff) as i16;
                    }
                    else if (moveVec[0]=="AH") && (moveVec[1]=="AH")
                    {
                        val2compare=(self.ax>>8) as i16;
                        i16ib=(self.ax>>8) as i16;
                    }
                    else
                    {
                        self.abort(&format!("unimplemented regs TEST rmb,rmb {} {}",moveVec[1],moveVec[0]));
                    }

                    let cmpval:i16=(val2compare&i16ib) as i16;
                    //if val2compare<i16ib { self.setSflag(true); }
                    //else { self.setSflag(false); }
                    self.doZflag(cmpval as u16);
                    self.doPflag(cmpval as u16);

                    self.ip+=2;
                }
            },
            0x88 =>
            {
                // MOV rmb,rb
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("MOV ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);
                *bytesRead=2;
                if (moveVec[0].contains("with 8bit")) { *bytesRead+=1; }

                if debugFlag==false
                {
                    // TODO all regs
                    if (moveVec[0]=="AL") && (moveVec[1]=="BH")
                    {
                        let bh:u16=(self.bx>>8)&0xff;
                        self.ax=(self.ax&0xff00)|bh;
                    }
                    else if (moveVec[0]=="AL") && (moveVec[1]=="AH")
                    {
                        let ah:u16=(self.ax>>8)&0xff;
                        self.ax=(self.ax&0xff00)|ah;
                    }
                    else if (moveVec[0]=="AL") && (moveVec[1]=="BL")
                    {
                        let bl:u16=self.bx&0xff;
                        self.ax=(self.ax&0xff00)|bl;
                    }
                    else if (moveVec[0]=="AH") && (moveVec[1]=="BH")
                    {
                        let bh:u16=self.bx>>8;
                        self.ax=(self.ax&0xff)|(bh<<8);
                    }
                    else if (moveVec[0]=="AH") && (moveVec[1]=="DL")
                    {
                        let dl:u16=self.dx&0xff;
                        self.ax=(self.ax&0xff)|(dl<<8);
                    }
                    else if (moveVec[0]=="AH") && (moveVec[1]=="AL")
                    {
                        let al:u16=self.ax&0xff;
                        self.ax=(self.ax&0xff)|(al<<8);
                    }
                    else if (moveVec[0].contains("[SI+Disp] with 8bit")) && (moveVec[1]=="AL")
                    {
                        let mut displType:u8=1;
                        let mut displacement:i32=pmachine.readMemory(theCS,theIP+2,pvga) as i8 as i32;
                        let mut si=self.si as i32;
                        si+=displacement;
                        pmachine.writeMemory(self.ds,si as u16,(self.ax&0xff) as u8,pvga);
                        self.ip+=1;
                    }
                    else
                    {
                        self.abort(&format!("0x88 reg combination {} {}",moveVec[0],moveVec[1]));
                    }
                    self.ip+=2;
                }
            },
            0x89 =>
            {
                // MOV rmw,rw
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str("MOV ");
                if moveVec[0]=="Direct Addr" 
                { 
                    let mut data=pmachine.readMemory16(theCS,theIP+2,pvga) as i32;
                    dbgString.push_str(&format!("[{:04x}]",data)); 
                }
                else { dbgString.push_str(&moveVec[0]); }
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[1]);

                if moveVec[0]=="Direct Addr" { *bytesRead=4; }
                else if (moveVec[0].contains("[SI+")) { *bytesRead=3; }
                else if (moveVec[0].contains("[DI+Disp] with 16bit")) { *bytesRead=4; }
                else if (moveVec[0].contains("[SI+Disp] with 8bit")) { *bytesRead=3; }
                else { *bytesRead=2; }

                if debugFlag==false
                {
                    if (moveVec[1]=="DX") && (moveVec[0]=="Direct Addr")
                    {
                        let mut offset=pmachine.readMemory16(theCS,theIP+2,pvga) as i32;
                        pmachine.writeMemory16(self.ds,offset as u16,self.dx,pvga);
                    }
                    else if (moveVec[0]=="AX") && (moveVec[1]=="DI")
                    {
                        self.ax=self.di;
                    }
                    else if (moveVec[0]=="[SI]") && (moveVec[1]=="AX")
                    {
                        let mut offset=pmachine.readMemory16(self.ds,self.si,pvga);
                        pmachine.writeMemory16(self.ds,offset as u16,self.ax,pvga);
                    }
                    else if (moveVec[0].contains("[SI+")) && (moveVec[1]=="DI")
                    {
                        let displacement:i32=pmachine.readMemory(theCS,theIP+2,pvga) as i8 as i32;
                        let mut si32:i32=self.si as i32;
                        si32+=displacement;
                        //*retErr=format!("si+displ is {:04x} displacement is {}",si32,displacement);
                        pmachine.writeMemory16(self.ds,si32 as u16,self.di,pvga);
                        self.ip+=1;
                    }
                    else if (moveVec[0].contains("[SI+Disp] with 8bit")) && (moveVec[1]=="AX")
                    {
                        let displacement:i32=pmachine.readMemory(theCS,theIP+2,pvga) as i8 as i32;
                        let mut si32:i32=self.si as i32;
                        si32+=displacement;
                        *retErr=format!("si+displ is {:04x} displacement is {}",si32,displacement);
                        pmachine.writeMemory16(self.ds,si32 as u16,self.ax,pvga);
                        self.ip+=1;
                    }
                    else if (moveVec[0].contains("[DI+Disp] with 16bit")) && (moveVec[1]=="AX")
                    {
                        let displacement:i32=pmachine.readMemory16(theCS,theIP+2,pvga) as i16 as i32;
                        let mut di32:i32=self.di as i32;
                        di32+=displacement;
                        *retErr=format!("di+displ is {:04x} displacement is {}",di32,displacement);
                        pmachine.writeMemory16(self.ds,di32 as u16,self.ax,pvga);
                        self.ip+=2;
                    }
                    else
                    {
                        self.abort(&format!("unhandled 0x89 {} {}",moveVec[0],moveVec[1]));
                    }

                    if moveVec[0]=="Direct Addr" { self.ip+=4; }
                    else { self.ip+=2; }
                }
            },
            0x8b =>
            {
                // MOV rw,rmw
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                dbgString.push_str(&format!("MOV {},{}",moveVec[1],moveVec[0])); 
                *bytesRead+=2;

                if debugFlag==false
                {
                    // TODO
                    if (moveVec[1]=="AX") && (moveVec[0]=="[DI]")
                    {
                        let data=pmachine.readMemory16(self.ds,self.di,pvga);
                        self.ax=data;
                    }
                    self.ip+=2;
                }
            },
            0x8e =>
            {
                // MOV reg/mem to segreg 
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,1,1);

                dbgString.push_str("MOV ");
                dbgString.push_str(&moveVec[1]);
                dbgString.push_str(",");
                dbgString.push_str(&moveVec[0]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO fully
                    let mut val:u16=0;
                    if moveVec[0]=="AX" { val=self.ax; }
                    if moveVec[1]=="DS" { self.ds=val; }
                    if moveVec[1]=="ES" { self.es=val; }
                    self.ip+=2;
                }
            },
            0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97  =>
            {
                // XCHG reg16,reg16
                let regHashMap = HashMap::from(
                    [
                        (0x91,"AX,CX"),(0x92,"AX,DX"),(0x93,"AX,BX"),(0x94,"AX,SP"),(0x95,"AX,BP"),(0x96,"AX,SI"),(0x97,"AX,DI")
                    ]
                );

                dbgString=format!("XCHG {}",regHashMap[&opcode]);
                *bytesRead=1;

                if debugFlag==false
                {
                    self.xchgRegs(regHashMap[&opcode].to_string());
                    self.ip+=1;
                }
            },
            0x9c =>
            {
                // PUSHF
                dbgString="PUSHF".to_string();
                *bytesRead=1;

                if debugFlag==false
                {
                    pmachine.push16(self.flags,self.ss,self.sp);
                    self.sp-=2;
                    self.ip+=1;
                }
            },
            0x9d =>
            {
                // POPF
                dbgString="POPF".to_string();
                *bytesRead=1;

                if debugFlag==false
                {
                    self.flags=pmachine.pop16(self.ss,self.sp);
                    self.sp+=2;
                    self.ip+=1;
                }
            },
            0xa0 =>
            {
                // MOV AL,[offset]
                let offset:u16=pmachine.readMemory16(theCS,theIP+1,pvga);

                dbgString=format!("MOV AL,[{:04x}]",offset);
                *bytesRead+=3;

                if debugFlag==false
                {
                    let data:u8=pmachine.readMemory(self.ds,offset,pvga);
                    self.ax=(self.ax&0xff00)|(data as u16);
                    self.ip+=3;
                }
            },
            0xa2 =>
            {
                // MOV rmb,AL
                let offset:u16=pmachine.readMemory16(theCS,theIP+1,pvga);

                if segOverride!="".to_string() { dbgString=format!("{} MOV ",segOverride); }
                else { dbgString=format!("MOV "); }

                dbgString.push_str(&format!("[{:04x}]",offset)); 
                dbgString.push_str(",AL");
                *bytesRead+=3;

                if debugFlag==false
                {
                    // TODO other segs override
                    let mut writeSeg:u16=self.ds;
                    if segOverride=="CS" { writeSeg=self.cs; }

                    pmachine.writeMemory(writeSeg,offset,(self.ax&0xff) as u8,pvga);

                    self.ip+=3;
                    if segOverride!="".to_string() { self.ip+=1; }
                }
            },
            0xa3 =>
            {
                // MOV rmw,AX
                let offset:u16=pmachine.readMemory16(theCS,theIP+1,pvga);

                dbgString.push_str(&format!("[{:04x}],AX",offset)); 
                *bytesRead+=3;

                if debugFlag==false
                {
                    pmachine.writeMemory16(self.ds,offset,self.ax,pvga);
                    self.ip+=3;
                }
            },
            0xa5 =>
            {
                // MOVSW
                if segOverride!="".to_string() { dbgString=format!("{} MOVSW",segOverride); }
                else { dbgString=format!("MOVSW ES:DI,DS:SI"); }
                *bytesRead+=1;

                if debugFlag==false
                {
                    // TODO other segs override
                    let mut readSeg:u16=self.ds;
                    if segOverride=="CS" { readSeg=self.cs; }
                    else { self.abort("movsw unhandled seg override"); }

                    let dataw=pmachine.readMemory16(readSeg,self.si,pvga);
                    pmachine.writeMemory16(self.es,self.di,dataw,pvga);

                    if self.getDflag() { self.si-=2; self.di-=2; }
                    else { self.si+=2; self.di+=2; }

                    self.ip+=1;
                    if segOverride!="".to_string() { self.ip+=1; }
                }
            },
            0xa8 =>
            {
                // TEST AL,ib
                let intNum=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("TEST AL,0x{:02x}",intNum);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO all flags
                    let mut val2compare:i16=(self.ax&0xff) as i16;
                    let i16ib=intNum as i16;

                    let cmpval:i16=(val2compare&i16ib) as i16;
                    //if val2compare<i16ib { self.setSflag(true); }
                    //else { self.setSflag(false); }
                    if cmpval==0 { self.setZflag(true); }
                    else { self.setZflag(false); }
                    if (cmpval%2)==0 { self.setPflag(true); }
                    else { self.setPflag(false); }

                    self.ip+=2;
                }
            },
            0xaa =>
            {
                // STOSB
                dbgString=format!("STOSB");
                *bytesRead=1;

                if debugFlag==false
                {
                    pmachine.writeMemory(self.es,self.di,(self.ax&0xff) as u8,pvga);
                    if self.getDflag() { self.di-=1; }
                    else { self.di+=1; }
                    self.ip+=1;
                }
            },
            0xab =>
            {
                // STOSW
                if repOverride=="REPE" { dbgString.push_str("REPE "); }
                dbgString.push_str(&format!("STOSW"));
                *bytesRead+=1;

                if debugFlag==false
                {
                    if repOverride=="REPE"
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
                        else { self.di+=2; }
                        self.ip+=1;
                    }
                }
            },
            0xad =>
            {
                // LODSW
                if segOverride!="".to_string() { dbgString=format!("{} LODSW",segOverride); }
                else { dbgString=format!("LODSW"); }
                *bytesRead+=1;

                if debugFlag==false
                {
                    // TODO other segs override
                    let mut readSeg:u16=self.ds;
                    if segOverride=="CS" { readSeg=self.cs; }

                    let dataw=pmachine.readMemory16(readSeg,self.si,pvga);
                    self.ax=dataw;

                    if self.getDflag() { self.si-=2; }
                    else { self.si+=2; }

                    self.ip+=1;
                    if segOverride!="".to_string() { self.ip+=1; }
                }
            },
            0xb0 | 0xb1 | 0xb2 | 0xb3 | 0xb4 | 0xb5 | 0xb6 | 0xb7 =>
            {
                // MOV 8bit reg,ib
                let intNum:u16=pmachine.readMemory(theCS,theIP+1,pvga) as u16;
                let regHashMap = HashMap::from(
                    [
                        (0xb0,"AL"),(0xb1,"CL"),(0xb2,"DL"),(0xb3,"BL"),(0xb4,"AH"),(0xb5,"CH"),(0xb6,"DH"),(0xb7,"BH")
                    ]
                );
                dbgString=format!("MOV {},0x{:02x}",regHashMap[&opcode],intNum);
                *bytesRead=2;

                if debugFlag==false
                {
                    if regHashMap[&opcode]=="AL" { self.ax=(self.ax&0xff00)|(intNum&0xff); }
                    if regHashMap[&opcode]=="CL" { self.cx=(self.cx&0xff00)|(intNum&0xff); }
                    if regHashMap[&opcode]=="DL" { self.dx=(self.dx&0xff00)|(intNum&0xff); }
                    if regHashMap[&opcode]=="BL" { self.bx=(self.bx&0xff00)|(intNum&0xff); }
                    if regHashMap[&opcode]=="AH" { self.ax=(self.ax&0xff)|(intNum<<8); }
                    if regHashMap[&opcode]=="CH" { self.cx=(self.cx&0xff)|(intNum<<8); }
                    if regHashMap[&opcode]=="DH" { self.dx=(self.dx&0xff)|(intNum<<8); }
                    if regHashMap[&opcode]=="BH" { self.bx=(self.bx&0xff)|(intNum<<8); }
                    self.ip+=2;
                }
            },
            0xbc | 0xb8 | 0xb9 | 0xbe | 0xbf | 0xbb | 0xbd =>
            {
                // MOV 16bit reg, immediate data
                let immediateData=pmachine.readMemory16(theCS,theIP+1,pvga);
                let regHashMap = HashMap::from(
                    [
                        (0xbc,"SP"),(0xb8,"AX"),(0xb9,"CX"),(0xbe,"SI"),(0xbf,"DI"),(0xbb,"BX"),(0xbd,"BP")
                    ]
                );

                dbgString=format!("MOV {},0x{:04x}",regHashMap[&opcode],immediateData);
                *bytesRead=3;

                if debugFlag==false
                {
                    if opcode==0xbc { self.sp=immediateData; }
                    else if opcode==0xb8 { self.ax=immediateData; }
                    else if opcode==0xb9 { self.cx=immediateData; }
                    else if opcode==0xbe { self.si=immediateData; }
                    else if opcode==0xbf { self.di=immediateData; }
                    else if opcode==0xbb { self.bx=immediateData; }
                    else if opcode==0xbd { self.bp=immediateData; }

                    self.ip+=3;
                }
            },
            0xc3 =>
            {
                // RET (near)
                dbgString=format!("RET");
                *bytesRead=1;

                if debugFlag==false
                {
                    let newip=pmachine.pop16(self.ss,self.sp);
                    self.sp+=2;
                    self.ip=newip;
                }
            },
            0xc6 =>
            {
                // MOV rmb,ib
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let moveVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                let ib:u8=pmachine.readMemory(theCS,theIP+3,pvga);

                dbgString.push_str("MOV ");
                dbgString.push_str(&moveVec[0]);
                dbgString.push_str(",");
                dbgString.push_str(&format!("0x{:02x}",ib));
                *bytesRead=3;
                if (moveVec[0].contains("with 8bit")) { *bytesRead+=1; }

                if debugFlag==false
                {
                    // TODO all regs
                    if moveVec[0].contains("[SI+Disp] with 8bit")
                    {
                        let mut displType:u8=1;
                        let mut displacement:i16=pmachine.readMemory(theCS,theIP+2,pvga) as i8 as i16;
                        let mut si=self.si as i16;
                        si+=displacement;

                        //*retErr=format!("si+displ is {:04x} displacement is {}",si,displacement);

                        pmachine.writeMemory(self.ds,si as u16,ib,pvga);
                        self.ip+=1;
                    }
                    self.ip+=3;
                }
            },
            0xcd =>
            {
                // INT
                let intNum=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("INT 0x{:02x}",intNum);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO flags
                    pmachine.handleINT(self,intNum,pvga);
                    self.ip+=2;
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
                else if (reg==5)
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
            0xd1 =>
            {
                // SHL rmw,1
                let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                dbgString=format!("SHL {},1",regVec[0]);
                *bytesRead=2;

                if debugFlag==false
                {
                    // TODO Aux flag
                    let mut reg:u16=0;
                    if regVec[0]=="CX" { reg=self.cx; self.cx<<=1; }
                    else if regVec[0]=="AX" { reg=self.ax; self.ax<<=1; }
                    else
                    {
                        self.abort(&format!("unhandled reg {} in 0xd1",regVec[0]));
                    }

                    let lastBit:bool=(reg&0x8000)!=0;
                    self.setCflag(lastBit);
                    self.doZflag(reg<<1);
                    self.doPflag(reg<<1);
                    self.ip+=2;
                }
            },
            0xd2 =>
            {
                // multi-instruction opcode
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if reg==4
                {
                    // SHL rmb,CL
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    dbgString=format!("SHL {},CL",regVec[0]);
                    *bytesRead=2;
    
                    if debugFlag==false
                    {
                        // TODO other regs, a,s flags
                        let mut val2shift:u16=0;
                        if regVec[0]=="AL" 
                        { 
                            val2shift=self.ax&0xff; 
                            let lastBit:bool=(val2shift&0x80)!=0;
                            self.setCflag(lastBit);

                            val2shift<<=(self.cx&0xff);
                            self.ax=(self.ax&0xff00)|(val2shift&0xff);
    
                            self.doZflag(val2shift as u16);
                            self.doPflag(val2shift as u16);
                        }

                        self.ip+=2;
                    }
                }
            },
            0xd7 =>
            {
                // XLAT
                if segOverride!="".to_string() { dbgString=format!("{} XLAT",segOverride); }
                else { dbgString=format!("XLAT"); }

                *bytesRead+=1;

                if debugFlag==false
                {
                    // TODO other segs
                    let mut segment=self.ds;
                    if segOverride=="CS" { segment=self.cs; }
                    let mut al=self.ax&0xff;
                    let tableval:u16=pmachine.readMemory(segment,self.bx+al,pvga) as u16;
                    self.ax=(self.ax&0xff00)|tableval;
                    
                    self.ip+=1;
                    if segOverride!="".to_string() { self.ip+=1; }
                }
            },
            0xe2 =>
            {
                // LOOP short
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("LOOP 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    self.cx-=1;
                    if self.cx!=0
                    {
                        let delta:i8=jumpAmt as i8;
                        self.ip=self.ip.wrapping_add((delta+2) as u16);
                    }
                    else
                    {
                        self.ip+=2;
                    }
                }
            },
            0xe8 =>
            {
                // CALL addr relative
                let addrRel=pmachine.readMemory16(theCS,theIP+1,pvga);
                dbgString=format!("CALL 0x{:04x}",addrRel);
                *bytesRead=3;

                if debugFlag==false
                {
                    pmachine.push16(self.ip+3,self.ss,self.sp);
                    self.sp-=2;
                    self.ip=self.ip.wrapping_add((addrRel+3) as u16);
                }
            },
            0xeb =>
            {
                // short jump
                let jumpAmt=pmachine.readMemory(theCS,theIP+1,pvga);
                dbgString=format!("JMP short 0x{:02x}",jumpAmt);
                *bytesRead=2;

                if debugFlag==false
                {
                    let delta:i8=jumpAmt as i8;
                    self.ip=self.ip.wrapping_add((delta+2) as u16);
                }
            },
            0xf6 =>
            {
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if (reg==0)
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
    
                        let cmpval:i16=(val2compare&i16ib) as i16;
                        self.doZflag(cmpval as u16);
                        self.doPflag(cmpval as u16);
    
                        self.ip+=3;
                    }
                }
            },
            0xf7 =>
            {
                // another of those nice multi-instruction opcodes
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if (reg==3)
                {
                    // NEG rmw
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    dbgString=format!("NEG {}",regVec[0]);
                    *bytesRead=2;
    
                    if debugFlag==false
                    {
                        // TODO
                        let mut inum:i32=0;
                        if regVec[0]=="BX"
                        {
                            inum=self.bx as i16 as i32;
                            inum=0-inum;
                            self.bx=inum as u16;

                            self.doZflag(self.bx);
                            self.doSflag(self.bx,16);
                            self.doPflag(self.bx);
                        }
                        self.ip+=2;
                    }
                }
                else if (reg==6)
                {
                    // DIV rmw
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);
                    dbgString=format!("DIV {}",regVec[0]);
                    *bytesRead=2;
    
                    if debugFlag==false
                    {
                        // TODO other regs, flags
                        if regVec[0]=="CX"
                        {
                            let dx32:u32=self.dx as u32;
                            let ax32:u32=self.ax as u32;
                            let cx32:u32=self.cx as u32;
                            let mut val2divide:u32=(ax32|(dx32<<16));
                            let modulo=val2divide%(self.cx as u32);
                            let quotient=val2divide/cx32;
                            self.dx=modulo as u16;
                            self.ax=quotient as u16;

                            self.doZflag(quotient as u16);
                            //self.doPflag(quotient as u16); // todo check p flag
                        }
                        self.ip+=2;
                    }
                }
            },
            0xf8 =>
            {
                // CLC
                dbgString=format!("CLC");
                *bytesRead=1;

                if debugFlag==false
                {
                    self.setCflag(false);
                    self.ip+=1;
                }
            },
            0xfc =>
            {
                // CLD
                dbgString=format!("CLD");
                *bytesRead=1;

                if debugFlag==false
                {
                    self.setDflag(false);
                    self.ip+=1;
                }
            },
            0xfe =>
            {
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if (reg==0)
                {
                    // INC rmb
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                    dbgString=format!("INC {}",regVec[0]);
                    *bytesRead=2;
    
                    if debugFlag==false
                    {
                        if regVec[0]=="AH"
                        {
                            let mut ah:u16=(self.ax>>8) as u16;
                            ah+=1;
                            self.ax=(self.ax&0xff)|(ah<<8);
                        }
                        self.ip+=2;
                    }
                }
                else if reg==1
                {
                    // DEC rmb
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                    dbgString=format!("DEC {}",regVec[0]);
                    *bytesRead=2;
                    if regVec[0]=="Direct Addr" { *bytesRead+=2; }
    
                    if debugFlag==false
                    {
                        // TODO flags, all
                        if regVec[0]=="Direct Addr"
                        {
                            let addr=pmachine.readMemory16(theCS,theIP+2,pvga);
                            let mut val:u8=pmachine.readMemory(self.ds,addr,pvga);
                            val-=1;
                            pmachine.writeMemory(self.ds,addr,val,pvga);
                        }
                        self.ip+=2;
                    }
                }
                else
                {
                    self.abort(&format!("Unhandled 0xfe reg={}",reg));
                }
            },
            0xff =>
            {
                let instrType=pmachine.readMemory(theCS,theIP+1,pvga);
                let reg:usize=((instrType>>3)&0x07).into();

                if (reg==2)
                {
                    // CALL rw
                    let addressingModeByte=pmachine.readMemory(theCS,theIP+1,pvga);
                    let regVec:Vec<String>=self.debugDecodeAddressingModeByte(addressingModeByte,0,opcode&1);

                    dbgString=format!("CALL {}",regVec[0]);
                    *bytesRead=2;
    
                    let mut addrRel:u16=0;
                    if regVec[0]=="BP" { addrRel=self.bp; }
                    if debugFlag==false
                    {
                        pmachine.push16(self.ip+2,self.ss,self.sp);
                        self.sp-=2;
                        self.ip=addrRel;
                    }
                }
            },
            _ =>
            {
                *retErr=format!("x86cpu::Unhandled opcode 0x{:02x}",opcode);
                dbgString="UNKNOWN".to_string();
                *bytesRead=0;
            }
        }

        dbgAddress.push_str(&dbgString);
        return dbgAddress;
    }
}
