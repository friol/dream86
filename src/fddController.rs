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

    pub fn readDiskSectors(&self,pmachine:&mut machine,pvga:&mut vga,numOfSectorsToRead:u64,
                           sectorNumber:u64,cylinderNumber:u64,_headNumber:u64,
                           loAddr:u16,hiAddr:u16)
    {
        // we assume we have a 1.44 diskette in
        let bytesPerSector=512;
        let sectorsPerTrack=18; // 3 1/2
        //let sectorsPerTrack=9; // 5 1/4

        /*LBA = (Cylinder × HeadsPerCylinder + Head) × SectorPerTrack + (Sector − 1)
          For a 1.44Mb 3.5" SectorsPerTrack = 18, MaxTrack = 80 HeadsPerCylinder = 2 BytesPerSector = 512*/

        let lba:u64=(((cylinderNumber*2)+_headNumber)*sectorsPerTrack)+(sectorNumber);
        let imgOffset=lba*bytesPerSector;

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
