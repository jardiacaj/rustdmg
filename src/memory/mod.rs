pub mod cartridge;
pub mod bootrom;

use cartridge::Cartridge;
use cartridge::ROM_BANK_SIZE;
use bootrom::BootROM;
use crate::memory::bootrom::BOOT_ROM_SIZE;

const WORK_RAM_BANK_SIZE: u16 = 0x1000;
const WORK_RAM_BASE_ADDRESS: u16 = 0xC000;
const VIDEO_RAM_SIZE: u16 = 0x2000;
const VIDEO_RAM_BASE_ADDRESS: u16 = 0x8000;

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

pub struct Memory {
    pub boot_rom_active: bool,
    pub boot_rom: BootROM,
    pub cartridge: cartridge::Cartridge,
    pub work_ram: RAMBank,
    pub video_ram: RAMBank,
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
}

impl Memory {
    pub fn read(&mut self, address: u16) -> u8 {
        self.get_memory_zone_from_address(address).read(address)
    }
    pub fn write(&mut self, address: u16, value: u8) {
        self.get_memory_zone_from_address(address).write(address, value)
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

    pub fn new(boot_rom: BootROM, cartridge: Cartridge) -> Memory {
        Memory {
            boot_rom_active: true,
            boot_rom,
            cartridge,
            work_ram: Memory::new_work_ram(),
            video_ram: Memory::new_video_ram(),
        }
    }

    pub fn new_from_vecs(boot_rom_data: Vec<u8>, cart_rom_bank_zero_data: Vec<u8>) -> Memory {
        let size = boot_rom_data.len() as u16;
        let boot_rom = BootROM{data: boot_rom_data, offset: 0, size};
        Memory {
            boot_rom_active: true,
            boot_rom,
            cartridge: Cartridge::new_dummy_cartridge(),
            work_ram: Memory::new_work_ram(),
            video_ram: Memory::new_video_ram(),
        }
    }

    fn get_memory_zone_from_address(&mut self, address: u16) -> Box<&mut MemoryZone> {
        if self.boot_rom_active && address < self.boot_rom.size { return Box::new(&mut self.boot_rom) };
        if address < ROM_BANK_SIZE as u16 { return Box::new(&mut self.cartridge.rom_banks[0])};
        if address < (ROM_BANK_SIZE * 2) as u16 { panic!("Rom banking not implemented"); };
        if address < 0xA000 { return Box::new(&mut self.video_ram); };
        if address < 0xC000 { panic!("External ram not implemented"); };
        if address < 0xD000 { return Box::new(&mut self.work_ram); };
        panic!("Invalid memory address 0x{:X?}", address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_boot_rom_zone() {
        let mut memory = Memory::new_from_vecs(vec![0, 0x55], vec![]);
        assert_eq!(memory.get_memory_zone_from_address(1).read(1), 0x55);
    }
    #[test]
    fn get_work_ram_zone() {
        let mut memory = Memory::new_from_vecs(vec![], vec![]);
        memory.work_ram.data[0x12] = 0xFF;
        assert_eq!(memory.get_memory_zone_from_address(0xC012).read(0xC012), 0xFF);
    }
    #[test]
    fn get_video_ram_zone() {
        let mut memory = Memory::new_from_vecs(vec![], vec![]);
        memory.video_ram.data[0x12] = 0xFF;
        assert_eq!(memory.get_memory_zone_from_address(0x8012).read(0x8012), 0xFF);
    }
}