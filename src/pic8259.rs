/* PIC 8259 - programmable interrupt controller */

//use std::process;

//use crate::machine::machine;


pub struct pic8259
{
    imr:u8, // irq mask register
    irr:u8, // request register
    isr:u8, // service register
    icwstep: u8,
    icw: Vec<u8>,
    ocw: Vec<u8>,
    readmode: u8
}

impl pic8259
{
    pub fn new() -> Self 
    {
        let icwVec=Vec::from([0,0,0,0,0]);
        let ocwVec=Vec::from([0,0,0,0,0]);

        pic8259
        {
            imr: 0,
            irr: 0,
            isr: 0,
            icwstep: 0,
            icw: icwVec,
            ocw: ocwVec,
            readmode: 0
        }
    }
}
