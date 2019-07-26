pub mod cartridge;
pub mod bootrom;

use cartridge::Cartridge;
use bootrom::BootROM;

pub trait MemoryZone {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub struct Memory {
    pub boot_rom: BootROM,
    pub cartridge: cartridge::Cartridge,
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

impl MemoryZone for Memory {
    fn read(&self, address: u16) -> u8 { self.boot_rom.read(address) }
    fn write(&mut self, address: u16, value: u8) { self.boot_rom.write(address, value) }
}

impl Memory {
    pub fn new_from_vec(data: Vec<u8>) -> Memory {
        let size = data.len() as u16;
        let boot_rom = BootROM{data, offset: 0, size};
        Memory { boot_rom, cartridge: Cartridge::new_dummy_cartridge()}
    }
}
