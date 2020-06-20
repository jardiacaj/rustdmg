use super::CPU;
use super::Flags;
use crate::memory::MemoryZone;
use crate::cpu::register::DMGRegister;
use crate::cpu::register::Subregister;

macro_rules! push {
    ($opcode:literal, $register:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("PUSH ", $register_name),
            description: concat!("Push ", $register_name),
            length_in_bytes: 1, cycles: "16", flags_changed: "",
            implementation: |cpu| {
                cpu.push_u16_to_stack(cpu.$register.read());
                cpu.cycle_count += 16;
            }
        }
    )
}

macro_rules! ld_8bit_register_immediate {
    ($opcode:literal, $register:ident, $write_method:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, ",d8"),
            description: concat!("Load immediate to ", $register_name),
            length_in_bytes: 2, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                let immediate = cpu.pop_u8_from_pc();
                cpu.$register.$write_method(immediate);
                cpu.cycle_count += 8;
            }
        }
    )
}

macro_rules! ld_8bit_register_register {
    ($opcode:literal,
     $register_dest:ident, $write_method_dest:ident, $register_name_dest:expr,
     $register_orig:ident, $read_method_orig:ident, $register_name_orig:expr
    ) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("LD ", $register_name_dest, ",", $register_name_orig),
            description: concat!("Load ", $register_name_orig, " to ", $register_name_dest),
            length_in_bytes: 1, cycles: "4", flags_changed: "",
            implementation: |cpu| {
                cpu.$register_dest.$write_method_dest(cpu.$register_orig.$read_method_orig());
                cpu.cycle_count += 4;
            }
        }
    )
}

macro_rules! ld_16bit_register_immediate {
    ($opcode:literal, $register:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, ",d16"),
            description: concat!("Load immediate to ", $register_name),
            length_in_bytes: 3, cycles: "12", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 12;
                let immediate = cpu.pop_u16_from_pc();
                cpu.$register.write(immediate);
            }
        }
    )
}

macro_rules! ld_register_pointer {
    ($opcode: literal,
     $register:ident, $write_method:ident, $register_name:expr,
     $pointer:ident, $pointer_name:expr) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, " (", $pointer_name, ")"),
            description: concat!("Put pointer ", $pointer_name, " in ", $register_name),
            length_in_bytes: 1, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.$register.$write_method(cpu.memory.read(cpu.$pointer.read()));
            }
        }
    );
    ($opcode: literal,
     $register:ident, $write_method:ident, $register_name:expr,
     $pointer:ident, $pointer_name:expr,
     $pointer_addition: literal, $pointer_addition_symbol:expr) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, " (", $pointer_name, $pointer_addition_symbol, ")"),
            description: concat!("Put pointer ", $pointer_name, " in ", $register_name, $pointer_addition_symbol),
            length_in_bytes: 1, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.$register.$write_method(cpu.memory.read(cpu.$pointer.read()));
                cpu.$pointer.overflowing_add($pointer_addition);
            }
        }
    )
}


macro_rules! rotate_left_trough_carry {
    ($opcode: literal,
     $register:ident, $read_method:ident, $write_method:ident, $register_name:expr, regular) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("RL ", $register_name),
            description: concat!("Rotate ", $register_name, " left trough carry"),
            length_in_bytes: 2, cycles: "8", flags_changed: "Z00C",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.reg_af.flags.remove(Flags::N);
                cpu.reg_af.flags.remove(Flags::H);
                let set_carry = (cpu.$register.$read_method() & 0b10000000) != 0;
                let mut new_value = cpu.$register.$read_method() << 1;
                if cpu.reg_af.flags.contains(Flags::C) { new_value += 1; }
                cpu.$register.$write_method(new_value);
                cpu.reg_af.flags.set(Flags::C, set_carry);
                cpu.reg_af.flags.set(Flags::Z, new_value == 0);
            }
        }
    );
    ($opcode: literal,
     $register:ident, $read_method:ident, $write_method:ident, $register_name:expr, fast) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("RL", $register_name),
            description: concat!("Rotate ", $register_name, " left trough carry (fast)"),
            length_in_bytes: 1, cycles: "4", flags_changed: "000C",
            implementation: |cpu| {
                cpu.cycle_count += 4;
                let set_carry = (cpu.$register.$read_method() & 0b10000000) != 0;
                let mut new_value = cpu.$register.$read_method() << 1;
                if cpu.reg_af.flags.contains(Flags::C) { new_value += 1; }
                cpu.$register.$write_method(new_value);
                cpu.reg_af.flags.clear();
                cpu.reg_af.flags.set(Flags::C, set_carry);
            }
        }
    )
}


