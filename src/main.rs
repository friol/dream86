
/* 
    
    dream86 - 2o22 
    x86/PC emulator in Rust

*/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::env;
use std::time::Instant;
use std::process;


mod vga;
mod pic8259;
mod machine;
mod x86cpu;
mod fddController;
mod guiif;

//
//
//

fn main()
{
    let mut _breakIt=false;

    let args: Vec<String> = env::args().collect();
    if args.len()!=4
    {
        println!("syntax: dream86.exe <disk image full path> <com name> <runmode>");        
        process::exit(0x0);
    }

    let diskImageName=String::from(&args[1]);
    let comName=String::from(&args[2]);
    let runMode=String::from(&args[3]).parse::<u8>().unwrap();

    //

    let _thePIC=pic8259::pic8259::new();
    let mut theVGA=vga::vga::new("./fonts/9x16.png","./fonts/cga8.png");
    let theDisk=fddController::fddController::new(&diskImageName);
    let mut theMachine=machine::machine::new(&comName,0x100000,runMode,machine::machineType::machineEGA);
    let mut theCPU=x86cpu::x86cpu::new(runMode);
    let mut theGUI=guiif::guiif::new(0x02,theCPU.cs,theCPU.ip);

    let mut goOut=false;
    while !goOut
    {
        let startTime = Instant::now();
        theGUI.clearScreen();
        theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
        theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
        theGUI.drawMemory(&mut theVGA,&theMachine,0x3322,0x29f4,80);
        theGUI.drawVideoInfo(&mut theVGA);
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
            theVGA.update();
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
                theVGA.update();

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
/*
            //let mut f = match File::open("./programs/tests/res_add.bin") {
            //let mut f = match File::open("./programs/tests/res_cmpneg.bin") {
            //let mut f = match File::open("./programs/tests/res_shifts.bin") {
            let mut f = match File::open("./programs/tests/res_sub.bin") {
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
                if (curb!=data[_b]) && (data[_b]!=0)
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
                theVGA.update();
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
                theVGA.update();
                inum+=1;

                //if (theCPU.cs==0xdeb) && (theCPU.ip==0x100) // start of .com
                //if (theCPU.cs==0x5250) && (theCPU.ip==0x4da4) // arkanoid before splash screen
                //if (theCPU.cs==0xdeb) && (theCPU.ip==0x4110)
                //if (theCPU.cs==0xdeb) && (theCPU.ip==0x413a)
                //if _breakIt && ((theCPU.cs==0x2f2) && (theCPU.ip==0x1460)) // int 21h
                //if _breakIt && ((theCPU.cs==0x19) && (theCPU.ip==0x40f8)) // int 21h dos 6.22
                //if (theCPU.cs==0x19) && (theCPU.ip==0xa872)
                //if (theCPU.cs==0x19) && (theCPU.ip==0x7736)
                //if (theCPU.cs==0xdfb) && (theCPU.ip==0x4adc)
                //if (theCPU.cs==0x0e88) && (theCPU.ip==0x02ed)
                //if (theCPU.cs==0xe00) && (theCPU.ip==0x4c7)
                //if (theCPU.cs==0xe00) && (theCPU.ip==0x3ff)
                //if (theCPU.cs==0x1301) && (theCPU.ip==0x36ea)
                //if (theCPU.cs==0x24ac) && (theCPU.ip==0x2c6)
                //if (theCPU.cs==0x24ac) && (theCPU.ip==0x11a9)
                //if (theCPU.cs==0x24ac) && (theCPU.ip==0x340)
                //if (theCPU.cs==0x1a79) && (theCPU.ip==0x00d7)
                //if (theCPU.cs==0x0c3a) && (theCPU.ip==0x138)
                //if (theCPU.cs==0x70) && (theCPU.ip==0x1bcd)
                //if (theCPU.cs==0x8ec1) && (theCPU.ip==0x4bf)
                //if (theCPU.cs==0x9f84) && (theCPU.ip==0x420)
                //if (theCPU.cs==0x70) && (theCPU.ip==0x1e41)
                //if (theCPU.cs==0x8ec1) && (theCPU.ip==0x0536)
                if false
                {
                    bailOut=true;
                }

                if inum>2000
                {
                    theGUI.clearScreen();
                    theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
                    theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
                    theGUI.drawMemory(&mut theVGA,&theMachine,0x3322,0x29f4,80);
                    theGUI.drawVideoInfo(&mut theVGA);
                    theVGA.fbTobuf32(&mut theGUI);
                    theGUI.updateVideoWindow(&theVGA);

                    if theGUI.checkExit()
                    {
                        bailOut=true;
                    }

                    theGUI.processKeys(&mut theMachine,&mut theCPU,&mut theVGA);
                    
                    //thread::sleep(time::Duration::from_millis(4));                    
                    inum=0;
                }
            }
        }
    }
}
