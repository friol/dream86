/* dream86 - machine 2o22 */

use std::fs::File;
use std::io::prelude::*;
use rand::Rng;
use std::process;

use crate::vga::vga;
use crate::x86cpu::x86cpu;

pub struct machine 
{
    pub ram: Vec<u8>,
    pub stackey: Vec<u8>,
    pub internalClockTicker: u64,
    pub clockTicker: u64,
    pub keyboardQueue: Vec<u8>
}

impl machine 
{
    fn loadBIOS(mem:&mut Vec<u8>,fname:&str)
    {
        // Load BIOS image into F000:0100
        let biosBase:usize=0xF0100;
        
        let mut f = match File::open(fname) {
            Ok(f) => f,
            Err(_e) => {
                println!("Unable to open file {}",fname);
                return;
            }
        };
        let biosLen:usize=f.metadata().unwrap().len() as usize;
        let mut data = Vec::new();
        f.read_to_end(&mut data).ok();        

        //println!("BIOS \"{}\" len is {}",fname,biosLen);

        for i in 0..biosLen
        {
            mem[biosBase+i]=data[i];
        }
    }

    fn loadCOMFile(mem:&mut Vec<u8>,fname:&str)
    {
        // Load .com image into F000:0100
        let comBase:usize=0xF0100;
        //let comBase:usize=0x8130;
        
        let mut f = match File::open(fname) {
            Ok(f) => f,
            Err(e) => {
                println!("Unable to open file {} error:{}",fname,e);
                return;
            }
        };
        let comLen:usize=f.metadata().unwrap().len() as usize;
        let mut data = Vec::new();
        f.read_to_end(&mut data).ok();        

        for i in 0..comLen
        {
            mem[comBase+i]=data[i];
        }
    }

    pub fn addKeystroke(&mut self,ks:u8)
    {
        self.keyboardQueue.push(ks);
    }

    pub fn handleINT(&mut self,pcpu:&mut x86cpu,intNum:u8,pvga:&mut vga) -> bool
    {
        if intNum==0x10
        {
            // VGA int
            if (pcpu.ax&0xff00)==0
            {
                // set videomode
                pvga.setVideomode(pcpu.ax&0xff);
                return true;
            }
        }
        else if intNum==0x1a
        {
            // read system clock counter, AH=0
            if (pcpu.ax&0xff00)==0
            {
                pcpu.ax&=0xff00; // midnight flag 0 in AL
                pcpu.cx=((self.clockTicker&0xffff0000)>>16) as u16;
                pcpu.dx=(self.clockTicker&0xffff) as u16;
                return true;
            }
        }
        else if intNum==0x21
        {
            // AH=0x4c - exit to DOS
            if (pcpu.ax&0xff00)==0x4c00
            {
                println!("Program terminated by int 21h");
                process::exit(0x0);
            }
            // AH=0x09 - print string to stdout
            if (pcpu.ax&0xff00)==0x0900
            {
                let mut stringVec:Vec<u8>=Vec::new();
                let mut curIdx=pcpu.dx;

                let mut b=self.readMemory(pcpu.ds,curIdx,pvga);
                while (b as char).to_string()!="$"
                {
                    stringVec.push(b);
                    curIdx+=1;
                    b=self.readMemory(pcpu.ds,curIdx,pvga);
                }

                for ch in stringVec
                {
                    pvga.outputCharToStdout(ch);
                }

                return true;
            }
        }
        else if intNum==0x16
        {
            // AH=0 - get keystroke
            if (pcpu.ax&0xff00)==0
            {
                if self.keyboardQueue.len()==0
                {
                    return false;
                }
                else
                {
                    let scanCode:u8=self.keyboardQueue[self.keyboardQueue.len()-1];
                    self.keyboardQueue.pop();
                    pcpu.ax=(scanCode as u16)<<8;
                    return true;
                }
            }
            // AH=1 - get keyboard status
            else if (pcpu.ax&0xff00)==0x0100
            {
                if self.keyboardQueue.len()==0
                {
                    pcpu.ax=0;   
                    pcpu.setZflag(true);             
                }
                else
                {
                    pcpu.setZflag(false);   
                    let scanCode:u8=self.keyboardQueue[self.keyboardQueue.len()-1];
                    pcpu.ax=(scanCode as u16)<<8;
                }
                return true;
            }
            // AH=2 - read keyboard flags
            else if (pcpu.ax&0xff00)==0x0200
            {
                /*
                |7|6|5|4|3|2|1|0|  AL or BIOS Data Area 40:17
                | | | | | | | `---- right shift key depressed
                | | | | | | `----- left shift key depressed
                | | | | | `------ CTRL key depressed
                | | | | `------- ALT key depressed
                | | | `-------- scroll-lock is active
                | | `--------- num-lock is active
                | `---------- caps-lock is active
                `----------- insert is active                
                */

                let mut al=0;
                if self.keyboardQueue.len()==0
                {
                    pcpu.ax=0xff00;   
                }
                else
                {
                    let scanCode:u8=self.keyboardQueue[self.keyboardQueue.len()-1];
                    if scanCode==0xff
                    {
                        al|=2;
                        self.keyboardQueue.pop();
                    }
                    pcpu.ax=0xff00|al;
                }

                return true;
            }
        }

        return true;
    }

