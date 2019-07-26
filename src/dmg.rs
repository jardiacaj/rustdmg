use super::memory::cartridge::Cartridge;
use super::memory;
use super::cpu::CPU;
use std::io;

pub struct DMG {
    cpu: CPU,
}

impl DMG {
    pub fn new(rom_file_path: &String) -> io::Result<DMG> {
        let cartridge = Cartridge::read_cartridge_from_romfile(rom_file_path)?;
        let boot_rom = memory::BootROM::new("DMG_ROM.bin")?;
        let memory = memory::Memory{
            boot_rom,
            cartridge,
        };
        let cpu = CPU::create(memory);
        Ok(DMG{cpu})
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step();
        }
    }
}