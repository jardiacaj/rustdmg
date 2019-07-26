pub mod cartridge;

use std::fs;
use std::io;
use std::io::Read;

const BOOT_ROM_SIZE: usize = 256;

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

impl Memory {
    pub fn read(&self, address: u16) -> u8 { self.boot_rom.read(address) }
    pub fn write(&mut self, address: u16, value: u8) { self.boot_rom.write(address, value) }

}

trait MemoryZone {
    fn read(&self, address: u16) -> u8;
    fn write(&self, address: u16, value: u8);
}

pub struct BootROM {
    offset: u16,
    size: u16,
    data: Vec<u8>,
}

impl BootROM {
    pub fn new(boot_rom_file_path: &str) -> io::Result<BootROM> {
        let file_metadata = fs::metadata(boot_rom_file_path)?;

        if file_metadata.len() as usize != BOOT_ROM_SIZE {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Bad boot ROM file size"));
        }

        let mut file = fs::File::open(boot_rom_file_path)?;
        let mut data: Vec<u8> = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(
            BootROM{
                data,
                offset: 0,
                size: BOOT_ROM_SIZE as u16,
            }
        )
    }
}

impl MemoryZone for BootROM {
    fn read(&self, address: u16) -> u8 { self.data[address as usize] }
    fn write(&self, address: u16, value: u8) { panic!("Trying to write to boot ROM"); }
}