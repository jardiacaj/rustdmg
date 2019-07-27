pub mod register;
pub mod instruction;

use super::memory::Memory;
use super::memory::MemoryZone;
use register::*;
use instruction::*;
use bitflags::bitflags;

bitflags! {
    pub struct Flags: u8 {
        const Z = 0b10000000;
        const N = 0b01000000;
        const H = 0b00100000;
        const C = 0b00010000;
    }
}

pub struct CPU {
    flags: Flags,
    reg_a: Register8bit,
    reg_bc: Register16bit,
    reg_de: Register16bit,
    reg_hl: Register16bit,
    stack_pointer: Register16bit,
    program_counter: Register16bit,
    memory: Memory,
}

impl CPU {
    pub fn create(memory: Memory) -> CPU {
        CPU{
            flags: Flags{bits: 0},
            reg_a: Register8bit{value: 0},
            reg_bc: Register16bit{value: 0},
            reg_de: Register16bit{value: 0},
            reg_hl: Register16bit{value: 0},
            stack_pointer: Register16bit{value: 0},
            program_counter: Register16bit{value: 0},
            memory,
        }
    }

    fn pop_u8_from_pc(&mut self) -> u8 {
        let result = self.memory.read(self.program_counter.value);
        self.program_counter.value += 1;
        result
    }

    fn pop_u16_from_pc(&mut self) -> u16 {
        let mut result: u16;
        result = self.pop_u8_from_pc() as u16;
        result += (self.pop_u8_from_pc() as u16) << 8;
        result
    }

    fn run_op(&mut self) {
        let opcode = self.pop_u8_from_pc();
        println!("OP: {:X?}", opcode);

        let instruction_index = CPU::get_instruction_index_from_opcode(opcode);
        let implementation = INSTRUCTIONS_NOCB[instruction_index].implementation;
        implementation(self);
    }

    fn run_cb_op(&mut self) {
        let opcode = self.pop_u8_from_pc();
        println!("CB OP: {:X?}", opcode);

        let instruction_index = CPU::get_cb_instruction_index_from_opcode(opcode);
        let implementation = INSTRUCTIONS_CB[instruction_index].implementation;
        implementation(self);
    }

    /// FIXME this should go away once all instructions are implemented, then opcodes will be
    /// array indexes
    fn get_instruction_index_from_opcode(opcode: u8) -> usize {
        match INSTRUCTIONS_NOCB
            .iter()
            .enumerate()
            .find(|enumerated_instruction| enumerated_instruction.1.opcode == opcode) {
            Some(enumerated_instruction) => return enumerated_instruction.0,
            None => panic!("Bad opcode 0x{:X?}", opcode),
        }
    }

    /// FIXME this should go away once all instructions are implemented, then opcodes will be
    /// array indexes
    fn get_cb_instruction_index_from_opcode(opcode: u8) -> usize {
        match INSTRUCTIONS_CB
            .iter()
            .enumerate()
            .find(|enumerated_instruction| enumerated_instruction.1.opcode == opcode) {
            Some(enumerated_instruction) => return enumerated_instruction.0,
            None => panic!("Bad opcode 0x{:X?}", opcode),
        }
    }

    pub fn step(&mut self) {
        println!("PC: {:X?}", self.program_counter.value);
        self.run_op()
    }
}

#[cfg(test)]
mod tests {
    use super::CPU;
    use super::Flags;
    use crate::memory::Memory;
    use crate::memory::MemoryZone;

    #[test]
    fn xor_a() {
        let mut cpu = CPU::create(
            Memory::new_from_vecs(vec![0xAF], vec![]));
        cpu.reg_a.value = 0x4F;
        cpu.step();
        assert_eq!(cpu.reg_a.value, 0);
        assert_eq!(cpu.flags, Flags::Z)
    }

    #[test]
    fn ld_hl_d16() {
        let mut cpu = CPU::create(
            Memory::new_from_vecs(vec![0x21, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.reg_hl.value, 0x1234);
    }

    #[test]
    fn ld_sp_d16() {
        let mut cpu = CPU::create(
            Memory::new_from_vecs(vec![0x31, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.stack_pointer.value, 0x1234);
    }

    #[test]
    fn ld_pointer_hl_a_and_decrement() {
        let mut cpu = CPU::create(
            Memory::new_from_vecs(vec![0x32], vec![]));
        cpu.reg_a.value = 0xF0;
        cpu.reg_hl.value = 0xC123;
        cpu.step();
        assert_eq!(cpu.memory.read(0xC123), 0xF0);
        assert_eq!(cpu.reg_hl.value, 0xC122);
    }

    #[test]
    fn bit_7_h_to_one() {
        let mut cpu = CPU::create(Memory::new_from_vecs(vec![0xCB, 0x7C], vec![]));
        cpu.reg_hl.value = 0xF000;
        cpu.step();
        assert!(!cpu.flags.contains(Flags::N));
        assert!(cpu.flags.contains(Flags::H));
        assert!(!cpu.flags.contains(Flags::Z));
    }

    #[test]
    fn bit_7_h_to_zero() {
        let mut cpu = CPU::create(Memory::new_from_vecs(vec![0xCB, 0x7C], vec![]));
        cpu.reg_hl.value = 0x0F00;
        cpu.step();
        assert!(!cpu.flags.contains(Flags::N));
        assert!(cpu.flags.contains(Flags::H));
        assert!(cpu.flags.contains(Flags::Z));
    }
}