    pub fn push16(&mut self,val:u16,segment:u16,address:u16)
    {
        let i64seg:i64=segment.into();
        let i64addr:i64=address.into();
        let flatAddr:i64=i64addr|(i64seg*16);

        self.ram[flatAddr as usize]=(val&0xff) as u8;
        self.ram[(flatAddr+1) as usize]=((val>>8)&0xff) as u8;

        self.stackey.push((val&0xff) as u8);
        self.stackey.push(((val>>8)&0xff) as u8);
    }

    pub fn pop16(&mut self,segment:u16,address:u16) -> u16
    {
        let i64seg:i64=segment.into();
        let i64addr:i64=address.into();
        let flatAddr:i64=i64addr|(i64seg*16);

        let mut retval:u16=0;

        retval|=self.ram[(flatAddr+2) as usize] as u16;
        let mut upperPart:u16=self.ram[(flatAddr+3) as usize].into();
        upperPart<<=8;
        retval|=upperPart;

        self.stackey.pop();
        self.stackey.pop();

        return retval;
    }

    pub fn readMemory(&self,segment:u16,address:u16,pvga:&vga) -> u8
    {
        let i64seg:i64=segment.into();
        let i64addr:i64=address.into();
        let flatAddr:i64=i64addr|(i64seg*16);

        if ((flatAddr>=0xa0000) && (flatAddr<=0xaffff)) ||
           ((flatAddr>=0xb8000) && (flatAddr<=0xbffff))
        {
            // VGA framebuffer
            return pvga.readMemory(flatAddr);
        }

        return self.ram[flatAddr as usize];
    }

    pub fn readMemory16(&self,segment:u16,address:u16,pvga:&vga) -> u16
    {
        let i64seg:i64=segment.into();
        let i64addr:i64=address.into();
        let flatAddr:i64=i64addr|(i64seg*16);

        if (flatAddr>=0xa0000) && (flatAddr<=0xaffff) ||
           ((flatAddr>=0xb8000) && (flatAddr<=0xbffff))
        {
            return pvga.readMemory16(flatAddr);
        }

        let lobyte:u16=self.ram[flatAddr as usize] as u16;
        let hibyte:u16=self.ram[(flatAddr+1) as usize] as u16;

        return lobyte|(hibyte<<8);
    }

    pub fn writeMemory(&mut self,segment:u16,address:u16,val:u8,pvga:&mut vga)
    {
        let i64seg:i64=segment.into();
        let i64addr:i64=address.into();
        let flatAddr:i64=i64addr|(i64seg*16);

        if (flatAddr>=0xa0000) && (flatAddr<=0xaffff) ||
           ((flatAddr>=0xb8000) && (flatAddr<=0xbffff))
        {
            // VGA framebuffer
            pvga.writeMemory(flatAddr,val);
        }
        else
        {
            self.ram[flatAddr as usize]=val;
        }
    }

    pub fn writeMemory16(&mut self,segment:u16,address:u16,val:u16,pvga:&mut vga)
    {
        let i64seg:i64=segment.into();
        let i64addr:i64=address.into();
        let flatAddr:i64=i64addr|(i64seg*16);

        if (flatAddr>=0xa0000) && (flatAddr<=0xaffff) ||
           ((flatAddr>=0xb8000) && (flatAddr<=0xbffff))
        {
            pvga.writeMemory16(flatAddr,val);
        }
        else
        {
            self.ram[flatAddr as usize]=(val&0xff) as u8;
            self.ram[(flatAddr+1) as usize]=(val>>8) as u8;
        }
    }

    pub fn update(&mut self)
    {
        // todo: update 18.206 times per second
        // assume 100.000 instructions per seconds
        self.internalClockTicker+=1;
        if self.internalClockTicker>=5555
        {
            self.internalClockTicker=0;
            self.clockTicker+=1;
        }
    }

    pub fn new(comFullPath:&str,ramSize:usize) -> Self 
    {
        let mut machineRAM:Vec<u8>=Vec::with_capacity(ramSize);
        for _i in 0..ramSize
        {
            let num = rand::thread_rng().gen_range(0..256);
            machineRAM.push(num as u8);
        }

        //Self::loadBIOS(&mut machineRAM,"./bios/bios_cga");
        Self::loadCOMFile(&mut machineRAM,comFullPath);

        let thestack:Vec<u8>=Vec::new();
        let kq:Vec<u8>=Vec::new();

        machine 
        {
            ram: machineRAM,
            stackey: thestack,
            internalClockTicker: 0,
            clockTicker: 0,
            keyboardQueue: kq
        }
    }
}
