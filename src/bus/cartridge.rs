use super::*;

use std::fs;
use std::io;
use std::io::Read;
use std::str;


const CARTRIDGE_TYPES: [CartridgeType; 26] = [
    CartridgeType{code: 0x00, name:"ROM only", supported: true},
    CartridgeType{code: 0x01, name:"ROM+MBC1", supported: false},
    CartridgeType{code: 0x02, name:"ROM+MBC1+RAM", supported: false},
    CartridgeType{code: 0x03, name:"ROM+MBC1+RAM+BATT", supported: false},
    CartridgeType{code: 0x05, name:"ROM+MBC2", supported: false},
    CartridgeType{code: 0x06, name:"ROM+MBC2+BATTERY", supported: false},
    CartridgeType{code: 0x08, name:"ROM+RAM", supported: false},
    CartridgeType{code: 0x09, name:"ROM+RAM+BATTERY", supported: false},
    CartridgeType{code: 0x0B, name:"ROM+MMM01", supported: false},
    CartridgeType{code: 0x0C, name:"ROM+MMM01+SRAM", supported: false},
    CartridgeType{code: 0x0D, name:"ROM+MMM01+SRAM+BATT", supported: false},
    CartridgeType{code: 0x0F, name:"ROM+MBC3+TIMER+BATT", supported: false},
    CartridgeType{code: 0x10, name:"ROM+MBC3+TIMER+RAM+BATT", supported: false},
    CartridgeType{code: 0x11, name:"ROM+MBC", supported: false},
    CartridgeType{code: 0x12, name:"ROM+MBC3+RAM", supported: false},
    CartridgeType{code: 0x13, name:"ROM+MBC3+RAM+BATT", supported: false},
    CartridgeType{code: 0x19, name:"ROM+MBC5", supported: false},
    CartridgeType{code: 0x1A, name:"ROM+MBC5+RAM", supported: false},
    CartridgeType{code: 0x1B, name:"ROM+MBC5+RAM+BATT", supported: false},
    CartridgeType{code: 0x1C, name:"ROM+MBC5+RUMBLE", supported: false},
    CartridgeType{code: 0x1D, name:"ROM+MBC5+RUMBLE+SRAM", supported: false},
    CartridgeType{code: 0x1E, name:"ROM+MBC5+RUMBLE+SRAM+BATT", supported: false},
    CartridgeType{code: 0x1F, name:"Pocket Camera", supported: false},
    CartridgeType{code: 0xFD, name:"Bandai TAMA5", supported: false},
    CartridgeType{code: 0xFE, name:"Hudson HuC-3", supported: false},
    CartridgeType{code: 0xFF, name:"Hudson HuC-1", supported: false},
];

const CARTRIDGE_ROM_SIZES: [CartridgeRomSize; 10] = [
    CartridgeRomSize {code: 0x00, name:"256Kbit", num_banks: 2},
    CartridgeRomSize {code: 0x01, name:"512Kbit", num_banks: 4},
    CartridgeRomSize {code: 0x02, name:"1Mbit", num_banks: 8},
    CartridgeRomSize {code: 0x03, name:"2Mbit", num_banks: 16},
    CartridgeRomSize {code: 0x04, name:"4Mbit", num_banks: 32},
    CartridgeRomSize {code: 0x05, name:"8Mbit", num_banks: 64},
    CartridgeRomSize {code: 0x06, name:"16Mbit", num_banks: 128},
    CartridgeRomSize {code: 0x52, name:"9Mbit", num_banks: 72},
    CartridgeRomSize {code: 0x53, name:"10Mbit", num_banks: 80},
    CartridgeRomSize {code: 0x54, name:"12Mbit", num_banks: 96},
];

pub struct CartridgeType<'a> {
    pub name: &'a str,
    pub supported: bool,
    pub code: u8,
}

pub struct CartridgeRomSize<'a> {
    pub name: &'a str,
    pub num_banks: u8,
    pub code: u8,
}

