use super::MemoryZone;

use std::fs;
use std::io;
use std::io::Read;

pub const BOOT_ROM_SIZE: usize = 256;

pub struct BootROM { pub data: Vec<u8> }

impl BootROM {
    pub fn new(boot_rom_file_path: &str) -> io::Result<BootROM> {
        let file_metadata = fs::metadata(boot_rom_file_path)?;

        if file_metadata.len() as usize != BOOT_ROM_SIZE {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Bad boot ROM file size"));
        }

        let mut file = fs::File::open(boot_rom_file_path)?;
        let mut data: Vec<u8> = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(BootROM{data})
    }
}

impl MemoryZone for BootROM {
    fn read(&self, address: u16) -> u8 { self.data[address as usize] }
    fn write(&mut self, address: u16, value: u8) { panic!("Trying to write to boot ROM"); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Trying to write to boot ROM")]
    fn write_panics() {
        let mut bootrom = BootROM{data:vec![0]};
        bootrom.write(0, 0);
    }

    #[test]
    fn read() {
        let mut bootrom = BootROM{data:vec![123, 234]};
        assert_eq!(bootrom.read(1), 234);
    }
}