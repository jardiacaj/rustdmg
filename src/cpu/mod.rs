pub mod register;
pub mod instruction;

use super::memory::MemoryManager;
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
    memory: MemoryManager,
    cycle_count: u64,
}

impl CPU {
    pub fn create(memory: MemoryManager) -> CPU {
        CPU {
            flags: Flags{bits: 0},
            reg_a: Register8bit{value: 0},
            reg_bc: Register16bit{value: 0},
            reg_de: Register16bit{value: 0},
            reg_hl: Register16bit{value: 0},
            stack_pointer: Register16bit{value: 0},
            program_counter: Register16bit{value: 0},
            memory,
            cycle_count: 0,
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
        println!("OP: {:02X}", opcode);

        let instruction_index = CPU::get_instruction_index_from_opcode(opcode);
        let implementation = INSTRUCTIONS_NOCB[instruction_index].implementation;
        implementation(self);
    }

    fn run_cb_op(&mut self) {
        let opcode = self.pop_u8_from_pc();
        println!("CB OP: {:02X}", opcode);

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
            None => panic!("Bad opcode {:#02X?}", opcode),
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
            None => panic!("Bad opcode {:#02X?}", opcode),
        }
    }

    pub fn step(&mut self) {
        println!("PC: {:02X}", self.program_counter.value);
        self.run_op()
    }
}
