use super::CPU;

pub const INSTRUCTIONS_NOCB: [Instruction; 8] = [
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


    Instruction{opcode: 0x21, mnemonic: "LD HL,d16", description: "Load immediate to HL", is_cb: false,
        length_in_bytes: 3, cycles: 12, flags_changed: "",
        implementation: |cpu| cpu.reg_hl.value = cpu.pop_u16_from_pc() },

    Instruction{opcode: 0x31, mnemonic: "LD SP,d16", description: "Load immediate to SP", is_cb: false,
        length_in_bytes: 3, cycles: 12, flags_changed: "",
        implementation: |cpu| cpu.stack_pointer.value = cpu.pop_u16_from_pc() },

    Instruction{opcode: 0xAF, mnemonic: "XOR A", description: "XOR A with A (zeroes A)", is_cb: false,
        length_in_bytes: 1, cycles: 4, flags_changed: "Z000",
        implementation: |cpu| cpu.reg_a.value = 0 },

];

pub struct Instruction <'a> {
    pub is_cb: bool,
    pub opcode: u8,
    pub mnemonic: &'a str,
    pub description: &'a str,
    pub length_in_bytes: u8,
    pub cycles: u8,
    pub flags_changed: &'a str,
    pub implementation: fn(&mut CPU),
}
