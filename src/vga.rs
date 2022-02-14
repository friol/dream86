/* the VGA - dream86 */

use std::process;

use crate::guiif::guiif;

pub struct vga
{
    pub mode: u16,
    pub framebuffer: Vec<u8>,
    pub cgaFramebuffer: Vec<u8>,
    pub font9x16data:Vec<Vec<u32>>,
    pub font9x16width:u32,
    pub font9x16height:u32,
    pub font8x8data:Vec<Vec<u32>>,
    pub font8x8width:u32,
    pub font8x8height:u32,
    pub cursorx:usize,
    pub cursory:usize,
    pub vgaPalette: Vec<u32>,
    pub vgaPaletteCurColor: u8,
    pub vgaPaletteIndexRGB: u8,
    pub egaRegister3ceSelected: u8,
    pub egaRegister3cfValues: Vec<u8>,
    pub egaRegister3c4Selected: u8,
    pub egaRegister3c5Values: Vec<u8>,
    pub egaRegister3b4Selected: u8,
    pub egaRegister3b5Values: Vec<u8>,
    pub egaDataLatch: Vec<u8>,
    pub scanlineCounter: u32,
    pub cgaPaletteSelected: u8
}

impl vga
{
    pub fn setVideomode(&mut self,videomodeNum:u16)
    {
        self.cursorx=0;
        self.cursory=0;

        if videomodeNum==0x13
        {
            // VGA 320x200
            self.mode=0x13;
            for idx in 0..self.framebuffer.len()
            {
                self.framebuffer[idx]=0;
            }
        }
        else if (videomodeNum==0x00) || (videomodeNum==0x01)
        {
            // 40x25 textmode 9x16
            self.mode=videomodeNum;
        }
        else if (videomodeNum==0x02) || (videomodeNum==0x03)
        {
            // 80x25 textmode 9x16
            self.mode=videomodeNum;
        }
        else if (videomodeNum==0x04) || (videomodeNum==0x05)
        {
            // CGA 320x200 4 colours            
            self.mode=videomodeNum;
            for idx in 0..self.cgaFramebuffer.len()
            {
                // CGA init pattern (?)
                if idx<(80*400)
                {
                    self.cgaFramebuffer[idx]=0;
                }
                else
                {
                    if (idx%2)==0 { self.cgaFramebuffer[idx]=0x07; }
                    else { self.cgaFramebuffer[idx]=0x20; }
                }
            }
        }
        else if videomodeNum==0x06
        {
            // 640x200 2 colours
            self.mode=0x06;
            for idx in 0..self.cgaFramebuffer.len()
            {
                self.cgaFramebuffer[idx]=0;
            }
        }
        else if videomodeNum==0x0d
        {
            // EGA 320x200x16
            self.mode=0x0d;
            for idx in 0..self.framebuffer.len()
            {
                self.framebuffer[idx]=0;
            }
        }
        else if videomodeNum==0x10
        {
            // EGA 640x350x16
            self.mode=0x10;
            for idx in 0..self.framebuffer.len()
            {
                self.framebuffer[idx]=0;
            }
        }
        else
        {
            println!("Bailing out: vga::cannot switch to mode {:02x}",videomodeNum);
            process::exit(0x0100);
        }
    }

    pub fn getNumberOfColumns(&self) -> u16
    {
        if (self.mode==0x00) || (self.mode==0x01)
        {
            return 40;
        }

        return 80;
    }

    pub fn getCursorPosition(&self) -> (usize,usize)
    {
        return (self.cursorx,self.cursory);
    }

    pub fn setCursorPosition(&mut self,px:u16,py:u16)
    {
        self.cursorx=px as usize;
        self.cursory=py as usize;
    }

    pub fn putpixel(&mut self,color:u8,column:u16,row:u16)
    {
        if self.mode==0x04 || self.mode==0x05
        {
            let mut addr=(row/640)+(column/4);
            if (row%2)>0 { addr+=0x2000; }
            let bit2=(column%4)*2;
            self.cgaFramebuffer[addr as usize]=self.cgaFramebuffer[addr as usize]&((color&0x3)<<(6-bit2));
        }
        else
        {
            println!("Bailing out: vga::putpixel for unhandled mode {:02x}",self.mode);
            process::exit(0x0100);
        }
    }

    //

