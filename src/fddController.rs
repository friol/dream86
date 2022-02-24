
/* FDD/HD high level emulation */

use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::process;
use std::fs;


use crate::vga::vga;
use crate::machine::machine;

#[derive(PartialEq)]
pub enum mediaType
{
    hardDisk,
    floppy144
}

pub struct fddController
{
    fddFullPath: String,
    diskType: mediaType
}

impl fddController
{
    pub fn new(diskImage:&String) -> Self 
    {
        let mut mType=mediaType::floppy144;
        let fileLen = fs::metadata(diskImage).unwrap().len();
        if fileLen>1474560
        {
            mType=mediaType::hardDisk;
        }

        fddController
        {
            fddFullPath: diskImage.clone(),
            diskType: mType
        }
    }

    pub fn readDiskSectors(&self,pmachine:&mut machine,pvga:&mut vga,numOfSectorsToRead:u64,
                           sectorNumber:u64,cylinderNumber:u64,_headNumber:u64,
                           loAddr:u16,hiAddr:u16)
    {
        let bytesPerSector=512;
        let sectorsPerTrack;
        let headsPerCylinder;

        if self.diskType==mediaType::floppy144
        {
            sectorsPerTrack=18;
            headsPerCylinder=2;
        }
        else
        {
            sectorsPerTrack=63;
            headsPerCylinder=16;
        }

        /* LBA = (Cylinder × HeadsPerCylinder + Head) × SectorPerTrack + (Sector − 1) */

        let lba:u64=(((cylinderNumber*headsPerCylinder)+_headNumber)*sectorsPerTrack)+(sectorNumber);
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
