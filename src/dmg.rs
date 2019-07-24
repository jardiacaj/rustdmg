use super::memory::cartridge::Cartridge;
use super::memory;
use super::cpu::CPU;

pub struct DMG {
    cpu: CPU,
}

impl DMG {
    pub fn run_rom(rom_file_path: &str) -> DMG {
        let mut dmg = DMG::init(rom_file_path);
        dmg.run();
        dmg
    }

    pub fn init(rom_file_path: &str) -> DMG {
        let cartridge = match Cartridge::read_cartridge_from_romfile(rom_file_path) {
            Err(e) => panic!("Failed to read cartridge"),
            Ok(cartridge) => cartridge
        };
        let boot_rom = match memory::read_boot_rom_from_romfile("DMG_ROM.bin") {
            Err(e) => panic!("Failed to read boot ROM"),
            Ok(boot_rom) => boot_rom
        };
        let memory = memory::Memory{
            boot_rom,
            cartridge,
        };
        let cpu = CPU::create(memory);
        DMG {
            cpu,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step();
        }
    }
}