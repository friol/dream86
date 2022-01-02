/* dream86 - 2o22 - v0.4 */

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::process;
use std::{thread, time};
use std::collections::HashMap;

mod vga;
mod machine;
mod x86cpu;
mod guiif;


fn main()
{
    let mut theVGA=vga::vga::new();
    let mut theMachine=machine::machine::new("./programs/pillman.com",0x100000);
    //let mut theMachine=machine::machine::new("./programs/invaders.com",0x100000);
    //let mut theMachine=machine::machine::new("./programs/SIN.com",0x100000);
    let mut theCPU=x86cpu::x86cpu::new();

    let mut theGUI=guiif::guiif::new();

    let mut errStr:String=String::from("");
    let mut goOut=false;
    while !goOut
    {
        theGUI.clearScreen();
        theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU);
        theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions);
        theGUI.drawMemory(&theVGA,0xa000,0xe626,80);
        theGUI.printDebugErr(errStr.clone());
        theVGA.fbTobuf32(&mut theGUI.frameBuffer);
        theGUI.updateVideoWindow();

        //

        let act=theGUI.getKeyAction();
        if act==guiif::keyAction::actionQuit
        {
            goOut=true;
        }
        else if act==guiif::keyAction::actionStep
        {
            let mut bytesRead=0;
            theCPU.executeOne(&mut theMachine,&mut theVGA,false,&mut bytesRead,&0,&0,&mut errStr);
            theMachine.update();
        }
        else if act==guiif::keyAction::actionRunToRet
        {
            let mut bytesRead=1;
            let mut dbgstr=String::from("");
            let mut iterations:u64=0;
            while (bytesRead!=0) && (!dbgstr.contains("RET"))
            {
                let mut errStr:String=String::from("");
                dbgstr=theCPU.executeOne(&mut theMachine,&mut theVGA,false,&mut bytesRead,&0,&0,&mut errStr);
                theMachine.update();

                if (iterations%1000)==0
                {
                    theGUI.clearScreen();
                    theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU);
                    theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions);
                    theVGA.fbTobuf32(&mut theGUI.frameBuffer);
                    theGUI.updateVideoWindow();
                }
                iterations+=1;
            }
        }
        else if act==guiif::keyAction::actionRunToAddr
        {
            let mut bytesRead=1;
            while theCPU.ip!=0x190
            {
                theCPU.executeOne(&mut theMachine,&mut theVGA,false,&mut bytesRead,&0,&0,&mut errStr);
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
            let bpPos:u16=theGUI.getRuntoIp(&mut theCPU,&mut theMachine,&mut theVGA);
            while theCPU.ip!=bpPos
            {
                theCPU.executeOne(&mut theMachine,&mut theVGA,false,&mut bytesRead,&0,&0,&mut errStr);
                theMachine.update();
            }
        }
        else if act==guiif::keyAction::actionRun
        {
            let mut bytesRead=1;
            let mut inum:u64=0;
            let mut bailOut=false;
            while !bailOut
            {
                theCPU.executeOne(&mut theMachine,&mut theVGA,false,&mut bytesRead,&0,&0,&mut errStr);
                theMachine.update();
                inum+=1;

                if inum>3000
                {
                    theGUI.clearScreen();
                    theGUI.drawDebugArea(&mut theMachine,&mut theVGA,&mut theCPU);
                    theGUI.drawRegisters(&theCPU.getRegisters(),&theCPU.flags,&theCPU.totInstructions);
                    theGUI.drawMemory(&theVGA,0xa000,0xe626,80);
                    theGUI.printDebugErr(errStr.clone());
                    theVGA.fbTobuf32(&mut theGUI.frameBuffer);
                    theGUI.updateVideoWindow();

                    if theGUI.checkEscPressed()
                    {
                        bailOut=true;
                    }
                    if theGUI.checkLeftPressed()
                    {
                        theMachine.addKeystroke(0x4b);
                    }
                    if theGUI.checkRightPressed()
                    {
                        theMachine.addKeystroke(0x4d);
                    }
                    if theGUI.checkUpPressed()
                    {
                        theMachine.addKeystroke(0x48);
                    }
                    if theGUI.checkDownPressed()
                    {
                        theMachine.addKeystroke(0x50);
                    }
                    
                    //thread::sleep(time::Duration::from_millis(4));                    
                    inum=0;
                }
            }
        }
    }
}
