pub mod rustdmg {
    pub mod dmg {
        use super::cartridge;

        pub fn run(rom_file_path: &str) {
            let cartridge = match cartridge::Cartridge::read_cartridge_from_romfile(rom_file_path) {
                Err(e) => panic!("Failed to read cartridge"),
                Ok(cartridge) => cartridge
            };
        }
    }
    mod cartridge {
        use std::fs;
        use std::io;
        use std::io::Read;
        use std::str;
        use std::collections::HashMap;

        const ROM_BANK_SIZE: usize = 0x4000;

        pub struct CartridgeType {
            pub name: String,
            pub supported: bool,
        }

        pub struct RomSize {
            pub name: String,
            pub num_banks: u8,
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
                let mut file_content: Vec<u8> = Vec::new();
                file.read_to_end(&mut file_content).unwrap();

                Ok(Cartridge::parse_cartridge_from_blob(file_content))
            }

            fn parse_cartridge_from_blob(blob: Vec<u8>) -> Cartridge {
                let mut rom_banks: Vec<Vec<u8>> = Vec::new();
                let num_banks_in_file = blob.len() / ROM_BANK_SIZE;
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

            pub fn get_cartridge_type(&self) -> CartridgeType {
                match self.rom_banks[0][0x0147] {
                    0x00 => CartridgeType{name: "ROM only".to_string(), supported: true},
                    0x01 => CartridgeType{name:"ROM+MBC1".to_string(), supported: false},
                    0x02 => CartridgeType{name:"ROM+MBC1+RAM".to_string(), supported: false},
                    0x03 => CartridgeType{name:"ROM+MBC1+RAM+BATT".to_string(), supported: false},
                    0x05 => CartridgeType{name:"ROM+MBC2".to_string(), supported: false},
                    0x06 => CartridgeType{name:"ROM+MBC2+BATTERY".to_string(), supported: false},
                    0x08 => CartridgeType{name:"ROM+RAM".to_string(), supported: false},
                    0x09 => CartridgeType{name:"ROM+RAM+BATTERY".to_string(), supported: false},
                    0x0B => CartridgeType{name:"ROM+MMM01".to_string(), supported: false},
                    0x0C => CartridgeType{name:"ROM+MMM01+SRAM".to_string(), supported: false},
                    0x0D => CartridgeType{name:"ROM+MMM01+SRAM+BATT".to_string(), supported: false},
                    0x0F => CartridgeType{name:"ROM+MBC3+TIMER+BATT".to_string(), supported: false},
                    0x10 => CartridgeType{name:"ROM+MBC3+TIMER+RAM+BATT".to_string(), supported: false},
                    0x11 => CartridgeType{name:"ROM+MBC".to_string(), supported: false},
                    0x12 => CartridgeType{name:"ROM+MBC3+RAM".to_string(), supported: false},
                    0x13 => CartridgeType{name:"ROM+MBC3+RAM+BATT".to_string(), supported: false},
                    0x19 => CartridgeType{name:"ROM+MBC5".to_string(), supported: false},
                    0x1A => CartridgeType{name:"ROM+MBC5+RAM".to_string(), supported: false},
                    0x1B => CartridgeType{name:"ROM+MBC5+RAM+BATT".to_string(), supported: false},
                    0x1C => CartridgeType{name:"ROM+MBC5+RUMBLE".to_string(), supported: false},
                    0x1D => CartridgeType{name:"ROM+MBC5+RUMBLE+SRAM".to_string(), supported: false},
                    0x1E => CartridgeType{name:"ROM+MBC5+RUMBLE+SRAM+BATT".to_string(), supported: false},
                    0x1F => CartridgeType{name:"Pocket Camera".to_string(), supported: false},
                    0xFD => CartridgeType{name:"Bandai TAMA5".to_string(), supported: false},
                    0xFE => CartridgeType{name:"Hudson HuC-3".to_string(), supported: false},
                    0xFF => CartridgeType{name:"Hudson HuC-1".to_string(), supported: false},
                    _ => panic!("Invalid cartridge type"),
                }
            }

            pub fn get_rom_size(&self) -> RomSize {
                match self.rom_banks[0][0x0148] {
                    0x00 => RomSize{name:"256Kbit".to_string(), num_banks: 2},
                    0x01 => RomSize{name:"512Kbit".to_string(), num_banks: 4},
                    0x02 => RomSize{name:"1Mbit".to_string(), num_banks: 8},
                    0x03 => RomSize{name:"2Mbit".to_string(), num_banks: 16},
                    0x04 => RomSize{name:"4Mbit".to_string(), num_banks: 32},
                    0x05 => RomSize{name:"8Mbit".to_string(), num_banks: 64},
                    0x06 => RomSize{name:"16Mbit".to_string(), num_banks: 128},
                    0x52 => RomSize{name:"9Mbit".to_string(), num_banks: 72},
                    0x53 => RomSize{name:"10Mbit".to_string(), num_banks: 80},
                    0x54 => RomSize{name:"12Mbit".to_string(), num_banks: 96},
                    _ => panic!("Invalid rom size"),
                }
            }
        }
    }

    mod memory {
        use super::cartridge::Cartridge;

        struct Memory {
            ordered_zones: Vec<MemoryZone>,
            rom_bank_fixed: MemoryZone,
            rom_bank_switchable: MemoryZone,
            vram: MemoryZone,
            cartridge_ram: MemoryZone,
            work_ram_fixed: MemoryZone,
            work_ram_switchable: MemoryZone,
            work_ram_echo: MemoryZone,
            oam: MemoryZone,
            not_usable: MemoryZone,
            io_ram: MemoryZone,
            hi_ram: MemoryZone,
            interrupt_enable_register: MemoryZone,
        }

        impl Memory {
            fn create(cartridge: Cartridge) {
                //                Memory {
                //                    rom_bank_fixed: MemoryZone {
                //                        name: "Fixed ROM bank",
                //                        offset: 0,
                //                        size: 0x4000
                //                    }
                //                }
            }
        }

        struct MemoryZone {
            name: String,
            offset: u16,
            size: u16,
        }
    }

}
