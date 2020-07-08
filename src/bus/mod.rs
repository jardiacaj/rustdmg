pub mod cartridge;
pub mod bootrom;
pub mod io_ports;
pub mod ram_bank;

use std::cell::RefCell;
use std::rc::Rc;

use cartridge::Cartridge;
use bootrom::BootROM;
use io_ports::IOPorts;
use ram_bank::RAMBank;
use crate::ppu::PPU;

const ROM_BANK_SIZE: usize = 0x4000;
const BOOT_ROM_SIZE: usize = 256;
const HIGH_RAM_BANK_SIZE: u16 = 0x007F;
const HIGH_RAM_BASE_ADDRESS: u16 = 0xFF80;
const WORK_RAM_BANK_SIZE: u16 = 0x2000;
const WORK_RAM_BASE_ADDRESS: u16 = 0xC000;
const VIDEO_RAM_SIZE: u16 = 0x2000;
const VIDEO_RAM_BASE_ADDRESS: u16 = 0x8000;
const IO_PORTS_SIZE: u16 = 0x80;
const IO_PORTS_BASE_ADDRESS: u16 = 0xFF00;


pub trait MemoryZone {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub struct Bus {
    pub boot_rom_active: bool,
    pub boot_rom: BootROM,
    pub cartridge: cartridge::Cartridge,
    pub work_ram: RAMBank,
    pub video_ram: RAMBank,
    pub io_ports: IOPorts,
    pub high_ram: RAMBank,
//            rom_bank_fixed: MemoryZone,
//            rom_bank_switchable: MemoryZone,
//            vram: MemoryZone,
//            cartridge_ram: MemoryZone,
//            work_ram_fixed: MemoryZone,
//            work_ram_switchable: MemoryZone,
//            work_ram_echo: MemoryZone,
//            oam: MemoryZone,
//            not_usable: MemoryZone,
//            io_ram: MemoryZone,
//            hi_ram: MemoryZone,
//            interrupt_enable_register: MemoryZone,
    ppu: Rc<RefCell<PPU>>,
}

impl Bus {
    pub fn read(&mut self, address: u16) -> u8 {
        self.get_memory_zone_from_address(address).read(address)
    }
    pub fn write(&mut self, address: u16, value: u8) {
        if address == 0xFF50 && value == 1 { self.boot_rom_active = false };
        self.get_memory_zone_from_address(address).write(address, value)
    }

    pub fn cycle(&mut self) {
        self.ppu.borrow_mut().cycle();
    }

    fn new_video_ram() -> RAMBank {
        RAMBank {
            base_address: VIDEO_RAM_BASE_ADDRESS,
            data: vec![0; VIDEO_RAM_SIZE as usize]
        }
    }

    fn new_work_ram() -> RAMBank {
        RAMBank {
            base_address: WORK_RAM_BASE_ADDRESS,
            data: vec![0; WORK_RAM_BANK_SIZE as usize]
        }
    }

    fn new_high_ram() -> RAMBank {
        RAMBank {
            base_address: HIGH_RAM_BASE_ADDRESS,
            data: vec![0; HIGH_RAM_BANK_SIZE as usize]
        }
    }

    pub fn new (boot_rom: BootROM, cartridge: Cartridge, ppu: PPU) -> Bus {
        let ppu_ref = Rc::new(RefCell::new(ppu));
        let io_ports = IOPorts::new(Rc::clone(&ppu_ref));
        Bus {
            boot_rom_active: true,
            boot_rom,
            cartridge,
            work_ram: Bus::new_work_ram(),
            video_ram: Bus::new_video_ram(),
            io_ports,
            high_ram: Bus::new_high_ram(),
            ppu: Rc::clone(&ppu_ref),
        }
    }

    pub fn new_from_vecs(boot_rom_data: Vec<u8>, cart_rom_bank_zero_data: Vec<u8>) -> Bus {
        let boot_rom = BootROM{data: boot_rom_data};
        let ppu: PPU = PPU::new();
        let ppu_ref = Rc::new(RefCell::new(ppu));
        let io_ports = IOPorts::new(Rc::clone(&ppu_ref));
        Bus {
            boot_rom_active: true,
            boot_rom,
            cartridge: Cartridge::new_dummy_cartridge(cart_rom_bank_zero_data),
            work_ram: Bus::new_work_ram(),
            video_ram: Bus::new_video_ram(),
            io_ports,
            high_ram: Bus::new_high_ram(),
            ppu: Rc::clone(&ppu_ref),
        }
    }

    fn get_memory_zone_from_address(&mut self, address: u16) -> Box<&mut MemoryZone> {
        if self.boot_rom_active && address < BOOT_ROM_SIZE as u16 { return Box::new(&mut self.boot_rom) };
        if address < ROM_BANK_SIZE as u16 { return Box::new(&mut self.cartridge.rom_banks[0])};
        if address < (ROM_BANK_SIZE * 2) as u16 { panic!("Rom banking not implemented"); };
        if address < 0xA000 { return Box::new(&mut self.video_ram); };
        if address < 0xC000 { panic!("External ram not implemented"); };
        if address < 0xE000 { return Box::new(&mut self.work_ram); };
        if address >= IO_PORTS_BASE_ADDRESS && address < IO_PORTS_BASE_ADDRESS + IO_PORTS_SIZE {
            return Box::new(&mut self.io_ports);
        }
        if address >= HIGH_RAM_BASE_ADDRESS && address < HIGH_RAM_BASE_ADDRESS + HIGH_RAM_BANK_SIZE {
            return Box::new(&mut self.high_ram);
        }
        panic!("Invalid bus address {:#02X?}", address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_boot_rom_zone() {
        let mut bus = Bus::new_from_vecs(vec![0, 0x55], vec![]);
        assert_eq!(bus.get_memory_zone_from_address(1).read(1), 0x55);
    }
    #[test]
    fn get_work_ram_zone() {
        let mut bus = Bus::new_from_vecs(vec![], vec![]);
        bus.work_ram.data[0x12] = 0xFF;
        assert_eq!(bus.get_memory_zone_from_address(0xC012).read(0xC012), 0xFF);
    }
    #[test]
    fn get_video_ram_zone() {
        let mut bus = Bus::new_from_vecs(vec![], vec![]);
        bus.video_ram.data[0x12] = 0xFF;
        assert_eq!(bus.get_memory_zone_from_address(0x8012).read(0x8012), 0xFF);
    }

    #[test]
    fn read_ff44_lcdc_y_coordinate() {
        let mut bus = Bus::new_from_vecs(vec![], vec![]);
        bus.ppu.borrow_mut().current_line = 123;
        assert_eq!(bus.read(0xFF44), 123);

    }

    #[test]
    fn write_ff50_disable_boot_rom() {
        let mut bus = Bus::new_from_vecs(vec![0x12], vec![0x34]);
        assert_eq!(bus.read(0x0000), 0x12);
        assert_eq!(bus.boot_rom_active, true);
        bus.write(0xFF50, 1);
        assert_eq!(bus.boot_rom_active, false);
        assert_eq!(bus.read(0x0000), 0x34);

    }
}