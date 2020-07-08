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
    reg_instruction: u8,
    reg_instruction_is_cb: bool,
    instruction_address: u16,
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
                        implementation: |cpu| { cpu.print_instruction(); panic!("Bad opcode!") }
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
                        implementation: |cpu| { cpu.print_instruction(); panic!("Bad CB opcode!") }
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
            reg_instruction: 0,
            reg_instruction_is_cb: false,
            instruction_address: 0,
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
    fn print_instruction(&mut self) {
        let instruction: &Instruction;
        if self.reg_instruction_is_cb {
            instruction = &self.cb_instruction_vector[self.reg_instruction as usize];
            print!("CB OP: {:02X}", instruction.opcode);
        } else {
            instruction = &self.instruction_vector[self.reg_instruction as usize];
            print!("OP: {:02X}", instruction.opcode);
        }
        let data_address_offset: u16 = match self.reg_instruction_is_cb {
            true => 1,
            false => 0,
        };

        print!(" -- {}", instruction.mnemonic);

        if instruction.length_in_bytes > 1 {
            print!(" -- ");
        }
        if instruction.length_in_bytes == 3 {
            print!("{:02X}", self.bus.read(self.program_counter.read() + data_address_offset + 1));
        }
        if instruction.length_in_bytes > 1 {
            print!("{:02X}", self.bus.read(self.program_counter.read() + data_address_offset));
        }
        println!();
        println!("Cycle {}", self.cycle_count);
    }

    fn run_op(&mut self) {
        self.instruction_address = self.program_counter.read();
        self.reg_instruction = self.pop_u8_from_pc();
        self.reg_instruction_is_cb = false;

        let instruction = &self.instruction_vector[self.reg_instruction as usize];
        let implementation = instruction.implementation;
        let cycles_before_op = self.cycle_count;

        if self.debug { self.print_instruction() };
        implementation(self);

        for _i in cycles_before_op..self.cycle_count {
            self.bus.cycle();
        }
    }

    fn run_cb_op(&mut self) {
        self.reg_instruction = self.pop_u8_from_pc();
        self.reg_instruction_is_cb = true;

        let instruction = &self.cb_instruction_vector[self.reg_instruction as usize];
        let implementation = instruction.implementation;

        if self.debug { self.print_instruction() };
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

#[cfg(test)]
mod tests {
    use super::CPU;
    use super::Flags;
    use crate::bus::Bus;
    use crate::cpu::register::DMGRegister;

    #[test]
    fn cpu_internal_registers() {
        // XOR A
        // BIT 7,H
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0xAF, 0xCB, 0x7C], vec![]));
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.instruction_address, 0x0000);
        assert_eq!(cpu.reg_instruction_is_cb, false);
        assert_eq!(cpu.reg_instruction, 0xAF);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x0003);
        assert_eq!(cpu.instruction_address, 0x0001);
        assert_eq!(cpu.reg_instruction_is_cb, true);
        assert_eq!(cpu.reg_instruction, 0x7C);
    }

}