/* dream86 - machine 2o22 */

use std::fs::File;
use std::io::prelude::*;
use rand::Rng;
use std::process;

use crate::vga::vga;
use crate::x86cpu::x86cpu;
use crate::fddController::fddController;

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

    fn loadBinFile(mem:&mut Vec<u8>,fname:&str)
    {
        // Load image into F000:0000
        let comBase:usize=0xF0000;
        
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

    // returns if we should go on with the code
    pub fn handleINT(&mut self,intNum:u8,pcpu:&mut x86cpu,pvga:&mut vga,pdisk:&fddController) -> bool
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
            else if (pcpu.ax&0xff00)==0x0e00
            {
                // AH=0e - output char to stdout
                let ch:u8=(pcpu.ax&0xff) as u8;
                pvga.outputCharToStdout(ch); 
            }
            else if (pcpu.ax&0xff00)==0x0200
            {
                // INT 10,2 - Set Cursor Position
                pvga.setCursorPosition(pcpu.dx&0xff,pcpu.dx>>8);
                return true;
            }
            else if (pcpu.ax&0xff00)==0x0300
            {
                // INT 10,3 - Read Cursor Position and Size
                // DH = row
	            // DL = column
                let cp=pvga.getCursorPosition();
                pcpu.dx=(cp.0 as u16)|((cp.1 as u16)<<8);
                return true;
            }
            else
            {
                println!("Unknown interrupt");
                println!("{:02x},{:02x}",intNum,pcpu.ax>>8);
                process::exit(0x0100);
            }
        }
        else if intNum==0x13
        {
            // disk stuff
            if (pcpu.ax&0xff00)==0x0
            {
                // INT 13,0 - Reset Disk System
                // we assume dl=drive number is 0

                pcpu.ax=0; // disk status AH=0
                pcpu.setCflag(false); // CF = 0 if successful
            }
            else if (pcpu.ax&0xff00)==0x0200
            {
                // INT 13,2 - Read Disk Sectors
                let driveNumber=pcpu.dx&0xff;
                let numOfSectorsToRead:u64=(pcpu.ax&0x7f) as u64;
                let sectorNumber:u64=((pcpu.cx&0x3f)-1) as u64;

                //let trackNumber:u64=((((pcpu.cx>>6)&0xff)<<8)|(pcpu.cx>>8)) as u64;
                //cylinder := ( (CX and 0xFF00) shr 8 ) or ( (CX and 0xC0) shl 2)
                let cylinderNumber:u64=((pcpu.cx>>8)|((pcpu.cx&0xc0)<<2)) as u64;
                
                let headNumber:u64=(pcpu.dx>>8) as u64;
                let loAddr=pcpu.bx;
                let hiAddr=pcpu.es;

                if driveNumber!=0
                {
                    println!("INT 13,2: Trying to read sectors from a drive that is not A:");
                    process::exit(0x0100);
                    //pcpu.ax=0x0700;
                    //pcpu.setCflag(true);
                    //return true;
                }

                if numOfSectorsToRead==0
                {
                    println!("Trying to read 0 sectors");
                    process::exit(0x0100);
                }

                pdisk.readDiskSectors(self,pvga,numOfSectorsToRead,sectorNumber,cylinderNumber,headNumber,loAddr,hiAddr);
        
                pcpu.ax=numOfSectorsToRead as u16;
                pcpu.setCflag(false); // CF = 0 if successful

                return true;
            }         
            else if (pcpu.ax&0xff00)==0x0800
            {
                // INT 13,8 - Get Current Drive Parameters (XT & newer)   
                // DL = drive number (0=A:, 1=2nd floppy, 80h=drive 0, 81h=drive 1)
                if (pcpu.dx&0xff)==0x80
                {
                    // hard drive?
                    pcpu.ax=(pcpu.ax&0xff)|(0x07<<8);
                    pcpu.setCflag(true);
                }
                else if (pcpu.dx&0xff)==0
                {
                    // A: drive
                    // BL = CMOS drive type
                    // 01 - 5¬  360K	     03 - 3«  720K
                    // 02 - 5¬  1.2Mb	     04 - 3« 1.44Mb                    // TODO
                    pcpu.ax=0;
                    pcpu.bx=(pcpu.bx&0xff00)|0x04; // 1.44mb diskette
                    pcpu.cx=0x4f12;
                    pcpu.dx=0x0101;
                    pcpu.setCflag(false); // CF = 0 if successful
                }
                else
                {
                    println!("Other drive type int 13,8");
                    process::exit(0x0100);
                }

                return true;
            }
            else if (pcpu.ax&0xff00)==0x1500
            {
                // INT 13,15 - Read DASD Type (XT BIOS from 1/10/86 & newer)
                assert_eq!(pcpu.dx&0xff,0); // drive a:
                pcpu.ax=0x0200|(pcpu.ax&0xff);
                pcpu.setCflag(false); // CF = 0 if successful
                return true;
            }
            else if (pcpu.ax&0xff00)==0x1600
            {
                // INT 13,16 - Change of Disk Status (XT BIOS from 1/10/86 & newer)
                assert_eq!(pcpu.dx&0xff,0); // drive a:
                pcpu.ax=0x0000|(pcpu.ax&0xff);
                pcpu.setCflag(false); // CF = 0 if successful
                return true;
            }
            else
            {
                println!("Unknown interrupt");
                println!("{:02x},{:02x}",intNum,pcpu.ax>>8);
                process::exit(0x0100);
            }
        }
        else if intNum==0x11
        {
            // INT 11 - BIOS Equipment Determination / BIOS Equipment Flags
            /*
                AX contains the following bit flags:

                    |F|E|D|C|B|A|9|8|7|6|5|4|3|2|1|0|  AX
                    | | | | | | | | | | | | | | | `---- IPL diskette installed
                    | | | | | | | | | | | | | | `----- math coprocessor
                    | | | | | | | | | | | | `-------- old PC system board RAM < 256K
                    | | | | | | | | | | | | | `----- pointing device installed (PS/2)
                    | | | | | | | | | | | | `------ not used on PS/2
                    | | | | | | | | | | `--------- initial video mode
                    | | | | | | | | `------------ # of diskette drives, less 1
                    | | | | | | | `------------- 0 if DMA installed
                    | | | | `------------------ number of serial ports
                    | | | `------------------- game adapter installed
                    | | `-------------------- unused, internal modem (PS/2)
                    `----------------------- number of printer ports     
                    
                    - bits 3 & 2,  system board RAM if less than 256K motherboard
                        00 - 16K		     01 - 32K
                        10 - 16K		     11 - 64K (normal)

                    - bits 5 & 4,  initial video mode
                        00 - unused 	     01 - 40x25 color
                        10 - 80x25 color	     11 - 80x25 monochrome


                    - bits 7 & 6,  number of disk drives attached, when bit 0=1
                        00 - 1 drive	     01 - 2 drives
                        10 - 3 drive	     11 - 4 drives                    
            */            

            pcpu.ax=0x5115; // 101 0100 0100 0101
            return true;
        }
        else if intNum==0x12
        {
            // INT 12 - Memory Size Determination
            pcpu.ax=0x280; // TODO configurable size
            return true;            
        }
        else if intNum==0x14
        {
            // INT 14,0 - Initialize Communications Port Parameters
            // TODO
            return true;
        }
        else if intNum==0x15
        {
            // INT 15,C0 - Return System Configuration Parameters (PS/2 only)
            if (pcpu.ax&0xff00)==0xc000
            {
                // TODO
                pcpu.ax=pcpu.ax&0xff;
                pcpu.bx=0;
                pcpu.setCflag(false); // CF = 0 if successful
                return true;
            }
            else if (pcpu.ax&0xff00)==0x4100
            {
                // INT 15,41 - Wait on External Event (convertible only)
                // TODO
                pcpu.setCflag(true);
                return true;
            }
            else
            {
                println!("Unknown interrupt");
                println!("{},{}",intNum,pcpu.ax>>8);
                process::exit(0x0100);
            }
        }
        else if intNum==0x17
        {
            // INT 17,1 - Initialize Printer Port
            // TODO
            return true;
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
            else if (pcpu.ax&0xff00)==0x0200
            {
                // INT 1A,2 - Read Time From Real Time Clock (XT 286,AT,PS/2)
                // TODO
                /*
                    CH = hours in BCD
                    CL = minutes in BCD
                    DH = seconds in BCD
                    DL = 1 if daylight savings time option
                */
                pcpu.cx=0x2324;
                pcpu.dx=0x0100;
                pcpu.setCflag(false); // CF = 0 if successful
                return true;
            }
            else if (pcpu.ax&0xff00)==0x0400
            {
                // INT 1A,4 - Read Real Time Clock Date (XT 286,AT,PS/2)
                // TODO
                pcpu.cx=0x2022; // 2022 forevah
                pcpu.dx=0x0101; 
                pcpu.setCflag(false); // CF = 0 if successful
                return true;
            }
            else
            {
                println!("Unknown interrupt");
                println!("{},{}",intNum,pcpu.ax>>8);
                process::exit(0x0100);
            }
        }
        else if intNum==0x29
        {
            // INT 29 - DOS Fast Character I/O (Undocumented 2.x+)
            let ch:u8=(pcpu.ax&0xff) as u8;
            pvga.outputCharToStdout(ch); 
            return true;
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
                    pcpu.ax=((scanCode as u16)<<8)|(scanCode as u16);
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
            else
            {
                println!("Unknown interrupt");
                println!("{:02x},{:02x}",intNum,pcpu.ax>>8);
                process::exit(0x0100);
            }
        }
        else if intNum==0x24
        {
            // do nothing for now
            return true;
        }
        else
        {
            println!("Unknown interrupt");
            println!("{}",intNum);
            process::exit(0x0100);
        }

        return true;
    }

    pub fn push16(&mut self,val:u16,segment:u16,address:u16)
    {
        let i64seg:i64=segment.into();
        let i64addr:i64=address.into();
        let flatAddr:i64=i64addr+(i64seg*16);

        self.ram[(flatAddr-2) as usize]=(val&0xff) as u8;
        self.ram[(flatAddr-1) as usize]=((val>>8)&0xff) as u8;

        self.stackey.push((val&0xff) as u8);
        self.stackey.push(((val>>8)&0xff) as u8);
    }

    pub fn pop16(&mut self,segment:u16,address:u16) -> u16
    {
        let i64seg:i64=segment.into();
        let i64addr:i64=address.into();
        let flatAddr:i64=i64addr+(i64seg*16);

        let mut retval:u16=0;

        retval|=self.ram[(flatAddr) as usize] as u16;
        let mut upperPart:u16=self.ram[(flatAddr+1) as usize].into();
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
        let flatAddr:i64=i64addr+(i64seg*16);

        if ((flatAddr>=0xa0000) && (flatAddr<=0xaffff)) || ((flatAddr>=0xb8000) && (flatAddr<=0xbffff))
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
        let flatAddr:i64=i64addr+(i64seg*16);

        if ((flatAddr>=0xa0000) && (flatAddr<=0xaffff)) || ((flatAddr>=0xb8000) && (flatAddr<=0xbffff))
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
        let flatAddr:i64=i64addr+(i64seg*16);

        if ((flatAddr>=0xa0000) && (flatAddr<=0xaffff)) || ((flatAddr>=0xb8000) && (flatAddr<=0xbffff))
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
        let flatAddr:i64=i64addr+(i64seg*16);

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

    pub fn new(_comFullPath:&str,ramSize:usize,mode:u8) -> Self 
    {
        let mut machineRAM:Vec<u8>=Vec::with_capacity(ramSize);
        for _i in 0..ramSize
        {
            let num = rand::thread_rng().gen_range(0..256);
            machineRAM.push(num as u8);
        }

        if mode==0 { Self::loadBIOS(&mut machineRAM,"./bios/bios_cga"); }
        else if mode==2 { Self::loadBinFile(&mut machineRAM,_comFullPath); }
        else { Self::loadCOMFile(&mut machineRAM,_comFullPath); }

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
