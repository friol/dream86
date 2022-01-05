/* gui interface - dream86 */


use std::collections::HashMap;
use std::io::{stdout, Write};
use crossterm::{ExecutableCommand, QueueableCommand,terminal, cursor, style::{self, Stylize}};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};

extern crate minifb;
use minifb::{Key, Scale, Window, WindowOptions};

use crate::machine::machine;
use crate::x86cpu::x86cpu;
use crate::vga::vga;

#[derive(PartialEq)]
pub enum keyAction 
{
    actionQuit,
    actionStep,
    actionRunToRet,
    actionRunToAddr,
    actionRunToCursor,
    actionRun,
    actionNone,
    actionIncDebugCursor,
    actionDecDebugCursor,
}

pub struct guiif
{
    pub dbgcs: u16,
    pub dbgip: u16,
    pub dbgCursorLine: u16,
    pub dbgInstrLine: u16,
    pub dbgRegline: u16,
    pub dbgMemoryLine: u16,
    pub frameBuffer: Vec<u32>,
    pub videoWindow: Window
}

impl guiif
{
    pub fn new() -> Self 
    {
        let mut stdout = stdout();
        stdout.execute(terminal::Clear(terminal::ClearType::All)).ok();

        let mut window = Window::new("dream86 v0.0.4",320,200,WindowOptions {
            scale: Scale::X2,
            ..WindowOptions::default()
        }).unwrap_or_else(|e| { panic!("{}", e); });    

        //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));        

        let mut buffer: Vec<u32> = vec![0; 320*200];

        for i in buffer.iter_mut() 
        {
            *i = 0;
        }
        window.update_with_buffer(&buffer,320,200).unwrap();
        
        guiif 
        {
            dbgcs: 0xf000, 
            dbgip: 0x100, 
            dbgCursorLine: 0, 
            dbgInstrLine: 9, 
            dbgRegline: 1,
            dbgMemoryLine: 25,
            frameBuffer: buffer,
            videoWindow: window
        }
    }    

    pub fn updateVideoWindow(&mut self)
    {
        self.videoWindow.update_with_buffer(&self.frameBuffer,320,200).unwrap();
    }

    pub fn checkEscPressed(&mut self) -> bool
    {
        return self.videoWindow.is_key_down(Key::Escape);
    }

    pub fn checkLeftPressed(&mut self) -> bool
    {
        return self.videoWindow.is_key_down(Key::Left);
    }

    pub fn checkRightPressed(&mut self) -> bool
    {
        return self.videoWindow.is_key_down(Key::Right);
    }

    pub fn checkUpPressed(&mut self) -> bool
    {
        return self.videoWindow.is_key_down(Key::Up);
    }

    pub fn checkDownPressed(&mut self) -> bool
    {
        return self.videoWindow.is_key_down(Key::Down);
    }

