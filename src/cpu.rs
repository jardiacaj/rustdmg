use super::memory::Memory;

pub struct CPU {
    flag_carry: bool,
    flag_half_carry: bool,
    flag_negative: bool,
    flag_zero: bool,
    reg_a: u8,
    reg_bc: u16,
    reg_de: u16,
    reg_hl: u16,
    stack_pointer: u16,
    program_counter: u16,
    memory: Memory,
}

impl CPU {
    pub fn create(memory: Memory) -> CPU {
        CPU{
            flag_carry: false,
            flag_half_carry: false,
            flag_negative: false,
            flag_zero: false,
            reg_a: 0,
            reg_bc: 0,
            reg_de: 0,
            reg_hl: 0,
            stack_pointer: 0,
            program_counter: 0,
            memory,
        }
    }

    fn pop_u8_from_pc(&mut self) -> u8 {
        let result = self.memory.read(self.program_counter);
        self.program_counter += 1;
        result
    }

    fn pop_u16_from_pc(&mut self) -> u16 {
        let mut result: u16;
        result = self.pop_u8_from_pc() as u16;
        result += (self.pop_u8_from_pc() as u16) << 8;
        result
    }

    fn run_op(&mut self) {
        let op = self.pop_u8_from_pc();
        println!("OP: {:X?}", op);
        match op {
            0x31 => self.ld_sp_d16(),
            _ => panic!("Bad opcode {:X?}", op),
        }
    }

    pub fn step(&mut self) {
        println!("PC: {:X?}", self.program_counter);
        self.run_op()
    }

    fn ld_sp_d16(&mut self) { self.stack_pointer = self.pop_u16_from_pc(); }
}