use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::ppu::PPU;


const IO_SOUND_CHANNEL_CONTROL_NR50: u16 = 0xFF24;
const IO_SOUND_ON_OFF_NR52: u16 = 0xFF26;
const IO_SOUND_CH1_SOUND_LENGTH_WAVE_PATTERN_DUTY_NR11: u16 = 0xFF11;
const IO_SOUND_CH1_VOLUME_ENVELOPE_NR12: u16 = 0xFF12;
const IO_SOUND_CH1_FREQUENCY_LO_NR13: u16 = 0xFF13;
const IO_SOUND_CH1_FREQUENCY_HI_NR14: u16 = 0xFF14;
const IO_SOUND_OUTPUT_TERMINAL_NR51: u16 = 0xFF25;

const IO_LCD_CONTROL: u16 = 0xFF40;
const IO_LCD_SCROLL_Y: u16 = 0xFF42;
const IO_LCD_Y_COORDINATE: u16 = 0xFF44;
const IO_LDC_BG_PALETTE_DATA: u16 = 0xFF47;

pub struct IOPorts {
    pub data: Vec<u8>,
    ppu: Rc<RefCell<PPU>>,
}

impl MemoryZone for IOPorts {
    fn read(&self, address: u16) -> u8 {
        match address {
            IO_LCD_Y_COORDINATE => { self.ppu.borrow().current_line }
            _ => {panic!("Reading from IO address {:04X}", address);}
        }
        // self.data[self.global_address_to_local_address(address) as usize]
    }
    fn write(&mut self, address: u16, value: u8) {
        match address {
            IO_SOUND_CHANNEL_CONTROL_NR50 => { println!("Not implemented"); }
            IO_SOUND_ON_OFF_NR52 => { println!("Not implemented"); }
            IO_SOUND_CH1_SOUND_LENGTH_WAVE_PATTERN_DUTY_NR11 => { println!("Not implemented"); }
            IO_SOUND_CH1_VOLUME_ENVELOPE_NR12 => { println!("Not implemented"); }
            IO_SOUND_CH1_FREQUENCY_LO_NR13 => { println!("Not implemented"); }
            IO_SOUND_CH1_FREQUENCY_HI_NR14 => { println!("Not implemented"); }
            IO_SOUND_OUTPUT_TERMINAL_NR51 => { println!("Not implemented"); }
            IO_LDC_BG_PALETTE_DATA => { println!("Not implemented"); }
            IO_LCD_SCROLL_Y => { println!("Not implemented"); }
            IO_LCD_CONTROL => { println!("Not implemented"); }
            _ => {panic!("Writing to IO: address {:04X} value {:02X}", address, value);}
        }
        let local_address = self.global_address_to_local_address(address) as usize;
        self.data[local_address] = value;
    }
}

impl IOPorts {
    fn global_address_to_local_address(&self, address: u16) -> u16 { address - IO_PORTS_BASE_ADDRESS }

    pub fn new(ppu: Rc<RefCell<PPU>>) -> IOPorts {
        IOPorts{
            data: vec![0; IO_PORTS_SIZE as usize], 
            ppu,
        }
    }
}