    pub fn readMemory(&mut self,addr:i64) -> u8
    {
        if (addr>=0xa0000) && (addr<=(0xaffff))
        {
            let egaOffs=addr&0x1fff;
            for plane in 0..4
            {
                self.egaDataLatch[plane]=self.framebuffer[(egaOffs+((plane as i64)*0x2000)) as usize];
            }
            return self.framebuffer[(addr-0xa0000) as usize];
        }
        else if (addr>=0xb8000) && (addr<=0xbffff)
        {
            return self.cgaFramebuffer[(addr-0xb8000) as usize];
        }

        return 0;
    }

    pub fn readMemory16(&mut self,addr:i64) -> u16
    {
        let lobyte=self.readMemory(addr) as u16;
        let hibyte=self.readMemory(addr+1) as u16;
        return lobyte|(hibyte<<8);
    }

    pub fn writeMemory(&mut self,addr:i64,val:u8)
    {
        if (addr>=0xa0000) && (addr<=0xaffff)
        {
            // if ega videomode is 0x0d|0x10 and write mode is 0
            if ((self.mode==0x0d)||(self.mode==0x10)) && (self.egaRegister3cfValues[5]==0)
            {
                let mut realVal=val;
                let bitplane=self.egaRegister3c5Values[2];
                let reg3_ega3cf=self.egaRegister3cfValues[3];
                let reg8_ega3cf=self.egaRegister3cfValues[8];

                let latchNum=((addr-0xa0000)>>13)&0x03;

                // data rotate reg bits 3&4: 
                if (reg3_ega3cf&0x18)==0x18
                {
                    // 1,1 -> data is XORed with latch data
                    realVal=realVal^self.egaDataLatch[latchNum as usize];
                }
                else if (reg3_ega3cf&0x18)==0x8
                {
                    // 0,1 -> data is AND'd with latch data
                    realVal=realVal&self.egaDataLatch[latchNum as usize];
                }
                else if (reg3_ega3cf&0x18)==0x10
                {
                    // 1,0 -> data is OR'd with latch data
                    realVal=realVal|self.egaDataLatch[latchNum as usize];
                }
                else if reg3_ega3cf!=0
                {
                    println!("Bailing out: vga::other function {:02x}",reg3_ega3cf);
                    process::exit(0x0100);
                }

                realVal = (realVal & reg8_ega3cf) | (self.egaDataLatch[latchNum as usize] & (!reg8_ega3cf));

                if (bitplane&0x01)>0
                {
                    self.framebuffer[((addr-0xa0000)&0xffff) as usize]=realVal;
                }
                if (bitplane&0x02)>0
                {
                    self.framebuffer[((addr-0xa0000+0x2000)&0xffff) as usize]=realVal;
                }
                if (bitplane&0x04)>0
                {
                    self.framebuffer[((addr-0xa0000+0x4000)&0xffff) as usize]=realVal;
                }
                if (bitplane&0x08)>0
                {
                    self.framebuffer[((addr-0xa0000+0x6000)&0xffff) as usize]=realVal;
                }
            }
            else if ((self.mode==0x0d)||(self.mode==0x10)) && (self.egaRegister3cfValues[5]==2)
            {
                let bitplane=0xff;//self.egaRegister3c5Values[2];
                let reg8_ega3cf=self.egaRegister3cfValues[8];

                let vala = if (val & 1)>0 { 0xff } else { 0 };
                let valb = if (val & 2)>0 { 0xff } else { 0 };
                let valc = if (val & 4)>0 { 0xff } else { 0 };
                let vald = if (val & 8)>0 { 0xff } else { 0 };

                if (bitplane&0x01)>0
                {
                    self.framebuffer[((addr-0xa0000)&0xffff) as usize]=(vala & reg8_ega3cf) | (self.egaDataLatch[0] & (!reg8_ega3cf));
                }
                if (bitplane&0x02)>0
                {
                    self.framebuffer[((addr-0xa0000+0x2000)&0xffff) as usize]=(valb & reg8_ega3cf) | (self.egaDataLatch[1] & (!reg8_ega3cf));
                }
                if (bitplane&0x04)>0
                {
                    self.framebuffer[((addr-0xa0000+0x4000)&0xffff) as usize]=(valc & reg8_ega3cf) | (self.egaDataLatch[2] & (!reg8_ega3cf));
                }
                if (bitplane&0x08)>0
                {
                    self.framebuffer[((addr-0xa0000+0x6000)&0xffff) as usize]=(vald & reg8_ega3cf) | (self.egaDataLatch[3] & (!reg8_ega3cf));
                }
            }
            else
            {
                self.framebuffer[(addr-0xa0000) as usize]=val;
            }
        }
        else if (addr>=0xb8000) && (addr<=0xbffff)
        {
            self.cgaFramebuffer[(addr-0xb8000) as usize]=val;
        }
    }