pub struct RomBank {
    pub bank_number: u8,
    pub data: Vec<u8>,
}

impl MemoryZone for RomBank {
    fn read(&self, address: u16) -> u8 {
        self.data[self.global_address_to_local_address(address) as usize]
    }
    fn write(&mut self, address: u16, value: u8) {
        let local_address = self.global_address_to_local_address(address) as usize;
        self.data[local_address] = value
    }
}

impl RomBank {
    fn global_address_to_local_address(&self, address: u16) -> u16 {
        if self.bank_number == 0 { return address; } else { return address - 0x4000; }
    }
}

pub struct Cartridge {
    pub name: String,
    pub rom_banks: Vec<RomBank>,
    blob: Vec<u8>,
}

impl Cartridge {
    pub fn new_dummy_cartridge(data: Vec<u8>) -> Cartridge {
        let rom_bank_zero = RomBank {
            bank_number: 0,
            data
        };
        Cartridge {name: "".to_string(), blob: vec![], rom_banks: vec![rom_bank_zero]}
    }

    pub fn read_cartridge_from_romfile(rom_file_path: &str) -> io::Result<Cartridge> {
        let file_metadata = fs::metadata(rom_file_path)?;

        if file_metadata.len() as usize % ROM_BANK_SIZE != 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Bad cartridge ROM file size"));
        }

        let mut file = fs::File::open(rom_file_path)?;
        let mut file_content: Vec<u8> = Vec::with_capacity(file_metadata.len() as usize);
        file.read_to_end(&mut file_content)?;
        Ok(Cartridge::parse_cartridge_from_blob(file_content)?)
    }

    fn parse_cartridge_from_blob(blob: Vec<u8>) -> io::Result<Cartridge> {
        let num_banks_in_file = blob.len() / ROM_BANK_SIZE;
        let mut rom_banks: Vec<RomBank> = Vec::with_capacity(num_banks_in_file);

        for bank_index in 0..num_banks_in_file {
            let bank_start_pos = bank_index * ROM_BANK_SIZE;
            let bank_end_pos = (bank_index + 1) * ROM_BANK_SIZE;
            rom_banks.push(
                RomBank{
                    bank_number: bank_index as u8,
                    data: blob[bank_start_pos..bank_end_pos].to_vec()
                }
            );
        }

        let name = match str::from_utf8(&blob[0x0134..0x0142]) {
            Ok(v) => v.to_string(),
            Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF8 in ROM name")),
        };

        let cartridge = Cartridge {
            blob,
            rom_banks,
            name,
        };

        let cartridge_type = cartridge.get_cartridge_type()?;
        let rom_size = cartridge.get_rom_size()?;

        println!();
        println!("==============");
        println!("Cartridge info");
        println!("Name: {}", cartridge.name);
        println!("Type : {}", cartridge_type.name);
        println!("Rom size: {} in {} banks", rom_size.name, rom_size.num_banks);
        println!("==============");

        if !cartridge_type.supported {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Cartridge type {} unsupported", cartridge_type.name)))
        }

        Ok(cartridge)
    }

    pub fn get_cartridge_type(&self) -> io::Result<&CartridgeType> {
        let type_code_in_rom = self.blob[0x0147];
        match CARTRIDGE_TYPES
            .iter()
            .find(|cart_type| cart_type.code == type_code_in_rom) {
            Some(cartridge_type) => return Ok(cartridge_type),
            None => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Cartridge type {:#02X?} unrecognized", type_code_in_rom))),
        }
    }

    pub fn get_rom_size(&self) -> io::Result<&CartridgeRomSize> {
        let type_size_in_rom = self.blob[0x0148];

        match CARTRIDGE_ROM_SIZES
            .iter()
            .find(|cart_size| cart_size.code == type_size_in_rom) {
            Some(cartridge_size) => return Ok(cartridge_size),
            None => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Cartridge size {:#02X?} unrecognized", type_size_in_rom))),
        }
    }
}