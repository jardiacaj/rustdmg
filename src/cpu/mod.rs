pub mod register;
pub mod instruction;

use super::memory::MemoryManager;
use super::memory::MemoryZone;
use register::*;
use instruction::*;


pub struct CPU {
    pub reg_af: AFRegister,
    pub reg_bc: Register16bit,
    pub reg_de: Register16bit,
    pub reg_hl: Register16bit,
    pub stack_pointer: Register16bit,
    pub program_counter: Register16bit,
    pub memory: MemoryManager,
    pub cycle_count: u64,
}

impl CPU {
    pub fn new(memory: MemoryManager) -> CPU {
        CPU {
            reg_af: AFRegister::new(),
            reg_bc: Register16bit::new(),
            reg_de: Register16bit::new(),
            reg_hl: Register16bit::new(),
            stack_pointer: Register16bit::new(),
            program_counter: Register16bit::new(),
            memory,
            cycle_count: 0,
        }
    }

    fn pop_u8_from_pc(&mut self) -> u8 {
        let result = self.memory.read(self.program_counter.read());
        self.program_counter.inc();
        result
    }

    fn pop_u16_from_pc(&mut self) -> u16 {
        let mut result: u16;
        result = self.pop_u8_from_pc() as u16;
        result += (self.pop_u8_from_pc() as u16) << 8;
        result
    }

    fn push_u8_to_stack(&mut self, value: u8) {
        self.stack_pointer.overflowing_add(0xFFFF);
        self.memory.write(self.stack_pointer.read(), value);
    }

    fn push_u16_to_stack(&mut self, value: u16) {
        self.push_u8_to_stack(value as u8);
        self.push_u8_to_stack((value >> 8) as u8);
    }

    fn pop_u8_from_stack(&mut self) -> u8 {
        let result = self.memory.read(self.stack_pointer.read());
        self.stack_pointer.overflowing_add(1);
        result
    }

    fn pop_u16_from_stack(&mut self) -> u16 {
        ((self.pop_u8_from_stack() as u16) << 8) | (self.pop_u8_from_stack() as u16)
    }

    // FIXME makes assumptions on PC
    fn print_instruction(&mut self, instruction: &Instruction, is_cb: bool) {
        let neg_offset: u16 = match is_cb {
            true => 1,
            false => 0,
        };

        print!("  {}", instruction.mnemonic);
        if instruction.length_in_bytes > 1 {
            print!(" -- {:02X}", self.memory.read(self.program_counter.read() - neg_offset));
        }
        if instruction.length_in_bytes > 2 {
            print!("{:02X}", self.memory.read(self.program_counter.read() - neg_offset + 1));
        }
        println!();
    }

    fn run_op(&mut self) {
        let opcode = self.pop_u8_from_pc();
        print!("OP: {:02X}", opcode);

        let instruction_index = CPU::get_instruction_index_from_opcode(opcode);
        let instruction = &INSTRUCTIONS_NOCB[instruction_index];
        let implementation = instruction.implementation;
        let cycles_before_op = self.cycle_count;

        self.print_instruction(instruction, false);
        implementation(self);

        for i in (cycles_before_op..self.cycle_count) {
            self.memory.cycle();
        }
    }

    fn run_cb_op(&mut self) {
        let opcode = self.pop_u8_from_pc();
        print!("CB OP: {:02X}", opcode);

        let instruction_index = CPU::get_cb_instruction_index_from_opcode(opcode);
        let instruction = &INSTRUCTIONS_CB[instruction_index];
        let implementation = instruction.implementation;

        self.print_instruction(instruction, true);
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
        println!();
        println!("PC: {:02X}", self.program_counter.read());
        self.run_op()
    }
}
