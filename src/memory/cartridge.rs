use std::fs;
use std::io::Read;
use std::str;

const ROM_BANK_SIZE: usize = 0x4000;

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

pub struct Cartridge {
    pub name: String,
    rom_banks: Vec<Vec<u8>>,
    blob: Vec<u8>,
}

impl Cartridge {
    pub fn read_cartridge_from_romfile(rom_file_path: &str) -> Result<Cartridge, String> {
        let file_metadata = match fs::metadata(rom_file_path){
            Err(e) => return Err(e.to_string()),
            Ok(file_metadata) => file_metadata,
        };

        if file_metadata.len() as usize % ROM_BANK_SIZE != 0 {
            return Err("Bad romfile size".to_string());
        }

        let mut file = fs::File::open(rom_file_path).unwrap();
        let mut file_content: Vec<u8> = Vec::with_capacity(file_metadata.len() as usize);
        file.read_to_end(&mut file_content).unwrap();

        Ok(Cartridge::parse_cartridge_from_blob(file_content))
    }

    fn parse_cartridge_from_blob(blob: Vec<u8>) -> Cartridge {
        let num_banks_in_file = blob.len() / ROM_BANK_SIZE;
        let mut rom_banks: Vec<Vec<u8>> = Vec::with_capacity(num_banks_in_file);

        for bank_index in 0..num_banks_in_file {
            let bank_start_pos = bank_index * ROM_BANK_SIZE;
            let bank_end_pos = (bank_index + 1) * ROM_BANK_SIZE;
            rom_banks.push(blob[bank_start_pos..bank_end_pos].to_vec());
        }

        let name = match str::from_utf8(&rom_banks[0][0x0134..0x0142]) {
            Ok(v) => v.to_string(),
            Err(e) => panic!("Invalid UTF-8 sequence in rom name: {}", e),
        };

        let cartridge = Cartridge {
            blob,
            rom_banks,
            name,
        };

        let cartridge_type = cartridge.get_cartridge_type();
        let rom_size = cartridge.get_rom_size();

        println!();
        println!("==============");
        println!("Cartridge info");
        println!("Name: {}", cartridge.name);
        println!("Type : {}", cartridge_type.name);
        println!("Rom size: {} in {} banks", rom_size.name, rom_size.num_banks);
        println!("==============");

        if !cartridge_type.supported {
            panic!("Cartridge type {} unsupported", cartridge_type.name)
        }

        cartridge
    }

    pub fn get_cartridge_type(&self) -> &CartridgeType {
        let type_code = self.rom_banks[0][0x0147];

        for cartridge_type_iterator in CARTRIDGE_TYPES.iter() {
            if cartridge_type_iterator.code == type_code { return cartridge_type_iterator }
        }

        panic!("Invalid cartridge type code: {:X?}", type_code);
    }

    pub fn get_rom_size(&self) -> &CartridgeRomSize {
        let type_size = self.rom_banks[0][0x0148];

        for cartridge_size_iterator in CARTRIDGE_ROM_SIZES.iter() {
            if cartridge_size_iterator.code == type_size { return cartridge_size_iterator }
        }

        panic!("Invalid rom size code: {:X?}", type_size);
    }
}