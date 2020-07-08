pub mod register;
pub mod instruction;

use super::bus::Bus;
use register::*;
use instruction::*;


pub struct CPU <'a> {
    pub reg_af: AFRegister,
    pub reg_bc: Register16bit,
    pub reg_de: Register16bit,
    pub reg_hl: Register16bit,
    pub stack_pointer: Register16bit,
    pub program_counter: Register16bit,
    pub bus: Bus,
    pub cycle_count: u64,
    pub instruction_vector: Vec<Instruction<'a>>, // FIXME this should be removed when all instructions are implemented
    pub cb_instruction_vector: Vec<Instruction<'a>>, // FIXME this should be removed when all instructions are implemented
    pub debug: bool,
}

impl<'a> CPU<'a> {
    pub fn new(bus: Bus) -> CPU<'a> {
        let mut instruction_vector = vec!();
        let mut cb_instruction_vector = vec!();

        for i in INSTRUCTIONS_NOCB.iter() {
            while (instruction_vector.len() as u8) < i.opcode {
                instruction_vector.push(
                    Instruction{opcode: instruction_vector.len() as u8, mnemonic: "NOT IMPLEMENTED", description: "NOT IMPLEMENTED",
                        length_in_bytes: 1, cycles: "0", flags_changed: "",
                        implementation: |_cpu| { panic!("Bad opcode!") }
                    }
                )
            }
            instruction_vector.push(i.clone());
        }

        for i in INSTRUCTIONS_CB.iter() {
            while (cb_instruction_vector.len() as u8) < i.opcode {
                cb_instruction_vector.push(
                    Instruction{opcode: cb_instruction_vector.len() as u8, mnemonic: "NOT IMPLEMENTED", description: "NOT IMPLEMENTED",
                        length_in_bytes: 1, cycles: "0", flags_changed: "",
                        implementation: |_cpu| { panic!("Bad CB opcode!") }
                    }
                )
            }
            cb_instruction_vector.push(i.clone());
        }

        CPU {
            reg_af: AFRegister::new(),
            reg_bc: Register16bit::new(),
            reg_de: Register16bit::new(),
            reg_hl: Register16bit::new(),
            stack_pointer: Register16bit::new(),
            program_counter: Register16bit::new(),
            bus,
            cycle_count: 0,
            instruction_vector,
            cb_instruction_vector,
            debug: false,
        }
    }

    fn pop_u8_from_pc(&mut self) -> u8 {
        let result = self.bus.read(self.program_counter.read());
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
        self.bus.write(self.stack_pointer.read(), value);
    }

    fn push_u16_to_stack(&mut self, value: u16) {
        self.push_u8_to_stack(value as u8);
        self.push_u8_to_stack((value >> 8) as u8);
    }

    fn pop_u8_from_stack(&mut self) -> u8 {
        let result = self.bus.read(self.stack_pointer.read());
        self.stack_pointer.overflowing_add(1);
        result
    }

    fn pop_u16_from_stack(&mut self) -> u16 {
        ((self.pop_u8_from_stack() as u16) << 8) | (self.pop_u8_from_stack() as u16)
    }

    // FIXME makes assumptions on PC
    fn print_instruction(&mut self, opcode: u8, is_cb: bool) {
        let instruction: &Instruction;
        if is_cb {
            instruction = &self.cb_instruction_vector[opcode as usize];
            print!("CB OP: {:02X}", opcode);
        } else {
            instruction = &self.instruction_vector[opcode as usize];
            print!("OP: {:02X}", opcode);
        }
        let neg_offset: u16 = match is_cb {
            true => 1,
            false => 0,
        };

        print!(" -- {}", instruction.mnemonic);

        if instruction.length_in_bytes > 1 {
            print!(" -- ");
        }
        if instruction.length_in_bytes == 3 {
            print!("{:02X}", self.bus.read(self.program_counter.read() - neg_offset + 1));
        }
        if instruction.length_in_bytes > 1 {
            print!("{:02X}", self.bus.read(self.program_counter.read() - neg_offset));
        }
        println!();
        println!("Cycle {}", self.cycle_count);
    }

    fn run_op(&mut self) {
        let opcode = self.pop_u8_from_pc();

        let instruction = &self.instruction_vector[opcode as usize];
        let implementation = instruction.implementation;
        let cycles_before_op = self.cycle_count;

        if self.debug { self.print_instruction(opcode, false) };
        implementation(self);

        for _i in cycles_before_op..self.cycle_count {
            self.bus.cycle();
        }
    }

    fn run_cb_op(&mut self) {
        let opcode = self.pop_u8_from_pc();

        let instruction = &self.cb_instruction_vector[opcode as usize];
        let implementation = instruction.implementation;

        if self.debug { self.print_instruction(opcode, true) };
        implementation(self);
    }

    pub fn step(&mut self) {
        if self.debug {
            println!();
            println!("PC: {:02X}", self.program_counter.read());
        }
        self.run_op()
    }
}
