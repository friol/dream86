/* the VGA - dream86 */

use crate::guiif::guiif;

pub struct vga
{
    pub mode: u16,
    pub framebuffer: Vec<u8>,
    pub cgaFramebuffer: Vec<u8>,
    pub font9x16data:Vec<Vec<u32>>,
    pub font9x16width:u32,
    pub font9x16height:u32,
    pub cursorx:usize,
    pub cursory:usize
}

impl vga
{
    pub fn setVideomode(&mut self,videomodeNum:u16)
    {
        if videomodeNum==0x13
        {
            // VGA 320x200
            self.mode=0x13;
            for idx in 0..self.framebuffer.len()
            {
                self.framebuffer[idx]=0;
            }
        }
        else if videomodeNum==0x01
        {
            // 40x25 textmode 9x16
            self.mode=0x1;
        }
        else if videomodeNum==0x02
        {
            // 80x25 textmode 9x16
            self.mode=0x2;
        }
        else if videomodeNum==0x04
        {
            // CGA 320x200 4 colours            
            self.mode=0x04;
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
    }

    //

    pub fn readMemory(&self,addr:i64) -> u8
    {
        if (addr>=0xa0000) && (addr<=(0xaffff))
        {
            return self.framebuffer[(addr-0xa0000) as usize];
        }
        else if (addr>=0xb8000) && (addr<=0xbffff)
        {
            return self.cgaFramebuffer[(addr-0xb8000) as usize];
        }

        return 0;
    }

    pub fn readMemory16(&self,addr:i64) -> u16
    {
        let lobyte=self.readMemory(addr) as u16;
        let hibyte=self.readMemory(addr+1) as u16;
        return lobyte|(hibyte<<8);
    }

    pub fn writeMemory(&mut self,addr:i64,val:u8)
    {
        if (addr>=0xa0000) && (addr<=0xaffff)
        {
            self.framebuffer[(addr-0xa0000) as usize]=val;
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

    pub fn outputCharToStdout(&mut self,ochar:u8)
    {
        // if in textmode
        if self.mode==2
        {
            if (ochar==10) || (ochar==13)
            {
                self.cursorx=0;
                self.cursory+=1;
                // TODO: handle scroll
            }
            else
            {
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*80)]=ochar;
                self.cgaFramebuffer[(self.cursorx*2)+(self.cursory*80)+1]=0x0f;
                self.cursorx+=1;
                if self.cursorx==80
                {
                    self.cursorx=0;
                    self.cursory+=1;
                }
            }
        }
    }

    fn drawTextmodeChar(&self,vecDest:&mut Vec<u32>,charDimX:u32,charDimY:u32,numCharsPerRow:u32,charNum:u32,row:u32,col:u32,scrInc:u32,fgCol:u32,bgCol:u32)
    {
        let mut srcx:u32=(charNum%numCharsPerRow)*charDimX;
        let mut srcy:u32=(charNum/numCharsPerRow)*charDimY;

        let dstx:u32=col*charDimX;
        let dsty:u32=row*charDimY;

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

    pub fn fbTobuf32(&self,gui:&mut guiif)
    {
        if self.mode!=gui.videoMode.into()
        {
            return;
        }

        let cgaPalette = Vec::from([0x000000,0x55ffff,0xff55ff,0xffffff]);

        let vgaPalette = Vec::from(
            [
                0x000000,0x0000aa,0x00aa00,0x00aaaa,0xaa0000,0xaa00aa,0xaa5500,0xaaaaaa,
                0x555555,0x5555ff,0x55ff55,0x55ffff,0xff5555,0xff55ff,0xffff55,0xffffff,
                0x000000,                0x141414,
                0x202020,                0x2c2c2c,                0x383838,                0x454545,
                0x515151,                0x616161,                0x717171,                0x828282,
                0x929292,                0xa2a2a2,                0xb6b6b6,                0xcbcbcb,
                0xe3e3e3,                0xffffff,                0x0000ff,                0x4100ff,
                0x7d00ff,                0xbe00ff,                0xff00ff,                0xff00be,
                0xff007d,                0xff0041,                0xff0000,                0xff4100,
                0xff7d00,                0xffbe00,                0xffff00,                0xbeff00,
                0x7dff00,                0x41ff00,                0x00ff00,                0x00ff41,
                0x00ff7d,                0x00ffbe,                0x00ffff,                0x00beff,
                0x007dff,                0x0041ff,                0x7d7dff,                0x9e7dff,
                0xbe7dff,                0xdf7dff,                0xff7dff,                0xff7ddf,
                0xff7dbe,                0xff7d9e,                0xff7d7d,
                0xff9e7d,                0xffbe7d,                0xffdf7d,                0xffff7d,
                0xdfff7d,                0xbeff7d,                0x9eff7d,                0x7dff7d,
                0x7dff9e,                0x7dffbe,                0x7dffdf,                0x7dffff,
                0x7ddfff,                0x7dbeff,                0x7d9eff,                0xb6b6ff,
                0xc7b6ff,                0xdbb6ff,                0xebb6ff,                0xffb6ff,
                0xffb6eb,                0xffb6db,                0xffb6c7,                0xffb6b6,
                0xffc7b6,                0xffdbb6,                0xffebb6,                0xffffb6,
                0xebffb6,                0xdbffb6,                0xc7ffb6,
                0xb6ffb6,                0xb6ffc7,                0xb6ffdb,                0xb6ffeb,
                0xb6ffff,                0xb6ebff,                0xb6dbff,                0xb6c7ff,
                0x000071,                0x1c0071,                0x380071,                0x550071,
                0x710071,                0x710055,                0x710038,                0x71001c,
                0x710000,                0x711c00,                0x713800,                0x715500,
                0x717100,                0x557100,                0x387100,                0x1c7100,
                0x007100,                0x00711c,                0x007138,                0x007155,
                0x007171,                0x005571,                0x003871,                0x001c71,
                0x383871,                0x453871,                0x553871,                0x613871,
                0x713871,                0x713861,                0x713855,                0x713845,
                0x713838,                0x714538,                0x715538,                0x716138,
                0x717138,                0x617138,                0x557138,                0x457138,
                0x387138,                0x387145,                0x387155,                0x387161,
                0x387171,                0x386171,                0x385571,                0x384571,
                0x515171,                0x595171,                0x615171,                0x695171,
                0x715171,                0x715169,                0x715161,                0x715159,
                0x715151,                0x715951,                0x716151,                0x716951,
                0x717151,                0x697151,                0x617151,                0x597151,
                0x517151,                0x517159,                0x517161,                0x517169,
                0x517171,                0x516971,                0x516171,                0x515971,
                0x000041,                0x100041,                0x200041,                0x300041,
                0x410041,                0x410030,                0x410020,                0x410010,
                0x410000,                0x411000,                0x412000,                0x413000,
                0x414100,                0x304100,                0x204100,                0x104100,
                0x004100,                0x004110,                0x004120,                0x004130,
                0x004141,                0x003041,                0x002041,                0x001041,
                0x202041,                0x282041,                0x302041,                0x382041,
                0x412041,                0x412038,                0x412030,                0x412028,
                0x412020,                0x412820,                0x413020,                0x413820,
                0x414120,                0x384120,                0x304120,                0x284120,
                0x204120,                0x204128,                0x204130,                0x204138,
                0x204141,                0x203841,                0x203041,                0x202841,
                0x2c2c41,                0x302c41,                0x342c41,                0x3c2c41,
                0x412c41,                0x412c3c,                0x412c34,                0x412c30,
                0x412c2c,                0x41302c,                0x41342c,                0x413c2c,
                0x41412c,                0x3c412c,                0x34412c,                0x30412c,
                0x2c412c,                0x2c4130,                0x2c4134,                0x2c413c,
                0x2c4141,                0x2c3c41,                0x2c3441,                0x2c3041,
                0x000000,                0x000000,                0x000000,                0x000000,
                0x000000,                0x000000,                0x000000,                0x000000
                ]
        );

        if self.mode==0x13
        {
            let mut idx:usize=0;
            for i in gui.frameBuffer.iter_mut() 
            {
                if idx<65536
                {
                    let bufVal=self.framebuffer[idx];
                    *i = vgaPalette[bufVal as usize];
                }
                idx+=1;
            }        
        }
        else if (self.mode==0x01) || (self.mode==0x02)
        {
            // mode 1 - 40x25 text mode, 9x16 chars, 360x400
            // mode 2 - 80x25 text mode, 9x16 chars, 720x400

            let mut resx=720; let mut resy=400; let mut rows=80; let mut cols=25;
            if self.mode==0x01 
            { 
                resx=360; resy=400; rows=40; cols=25; 
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
                self.drawTextmodeChar(&mut tempFb,
                    9,16,
                    32,
                    charNum as u32,
                    i/rows,
                    i%rows,
                    resx,
                    vgaPalette[fgCol as usize],vgaPalette[bgCol as usize]);
                    //vgaPalette[1],vgaPalette[2]);
                bufIdx+=2;
            }

            for i in gui.frameBuffer.iter_mut() 
            {
                let bufVal=tempFb[idx];
                *i = bufVal;
                idx+=1;
            }        
        }
        else if self.mode==0x04
        {
            let mut adder=0;
            let mut currow=0;
            let mut curbyte=0;
            let mut fbidx=0;
            let mut shifter=6;

            // even rows
            for pix in gui.frameBuffer.iter_mut()
            {
                let theByte=self.cgaFramebuffer[adder+fbidx];
                let b0:usize=((theByte>>shifter)&0x03) as usize;
                if adder==0 { *pix=cgaPalette[b0]; }
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
                if adder==0x2000 { *pix=cgaPalette[b0]; }
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
    }

    pub fn new(font9x16:&str) -> Self 
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

        // framebuffers

        let fbSize=65536; // 64k?
        let mut vgaFramebuf:Vec<u8>=Vec::with_capacity(fbSize);
        let mut cgaFramebuf:Vec<u8>=Vec::with_capacity(fbSize);
        for _i in 0..fbSize
        {
            vgaFramebuf.push(0);
            cgaFramebuf.push(0);
        }

        vga
        {
            mode: 2,
            framebuffer: vgaFramebuf,
            cgaFramebuffer: cgaFramebuf,
            font9x16data: font9x16vec,
            font9x16width: img_width,
            font9x16height: img_height,
            cursorx:0,
            cursory:0
        }
    }
}
