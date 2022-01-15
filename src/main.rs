/* dream86 - 2o22 */

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

//use std::process;
//use std::{thread, time};
//use std::rc::Rc;
//use std::cell::RefCell;
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
    //let mut theMachine=machine::machine::new("./programs/aSMtris.com",0x100000,1);
    let mut theMachine=machine::machine::new("./programs/pillman.com",0x100000,0);
    //let mut theMachine=machine::machine::new("./programs/invaders.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/fbird.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/bricks.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/rogue.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/sorryass.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/basic.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/dirojedc.com",0x100000,1);
    //let mut theMachine=machine::machine::new("./programs/CGADOTS.COM",0x100000);
    //let mut theMachine=machine::machine::new("../../testcga.com",0x100000);
    /*let mut theMachine=machine::machine::new("./programs/TROLL.com",0x100000,1);
    theMachine.writeMemory(0xf000,0x116,0x90,&mut theVGA);
    theMachine.writeMemory(0xf000,0x117,0x90,&mut theVGA);
    theMachine.writeMemory(0xf000,0x118,0x90,&mut theVGA);*/
    //let mut theMachine=machine::machine::new("./programs/SIN.com",0x100000,1);

    //let theDisk=fddController::fddController::new("./diskimages/pillman.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/invaders.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/tetros.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/basic.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/Dos3.3.img".to_string()); // ohohoh
    //let theDisk=fddController::fddController::new("./diskimages/dos3.31.microsoft.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/dos5.0.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/Dos6.22.img".to_string());
    //let theDisk=fddController::fddController::new("./diskimages/OLVDOS20.IMG".to_string()); // loops
    let mut theCPU=x86cpu::x86cpu::new();
    let mut theGUI=guiif::guiif::new(0x02,theCPU.cs,theCPU.ip);

    let mut goOut=false;
    while !goOut
    {
        let startTime = Instant::now();
        theGUI.clearScreen();
        theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
        theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
        theGUI.drawMemory(&theVGA,&theMachine,0x0070,0x0532,80);
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
            //while theCPU.ip!=0x201
            // get position of current invader
            while theCPU.ip!=0x25f
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
                // 0x0070:0x3597 - after banner xor dx,dx sp=6fc
                // 0x0070:0x36a4 - sp=6fe
                // 0x0070:0x36d5 - dos 3.3 tries to check hard drive (dl=0x80)
                // 0x0070:0x3708 - dos 3.3 drive a: check (dl=00)
                // 0x0070:0x37f5 - sp=700
                // 0x0070:0x3928 - int 15h
                // 0x0070:0x3996 - 3 calls
                // 0x0070:0x3f65 - cmp si, 0xffff (sign extended)


                //if theCPU.ip==0x7d6e
                //if theCPU.ip==0xdbd
                //if theCPU.ip==0x7dd0
                //if theCPU.ip==0x3506
                //if theCPU.ip==0x3668
                //if theCPU.ip==0x36a4
                //if theCPU.ip==0x36d5
                //if theCPU.ip==0x37f8 // mov ax,0xffff
                //if theCPU.ip==0x39bc
                //if theCPU.ip==0x3f65
                //if theCPU.ip==0x39e2
                if (theCPU.cs==0x9dfd) && (theCPU.ip==0x952)
                {
                    bailOut=true;
                }

                if inum>6000
                {
                    theGUI.clearScreen();
                    theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU,&theDisk);
                    theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions,&startTime);
                    theGUI.drawMemory(&theVGA,&theMachine,0xa000,0xe626,80);
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
