pub mod cartridge;
pub mod bootrom;

use cartridge::Cartridge;
use cartridge::ROM_BANK_SIZE;
use bootrom::BootROM;
use crate::bus::bootrom::BOOT_ROM_SIZE;
use crate::ppu::PPU;

const HIGH_RAM_BANK_SIZE: u16 = 0x007F;
const HIGH_RAM_BASE_ADDRESS: u16 = 0xFF80;
const WORK_RAM_BANK_SIZE: u16 = 0x1000;
const WORK_RAM_BASE_ADDRESS: u16 = 0xC000;
const VIDEO_RAM_SIZE: u16 = 0x2000;
const VIDEO_RAM_BASE_ADDRESS: u16 = 0x8000;
const IO_PORTS_SIZE: u16 = 0x80;
const IO_PORTS_BASE_ADDRESS: u16 = 0xFF00;

pub trait MemoryZone {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub struct RAMBank {
    pub data: Vec<u8>,
    pub base_address: u16,
}

impl MemoryZone for RAMBank {
    fn read(&self, address: u16) -> u8 {
        self.data[self.global_address_to_local_address(address) as usize]
    }
    fn write(&mut self, address: u16, value: u8) {
        let local_address = self.global_address_to_local_address(address) as usize;
        self.data[local_address] = value;
    }
}

impl RAMBank {
    fn global_address_to_local_address(&self, address: u16) -> u16 { address - self.base_address }
}

pub struct IOPorts {
    pub data: Vec<u8>
}

impl MemoryZone for IOPorts {
    fn read(&self, address: u16) -> u8 {
        println!("Reading from IO");
        self.data[self.global_address_to_local_address(address) as usize]
    }
    fn write(&mut self, address: u16, value: u8) {
        println!("Writing to IO");
        let local_address = self.global_address_to_local_address(address) as usize;
        self.data[local_address] = value;
    }
}

impl IOPorts {
    fn global_address_to_local_address(&self, address: u16) -> u16 { address - IO_PORTS_BASE_ADDRESS }
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
    ppu: PPU,
}

impl Bus {
    pub fn read(&mut self, address: u16) -> u8 {
        self.get_memory_zone_from_address(address).read(address)
    }
    pub fn write(&mut self, address: u16, value: u8) {
        self.get_memory_zone_from_address(address).write(address, value)
    }

    pub fn cycle(&mut self) {
        self.ppu.cycle();
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

    fn new_io_ports() -> IOPorts { IOPorts{data: vec![0; IO_PORTS_SIZE as usize]} }

    pub fn new(boot_rom: BootROM, cartridge: Cartridge, ppu: PPU) -> Bus {
        Bus {
            boot_rom_active: true,
            boot_rom,
            cartridge,
            work_ram: Bus::new_work_ram(),
            video_ram: Bus::new_video_ram(),
            io_ports: Bus::new_io_ports(),
            high_ram: Bus::new_high_ram(),
            ppu,
        }
    }

    pub fn new_from_vecs(boot_rom_data: Vec<u8>, cart_rom_bank_zero_data: Vec<u8>) -> Bus {
        let boot_rom = BootROM{data: boot_rom_data};
        Bus {
            boot_rom_active: true,
            boot_rom,
            cartridge: Cartridge::new_dummy_cartridge(),
            work_ram: Bus::new_work_ram(),
            video_ram: Bus::new_video_ram(),
            io_ports: Bus::new_io_ports(),
            high_ram: Bus::new_high_ram(),
            ppu: PPU::new(),
        }
    }

    fn get_memory_zone_from_address(&mut self, address: u16) -> Box<&mut MemoryZone> {
        if self.boot_rom_active && address < BOOT_ROM_SIZE as u16 { return Box::new(&mut self.boot_rom) };
        if address < ROM_BANK_SIZE as u16 { return Box::new(&mut self.cartridge.rom_banks[0])};
        if address < (ROM_BANK_SIZE * 2) as u16 { panic!("Rom banking not implemented"); };
        if address < 0xA000 { return Box::new(&mut self.video_ram); };
        if address < 0xC000 { panic!("External ram not implemented"); };
        if address < 0xD000 { return Box::new(&mut self.work_ram); };
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
}