pub const INSTRUCTIONS_NOCB: [Instruction; 81] = [
    Instruction{opcode: 0x00, mnemonic: "NOP", description: "No operation",
        length_in_bytes: 1, cycles: "4", flags_changed: "",
        implementation: |cpu| cpu.cycle_count += 4 },
    Instruction{opcode: 0x01, mnemonic: "LD BC,d16", description: "Load immediate to BC",
        length_in_bytes: 3, cycles: "12", flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x02, mnemonic: "LD (BC),A", description: "Put A to pointer BC",
        length_in_bytes: 1, cycles: "8", flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x03, mnemonic: "INC BC", description: "Increment BC",
        length_in_bytes: 1, cycles: "8", flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x04, mnemonic: "INC B", description: "Increment B",
        length_in_bytes: 1, cycles: "4", flags_changed: "Z0H",
        implementation: |cpu| panic!("Not implemented") },

    ld_8bit_register_immediate!(0x06, reg_bc, write_higher, "B"),

    Instruction{opcode: 0x0C, mnemonic: "INC C", description: "Increment C",
        length_in_bytes: 1, cycles: "4", flags_changed: "Z0H-",
        implementation: |cpu| {
            cpu.cycle_count += 4;
            let target_value = cpu.reg_bc.read_lower().overflowing_add(1).0;
            cpu.reg_bc.write_lower(target_value);
            cpu.reg_af.flags.remove(Flags::N);
            cpu.reg_af.flags.set(Flags::Z, target_value == 0);
            cpu.reg_af.flags.set(Flags::H, target_value & 0x0F == 0);
        } },

    ld_register_pointer!(0x0A, reg_af, write_a, "A", reg_bc, "BC"),

    ld_8bit_register_immediate!(0x0E, reg_bc, write_lower, "C"),
    ld_16bit_register_immediate!(0x11, reg_de, "DE"),
    ld_8bit_register_immediate!(0x16, reg_de, write_higher, "D"),
    rotate_left_trough_carry!(0x17, reg_af, read_higher, write_higher, "A", fast),
    ld_register_pointer!(0x1A, reg_af, write_a, "A", reg_de, "DE"),
    ld_8bit_register_immediate!(0x1E, reg_de, write_lower, "E"),

    Instruction{opcode: 0x20, mnemonic: "JR NZ,r8", description: "Jump relative if not zero",
        length_in_bytes: 2, cycles: "8 or 12", flags_changed: "",
        implementation: |cpu| {
            let jump_distance = ((cpu.pop_u8_from_pc() as i8) as u16);
            if cpu.reg_af.flags.contains(Flags::Z) {
                cpu.cycle_count += 8;
            } else {
                cpu.cycle_count += 12;
                cpu.program_counter.overflowing_add(jump_distance);
            }
        } },

    ld_16bit_register_immediate!(0x21, reg_hl, "HL"),
    ld_8bit_register_immediate!(0x26, reg_hl, write_higher, "H"),

    ld_register_pointer!(0x2A, reg_af, write_a, "A", reg_hl, "HL", 0x0001, "+"),
    ld_8bit_register_immediate!(0x2E, reg_hl, write_lower, "L"),

    Instruction{opcode: 0x31, mnemonic: "LD SP,d16", description: "Load immediate to SP",
        length_in_bytes: 3, cycles: "12", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 12;
            let immediate = cpu.pop_u16_from_pc();
            cpu.stack_pointer.write(immediate);
        } },

    Instruction{opcode: 0x32, mnemonic: "LD (HL-),A", description: "Put A to pointer HL and decrement HL",
        length_in_bytes: 1, cycles: "8", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 8;
            cpu.memory.write(cpu.reg_hl.read(), cpu.reg_af.read_a());
            cpu.reg_hl.overflowing_add(0xFFFF);
        } },

    ld_register_pointer!(0x3A, reg_af, write_a, "A", reg_hl, "HL", 0xFFFF, "-"),

    ld_8bit_register_immediate!(0x3E, reg_af, write_higher, "A"),

    ld_8bit_register_register!(0x40, reg_bc, write_higher, "B",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x41, reg_bc, write_higher, "B",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x42, reg_bc, write_higher, "B",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x43, reg_bc, write_higher, "B",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x44, reg_bc, write_higher, "B",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x45, reg_bc, write_higher, "B",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x46, reg_bc, write_higher, "B", reg_hl, "HL"),
    ld_8bit_register_register!(0x47, reg_bc, write_higher, "B",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x48, reg_bc, write_lower, "C",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x49, reg_bc, write_lower, "C",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x4A, reg_bc, write_lower, "C",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x4B, reg_bc, write_lower, "C",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x4C, reg_bc, write_lower, "C",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x4D, reg_bc, write_lower, "C",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x4E, reg_bc, write_lower, "C", reg_hl, "HL"),
    ld_8bit_register_register!(0x4F, reg_bc, write_lower, "C",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x50, reg_de, write_higher, "D",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x51, reg_de, write_higher, "D",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x52, reg_de, write_higher, "D",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x53, reg_de, write_higher, "D",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x54, reg_de, write_higher, "D",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x55, reg_de, write_higher, "D",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x56, reg_de, write_higher, "D", reg_hl, "HL"),
    ld_8bit_register_register!(0x57, reg_de, write_higher, "D",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x58, reg_de, write_lower, "E",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x59, reg_de, write_lower, "E",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x5A, reg_de, write_lower, "E",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x5B, reg_de, write_lower, "E",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x5C, reg_de, write_lower, "E",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x5D, reg_de, write_lower, "E",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x5E, reg_de, write_lower, "E", reg_hl, "HL"),
    ld_8bit_register_register!(0x5F, reg_de, write_lower, "E",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x60, reg_hl, write_higher, "H",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x61, reg_hl, write_higher, "H",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x62, reg_hl, write_higher, "H",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x63, reg_hl, write_higher, "H",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x64, reg_hl, write_higher, "H",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x65, reg_hl, write_higher, "H",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x66, reg_hl, write_higher, "H", reg_hl, "HL"),
    ld_8bit_register_register!(0x67, reg_hl, write_higher, "H",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x68, reg_hl, write_lower, "L",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x69, reg_hl, write_lower, "L",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x6A, reg_hl, write_lower, "L",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x6B, reg_hl, write_lower, "L",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x6C, reg_hl, write_lower, "L",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x6D, reg_hl, write_lower, "L",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x6E, reg_hl, write_lower, "L", reg_hl, "HL"),
    ld_8bit_register_register!(0x6F, reg_hl, write_lower, "L",  reg_af, read_higher, "A"),

    Instruction{opcode: 0x77, mnemonic: "LD (HL),A", description: "Put A to pointer HL",
        length_in_bytes: 1, cycles: "8", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 8;
            cpu.memory.write(cpu.reg_hl.read(), cpu.reg_af.read_a());
        } },

    Instruction{opcode: 0xAF, mnemonic: "XOR A", description: "XOR A with A (zeroes A)",
        length_in_bytes: 1, cycles: "4", flags_changed: "Z000",
        implementation: |cpu| {
            cpu.cycle_count += 4;
            cpu.reg_af.write_a(0);
            cpu.reg_af.flags.insert(Flags::Z);
        } },

    push!(0xC5, reg_bc, "BC"),

    Instruction{opcode: 0xCB, mnemonic: "CB", description: "CB prefix",
        length_in_bytes: 0, cycles: "0", flags_changed: "",
        implementation: |cpu| cpu.run_cb_op() },

    Instruction{opcode: 0xCD, mnemonic: "CALL", description: "Call",
        length_in_bytes: 3, cycles: "24", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 24;
            let new_pc = cpu.pop_u16_from_pc();
            cpu.push_u16_to_stack(cpu.program_counter.read());
            cpu.program_counter.write(new_pc);
        } },

    push!(0xD5, reg_de, "DE"),

    Instruction{opcode: 0xE0, mnemonic: "LD ($FF00+imm), A", description: "Put A to pointer 0xFF00 + immediate",
        length_in_bytes: 2, cycles: "12", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 12;
            let address = 0xFF00 + (cpu.pop_u8_from_pc() as u16);
            cpu.memory.write(address, cpu.reg_af.read_a());
        } },

    Instruction{opcode: 0xE2, mnemonic: "LD ($FF00+C), A", description: "Put A to pointer 0xFF00 + C",
        length_in_bytes: 1, cycles: "8", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 8;
            let address = 0xFF00 + (cpu.reg_bc.read_lower() as u16);
            cpu.memory.write(address, cpu.reg_af.read_a());
        } },

    push!(0xE5, reg_hl, "HL"),
    push!(0xF5, reg_af, "AF"),
];

