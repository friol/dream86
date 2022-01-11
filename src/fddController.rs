/* FDD fake controller ftw */

use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::process;


use crate::vga::vga;
use crate::machine::machine;


pub struct fddController
{
    fddFullPath: String
}

impl fddController
{
    pub fn new(diskImage:String) -> Self 
    {
        fddController
        {
            fddFullPath: diskImage.clone()
        }
    }

    pub fn readDiskSectors(&self,pmachine:&mut machine,pvga:&mut vga,numOfSectorsToRead:u64,sectorNumber:u64,trackNumber:u64,_headNumber:u64,loAddr:u16,hiAddr:u16)
    {
        // we assume we have a 1.44 diskette in
        let bytesPerSector=512;
        let sectorsPerTrack=18;
        let tracksPerSide=80;

        let mut imgOffset:u64=(trackNumber*sectorsPerTrack*bytesPerSector)+(sectorNumber*bytesPerSector);
        // add eventual heads
        imgOffset+=_headNumber*tracksPerSide*sectorsPerTrack*bytesPerSector;

        /*if (_headNumber==1) && (trackNumber==0) && (sectorNumber==1)
        {
            imgOffset=0x2600;
        }*/

        let mut f = match File::open(self.fddFullPath.clone()) {
            Ok(f) => f,
            Err(_e) => {
                println!("Unable to open file {}",self.fddFullPath);
                process::exit(0x100);
            }
        };
        f.seek(SeekFrom::Start(imgOffset)).ok();

        let mut memOffs:u16=loAddr;
        for _idx in 0..(numOfSectorsToRead*bytesPerSector)
        {
            let mut buf = vec![0u8; 1];
            f.read_exact(&mut buf).ok();
            pmachine.writeMemory(hiAddr,memOffs,buf[0],pvga);
            memOffs+=1;
        }

    }
}
