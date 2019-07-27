use super::CPU;
use super::Flags;
use crate::memory::MemoryZone;

pub const INSTRUCTIONS_NOCB: [Instruction; 10] = [
    Instruction{opcode: 0x00, mnemonic: "NOP", description: "No operation",
        length_in_bytes: 1, cycles: 4, flags_changed: "",
        implementation: |cpu| return },
    Instruction{opcode: 0x01, mnemonic: "LD BC,d16", description: "Load immediate to BC",
        length_in_bytes: 3, cycles: 12, flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x02, mnemonic: "LD (BC),A", description: "Put A to pointer BC",
        length_in_bytes: 1, cycles: 8, flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x03, mnemonic: "INC BC", description: "Increment BC",
        length_in_bytes: 1, cycles: 8, flags_changed: "",
        implementation: |cpu| panic!("Not implemented") },
    Instruction{opcode: 0x04, mnemonic: "INC B", description: "Increment B",
        length_in_bytes: 1, cycles: 4, flags_changed: "Z0H",
        implementation: |cpu| panic!("Not implemented") },


    Instruction{opcode: 0x21, mnemonic: "LD HL,d16", description: "Load immediate to HL",
        length_in_bytes: 3, cycles: 12, flags_changed: "",
        implementation: |cpu| cpu.reg_hl.value = cpu.pop_u16_from_pc() },

    Instruction{opcode: 0x31, mnemonic: "LD SP,d16", description: "Load immediate to SP",
        length_in_bytes: 3, cycles: 12, flags_changed: "",
        implementation: |cpu| cpu.stack_pointer.value = cpu.pop_u16_from_pc() },

    Instruction{opcode: 0x32, mnemonic: "LD (HL-),A", description: "Put A to pointer HL and decrement HL",
        length_in_bytes: 1, cycles: 8, flags_changed: "",
        implementation: |cpu| {
            cpu.memory.write(cpu.reg_hl.value, cpu.reg_a.value);
            cpu.reg_hl.value -= 1;
        } },

    Instruction{opcode: 0xAF, mnemonic: "XOR A", description: "XOR A with A (zeroes A)",
        length_in_bytes: 1, cycles: 4, flags_changed: "Z000",
        implementation: |cpu| { cpu.reg_a.value = 0; cpu.flags.insert(Flags::Z) } },

    Instruction{opcode: 0xCB, mnemonic: "CB", description: "CB prefix",
        length_in_bytes: 0, cycles: 0, flags_changed: "",
        implementation: |cpu| cpu.run_cb_op() },

];

pub const INSTRUCTIONS_CB: [Instruction; 1] = [

    Instruction{opcode: 0x7C, mnemonic: "BIT 7,H", description: "Test bit 7 of H",
        length_in_bytes: 2, cycles: 8, flags_changed: "Z01-",
        implementation: |cpu| {
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
    pub cycles: u8,
    pub flags_changed: &'a str,
    pub implementation: fn(&mut CPU),
}