pub const INSTRUCTIONS_CB: [Instruction; 8] = [

    rotate_left_trough_carry!(0x10, reg_bc, read_higher, write_higher, "B", regular),
    rotate_left_trough_carry!(0x11, reg_bc, read_lower, write_lower, "C", regular),
    rotate_left_trough_carry!(0x12, reg_de, read_higher, write_higher, "D", regular),
    rotate_left_trough_carry!(0x13, reg_de, read_lower, write_lower, "E", regular),
    rotate_left_trough_carry!(0x14, reg_hl, read_higher, write_higher, "H", regular),
    rotate_left_trough_carry!(0x15, reg_hl, read_lower, write_lower, "L", regular),

    rotate_left_trough_carry!(0x17, reg_af, read_lower, write_lower, "A", regular),


    Instruction{opcode: 0x7C, mnemonic: "BIT 7,H", description: "Test bit 7 of H",
        length_in_bytes: 2, cycles: "8", flags_changed: "Z01-",
        implementation: |cpu| {
            cpu.cycle_count += 8;
            cpu.reg_af.flags.remove(Flags::N);
            cpu.reg_af.flags.insert(Flags::H);
            cpu.reg_af.flags.set(Flags::Z, (cpu.reg_hl.read_higher() & (1 << 7)) == 0)
        } },

];

