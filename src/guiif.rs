/* gui interface - dream86 */

use std::time::Instant;
use std::collections::HashMap;
use std::io::{stdout, Write};
use crossterm::{ExecutableCommand, QueueableCommand,terminal, cursor, style::{self, Stylize}};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};

extern crate minifb;
use minifb::{Key,  KeyRepeat, Scale, Window, WindowOptions};

use crate::machine::machine;
use crate::fddController::fddController;
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
    pub videoWindow: Window,
    pub videoWinWidth: u32,
    pub videoWinHeight: u32,
    pub videoMode: u8
}

impl guiif
{
    pub fn new(videomode:u8,inCS:u16,inIP:u16) -> Self 
    {
        let mut stdout = stdout();
        stdout.execute(terminal::Clear(terminal::ClearType::All)).ok();

        let vwidth:u32=320;
        let vheight:u32=200;
        let window:Window=Window::new("dream86",vwidth as usize,vheight as usize,WindowOptions {
            scale: Scale::X2,
            ..WindowOptions::default()
        }).unwrap_or_else(|e| { panic!("{}", e); });
        let buffer:Vec<u32>=Vec::new();

        let mut newGUI=guiif 
        {
            dbgcs: inCS, 
            dbgip: inIP, 
            dbgCursorLine: 0, 
            dbgInstrLine: 9, 
            dbgRegline: 1,
            dbgMemoryLine: 30,
            frameBuffer: buffer,
            videoWindow: window,
            videoWinWidth: vwidth,
            videoWinHeight: vheight,
            videoMode: videomode
        };

        newGUI.initVideomode(videomode);
        return newGUI;
    }    

    fn initVideomode(&mut self,videomode:u8)
    {
        let mut vwidth:u32=0;
        let mut vheight:u32=0;
        let mut window:Window;
        let mut buffer:Vec<u32>;

        if videomode==0x13 { vwidth=320; vheight=200; }
        else if videomode==0x01 { vwidth=360; vheight=400; }
        else if videomode==0x02 { vwidth=720; vheight=400; }
        else if videomode==0x04 { vwidth=320; vheight=200; }

        if videomode==0x13 || videomode==0x04
        {
            window=Window::new("dream86",vwidth as usize,vheight as usize,WindowOptions {
                scale: Scale::X2,
                ..WindowOptions::default()
            }).unwrap_or_else(|e| { panic!("{}", e); });    
        }
        else
        {
            window=Window::new("dream86",vwidth as usize,vheight as usize,WindowOptions {
                /*scale: Scale::X2,*/
                ..WindowOptions::default()
            }).unwrap_or_else(|e| { panic!("{}", e); });    
        }

        buffer=Vec::with_capacity((vwidth as usize)*(vheight as usize));
        for _i in 0..(vwidth as usize)*(vheight as usize)
        {
            buffer.push(0);
        }
        window.update_with_buffer(&buffer,vwidth as usize,vheight as usize).unwrap();

        self.videoWinWidth=vwidth;
        self.videoWinHeight=vheight;
        self.frameBuffer=buffer;
        self.videoWindow=window;
        self.videoMode=videomode;
    }

    pub fn updateVideoWindow(&mut self,pvga:&vga)
    {
        // check if videomode changed
        if (*pvga).mode!=self.videoMode.into()
        {
            self.initVideomode((*pvga).mode as u8);
        }

        self.videoWindow.update_with_buffer(&self.frameBuffer,self.videoWinWidth as usize,self.videoWinHeight as usize).unwrap();
    }

    pub fn checkEscPressed(&mut self) -> bool
    {
        return self.videoWindow.is_key_down(Key::Escape);
    }

