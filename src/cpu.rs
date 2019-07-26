use super::memory::Memory;

const INSTRUCTIONS_NOCB: [Instruction; 7] = [
    Instruction{opcode: 0x00, mnemonic: "NOP", description: "No operation", is_cb: false,
        length_in_bytes: 1, cycles: 4, flags_changed: "",
        implementation: |cpu| return },
    Instruction{opcode: 0x01, mnemonic: "LD BC,d16", description: "Load immediate to BC", is_cb: false,
        length_in_bytes: 3, cycles: 12, flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x02, mnemonic: "LD (BC),A", description: "Put A to pointer BC", is_cb: false,
        length_in_bytes: 1, cycles: 8, flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x03, mnemonic: "INC BC", description: "Increment BC", is_cb: false,
        length_in_bytes: 1, cycles: 8, flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x04, mnemonic: "INC B", description: "Increment B", is_cb: false,
        length_in_bytes: 1, cycles: 4, flags_changed: "Z0H",
        implementation: |cpu| panic!("Not implemented") },


    Instruction{opcode: 0x31, mnemonic: "LD SP,d16", description: "Load immediate to SP", is_cb: false,
        length_in_bytes: 3, cycles: 12, flags_changed: "",
        implementation: |cpu| cpu.stack_pointer = cpu.pop_u16_from_pc() },

    Instruction{opcode: 0xAF, mnemonic: "XOR A", description: "XOR A with A (zeroes A)", is_cb: false,
        length_in_bytes: 1, cycles: 4, flags_changed: "Z000",
        implementation: |cpu| cpu.reg_a = 0 },

];

pub struct Instruction <'a>
{
    is_cb: bool,
    opcode: u8,
    mnemonic: &'a str,
    description: &'a str,
    length_in_bytes: u8,
    cycles: u8,
    flags_changed: &'a str,
    implementation: fn(&mut CPU),
}

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
        let opcode = self.pop_u8_from_pc();
        println!("OP: {:X?}", opcode);

        let instruction_index = CPU::get_instruction_index_from_opcode(opcode);
        let implementation = INSTRUCTIONS_NOCB[instruction_index].implementation;
        implementation(self);
    }

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
        println!("PC: {:X?}", self.program_counter);
        self.run_op()
    }
}