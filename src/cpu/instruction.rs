use super::CPU;
use super::Flags;
use crate::memory::MemoryZone;
use crate::cpu::register::DMGRegister;
use crate::cpu::register::Subregister;

macro_rules! ld_8bit_register_immediate {
    ($opcode:literal, $register:ident, $subregister:expr, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, ",d8"),
            description: concat!("Load immediate to {}", $register_name),
            length_in_bytes: 2, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                let immediate = cpu.pop_u8_from_pc();
                cpu.$register.write_subreg($subregister, immediate);
                cpu.cycle_count += 8;
            }
        }
    )
}

pub const INSTRUCTIONS_NOCB: [Instruction; 17] = [
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
            let target_value = cpu.reg_bc.read_subreg(Subregister::Lower).overflowing_add(1).0;
            cpu.reg_bc.write_subreg(
                Subregister::Lower,
                target_value
            );
            cpu.reg_af.flags.remove(Flags::N);
            cpu.reg_af.flags.set(Flags::Z, target_value == 0);
            cpu.reg_af.flags.set(Flags::H, target_value & 0x0F == 0);
        } },

    ld_8bit_register_immediate!(0x0E, reg_bc, Subregister::Lower, "C"),

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


    Instruction{opcode: 0x21, mnemonic: "LD HL,d16", description: "Load immediate to HL",
        length_in_bytes: 3, cycles: "12", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 12;
            let immediate = cpu.pop_u16_from_pc();
            cpu.reg_hl.write(immediate);
        } },

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

    ld_8bit_register_immediate!(0x3E, reg_af, Subregister::Higher, "A"),

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
            let address = 0xFF00 + (cpu.reg_bc.read_subreg(Subregister::Lower) as u16);
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
            cpu.reg_af.flags.set(Flags::Z, (cpu.reg_hl.read_subreg(Subregister::Higher) & (1 << 7)) == 0)
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
        cpu.reg_bc.write_subreg(Subregister::Lower, 0x4F);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_subreg(Subregister::Lower), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
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
        cpu.reg_bc.write_subreg(Subregister::Lower, 0x0F);
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
        assert_eq!(cpu.reg_bc.read_subreg(Subregister::Lower), 0xAA);
    }
}