    pub fn writeMemory16(&mut self,addr:i64,val:u16)
    {
        self.writeMemory(addr,(val&0xff) as u8);
        self.writeMemory(addr+1,((val>>8)&0xff) as u8);
    }

    pub fn clrScreenMode2(&mut self)
    {
        let mut pos=0;
        for _c in 0..80*2*25
        {
            self.cgaFramebuffer[pos]=0;
            pos+=1;
        }
    }

    fn handleScrollMode2(&mut self)
    {
        let mut pos=0;
        let columns=80;
        if self.cursory==25
        {
            for _row in 0..24
            {
                for _c in 0..columns*2
                {
                    self.cgaFramebuffer[pos]=self.cgaFramebuffer[pos+(columns*2)];
                    pos+=1;
                }
            }            

            for _c in 0..columns*2
            {
                self.cgaFramebuffer[pos]=0;
                pos+=1;
            }

            self.cursory-=1;
        }
    }

    pub fn readCharAttributeAtCursorPos(&self) -> u16
    {
        let numColumns=80;
        let ch:u16=self.cgaFramebuffer[((self.cursorx)*2)+(self.cursory*numColumns*2)] as u16;
        let attr:u16=self.cgaFramebuffer[((self.cursorx)*2)+(self.cursory*numColumns*2)+1] as u16;
        return ch|(attr<<8);
    }

    pub fn writeCharsWithAttribute(&mut self,ochar:u16,bgcol:u16,attrib:u16,nchars:u16)
    {
        // if in textmode
        if (self.mode==0) || (self.mode==1)
        {
            let numColumns=40;
            for _i in 0..nchars
            {
                self.cgaFramebuffer[((self.cursorx+_i as usize)*2)+(self.cursory*numColumns*2)]=ochar as u8;
                self.cgaFramebuffer[((self.cursorx+_i as usize)*2)+(self.cursory*numColumns*2)+1]=attrib as u8;
            }
        }
        else if (self.mode==2) || (self.mode==3)
        {
            let numColumns=80;
            for _i in 0..nchars
            {
                self.cgaFramebuffer[((self.cursorx+_i as usize)*2)+(self.cursory*numColumns*2)]=ochar as u8;
                self.cgaFramebuffer[((self.cursorx+_i as usize)*2)+(self.cursory*numColumns*2)+1]=attrib as u8;
            }
        }
        else if (self.mode==4) || (self.mode==5) || (self.mode==0x0d)
        {
            // CGA 320x200 modes
            // EGA 320x200 mode
            let mut tempFb:Vec<u32>=Vec::new();

            self.drawCharOnScreen(&mut tempFb,
                8,8,
                32,
                (ochar&0x7f) as u32,
                self.cursory as u32,
                self.cursorx as u32,
                320,
                attrib as u32,bgcol as u32);
        }
    }

