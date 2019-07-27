use super::CPU;
use super::Flags;
use crate::memory::MemoryZone;

pub const INSTRUCTIONS_NOCB: [Instruction; 11] = [
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


    Instruction{opcode: 0x20, mnemonic: "JR NZ,r8", description: "Jump relative if not zero",
        length_in_bytes: 2, cycles: "8 or 12", flags_changed: "",
        implementation: |cpu| {
            let jump_distance = ((cpu.pop_u8_from_pc() as i8) as u16);
            if cpu.flags.contains(Flags::Z) {
                cpu.cycle_count += 8;
            } else {
                cpu.cycle_count += 12;
                cpu.program_counter.value = cpu.program_counter.value.overflowing_add(jump_distance).0;
            }
        } },


    Instruction{opcode: 0x21, mnemonic: "LD HL,d16", description: "Load immediate to HL",
        length_in_bytes: 3, cycles: "12", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 12;
            cpu.reg_hl.value = cpu.pop_u16_from_pc();
        } },

    Instruction{opcode: 0x31, mnemonic: "LD SP,d16", description: "Load immediate to SP",
        length_in_bytes: 3, cycles: "12", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 12;
            cpu.stack_pointer.value = cpu.pop_u16_from_pc();
        } },

    Instruction{opcode: 0x32, mnemonic: "LD (HL-),A", description: "Put A to pointer HL and decrement HL",
        length_in_bytes: 1, cycles: "8", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 8;
            cpu.memory.write(cpu.reg_hl.value, cpu.reg_a.value);
            cpu.reg_hl.value -= 1;
        } },

    Instruction{opcode: 0xAF, mnemonic: "XOR A", description: "XOR A with A (zeroes A)",
        length_in_bytes: 1, cycles: "4", flags_changed: "Z000",
        implementation: |cpu| {
            cpu.cycle_count += 4;
            cpu.reg_a.value = 0; cpu.flags.insert(Flags::Z)
        } },

    Instruction{opcode: 0xCB, mnemonic: "CB", description: "CB prefix",
        length_in_bytes: 0, cycles: "0", flags_changed: "",
        implementation: |cpu| cpu.run_cb_op() },

];

pub const INSTRUCTIONS_CB: [Instruction; 1] = [

    Instruction{opcode: 0x7C, mnemonic: "BIT 7,H", description: "Test bit 7 of H",
        length_in_bytes: 2, cycles: "8", flags_changed: "Z01-",
        implementation: |cpu| {
            cpu.cycle_count += 8;
            cpu.flags.remove(Flags::N);
            cpu.flags.insert(Flags::H);
            cpu.flags.set(Flags::Z, (cpu.reg_hl.higher_8bit().value & (1 << 7)) == 0)
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

    #[test]
    fn xor_a() {
        let mut cpu = CPU::create(
            MemoryManager::new_from_vecs(vec![0xAF], vec![]));
        cpu.reg_a.value = 0x4F;
        cpu.step();
        assert_eq!(cpu.reg_a.value, 0);
        assert_eq!(cpu.flags, Flags::Z)
    }

    #[test]
    fn ld_hl_d16() {
        let mut cpu = CPU::create(
            MemoryManager::new_from_vecs(vec![0x21, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.reg_hl.value, 0x1234);
    }

    #[test]
    fn ld_sp_d16() {
        let mut cpu = CPU::create(
            MemoryManager::new_from_vecs(vec![0x31, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.stack_pointer.value, 0x1234);
    }

    #[test]
    fn ld_pointer_hl_a_and_decrement() {
        let mut cpu = CPU::create(
            MemoryManager::new_from_vecs(vec![0x32], vec![]));
        cpu.reg_a.value = 0xF0;
        cpu.reg_hl.value = 0xC123;
        cpu.step();
        assert_eq!(cpu.memory.read(0xC123), 0xF0);
        assert_eq!(cpu.reg_hl.value, 0xC122);
    }

    #[test]
    fn bit_7_h_to_one() {
        let mut cpu = CPU::create(MemoryManager::new_from_vecs(vec![0xCB, 0x7C], vec![]));
        cpu.reg_hl.value = 0xF000;
        cpu.step();
        assert!(!cpu.flags.contains(Flags::N));
        assert!(cpu.flags.contains(Flags::H));
        assert!(!cpu.flags.contains(Flags::Z));
    }

    #[test]
    fn bit_7_h_to_zero() {
        let mut cpu = CPU::create(MemoryManager::new_from_vecs(vec![0xCB, 0x7C], vec![]));
        cpu.reg_hl.value = 0x0F00;
        cpu.step();
        assert!(!cpu.flags.contains(Flags::N));
        assert!(cpu.flags.contains(Flags::H));
        assert!(cpu.flags.contains(Flags::Z));
    }

    #[test]
    fn jnz_no_jump() {
        let mut cpu = CPU::create(MemoryManager::new_from_vecs(vec![0x20, 0x33], vec![]));
        cpu.flags.insert(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.value, 0x02);
        assert_eq!(cpu.cycle_count, 8);
    }

    #[test]
    fn jnz_jump_positive() {
        let mut cpu = CPU::create(MemoryManager::new_from_vecs(vec![0x20, 0x33], vec![]));
        cpu.flags.remove(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.value, 0x35);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jnz_jump_negative() {
        // Jump -3
        let mut cpu = CPU::create(MemoryManager::new_from_vecs(vec![0x20, 0xFD], vec![]));
        cpu.flags.remove(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.value, 0xFFFF);
        assert_eq!(cpu.cycle_count, 12);
    }
}
