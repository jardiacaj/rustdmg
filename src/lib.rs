pub mod rustdmg {
    pub mod dmg {
        use super::cartridge::Cartridge;
        use super::memory;
        use super::memory::Memory;
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
    }

    mod cpu {
        use super::memory::Memory;

        pub struct CPU {
            flag_carry: bool,
            flag_half_carry: bool,
            flag_negative: bool,
            flag_zero: bool,
            reg_a: u8,
            reg_bc: u16,
            reg_de: u16,
            reg_hl: u16,
            stack_pointer: u16,
            program_counter: u16,
            memory: Memory,
        }

        impl CPU {
            pub fn create(memory: Memory) -> CPU {
                CPU{
                    flag_carry: false,
                    flag_half_carry: false,
                    flag_negative: false,
                    flag_zero: false,
                    reg_a: 0,
                    reg_bc: 0,
                    reg_de: 0,
                    reg_hl: 0,
                    stack_pointer: 0,
                    program_counter: 0,
                    memory,
                }
            }

            fn pop_op_from_pc(&mut self) -> u8 {
                let pc = self.program_counter;
                self.program_counter += 1;
                self.memory.read(pc)
            }

            fn run_op(&mut self) {
                let op = self.pop_op_from_pc();
                println!("OP: {:X?}", op);
                match self.pop_op_from_pc() {
                    0x31 => ld_sp_d16(),
                    _ => panic!("Bad opcode {:X?}", op),
                }
            }

            pub fn step(&mut self) {
                println!("PC: {:X?}", self.program_counter);
                self.run_op()
            }
        }
    }

    mod cartridge {
        use std::fs;
        use std::io::Read;
        use std::str;

        const ROM_BANK_SIZE: usize = 0x4000;

        pub struct CartridgeType {
            pub name: String,
            pub supported: bool,
        }

        pub struct CartridgeRomSize {
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

            pub fn get_rom_size(&self) -> CartridgeRomSize {
                match self.rom_banks[0][0x0148] {
                    0x00 => CartridgeRomSize {name:"256Kbit".to_string(), num_banks: 2},
                    0x01 => CartridgeRomSize {name:"512Kbit".to_string(), num_banks: 4},
                    0x02 => CartridgeRomSize {name:"1Mbit".to_string(), num_banks: 8},
                    0x03 => CartridgeRomSize {name:"2Mbit".to_string(), num_banks: 16},
                    0x04 => CartridgeRomSize {name:"4Mbit".to_string(), num_banks: 32},
                    0x05 => CartridgeRomSize {name:"8Mbit".to_string(), num_banks: 64},
                    0x06 => CartridgeRomSize {name:"16Mbit".to_string(), num_banks: 128},
                    0x52 => CartridgeRomSize {name:"9Mbit".to_string(), num_banks: 72},
                    0x53 => CartridgeRomSize {name:"10Mbit".to_string(), num_banks: 80},
                    0x54 => CartridgeRomSize {name:"12Mbit".to_string(), num_banks: 96},
                    _ => panic!("Invalid rom size"),
                }
            }
        }
    }

    mod memory {
        use std::fs;
        use std::io::Read;
        use super::cartridge::Cartridge;

        const BOOT_ROM_SIZE: usize = 256;

        pub fn read_boot_rom_from_romfile(boot_rom_file_path: &str) -> Result<MemoryZone, String> {
            let file_metadata = match fs::metadata(boot_rom_file_path) {
                Err(e) => return Err(e.to_string()),
                Ok(file_metadata) => file_metadata,
            };

            if file_metadata.len() as usize != BOOT_ROM_SIZE {
                return Err("Bad boot ROM file size".to_string());
            }

            let mut file = fs::File::open(boot_rom_file_path).expect("Failed to open boot rom");
            let mut data: Vec<u8> = Vec::new();
            file.read_to_end(&mut data).unwrap();

            Ok(
                MemoryZone{
                    name: "Boot ROM".to_string(),
                    data,
                    offset: 0,
                    size: BOOT_ROM_SIZE as u16,
                }
            )
        }

        pub struct Memory {
            pub boot_rom: MemoryZone,
            pub cartridge: Cartridge,
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
            pub fn create(boot_rom: MemoryZone, cartridge: Cartridge) -> Memory {
                Memory {
                    boot_rom,
                    cartridge
                }
            }

            pub fn read(&self, address: u16) -> u8 {
                self.boot_rom.data[address as usize]
            }

            pub fn write(&mut self, address: u16, value: u8) {
                self.boot_rom.data[address as usize] = value;
            }

        }

        pub struct MemoryZone {
            name: String,
            offset: u16,
            size: u16,
            data: Vec<u8>,
        }
    }

}
