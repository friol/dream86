
/* 
    dream86 - 2o22 

    my 1st PC was an 80386 with MS-DOS 3.30
    my father bought it, so this program (that runs MS-DOS 3.30) is dedicated to him

*/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

//use std::process;
//use std::{thread, time};
use std::time::Instant;

mod vga;
mod machine;
mod x86cpu;
mod fddController;
mod guiif;

fn main()
{
    let mut theVGA=vga::vga::new("./fonts/9x16.png");

    //let mut theMachine=machine::machine::new("./programs/dino.com",0x100000,1);
    let mut theMachine=machine::machine::new("./programs/pillman.com",0x100000,0);
    //let mut theMachine=machine::machine::new("./programs/invaders.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/fbird.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/bricks.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/rogue.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/sorryass.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/basic.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/test386.bin",0x100000,2);
    //let mut theMachine=machine::machine::new("./programs/dirojedc.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/CGADOTS.COM",0x100000);
    //let mut theMachine=machine::machine::new("../../testcga.com",0x100000);
    //let mut theMachine=machine::machine::new("./programs/SIN.com",0x100000,1);

    //let theDisk=fddController::fddController::new("./diskimages/pillman.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/invaders.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/tetros.img".to_string()); // Unhandled opcode 61 at 7d84
    //let theDisk=fddController::fddController::new("./diskimages/basic.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/toledo_atomchess_bootos.img".to_string());
    let theDisk=fddController::fddController::new("./diskimages/Dos3.3.img".to_string()); // ohohoh
    //let theDisk=fddController::fddController::new("./diskimages/freedos.img".to_string()); // goes awry
    //let theDisk=fddController::fddController::new("./diskimages/dos3.31.microsoft.img".to_string()); // goes awry
    //let theDisk=fddController::fddController::new("./diskimages/dos5.0.img".to_string()); // Unhandled opcode 83 at 0070:1b5d
    //let theDisk=fddController::fddController::new("./diskimages/Dos6.22.img".to_string()); // Unhandled opcode 83 at 0070:1ba3
    //let theDisk=fddController::fddController::new("./diskimages/OLVDOS20.IMG".to_string()); // loops // Unhandled opcode d0 at 7c9e
    //let theDisk=fddController::fddController::new("./diskimages/test8086.bin".to_string());
    let mut theCPU=x86cpu::x86cpu::new();
    let mut theGUI=guiif::guiif::new(0x02,theCPU.cs,theCPU.ip);

    let mut goOut=false;
    while !goOut
    {
        let startTime = Instant::now();
        theGUI.clearScreen();
        theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
        theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
        theGUI.drawMemory(&theVGA,&theMachine,0x9b28,0x0101,80);
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
            theMachine.update();
        }
        else if act==guiif::keyAction::actionRunToRet
        {
            let startTime = Instant::now();
            let mut bytesRead=1;
            let mut dbgstr=String::from("");
            let mut iterations:u64=0;
            while (bytesRead!=0) && (!dbgstr.contains("RET"))
            {
                dbgstr=theCPU.executeOne(&mut theMachine,&mut theVGA,&theDisk,false,&mut bytesRead,&0,&0);
                theMachine.update();

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
            let mut bytesRead=1;

            //while theCPU.ip!=0x6a07
            //while theCPU.ip!=0x6f4d
            while theCPU.ip!=0x153f
            {
                theCPU.executeOne(&mut theMachine,&mut theVGA,&theDisk,false,&mut bytesRead,&0,&0);
                theMachine.update();
            }

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
                theMachine.update();
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
                theCPU.executeOne(&mut theMachine,&mut theVGA,&theDisk,false,&mut bytesRead,&0,&0);
                theMachine.update();
                inum+=1;

                //if theCPU.ip==0x7d74 // dos 3.3 reads disk 2nd time here
                // 0x0070:0x356a - writes nec io.sys banner
                // 0x0070:0x36d5 - dos 3.3 tries to check hard drive (dl=0x80)
                // 0x0070:0x3708 - dos 3.3 drive a: check (dl=00)
                // 0x0070:0x3928 - int 15h
                // 0x0070:0x3f65 - cmp si, 0xffff (sign extended)

                //if (theCPU.cs==0x2f2) && (theCPU.ip==0x1460) // int 21h
                //if (theCPU.cs==0x9dfd) && (theCPU.ip==0xeea)
                //if (theCPU.cs==0xd08) && (theCPU.ip==0x11c8)
                //if (theCPU.cs==0x9b28) && (theCPU.ip==0x31a) // after dos command
                //if (theCPU.cs==0x151e) && (theCPU.ip==0x2db6)
                if false
                {
                    bailOut=true;
                }

                if inum>2000
                {
                    theGUI.clearScreen();
                    theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
                    theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
                    theGUI.drawMemory(&theVGA,&theMachine,0x9b28,0x0101,80);
                    theVGA.fbTobuf32(&mut theGUI);
                    theGUI.updateVideoWindow(&theVGA);

                    if theGUI.checkEscPressed()
                    {
                        bailOut=true;
                    }

                    theGUI.processKeys(&mut theMachine);
                    
                    //thread::sleep(time::Duration::from_millis(4));                    
                    inum=0;
                }
            }
        }
    }
}