    pub fn getKeyAction(&self) -> keyAction
    {
        match read().unwrap() {
            Event::Key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::CONTROL }) => return keyAction::actionQuit,
            Event::Key(KeyEvent { code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL }) => return keyAction::actionStep,
            Event::Key(KeyEvent { code: KeyCode::Char('e'), modifiers: KeyModifiers::CONTROL }) => return keyAction::actionRunToRet,
            Event::Key(KeyEvent { code: KeyCode::Char('d'), modifiers: KeyModifiers::CONTROL }) => return keyAction::actionRunToAddr,
            Event::Key(KeyEvent { code: KeyCode::Char('r'), modifiers: KeyModifiers::CONTROL }) => return keyAction::actionRun,
            Event::Key(KeyEvent { code: KeyCode::Char('t'), modifiers: KeyModifiers::CONTROL }) => return keyAction::actionRunToCursor,
            Event::Key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL }) => return keyAction::actionIncDebugCursor,
            Event::Key(KeyEvent { code: KeyCode::Char('o'), modifiers: KeyModifiers::CONTROL }) => return keyAction::actionDecDebugCursor,
            _ => (),
        }        

        return keyAction::actionNone;
    }

    pub fn incDebugCursor(&mut self)
    {
        self.dbgCursorLine+=1;
    }

    pub fn decDebugCursor(&mut self)
    {
        if self.dbgCursorLine>0
        {
            self.dbgCursorLine-=1;
        }
    }

    pub fn getRuntoIp(&self,theCPU:&mut x86cpu,theMachine:&mut machine,theVGA:&mut vga) -> u16
    {
        const NUM_INSTRUCTIONS:u32=15;
        let mut listOfInstructions:Vec<String>=Vec::new();
        let mut bytesRead:u8=1;
        let mut tempIp=theCPU.ip;

        let mut idx=0;
        while (bytesRead!=0) && (idx<NUM_INSTRUCTIONS)
        {
            let mut errStr:String=String::from("");
            let instr:String=theCPU.executeOne(theMachine,theVGA,true,&mut bytesRead,&self.dbgcs,&tempIp,&mut errStr);
            if bytesRead!=0
            {
                listOfInstructions.push(instr);
                tempIp+=bytesRead as u16;
            }
            idx+=1;
        }

        //

        let sin:String=listOfInstructions[self.dbgCursorLine as usize].clone();
        let addr=&sin[5..9];

        return u16::from_str_radix(addr,16).unwrap();
    }

    pub fn clearScreen(&self)
    {
        let mut stdout = stdout();
        stdout.execute(terminal::Clear(terminal::ClearType::All)).ok();
        stdout.queue(cursor::MoveTo(0,0)).ok();
    }

    pub fn drawInstructions(&self,instrs:&Vec<String>)
    {
        let mut ypos=self.dbgInstrLine;
        let mut stdout = stdout();
        for idx in 0..instrs.len()
        {
            stdout.queue(cursor::MoveTo(5,ypos)).ok();
            if idx==0 { stdout.queue(style::PrintStyledContent(instrs[idx].clone().white().negative())).ok(); }
            else { stdout.queue(style::PrintStyledContent(instrs[idx].clone().white())).ok(); }
            ypos+=1;
        }
        stdout.flush().ok();
    }

    pub fn printDebugErr(&self,err:String)
    {
        let mut stdout = stdout();
        stdout.queue(cursor::MoveTo(0,0)).ok();
        stdout.queue(style::PrintStyledContent(err.clone().white())).ok();
        stdout.flush().ok();
    }

    pub fn drawDebugArea(&mut self,theMachine:&mut machine,theVGA:&mut vga,theCPU:&mut x86cpu)
    {
        // stack

        let mut stdout = stdout();

        let mut ii=0;
        for el in &theMachine.stackey
        {
            stdout.queue(cursor::MoveTo(80,ii)).ok();
            let ss:String=format!("{:02x}",el);
            stdout.queue(style::PrintStyledContent(ss.to_string().white())).ok();
            ii+=1;
        }

        // instrs

        const NUM_INSTRUCTIONS:u32=15;
        let mut listOfInstructions:Vec<String>=Vec::new();
        let mut bytesRead:u8=1;
        let mut tempIp=theCPU.ip;

        let mut idx=0;
        while (bytesRead!=0) && (idx<NUM_INSTRUCTIONS)
        {
            let mut errStr:String=String::from("");
            let instr:String=theCPU.executeOne(theMachine,theVGA,true,&mut bytesRead,&self.dbgcs,&tempIp,&mut errStr);
            if bytesRead!=0
            {
                listOfInstructions.push(instr);
                tempIp+=bytesRead as u16;
            }

            idx+=1;
        }

        // draw pointer
        stdout.queue(cursor::MoveTo(0,self.dbgInstrLine+self.dbgCursorLine)).ok();
        stdout.queue(style::PrintStyledContent("==> ".white())).ok();

        // instrs.
        self.drawInstructions(&listOfInstructions);
    }

    pub fn drawRegisters(&self,regsMap:&HashMap<String,u16>,flags:&u16,totins:&u64)
    {
        let mut strReg:String=String::from("");
        strReg.push_str(&format!(
            "AX:{:04x} BX:{:04x} CX:{:04x} DX:{:04x} SI:{:04x} DI:{:04x} BP:{:04x} SP:{:04x}",
            regsMap["AX"],regsMap["BX"],regsMap["CX"],regsMap["DX"],regsMap["SI"],regsMap["DI"],regsMap["BP"],regsMap["SP"]
        ));

        let mut strReg2:String=String::from("");
        strReg2.push_str(&format!(
            "IP:{:04x} CS:{:04x} DS:{:04x} ES:{:04x} SS:{:04x} Instructions:{}",
            regsMap["IP"],regsMap["CS"],regsMap["DS"],regsMap["ES"],regsMap["SS"],totins
        ));

        let mut stdout = stdout();

        stdout.queue(cursor::MoveTo(0,self.dbgRegline)).ok();
        stdout.queue(style::PrintStyledContent("Registers                                                      ".blue().negative())).ok();
        stdout.queue(cursor::MoveTo(0,self.dbgRegline+1)).ok();
        stdout.queue(style::PrintStyledContent(strReg.white())).ok();
        stdout.queue(cursor::MoveTo(0,self.dbgRegline+2)).ok();
        stdout.queue(style::PrintStyledContent(strReg2.white())).ok();

        stdout.queue(cursor::MoveTo(0,self.dbgRegline+4)).ok();
        stdout.queue(style::PrintStyledContent("Flags                                                          ".blue().negative())).ok();
        stdout.queue(cursor::MoveTo(0,self.dbgRegline+5)).ok();
        stdout.queue(style::PrintStyledContent("XXXXODITSZXAXPXC".white())).ok();
        stdout.queue(cursor::MoveTo(0,self.dbgRegline+6)).ok();
        let mut flagsReg:String=String::from("");
        flagsReg.push_str(&format!("{:016b}",flags));
        stdout.queue(style::PrintStyledContent(flagsReg.white())).ok();

        stdout.queue(cursor::MoveTo(0,24)).ok();
        stdout.flush().ok();
    }

    pub fn drawMemory(&self,pvga:&vga,pmachine:&machine,startSegment:u16,startOffset:u16,numBytes:u16)
    {
        let mut varOffset:i64=startOffset.into();
        let mut stdout = stdout();

        stdout.queue(cursor::MoveTo(0,self.dbgMemoryLine)).ok();
        stdout.queue(style::PrintStyledContent("Memory                                                          ".blue().negative())).ok();

        for idx in 0..numBytes
        {
            if ((idx*4)%(20*4))==0
            {
                // print address
                stdout.queue(cursor::MoveTo(0,self.dbgMemoryLine+1+(idx/20))).ok();
                let ss:String=format!("{:04x}:{:04x}",startSegment,varOffset as u16);
                stdout.queue(style::PrintStyledContent(ss.to_string().white())).ok();
            }
            else
            {
                stdout.queue(cursor::MoveTo(9+((idx*4)%(20*4)),self.dbgMemoryLine+1+(idx/20))).ok();
                let ss:String=format!(" {:02x}",pmachine.readMemory(startSegment,varOffset as u16,pvga));
                stdout.queue(style::PrintStyledContent(ss.to_string().white())).ok();
                varOffset+=1;
            }

        }

    }
}
