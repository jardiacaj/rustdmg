use super::CPU;
use super::Flags;
use crate::memory::MemoryZone;
use crate::cpu::register::DMGRegister;
use crate::cpu::register::Subregister;

macro_rules! ld_8bit_register_immediate {
    ($opcode:literal, $register:ident, $write_method:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, ",d8"),
            description: concat!("Load immediate to {}", $register_name),
            length_in_bytes: 2, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                let immediate = cpu.pop_u8_from_pc();
                cpu.$register.$write_method(immediate);
                cpu.cycle_count += 8;
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
            description: concat!("Put pointer ", $pointer_name, " in ", $register_name),
            length_in_bytes: 1, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.$register.$write_method(cpu.memory.read(cpu.$pointer.read()));
                cpu.$pointer.overflowing_add($pointer_addition);
            }
        }
    )
}


pub const INSTRUCTIONS_NOCB: [Instruction; 28] = [
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

    ld_register_pointer!(0x1A, reg_af, write_a, "A", reg_de, "DE"),

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

    ld_register_pointer!(0x2A, reg_af, write_a, "A", reg_hl, "HL", 0x0001, "+"),

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

    ld_register_pointer!(0x46, reg_bc, write_higher, "B", reg_hl, "HL"),

    ld_register_pointer!(0x4E, reg_bc, write_lower, "C", reg_hl, "HL"),

    ld_register_pointer!(0x56, reg_de, write_higher, "D", reg_hl, "HL"),

    ld_register_pointer!(0x5E, reg_de, write_lower, "E", reg_hl, "HL"),

    ld_register_pointer!(0x66, reg_hl, write_higher, "H", reg_hl, "HL"),

    ld_register_pointer!(0x6E, reg_hl, write_lower, "L", reg_hl, "HL"),

    Instruction{opcode: 0xCB, mnemonic: "CB", description: "CB prefix",
        length_in_bytes: 0, cycles: "0", flags_changed: "",
        implementation: |cpu| cpu.run_cb_op() },

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

];

pub const INSTRUCTIONS_CB: [Instruction; 1] = [

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
    fn ld_c_immediate() {
        let mut cpu = CPU::new(MemoryManager::new_from_vecs(vec![0x0E, 0xAA], vec![]));
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 2);
        assert_eq!(cpu.reg_bc.read_lower(), 0xAA);
    }
}