    pub fn processKeys(&mut self,pmachine:&mut machine)
    {
        if self.checkLShiftPressed()
        {
            pmachine.addKeystroke(0xff);
        }

        self.videoWindow.get_keys_pressed(KeyRepeat::No).iter().for_each(|key| {
            match key {
                Key::A => pmachine.addKeystroke(0x61),
                Key::B => pmachine.addKeystroke(0x62),
                Key::C => pmachine.addKeystroke(0x63),
                Key::D => pmachine.addKeystroke(0x64),
                Key::E => pmachine.addKeystroke(0x65),
                Key::F => pmachine.addKeystroke(0x66),
                Key::G => pmachine.addKeystroke(0x67),
                Key::H => pmachine.addKeystroke(0x68),
                Key::I => pmachine.addKeystroke(0x69),
                Key::J => pmachine.addKeystroke(0x6a),
                Key::K => pmachine.addKeystroke(0x6b),
                Key::L => pmachine.addKeystroke(0x6c),
                Key::M => pmachine.addKeystroke(0x6d),
                Key::N => pmachine.addKeystroke(0x6e),
                Key::O => pmachine.addKeystroke(0x6f),
                Key::P => pmachine.addKeystroke(0x70),
                Key::Q => pmachine.addKeystroke(0x71),
                Key::R => pmachine.addKeystroke(0x72),
                Key::S => pmachine.addKeystroke(0x73),
                Key::T => pmachine.addKeystroke(0x74),
                Key::U => pmachine.addKeystroke(0x75),
                Key::V => pmachine.addKeystroke(0x76),
                Key::W => pmachine.addKeystroke(0x77),
                Key::X => pmachine.addKeystroke(0x78),
                Key::Y => pmachine.addKeystroke(0x79),
                Key::Z => pmachine.addKeystroke(0x7a),
                Key::Key0 => pmachine.addKeystroke(0x30),
                Key::Key1 => pmachine.addKeystroke(0x31),
                Key::Key2 => pmachine.addKeystroke(0x32),
                Key::Key3 => pmachine.addKeystroke(0x33),
                Key::Key4 => pmachine.addKeystroke(0x34),
                Key::Key5 => pmachine.addKeystroke(0x35),
                Key::Key6 => pmachine.addKeystroke(0x36),
                Key::Key7 => pmachine.addKeystroke(0x37),
                Key::Key8 => pmachine.addKeystroke(0x38),
                Key::Key9 => pmachine.addKeystroke(0x39),
                Key::F1 => pmachine.addKeystroke(0x3b),
                Key::Space => pmachine.addKeystroke(0x20),
                Key::Period => pmachine.addKeystroke(46),
                Key::NumPadAsterisk => pmachine.addKeystroke(42),
                Key::Backspace => pmachine.addKeystroke(0x08),
                Key::NumPadPlus => pmachine.addKeystroke(0x2b),
                Key::Minus => pmachine.addKeystroke(0x2d),
                Key::Enter => pmachine.addKeystroke(0x0d),
                Key::Up => pmachine.addKeystroke(0x48),
                Key::Down => pmachine.addKeystroke(0x50),
                Key::Left => pmachine.addKeystroke(0x4b),
                Key::Right => pmachine.addKeystroke(0x4d),
                Key::NumPadSlash => pmachine.addKeystroke(47),
                _ => return,
            }
        });
    }

    pub fn checkLShiftPressed(&mut self) -> bool
    {
        return self.videoWindow.is_key_down(Key::LeftShift);
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

    pub fn getRuntoIp(&self,theCPU:&mut x86cpu,theMachine:&mut machine,theVGA:&mut vga,theDisk:&fddController) -> u16
    {
        const NUM_INSTRUCTIONS:u32=15;
        let mut listOfInstructions:Vec<String>=Vec::new();
        let mut bytesRead:u8=1;
        let mut tempIp=theCPU.ip;

        let mut idx=0;
        while (bytesRead!=0) && (idx<NUM_INSTRUCTIONS)
        {
            let instr:String=theCPU.executeOne(theMachine,theVGA,theDisk,true,&mut bytesRead,&self.dbgcs,&tempIp);
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

    pub fn drawDebugArea(&mut self,theMachine:&mut machine,theVGA:&mut vga,theCPU:&mut x86cpu,theDisk:&fddController)
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

        const NUM_INSTRUCTIONS:u32=20;
        let mut listOfInstructions:Vec<String>=Vec::new();
        let mut bytesRead:u8=1;
        let mut tempIp=theCPU.ip;
        self.dbgcs=theCPU.cs;

        let mut idx=0;
        while (bytesRead!=0) && (idx<NUM_INSTRUCTIONS)
        {
            let instr:String=theCPU.executeOne(theMachine,theVGA,theDisk,true,&mut bytesRead,&self.dbgcs,&tempIp);
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

    pub fn drawRegisters(&self,regsMap:&HashMap<String,u16>,flags:&u16,totins:&u64,startTime:&Instant)
    {
        let mut elapsed = startTime.elapsed().as_secs();
        if elapsed==0 { elapsed=1; }

        let mut strReg:String=String::from("");
        strReg.push_str(&format!(
            "AX:{:04x} BX:{:04x} CX:{:04x} DX:{:04x} SI:{:04x} DI:{:04x} BP:{:04x} SP:{:04x}",
            regsMap["AX"],regsMap["BX"],regsMap["CX"],regsMap["DX"],regsMap["SI"],regsMap["DI"],regsMap["BP"],regsMap["SP"]
        ));

        let mut strReg2:String=String::from("");
        strReg2.push_str(&format!(
            "IP:{:04x} CS:{:04x} DS:{:04x} ES:{:04x} SS:{:04x} Instructions:{} IPS:{}",
            regsMap["IP"],regsMap["CS"],regsMap["DS"],regsMap["ES"],regsMap["SS"],totins,totins/elapsed
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

        let NUM_BYTES=17;
        for idx in 0..numBytes
        {
            if ((idx*4)%(NUM_BYTES*4))==0
            {
                // print address
                stdout.queue(cursor::MoveTo(0,self.dbgMemoryLine+1+(idx/NUM_BYTES))).ok();
                let ss:String=format!("{:04x}:{:04x}",startSegment,varOffset as u16);
                stdout.queue(style::PrintStyledContent(ss.to_string().white())).ok();
            }
            else
            {
                stdout.queue(cursor::MoveTo(9+((idx*4)%(NUM_BYTES*4)),self.dbgMemoryLine+1+(idx/NUM_BYTES))).ok();
                let ss:String=format!(" {:02x}",pmachine.readMemory(startSegment,varOffset as u16,pvga));
                stdout.queue(style::PrintStyledContent(ss.to_string().white())).ok();
                varOffset+=1;
            }
        }
    }
}
