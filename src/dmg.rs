use super::memory::cartridge::Cartridge;
use super::memory::bootrom::BootROM;
use super::memory;
use super::cpu::CPU;
use std::io;
use crate::ppu::PPU;

pub struct DMG {
    cpu: CPU,
}

impl DMG {
    pub fn new(rom_file_path: &String) -> io::Result<DMG> {
        let cartridge = Cartridge::read_cartridge_from_romfile(rom_file_path)?;
        let boot_rom = BootROM::new("DMG_ROM.bin")?;
        let ppu = PPU::new();
        let memory = memory::MemoryManager::new(boot_rom, cartridge, ppu);
        let cpu = CPU::new(memory);
        Ok(DMG{cpu})
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step();
        }
    }
}