pub struct Instruction <'a> {
    pub opcode: u8,
    pub mnemonic: &'a str,
    pub description: &'a str,
    pub length_in_bytes: u8,
    pub cycles: &'a str,
    pub flags_changed: &'a str,
    pub implementation: fn(&mut CPU),
}

#[cfg(test)]
mod tests {
    use super::CPU;
    use super::Flags;
    use crate::memory::MemoryManager;
    use crate::memory::MemoryZone;
    use crate::cpu::register::DMGRegister;
    use crate::cpu::register::Subregister;
    use bitflags::_core::num::FpCategory::Subnormal;

    #[test]
    fn xor_a() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0xAF], vec![]));
        cpu.reg_af.write_a(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0);
        assert_eq!(cpu.reg_af.flags, Flags::Z)
    }

    #[test]
    fn inc_c() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x0C], vec![]));
        cpu.reg_bc.write_lower(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_lower(), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn ld_de_d16() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x11, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.reg_de.read(), 0x1234);
    }

    #[test]
    fn ld_hl_d16() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x21, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.reg_hl.read(), 0x1234);
    }

    #[test]
    fn ld_sp_d16() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x31, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.stack_pointer.read(), 0x1234);
    }

    #[test]
    fn ld_pointer_hl_a_and_decrement() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x32], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.memory.read(0xC123), 0xF0);
        assert_eq!(cpu.reg_hl.read(), 0xC122);
    }

    #[test]
    fn ld_pointer_hl_a() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x77], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.memory.read(0xC123), 0xF0);
    }

    #[test]
    fn ld_a_pointer_de() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x1A, 0x55], vec![]));
        cpu.reg_de.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0x55);
    }

    #[test]
    fn ld_a_pointer_bc() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x0A, 0x55], vec![]));
        cpu.reg_bc.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0x55);
    }

    #[test]
    fn ld_a_pointer_hl_increment() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x2A, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0x55);
        assert_eq!(cpu.reg_hl.read(), 0x0002);
    }

    #[test]
    fn ld_a_pointer_hl_decrement() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x3A, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0x55);
        assert_eq!(cpu.reg_hl.read(), 0x0000);
    }

    #[test]
    fn ld_b_pointer_hl() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x46, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0x55);
    }

    #[test]
    fn ld_c_pointer_hl() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x4E, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_lower(), 0x55);
    }

    #[test]
    fn ld_d_pointer_hl() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x56, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_de.read_higher(), 0x55);
    }

    #[test]
    fn ld_e_pointer_hl() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x5E, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_de.read_lower(), 0x55);
    }

    #[test]
    fn ld_h_pointer_hl() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x66, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_hl.read_higher(), 0x55);
    }

    #[test]
    fn ld_l_pointer_hl() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0x6E, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_hl.read_lower(), 0x55);
    }

    #[test]
    fn ld_high_immediate_a() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0xE0, 0x45], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.step();
        assert_eq!(cpu.cycle_count, 12);
        assert_eq!(cpu.memory.read(0xFF45), 0xF0);
    }

    #[test]
    fn ld_pointer_c_a() {
        let mut cpu = CPU::new(
            MemoryManager::new_from_vecs(vec![0xE2], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.reg_bc.write_lower(0x0F);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.memory.read(0xFF0F), 0xF0);
    }

    #[test]
    fn bit_7_h_to_one() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xCB, 0x7C], vec![]));
        cpu.reg_hl.write(0xF000);
        cpu.step();
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
    }

    #[test]
    fn bit_7_h_to_zero() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xCB, 0x7C], vec![]));
        cpu.reg_hl.write(0x0F00);
        cpu.step();
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
        assert!(cpu.reg_af.flags.contains(Flags::Z));
    }

    #[test]
    fn jnz_no_jump() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x20, 0x33], vec![]));
        cpu.reg_af.flags.insert(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x02);
        assert_eq!(cpu.cycle_count, 8);
    }

    #[test]
    fn jnz_jump_positive() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x20, 0x33], vec![]));
        cpu.reg_af.flags.remove(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x35);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jnz_jump_negative() {
        // Jump -3
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x20, 0xFD], vec![]));
        cpu.reg_af.flags.remove(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0xFFFF);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn call() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xCD, 0x34, 0x12], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.step();
        assert_eq!(cpu.cycle_count, 24);
        assert_eq!(cpu.program_counter.read(), 0x1234);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.memory.read(0xCFFF), 0x03);
        assert_eq!(cpu.memory.read(0xCFFE), 0x00);
    }

    #[test]
    fn ld_b_b() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x40], vec![]));
        cpu.reg_bc.write_higher(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5);
    }

    #[test]
    fn ld_b_c() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x41], vec![]));
        cpu.reg_bc.write_lower(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5);
    }

    #[test]
    fn ld_b_d() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x42], vec![]));
        cpu.reg_de.write_higher(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5);
    }

    #[test]
    fn ld_b_e() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x43], vec![]));
        cpu.reg_de.write_lower(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5);
    }

    #[test]
    fn ld_b_h() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x44], vec![]));
        cpu.reg_hl.write_higher(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5)
    }

    #[test]
    fn ld_b_l() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x45], vec![]));
        cpu.reg_hl.write_lower(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5)
    }

    #[test]
    fn ld_b_a() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x47], vec![]));
        cpu.reg_af.write_higher(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5)
    }

    #[test]
    fn ld_b_immediate() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x06, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_bc.read_higher(), 0xBB);
    }

    #[test]
    fn ld_c_immediate() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x0E, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_bc.read_lower(), 0xBB);
    }

    #[test]
    fn ld_d_immediate() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x16, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_de.read_higher(), 0xBB);
    }

    #[test]
    fn ld_e_immediate() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x1E, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_de.read_lower(), 0xBB);
    }

    #[test]
    fn ld_h_immediate() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x26, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_hl.read_higher(), 0xBB);
    }

    #[test]
    fn ld_l_immediate() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x2E, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_hl.read_lower(), 0xBB);
    }

    #[test]
    fn ld_a_immediate() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x3E, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_af.read_higher(), 0xBB);
    }

    #[test]
    fn push_bc() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xC5], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.reg_bc.write(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.memory.read(0xCFFF), 0x34);
        assert_eq!(cpu.memory.read(0xCFFE), 0x12);
    }

    #[test]
    fn push_de() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xD5], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.reg_de.write(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.memory.read(0xCFFF), 0x34);
        assert_eq!(cpu.memory.read(0xCFFE), 0x12);
    }

    #[test]
    fn push_hl() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xE5], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.reg_hl.write(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.memory.read(0xCFFF), 0x34);
        assert_eq!(cpu.memory.read(0xCFFE), 0x12);
    }

    #[test]
    fn push_af() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xF5], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.reg_af.write(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.memory.read(0xCFFF), 0x34);
        assert_eq!(cpu.memory.read(0xCFFE), 0x12);
    }

    #[test]
    fn rl_c_no_carry() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xCB, 0x11], vec![]));
        cpu.reg_bc.write_lower(0b01010010);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_bc.read_lower(), 0b10100100);
        assert_eq!(cpu.reg_af.flags, Flags::empty());
    }

    #[test]
    fn rl_c_to_carry() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xCB, 0x11], vec![]));
        cpu.reg_bc.write_lower(0b11010010);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_bc.read_lower(), 0b10100100);
        assert_eq!(cpu.reg_af.flags, Flags::C);
    }

    #[test]
    fn rl_c_from_carry() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0xCB, 0x11], vec![]));
        cpu.reg_bc.write_lower(0);
        cpu.reg_af.flags.insert(Flags::C);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_bc.read_lower(), 1);
        assert_eq!(cpu.reg_af.flags, Flags::empty());
    }

    #[test]
    fn rla_no_carry() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x17], vec![]));
        cpu.reg_af.write_higher(0b01010010);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.read_higher(), 0b10100100);
        assert_eq!(cpu.reg_af.flags, Flags::empty());
    }

    #[test]
    fn rla_to_carry() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x17], vec![]));
        cpu.reg_af.write_higher(0b11010010);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.read_higher(), 0b10100100);
        assert_eq!(cpu.reg_af.flags, Flags::C);
    }

    #[test]
    fn rla_from_carry() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x17], vec![]));
        cpu.reg_af.write_higher(0);
        cpu.reg_af.flags.insert(Flags::C);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.read_higher(), 1);
        assert_eq!(cpu.reg_af.flags, Flags::empty());
    }

}
