
/* 
    
    dream86 - 2o22 
    x86/PC emulator in Rust

*/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::process;
//use std::{thread, time};
//use std::fs::File;
//use std::io::prelude::*;

use std::env;
use std::time::Instant;

mod vga;
mod machine;
mod x86cpu;
mod fddController;
mod guiif;

fn main()
{
    let mut _breakIt=false;
    
    let args: Vec<String> = env::args().collect();
    if args.len()!=4
    {
        println!("syntax: cargo run --release <disk image full path> <com name> <runmode>");        
        process::exit(0x0);
    }

    let diskImageName=String::from(&args[1]);
    let comName=String::from(&args[2]);
    let runMode=String::from(&args[3]).parse::<u8>().unwrap();

    //

    let mut theVGA=vga::vga::new("./fonts/9x16.png");
    let mut theMachine=machine::machine::new(&comName,0x100000,runMode);
    let theDisk=fddController::fddController::new(diskImageName);
    let mut theCPU=x86cpu::x86cpu::new();
    let mut theGUI=guiif::guiif::new(0x02,theCPU.cs,theCPU.ip);

    let mut goOut=false;
    while !goOut
    {
        let startTime = Instant::now();
        theGUI.clearScreen();
        theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
        theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
        theGUI.drawMemory(&theVGA,&theMachine,0x1665,0xfffe,80);
        theVGA.fbTobuf32(&mut theGUI);
        theGUI.updateVideoWindow(&theVGA);

        //

        let act=theGUI.getKeyAction();
        if act==guiif::keyAction::actionQuit
        {
            goOut=true;
        }
        else if act==guiif::keyAction::actionStep
        {
            let mut bytesRead=0;
            theCPU.executeOne(&mut theMachine,&mut theVGA,&theDisk,false,&mut bytesRead,&0,&0);
            theMachine.update(&mut theCPU);
        }
        else if act==guiif::keyAction::actionRunToRet
        {
            let startTime = Instant::now();
            let mut bytesRead=1;
            let mut _dbgstr=String::from("");
            let mut iterations:u64=0;

            let stopit=false;

            while (bytesRead!=0) && (!stopit)
            {
                _dbgstr=theCPU.executeOne(&mut theMachine,&mut theVGA,&theDisk,false,&mut bytesRead,&0,&0);
                theMachine.update(&mut theCPU);

                if (iterations%1000)==0
                {
                    theGUI.clearScreen();
                    theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
                    theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
                    theVGA.fbTobuf32(&mut theGUI);
                    theGUI.updateVideoWindow(&theVGA);
                }
                iterations+=1;
            }
        }
        else if act==guiif::keyAction::actionRunToAddr
        {
            /*let mut bytesRead=1;
            //while theCPU.ip!=0x6a07
            //while theCPU.ip!=0x6f4d
            while theCPU.ip!=0x153f
            {
                theCPU.executeOne(&mut theMachine,&mut theVGA,&theDisk,false,&mut bytesRead,&0,&0);
                theMachine.update();
            }*/
            _breakIt=true;
            //theMachine.writeMemory(0xe0b,0x4f5,0x4,&mut theVGA);
            //theMachine.writeMemory(0x977,0x3547,0x90,&mut theVGA);
/*
            //let mut f = match File::open("./programs/tests/res_add.bin") {
            //let mut f = match File::open("./programs/tests/res_cmpneg.bin") {
            //let mut f = match File::open("./programs/tests/res_shifts.bin") {
            let mut f = match File::open("./programs/tests/res_datatrnf.bin") {
                Ok(f) => f,
                Err(e) => {
                    println!("Unable to open file error:{}",e);
                    return;
                }
            };
            let comLen:usize=f.metadata().unwrap().len() as usize;
            let mut data = Vec::new();
            f.read_to_end(&mut data).ok();        

            for _b in 0..comLen
            {
                let curb=theMachine.readMemory(0x0,_b as u16,&mut theVGA);
                if curb!=data[_b]
                {
                    println!("fck at byte {:04x} curb {:02x} reference {:02x}",_b,curb,data[_b]);
                    return;
                }
            }
*/    
        }
        else if act==guiif::keyAction::actionIncDebugCursor
        {
            theGUI.incDebugCursor();            
        }
        else if act==guiif::keyAction::actionDecDebugCursor
        {
            theGUI.decDebugCursor();            
        }
        else if act==guiif::keyAction::actionRunToCursor
        {
            let mut bytesRead=1;
            let bpPos:u16=theGUI.getRuntoIp(&mut theCPU,&mut theMachine,&mut theVGA,&theDisk);
            while theCPU.ip!=bpPos
            {
                theCPU.executeOne(&mut theMachine,&mut theVGA,&theDisk,false,&mut bytesRead,&0,&0);
                theMachine.update(&mut theCPU);
            }
        }
        else if act==guiif::keyAction::actionRun
        {
            let startTime = Instant::now();
            let mut bytesRead=1;
            let mut inum:u64=0;
            let mut bailOut=false;
            while !bailOut
            {
                let _dbgstr=theCPU.executeOne(&mut theMachine,&mut theVGA,&theDisk,false,&mut bytesRead,&0,&0);
                theMachine.update(&mut theCPU);
                inum+=1;

                //if theCPU.ip==0x7d74 // dos 3.3 reads disk 2nd time here
                // 0x0070:0x356a - writes nec io.sys banner
                // 0x0070:0x36d5 - dos 3.3 tries to check hard drive (dl=0x80)
                // 0x0070:0x3708 - dos 3.3 drive a: check (dl=00)
                // 0x0070:0x3928 - int 15h
                // 0x0070:0x3f65 - cmp si, 0xffff (sign extended)

                //if (theCPU.cs==0x5250) && (theCPU.ip==0x4250)
                //if (theCPU.cs==0xdeb) && (theCPU.ip==0x108)
                //if (theCPU.cs==0xdeb) && (theCPU.ip==0x4110)
                //if (theCPU.cs==0xdeb) && (theCPU.ip==0x413a)
                //if _breakIt && ((theCPU.cs==0x2f2) && (theCPU.ip==0x1460)) // int 21h
                //if ((theCPU.cs==0x2219) && (theCPU.ip==0x40))
                //if ((theCPU.cs==0xe0b) && (theCPU.ip==0x1da9))
                //if (theCPU.cs==0xd08) && (theCPU.ip==0x12d)
                //if (theCPU.cs==0x2f2) && (theCPU.ip==0x3d47)
                //if (theCPU.cs==0x9dfd) && (theCPU.ip==0xeea)
                //if (theCPU.cs==0xdeb) && (theCPU.ip==0x03d8a)
                //if (theCPU.cs==0xdfb) && (theCPU.ip==0x0) // av.exe 
                //if (theCPU.cs==0xdfb) && (theCPU.ip==0x49f6) // av.exe 
                //if (theCPU.cs==0xdfb) && (theCPU.ip==0x4adc)
                //if (theCPU.cs==0xdeb) && (theCPU.ip==0x100)
                if false
                {
                    bailOut=true;
                }

                //if dbgstr.contains(" (23)")
                //{
                    /*if 
                        (!dbgstr.contains("AH,AH")) &&
                        (!dbgstr.contains("BH,BH")) &&
                        (!dbgstr.contains("CH,CH")) &&
                        (!dbgstr.contains("DH,DH")) &&
                        (!dbgstr.contains("AX,AX")) &&
                        (!dbgstr.contains("BX,BX")) &&
                        (!dbgstr.contains("CX,CX")) &&
                        (!dbgstr.contains("BP,BP")) &&
                        (!dbgstr.contains("SI,SI")) &&
                        (!dbgstr.contains("DI,DI")) &&
                        (!dbgstr.contains("DX,DX"))*/
                  /*  {
                        bailOut=true;
                    }*/
                //}

                if inum>2000
                {
                    theGUI.clearScreen();
                    theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
                    theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
                    theGUI.drawMemory(&theVGA,&theMachine,0x0dfb,0x49f0,80);
                    theVGA.fbTobuf32(&mut theGUI);
                    theGUI.updateVideoWindow(&theVGA);

                    if theGUI.checkEscPressed()
                    {
                        bailOut=true;
                    }

                    if theGUI.processKeys(&mut theMachine,&mut theCPU,&mut theVGA)
                    {
                        //bailOut=true;
                    }
                    
                    //thread::sleep(time::Duration::from_millis(4));                    
                    inum=0;
                }
            }
        }
    }
}
