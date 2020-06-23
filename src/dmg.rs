use super::bus::cartridge::Cartridge;
use super::bus::bootrom::BootROM;
use super::bus;
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
        let bus = bus::Bus::new(boot_rom, cartridge, ppu);
        let cpu = CPU::new(bus);
        Ok(DMG{cpu})
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step();
        }
    }
}