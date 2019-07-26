pub mod register;
pub mod instruction;

use super::memory::Memory;
use super::memory::MemoryZone;
use register::*;
use instruction::*;

pub struct CPU {
    flag_carry: bool,
    flag_half_carry: bool,
    flag_negative: bool,
    flag_zero: bool,
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
            flag_carry: false,
            flag_half_carry: false,
            flag_negative: false,
            flag_zero: false,
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

    pub fn step(&mut self) {
        println!("PC: {:X?}", self.program_counter.value);
        self.run_op()
    }
}

#[cfg(test)]
mod tests {
    use super::CPU;
    use super::super::memory::Memory;

    #[test]
    fn xor_a() {
        let mut cpu = CPU::create(Memory::new_from_vec(vec![0xAF]));
        cpu.reg_a.value = 0x4F;
        cpu.step();
        assert_eq!(cpu.reg_a.value, 0);
    }

    #[test]
    fn ld_hl_d16() {
        let mut cpu = CPU::create(Memory::new_from_vec(vec![0x21, 0x34, 0x12]));
        cpu.step();
        assert_eq!(cpu.reg_hl.value, 0x1234);
    }

    #[test]
    fn ld_sp_d16() {
        let mut cpu = CPU::create(Memory::new_from_vec(vec![0x31, 0x34, 0x12]));
        cpu.step();
        assert_eq!(cpu.stack_pointer.value, 0x1234);
    }
}