    pub fn outputCharToStdout(&mut self,ochar:u8)
    {
        // if in textmode
        if (self.mode==0) || (self.mode==1) || (self.mode==2) || (self.mode==3)
        {
            let charCol=7;
            let mut numColumns=80;
            if (self.mode==0) || (self.mode==1) 
            {
                numColumns=40;
            }

            if ochar==13
            {
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)]=0;
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)+1]=0x0;

                self.cursorx=0;
                self.cursory+=1;
                self.handleScrollMode2();
            }
            else if ochar==10
            {
            }
            else if ochar==8
            {
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)]=0;
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)+1]=0x0;
                self.cursorx-=1;
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)]=22;
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)+1]=charCol;
            }
            else
            {
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)]=ochar;
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)+1]=charCol;
                self.cursorx+=1;
                if self.cursorx==numColumns
                {
                    self.cursorx=0;
                    self.cursory+=1;
                    self.handleScrollMode2();
                }

                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)]=22;
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*numColumns*2)+1]=charCol;
            }
        }
    }

    fn putCGA320x200pixel(&mut self,curcol:u8,pixelx:u32,pixely:u32)
    {
        if pixelx>=320 { return };
        if pixely>=200 { return };

        let mut adder=0;
        if (pixely%2)>0 { adder=0x2000; }

        let curbyte=(pixelx/4)%80;
        let cur2bits=3-(pixelx%4);
        let curline=pixely/2;

        let maskArr=[0xfc,0xf3,0xcf,0x3f];
        self.cgaFramebuffer[(adder+curbyte+(curline*80)) as usize]&=maskArr[cur2bits as usize];
        self.cgaFramebuffer[(adder+curbyte+(curline*80)) as usize]|=(curcol&0x03)<<(cur2bits*2);
    }

    fn putEGA320x200pixel(&mut self,curcol:u8,pixelx:u32,pixely:u32)
    {
        if pixelx>=320 { return };
        if pixely>=200 { return };

        let curbyte=(pixelx/8)%40;
        let curbit=7-(pixelx%8);
        let curline=pixely;

        self.framebuffer[(curbyte+(curline*40)) as usize]&=!(1<<curbit);
        self.framebuffer[(curbyte+(curline*40)) as usize]|=(curcol&0x1)<<curbit;

        self.framebuffer[(0x2000+curbyte+(curline*40)) as usize]&=!(1<<curbit);
        self.framebuffer[(0x2000+curbyte+(curline*40)) as usize]|=((curcol&0x2)>>1)<<curbit;

        self.framebuffer[(0x4000+curbyte+(curline*40)) as usize]&=!(1<<curbit);
        self.framebuffer[(0x4000+curbyte+(curline*40)) as usize]|=((curcol&0x4)>>2)<<curbit;

        self.framebuffer[(0x6000+curbyte+(curline*40)) as usize]&=!(1<<curbit);
        self.framebuffer[(0x6000+curbyte+(curline*40)) as usize]|=((curcol&0x8)>>3)<<curbit;
    }

    fn drawCharOnScreen(&mut self,
        vecDest:&mut Vec<u32>,
        charDimX:u32,
        charDimY:u32,
        numCharsPerRow:u32,
        charNum:u32,
        row:u32,col:u32,
        scrInc:u32,
        fgCol:u32,bgCol:u32)
    {
        let mut srcx:u32=(charNum%numCharsPerRow)*charDimX;
        let mut srcy:u32=(charNum/numCharsPerRow)*charDimY;

        let mut dstx:u32=col*charDimX;
        let mut dsty:u32=row*charDimY;

        if (self.mode==0x02) || (self.mode==0x03)
        {
            let mut destPos=dstx+(dsty*scrInc);
            for _y in 0..charDimY
            {
                for _x in 0..charDimX
                {
                    let curVal=self.font9x16data[srcy as usize][srcx as usize];
                    let curCol=if curVal==0 { bgCol } else { fgCol };
                    vecDest[destPos as usize]=curCol;
                    destPos+=1;
                    srcx+=1;
                }
                destPos+=scrInc-charDimX;
                srcy+=1;
                srcx=(charNum%numCharsPerRow)*charDimX;
            }
        }
        else if (self.mode==0x04) || (self.mode==0x05)
        {
            for _y in 0..charDimY
            {
                for _x in 0..charDimX
                {
                    let curVal=self.font8x8data[srcy as usize][srcx as usize];
                    let curCol=if curVal==0 { bgCol } else { fgCol };
                    self.putCGA320x200pixel(curCol as u8,dstx,dsty);
                    dstx+=1;
                    srcx+=1;
                }
                dstx=col*charDimX;
                dsty+=1;

                srcy+=1;
                srcx=(charNum%numCharsPerRow)*charDimX;
            }

        }
        else if self.mode==0x0d
        {
            for _y in 0..charDimY
            {
                for _x in 0..charDimX
                {
                    let curVal=self.font8x8data[srcy as usize][srcx as usize];
                    if curVal!=0
                    {
                        self.putEGA320x200pixel(14 as u8,dstx,dsty);
                    }
                    dstx+=1;
                    srcx+=1;
                }
                dstx=col*charDimX;
                dsty+=1;

                srcy+=1;
                srcx=(charNum%numCharsPerRow)*charDimX;
            }
        }
    }

    // VGA palette index set
    pub fn write0x3c8(&mut self,val: u8)
    {
        self.vgaPaletteCurColor=val;
    }

    // VGA rgb value set
    pub fn write0x3c9(&mut self,val: u8)
    {
        if self.vgaPaletteIndexRGB==0
        {
            self.vgaPalette[self.vgaPaletteCurColor as usize]=(self.vgaPalette[self.vgaPaletteCurColor as usize]&0xffff00)|((val<<2) as u32);
        }
        else if self.vgaPaletteIndexRGB==1
        {
            self.vgaPalette[self.vgaPaletteCurColor as usize]=(self.vgaPalette[self.vgaPaletteCurColor as usize]&0xff00ff)|(((val<<2) as u32)<<8);
        }
        else
        {
            self.vgaPalette[self.vgaPaletteCurColor as usize]=(self.vgaPalette[self.vgaPaletteCurColor as usize]&0x00ffff)|(((val<<2) as u32)<<16);
        }

        self.vgaPaletteIndexRGB+=1;
        if self.vgaPaletteIndexRGB==3
        {
            self.vgaPaletteIndexRGB=0;
            self.vgaPaletteCurColor+=1;
        }
    }

    pub fn write0x3ce(&mut self,val: u8)
    {
        /*
            reg             use
            ----------------------------------------------------------------------
            0               Set/reset <what I don't know>
            1               enable set/reset
            2               Color compare
            3               Data rotate value
            4               Memory plane to read
            5               Mode register 1
            6               Mode register 2
            7               Ignore color compare
            8               Bit mask for plane change        
        */
        self.egaRegister3ceSelected=val;
    }

    pub fn write0x3cf(&mut self,val: u8)
    {
        self.egaRegister3cfValues[self.egaRegister3ceSelected as usize]=val;
    }

    pub fn write0x3c4(&mut self,val: u8)
    {
        self.egaRegister3c4Selected=val;
    }

    pub fn write0x3c5(&mut self,val: u8)
    {
        self.egaRegister3c5Values[self.egaRegister3c4Selected as usize]=val;
    }

    pub fn write0x3d9(&mut self,val:u8)
    {
        /*
            |7|6|5|4|3|2|1|0|  3D9 Color Select Register (3B9 not used)
            | | | | | `-------- RGB for background
            | | | | `--------- intensity
            | | | `---------- unused
            | | `----------- 1 = palette 1, 0=palette 0 (see below)
            `-------------- unused

            Palette 0 = green, red, brown
            Palette 1 = cyan, magenta, white
        */        

        if (val&0x20)>0
        {
            self.cgaPaletteSelected=1;
        }
        else
        {
            self.cgaPaletteSelected=0;
        }
    }

    pub fn read0x3da(&self) -> u8
    {
        // CGA status register	EGA/VGA: input status 1 register
        /*
            bit 7-4     not used
            bit 3 = 1   in vertical retrace
            bit 2 = 1   light pen switch is off
            bit 1 = 1   positive edge from light pen has set trigger
            bit 0 = 0   do not use memory
            = 1   memory access without interfering with display
        */          

        let mut retval:u8=0;
        if self.scanlineCounter>4000
        {
            // in vertical retrace
            retval|=0x09;
        }

        return retval;
    }

    pub fn write0x3b4(&mut self,val:u8)
    {
        self.egaRegister3b4Selected=val;
    }

    pub fn write0x3b5(&mut self,val:u8)
    {
        let mut newVal=val;
        if (self.egaRegister3b4Selected & 0x20)>0 { return; }

        if (self.egaRegister3b4Selected < 7) && ((self.egaRegister3b5Values[0x11] & 0x80)>0) { return; }

        if (self.egaRegister3b4Selected == 7) && ((self.egaRegister3b5Values[0x11] & 0x80)>0)
        {
            newVal = (self.egaRegister3b5Values[7] & !0x10) | (val & 0x10);        
        }

        self.egaRegister3b5Values[self.egaRegister3b4Selected as usize]=newVal;
    }

    pub fn read0x3b5(&self) -> u8
    {
        return self.egaRegister3b5Values[self.egaRegister3b4Selected as usize];
    }

    pub fn update(&mut self)
    {
        self.scanlineCounter+=1;
        self.scanlineCounter%=4166; // 6hz, 250.000ips
    }

    //
    // blitting
    //

    pub fn fbTobuf32(&mut self,gui:&mut guiif)
    {
        if self.mode!=gui.videoMode.into()
        {
            return;
        }

        let cgaPalette = Vec::from([0x000000,0x55ff55,0xff5555,0xffff55]);
        let cgaPalette2 = Vec::from([0x000000,0x55ffff,0xff55ff,0xffffff]);

        if self.mode==0x13
        {
            let mut idx:usize=0;
            for i in gui.frameBuffer.iter_mut() 
            {
                if idx<65536
                {
                    let bufVal=self.framebuffer[idx];
                    *i = self.vgaPalette[bufVal as usize];
                }
                idx+=1;
            }        
        }
        else if (self.mode==0x00) || (self.mode==0x01) || (self.mode==0x02)  || (self.mode==0x03)
        {
            // mode 0,1 - 40x25 text mode, 9x16 chars, 360x400
            // mode 2 - 80x25 text mode, 9x16 chars, 720x400

            let mut resx=720; let mut resy=400; let mut cols=80; let mut rows=25;
            if (self.mode==0x00) || (self.mode==0x01)
            { 
                resx=360; resy=400; cols=40; rows=25; 
            }

            let mut idx:usize=0;
            let mut tempFb:Vec<u32>=Vec::new();
            for _idx in 0..(resx*resy)            
            {
                tempFb.push(0);
            }

            let mut bufIdx=0;
            for i in 0..rows*cols
            {
                let attributes:u8=self.cgaFramebuffer[bufIdx+1];
                let fgCol=attributes&0x0f;
                let bgCol=(attributes>>4)&0x07;
                let charNum:u8=self.cgaFramebuffer[bufIdx];
                self.drawCharOnScreen(&mut tempFb,
                    9,16,
                    32,
                    charNum as u32,
                    i/cols,
                    i%cols,
                    resx,
                    self.vgaPalette[fgCol as usize],self.vgaPalette[bgCol as usize]);
                bufIdx+=2;
            }

            for i in gui.frameBuffer.iter_mut() 
            {
                let bufVal=tempFb[idx];
                *i = bufVal;
                idx+=1;
            }        
        }
        else if (self.mode==0x0d) || (self.mode==0x10)
        {
            // EGA 320x200x16
            // EGA 640x350x16
            let mut idx:usize=0;
            let mut curbit=7;
            
            for i in gui.frameBuffer.iter_mut() 
            {
                {
                    let p0=(self.framebuffer[idx]&(1<<curbit))>>curbit;
                    let p1=(self.framebuffer[idx+0x2000]&(1<<curbit))>>curbit;
                    let p2=(self.framebuffer[idx+0x4000]&(1<<curbit))>>curbit;
                    let p3=(self.framebuffer[idx+0x6000]&(1<<curbit))>>curbit;

                    let colidx=p0|(p1<<1)|(p2<<2)|(p3<<3);

                    *i = self.vgaPalette[colidx as usize];

                    curbit-=1;
                    if curbit<0
                    {
                        curbit=7;
                        idx+=1;
                    }
                }
            }               
        }
        else if self.mode==0x04 || self.mode==0x05
        {
            // CGA 320x200 4 colors
            let mut adder=0;
            let mut currow=0;
            let mut curbyte=0;
            let mut fbidx=0;
            let mut shifter=6;

            let mut cgaPal=cgaPalette;
            if self.mode==0x05 || self.cgaPaletteSelected==1 { cgaPal=cgaPalette2; }

            // even rows
            for pix in gui.frameBuffer.iter_mut()
            {
                let theByte=self.cgaFramebuffer[adder+fbidx];
                let b0:usize=((theByte>>shifter)&0x03) as usize;
                if adder==0 { *pix=cgaPal[b0]; }
                shifter-=2;

                if shifter<0
                {
                    shifter=6;
                    if adder==0 { fbidx+=1; }
                    curbyte+=1;
                    if curbyte==80
                    {
                        curbyte=0;                        
                        currow+=1;
                        if (currow%2)==0 { adder=0; }
                        else { adder=0x2000; }
                    }
                }
            }

            adder=0;
            currow=0;
            curbyte=0;
            fbidx=0;
            shifter=6;

            // odd rows
            for pix in gui.frameBuffer.iter_mut()
            {
                let theByte=self.cgaFramebuffer[adder+fbidx];
                let b0:usize=((theByte>>shifter)&0x03) as usize;
                if adder==0x2000 { *pix=cgaPal[b0]; }
                shifter-=2;

                if shifter<0
                {
                    shifter=6;
                    if adder==0x2000 { fbidx+=1; }
                    curbyte+=1;
                    if curbyte==80
                    {
                        curbyte=0;                        
                        currow+=1;
                        if (currow%2)==0 { adder=0; }
                        else { adder=0x2000; }
                    }
                }
            }
        }
        else if self.mode==0x06
        {
            let mut adder=0;
            let mut currow=0;
            let mut curbyte=0;
            let mut fbidx=0;
            let mut shifter=7;

            // even rows
            for pix in gui.frameBuffer.iter_mut()
            {
                let theByte=self.cgaFramebuffer[adder+fbidx];
                let b0:usize=((theByte>>shifter)&0x01) as usize;
                if adder==0 { if b0>0 { *pix=0xffffff; } else { *pix=0; } }
                shifter-=1;

                if shifter<0
                {
                    shifter=7;
                    if adder==0 { fbidx+=1; }
                    curbyte+=1;
                    if curbyte==80
                    {
                        curbyte=0;                        
                        currow+=1;
                        if (currow%2)==0 { adder=0; }
                        else { adder=0x2000; }
                    }
                }
            }

            adder=0;
            currow=0;
            curbyte=0;
            fbidx=0;
            shifter=6;

            // odd rows
            for pix in gui.frameBuffer.iter_mut()
            {
                let theByte=self.cgaFramebuffer[adder+fbidx];
                let b0:usize=((theByte>>shifter)&0x01) as usize;
                if adder==0x2000 { if b0>0 { *pix=0xffffff; } else { *pix=0; } }
                shifter-=1;

                if shifter<0
                {
                    shifter=7;
                    if adder==0x2000 { fbidx+=1; }
                    curbyte+=1;
                    if curbyte==80
                    {
                        curbyte=0;                        
                        currow+=1;
                        if (currow%2)==0 { adder=0; }
                        else { adder=0x2000; }
                    }
                }
            }
        }
    }

    pub fn new(font9x16:&str,font8x8:&str) -> Self 
    {
        // load fonts

        let myFont9x16 = image::open(font9x16).unwrap().to_rgb8();
        let img_width = myFont9x16.dimensions().0 as u32;
        let img_height = myFont9x16.dimensions().1 as u32;

        let mut font9x16vec:Vec<Vec<u32>>=Vec::new();
        for y in 0..img_height
        {
            let mut newLine:Vec<u32>=Vec::new();
            for x in 0..img_width
            {
                let imgPixel=myFont9x16.get_pixel(x,y);
                let r=imgPixel[0] as u32; let g=imgPixel[1] as u32; let b=imgPixel[2] as u32;
                let u32val:u32=r|(g<<8)|(b<<16);
                newLine.push(u32val);
            }
            font9x16vec.push(newLine);
        }

        let myFont8x8 = image::open(font8x8).unwrap().to_rgb8();
        let img_width8 = myFont8x8.dimensions().0 as u32;
        let img_height8 = myFont8x8.dimensions().1 as u32;

        let mut font8x8vec:Vec<Vec<u32>>=Vec::new();
        for y in 0..img_height8
        {
            let mut newLine:Vec<u32>=Vec::new();
            for x in 0..img_width8
            {
                let imgPixel=myFont8x8.get_pixel(x,y);
                let r=imgPixel[0] as u32; let g=imgPixel[1] as u32; let b=imgPixel[2] as u32;
                let u32val:u32=r|(g<<8)|(b<<16);
                newLine.push(u32val);
            }
            font8x8vec.push(newLine);
        }

        // framebuffers

        let fbSize=65536; // 64k?
        let mut vgaFramebuf:Vec<u8>=Vec::with_capacity(fbSize);
        let mut cgaFramebuf:Vec<u8>=Vec::with_capacity(fbSize);
        for _i in 0..fbSize
        {
            vgaFramebuf.push(0);
            cgaFramebuf.push(0);
        }

        let defaultVgaPalette = Vec::from(
            [
                0x000000,0x0000aa,0x00aa00,0x00aaaa,0xaa0000,0xaa00aa,0xaa5500,0xaaaaaa,
                0x555555,0x5555ff,0x55ff55,0x55ffff,0xff5555,0xff55ff,0xffff55,0xffffff,
                0x000000,0x141414,0x202020,0x2c2c2c,0x383838,0x454545,
                0x515151,0x616161,0x717171,0x828282,0x929292,0xa2a2a2,0xb6b6b6,0xcbcbcb,
                0xe3e3e3,0xffffff,0x0000ff,0x4100ff,0x7d00ff,0xbe00ff,0xff00ff,0xff00be,
                0xff007d,0xff0041,0xff0000,0xff4100,0xff7d00,0xffbe00,0xffff00,0xbeff00,
                0x7dff00,0x41ff00,0x00ff00,0x00ff41,0x00ff7d,0x00ffbe,0x00ffff,0x00beff,
                0x007dff,0x0041ff,0x7d7dff,0x9e7dff,0xbe7dff,0xdf7dff,0xff7dff,0xff7ddf,
                0xff7dbe,0xff7d9e,0xff7d7d,0xff9e7d,0xffbe7d,0xffdf7d,0xffff7d,
                0xdfff7d,0xbeff7d,0x9eff7d,0x7dff7d,0x7dff9e,0x7dffbe,0x7dffdf,0x7dffff,
                0x7ddfff,0x7dbeff,0x7d9eff,0xb6b6ff,0xc7b6ff,0xdbb6ff,0xebb6ff,0xffb6ff,
                0xffb6eb,0xffb6db,0xffb6c7,0xffb6b6,0xffc7b6,0xffdbb6,0xffebb6,0xffffb6,
                0xebffb6,0xdbffb6,0xc7ffb6,0xb6ffb6,0xb6ffc7,0xb6ffdb,0xb6ffeb,
                0xb6ffff,0xb6ebff,0xb6dbff,0xb6c7ff,0x000071,0x1c0071,0x380071,0x550071,
                0x710071,0x710055,0x710038,0x71001c,0x710000,0x711c00,0x713800,0x715500,
                0x717100,0x557100,0x387100,0x1c7100,0x007100,0x00711c,0x007138,0x007155,
                0x007171,0x005571,0x003871,0x001c71,0x383871,0x453871,0x553871,0x613871,
                0x713871,0x713861,0x713855,0x713845,0x713838,0x714538,0x715538,0x716138,
                0x717138,0x617138,0x557138,0x457138,0x387138,0x387145,0x387155,0x387161,
                0x387171,0x386171,0x385571,0x384571,0x515171,0x595171,0x615171,0x695171,
                0x715171,0x715169,0x715161,0x715159,0x715151,0x715951,0x716151,0x716951,
                0x717151,0x697151,0x617151,0x597151,0x517151,0x517159,0x517161,0x517169,
                0x517171,0x516971,0x516171,0x515971,0x000041,0x100041,0x200041,0x300041,
                0x410041,0x410030,0x410020,0x410010,0x410000,0x411000,0x412000,0x413000,
                0x414100,0x304100,0x204100,0x104100,0x004100,0x004110,0x004120,0x004130,
                0x004141,0x003041,0x002041,0x001041,0x202041,0x282041,0x302041,0x382041,
                0x412041,0x412038,0x412030,0x412028,0x412020,0x412820,0x413020,0x413820,
                0x414120,0x384120,0x304120,0x284120,0x204120,0x204128,0x204130,0x204138,
                0x204141,0x203841,0x203041,0x202841,0x2c2c41,0x302c41,0x342c41,0x3c2c41,
                0x412c41,0x412c3c,0x412c34,0x412c30,0x412c2c,0x41302c,0x41342c,0x413c2c,
                0x41412c,0x3c412c,0x34412c,0x30412c,0x2c412c,0x2c4130,0x2c4134,0x2c413c,
                0x2c4141,0x2c3c41,0x2c3441,0x2c3041,0x000000,0x000000,0x000000,0x000000,
                0x000000,0x000000,0x000000,0x000000
                ]
        );

        let reg3c5Values=Vec::from([0,0,0,0,0]); // 5 registers
        let reg3cfValues=Vec::from([0,0,0,0,0,0,0,0,0]); // 9 registers
        let reg3b5Values=Vec::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]); // 64 registers for vga
        let latches=Vec::from([0,0,0,0]);

        vga
        {
            mode: 2,
            framebuffer: vgaFramebuf,
            cgaFramebuffer: cgaFramebuf,
            font9x16data: font9x16vec,
            font9x16width: img_width,
            font9x16height: img_height,
            font8x8data: font8x8vec,
            font8x8width: img_width8,
            font8x8height: img_height8,
            cursorx:0,
            cursory:0,
            vgaPalette: defaultVgaPalette,
            vgaPaletteCurColor: 0,
            vgaPaletteIndexRGB: 0,
            egaRegister3c4Selected: 0,
            egaRegister3c5Values: reg3c5Values,
            egaRegister3ceSelected: 0,
            egaRegister3cfValues: reg3cfValues,
            egaRegister3b4Selected: 0,
            egaRegister3b5Values: reg3b5Values,
            egaDataLatch: latches,
            scanlineCounter: 0,
            cgaPaletteSelected: 0
        }
    }
}
