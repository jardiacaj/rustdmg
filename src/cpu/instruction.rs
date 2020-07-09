use super::CPU;
use super::Flags;
use crate::cpu::register::DMGRegister;

#[derive(Clone)]
pub struct Instruction <'a> {
    pub opcode: u8,
    pub mnemonic: &'a str,
    pub description: &'a str,
    pub length_in_bytes: u8,
    pub cycles: &'a str,
    pub flags_changed: &'a str,
    pub implementation: fn(&mut CPU),
}

macro_rules! push {
    ($opcode:literal, $register:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("PUSH ", $register_name),
            description: concat!("Push ", $register_name),
            length_in_bytes: 1, cycles: "16", flags_changed: "",
            implementation: |cpu| {
                cpu.push_u16_to_stack(cpu.$register.read());
                cpu.cycle_count += 16;
            }
        }
    )
}

macro_rules! pop {
    ($opcode:literal, $register:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("Pop ", $register_name),
            description: concat!("Pop ", $register_name),
            length_in_bytes: 1, cycles: "12", flags_changed: "",
            implementation: |cpu| {
                let popped_value = cpu.pop_u16_from_stack();
                cpu.$register.write(popped_value);
                cpu.cycle_count += 12;
            }
        }
    )
}

macro_rules! inc_u8 {
    ($opcode:literal, $register:ident, $write_method:ident, $read_method:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("INC ", $register_name),
            description: concat!("Increment ", $register_name),
            length_in_bytes: 1, cycles: "4", flags_changed: "Z0H-",
            implementation: |cpu| {
                let target_value = cpu.$register.$read_method().overflowing_add(1).0;
                cpu.$register.$write_method(target_value);
                cpu.reg_af.flags.remove(Flags::N);
                cpu.reg_af.flags.set(Flags::Z, target_value == 0);
                cpu.reg_af.flags.set(Flags::H, target_value & 0x0F == 0);
                cpu.cycle_count += 4;
            }
        }
    )
}

macro_rules! inc_u16 {
    ($opcode:literal, $register:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("INC ", $register_name),
            description: concat!("Increment ", $register_name),
            length_in_bytes: 1, cycles: "8", flags_changed: "----",
            implementation: |cpu| {
                cpu.$register.overflowing_add(1);
                cpu.cycle_count += 8;
            }
        }
    )
}

macro_rules! dec_u8 {
    ($opcode:literal, $register:ident, $write_method:ident, $read_method:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("DEC ", $register_name),
            description: concat!("Decrement ", $register_name),
            length_in_bytes: 1, cycles: "4", flags_changed: "Z1H-",
            implementation: |cpu| {
                let target_value = cpu.$register.$read_method().overflowing_add(0xFF).0;
                cpu.$register.$write_method(target_value);
                cpu.reg_af.flags.insert(Flags::N);
                cpu.reg_af.flags.set(Flags::Z, target_value == 0);
                cpu.reg_af.flags.set(Flags::H, target_value & 0x0F == 0x0F);
                cpu.cycle_count += 4;
            }
        }
    )
}

macro_rules! dec_u16 {
    ($opcode:literal, $register:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("DEC ", $register_name),
            description: concat!("Decrement ", $register_name),
            length_in_bytes: 1, cycles: "8", flags_changed: "----",
            implementation: |cpu| {
                cpu.$register.overflowing_add(0xFFFF);
                cpu.cycle_count += 8;
            }
        }
    )
}

macro_rules! ld_8bit_register_immediate {
    ($opcode:literal, $register:ident, $write_method:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, ",d8"),
            description: concat!("Load immediate to ", $register_name),
            length_in_bytes: 2, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                let immediate = cpu.pop_u8_from_pc();
                cpu.$register.$write_method(immediate);
                cpu.cycle_count += 8;
            }
        }
    )
}

macro_rules! ld_8bit_register_register {
    ($opcode:literal,
     $register_dest:ident, $write_method_dest:ident, $register_name_dest:expr,
     $register_orig:ident, $read_method_orig:ident, $register_name_orig:expr
    ) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("LD ", $register_name_dest, ",", $register_name_orig),
            description: concat!("Load ", $register_name_orig, " to ", $register_name_dest),
            length_in_bytes: 1, cycles: "4", flags_changed: "",
            implementation: |cpu| {
                cpu.$register_dest.$write_method_dest(cpu.$register_orig.$read_method_orig());
                cpu.cycle_count += 4;
            }
        }
    )
}

macro_rules! ld_16bit_register_immediate {
    ($opcode:literal, $register:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, ",d16"),
            description: concat!("Load immediate to ", $register_name),
            length_in_bytes: 3, cycles: "12", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 12;
                let immediate = cpu.pop_u16_from_pc();
                cpu.$register.write(immediate);
            }
        }
    )
}

macro_rules! ld_register_pointer {
    ($opcode: literal,
     $register:ident, $write_method:ident, $register_name:expr,
     $pointer:ident, $pointer_name:expr) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, " (", $pointer_name, ")"),
            description: concat!("Put pointer ", $pointer_name, " in ", $register_name),
            length_in_bytes: 1, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.$register.$write_method(cpu.bus.read(cpu.$pointer.read()));
            }
        }
    );
    ($opcode: literal,
     $register:ident, $write_method:ident, $register_name:expr,
     $pointer:ident, $pointer_name:expr,
     $pointer_addition: literal, $pointer_addition_symbol:expr) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("LD ", $register_name, " (", $pointer_name, $pointer_addition_symbol, ")"),
            description: concat!("Put pointer ", $pointer_name, " in ", $register_name, $pointer_addition_symbol),
            length_in_bytes: 1, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.$register.$write_method(cpu.bus.read(cpu.$pointer.read()));
                cpu.$pointer.overflowing_add($pointer_addition);
            }
        }
    )
}

macro_rules! ld_pointer_register {
    ($opcode: literal,
     $pointer:ident, $pointer_name:expr,
     $register:ident, $read_method:ident, $register_name:expr) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("LD (", $pointer_name, ") ", $register_name),
            description: concat!("Put ", $register_name, " in pointer ", $pointer_name),
            length_in_bytes: 1, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.bus.write(cpu.$pointer.read(), cpu.$register.$read_method());
            }
        }
    );
    ($opcode: literal,
     $pointer:ident, $pointer_name:expr,
     $register:ident, $read_method:ident, $register_name:expr,
     $pointer_addition: literal, $pointer_addition_symbol:expr) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("LD (", $pointer_name, $pointer_addition_symbol, ") ", $register_name),
            description: concat!("Put ", $register_name, " in pointer ", $pointer_name, " and ", $pointer_addition_symbol),
            length_in_bytes: 1, cycles: "8", flags_changed: "",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.bus.write(cpu.$pointer.read(), cpu.$register.$read_method());
                cpu.$pointer.overflowing_add($pointer_addition);
            }
        }
    )
}


macro_rules! rotate_left_trough_carry {
    ($opcode: literal,
     $register:ident, $read_method:ident, $write_method:ident, $register_name:expr, regular) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("RL ", $register_name),
            description: concat!("Rotate ", $register_name, " left trough carry"),
            length_in_bytes: 2, cycles: "8", flags_changed: "Z00C",
            implementation: |cpu| {
                cpu.cycle_count += 8;
                cpu.reg_af.flags.remove(Flags::N);
                cpu.reg_af.flags.remove(Flags::H);
                let set_carry = (cpu.$register.$read_method() & 0b10000000) != 0;
                let mut new_value = cpu.$register.$read_method() << 1;
                if cpu.reg_af.flags.contains(Flags::C) { new_value += 1; }
                cpu.$register.$write_method(new_value);
                cpu.reg_af.flags.set(Flags::C, set_carry);
                cpu.reg_af.flags.set(Flags::Z, new_value == 0);
            }
        }
    );
    ($opcode: literal,
     $register:ident, $read_method:ident, $write_method:ident, $register_name:expr, fast) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("RL", $register_name),
            description: concat!("Rotate ", $register_name, " left trough carry (fast)"),
            length_in_bytes: 1, cycles: "4", flags_changed: "000C",
            implementation: |cpu| {
                cpu.cycle_count += 4;
                let set_carry = (cpu.$register.$read_method() & 0b10000000) != 0;
                let mut new_value = cpu.$register.$read_method() << 1;
                if cpu.reg_af.flags.contains(Flags::C) { new_value += 1; }
                cpu.$register.$write_method(new_value);
                cpu.reg_af.flags.clear();
                cpu.reg_af.flags.set(Flags::C, set_carry);
            }
        }
    )
}

macro_rules! jump_relative {
    ($opcode: literal, $flag:expr, $true_or_false:literal, $condition_text:literal) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("JR ", $condition_text, ", r8"),
            description: concat!("Jump relative if ", $condition_text),
            length_in_bytes: 2, cycles: "12/8", flags_changed: "----",
            implementation: |cpu| {
                let jump_distance = cpu.pop_u8_from_pc() as i8;
                if cpu.reg_af.flags.contains($flag) == $true_or_false {
                    cpu.cycle_count += 12;
                    cpu.program_counter.overflowing_add(i16::from(jump_distance) as u16);
                } else {
                    cpu.cycle_count += 8;
                }
            }
        }
    );
    ($opcode: literal) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("JR r8"),
            description: concat!("Jump relative"),
            length_in_bytes: 2, cycles: "12", flags_changed: "----",
            implementation: |cpu| {
                let jump_distance = cpu.pop_u8_from_pc() as i8;
                cpu.cycle_count += 12;
                cpu.program_counter.overflowing_add(i16::from(jump_distance) as u16);
            }
        }
    )
}

macro_rules! jump {
    ($opcode: literal, $flag:expr, $true_or_false:literal, $condition_text:literal) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("JP ", $condition_text, ", d16"),
            description: concat!("Jump if ", $condition_text),
            length_in_bytes: 3, cycles: "12", flags_changed: "----",
            implementation: |cpu| {
                let jump_address = cpu.pop_u16_from_pc();
                if cpu.reg_af.flags.contains($flag) == $true_or_false {
                    cpu.cycle_count += 16;
                    cpu.program_counter.write(jump_address);
                } else {
                    cpu.cycle_count += 12;
                }
            }
        }
    );
    ($opcode: literal) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("JP d16"),
            description: concat!("Jump"),
            length_in_bytes: 3, cycles: "12", flags_changed: "----",
            implementation: |cpu| {
                let jump_address = cpu.pop_u16_from_pc();
                cpu.cycle_count += 12;
                cpu.program_counter.write(jump_address);
            }
        }
    );
    ($opcode: literal, hl) => (
        Instruction{opcode: $opcode,
            mnemonic: concat!("JP (HL))"),
            description: concat!("Jump (HL)"),
            length_in_bytes: 1, cycles: "4", flags_changed: "----",
            implementation: |cpu| {
                cpu.cycle_count += 4;
                cpu.program_counter.write(cpu.bus.read(cpu.reg_hl.read()) as u16);
            }
        }
    )
}

fn set_cpu_flags_for_add(cpu: &mut CPU, value: u8) {
    let a = cpu.reg_af.read_a();
    cpu.reg_af.flags.set(Flags::Z, a.overflowing_add(value).0 == 0);
    cpu.reg_af.flags.set(Flags::C, a.overflowing_add(value).1);
    cpu.reg_af.flags.set(Flags::H, (a & 0x0F) + (value & 0x0F) > 0x0F);
    cpu.reg_af.flags.remove(Flags::N);
}

macro_rules! add {
    ($opcode:literal, $register:ident, $read_method:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("ADD ", $register_name),
            description: concat!("Add ", $register_name, " to A"),
            length_in_bytes: 1, cycles: "4", flags_changed: "Z0HC",
            implementation: |cpu| {
                let addend = cpu.$register.$read_method();
                set_cpu_flags_for_add(cpu, addend);
                let (target_value, _carried) = cpu.reg_af.read_a().overflowing_add(addend);
                cpu.reg_af.write_a(target_value);
                cpu.cycle_count += 4;
            }
        }
    );
    ($opcode:literal, hl) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("ADD (HL)"),
            description: concat!("ADD (HL) to A"),
            length_in_bytes: 1, cycles: "8", flags_changed: "Z0HC",
            implementation: |cpu| {
                let addend = cpu.bus.read(cpu.reg_hl.read());
                set_cpu_flags_for_add(cpu, addend);
                let (target_value, _carried) = cpu.reg_af.read_a().overflowing_add(addend);
                cpu.reg_af.write_a(target_value);
                cpu.cycle_count += 8;
            }
        }
    );
    ($opcode:literal, immediate) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("ADD d8"),
            description: concat!("Add immediate to A"),
            length_in_bytes: 2, cycles: "8", flags_changed: "Z0HC",
            implementation: |cpu| {
                let addend = cpu.pop_u8_from_pc();
                set_cpu_flags_for_add(cpu, addend);
                let (target_value, _carried) = cpu.reg_af.read_a().overflowing_add(addend);
                cpu.reg_af.write_a(target_value);
                cpu.cycle_count += 8;
            }
        }
    )
}

fn set_cpu_flags_for_sub_or_cp(cpu: &mut CPU, value: u8) {
    let a = cpu.reg_af.read_a();
    cpu.reg_af.flags.set(Flags::Z, a == value);
    cpu.reg_af.flags.set(Flags::C, a < value);
    cpu.reg_af.flags.set(Flags::H, (a & 0x0F) < (value & 0x0F));
    cpu.reg_af.flags.insert(Flags::N);
}

macro_rules! sub {
    ($opcode:literal, $register:ident, $read_method:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("SUB ", $register_name),
            description: concat!("Substract ", $register_name, " from A"),
            length_in_bytes: 1, cycles: "4", flags_changed: "Z1HC",
            implementation: |cpu| {
                let subtrahend = cpu.$register.$read_method();
                set_cpu_flags_for_sub_or_cp(cpu, subtrahend);
                let (target_value, _carried) = cpu.reg_af.read_a().overflowing_sub(subtrahend);
                cpu.reg_af.write_a(target_value);
                cpu.cycle_count += 4;
            }
        }
    );
    ($opcode:literal, hl) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("SUB (HL)"),
            description: concat!("Substract (HL) from A"),
            length_in_bytes: 1, cycles: "8", flags_changed: "Z1HC",
            implementation: |cpu| {
                let subtrahend = cpu.bus.read(cpu.reg_hl.read());
                set_cpu_flags_for_sub_or_cp(cpu, subtrahend);
                let (target_value, _carried) = cpu.reg_af.read_a().overflowing_sub(subtrahend);
                cpu.reg_af.write_a(target_value);
                cpu.cycle_count += 8;
            }
        }
    );
    ($opcode:literal, immediate) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("SUB d8"),
            description: concat!("Substract immediate from A"),
            length_in_bytes: 2, cycles: "8", flags_changed: "Z1HC",
            implementation: |cpu| {
                let subtrahend = cpu.pop_u8_from_pc();
                set_cpu_flags_for_sub_or_cp(cpu, subtrahend);
                let (target_value, _carried) = cpu.reg_af.read_a().overflowing_sub(subtrahend);
                cpu.reg_af.write_a(target_value);
                cpu.cycle_count += 8;
            }
        }
    )
}

macro_rules! cp {
    ($opcode:literal, $register:ident, $read_method:ident, $register_name:expr) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("CP ", $register_name),
            description: concat!("Compare ", $register_name, " with A"),
            length_in_bytes: 1, cycles: "4", flags_changed: "Z1HC",
            implementation: |cpu| {
                let subtrahend = cpu.$register.$read_method();
                set_cpu_flags_for_sub_or_cp(cpu, subtrahend);
                cpu.cycle_count += 4;
            }
        }
    );
    ($opcode:literal, hl) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("CP (HL)"),
            description: concat!("Compare (HL) with A"),
            length_in_bytes: 1, cycles: "8", flags_changed: "Z1HC",
            implementation: |cpu| {
                let subtrahend = cpu.bus.read(cpu.reg_hl.read());
                set_cpu_flags_for_sub_or_cp(cpu, subtrahend);
                cpu.cycle_count += 8;
            }
        }
    );
    ($opcode:literal, immediate) => (
        Instruction{
            opcode: $opcode,
            mnemonic: concat!("CP d8"),
            description: concat!("Compare immediate with A"),
            length_in_bytes: 2, cycles: "8", flags_changed: "Z1HC",
            implementation: |cpu| {
                let subtrahend = cpu.pop_u8_from_pc();
                set_cpu_flags_for_sub_or_cp(cpu, subtrahend);
                cpu.cycle_count += 8;
            }
        }
    )
}


pub const INSTRUCTIONS_NOCB: [Instruction; 162] = [
    Instruction{opcode: 0x00, mnemonic: "NOP", description: "No operation",
        length_in_bytes: 1, cycles: "4", flags_changed: "",
        implementation: |cpu| cpu.cycle_count += 4 },
    Instruction{opcode: 0x01, mnemonic: "LD BC,d16", description: "Load immediate to BC",
        length_in_bytes: 3, cycles: "12", flags_changed: "",
        implementation: |_cpu| panic!("Not implemented") },
    ld_pointer_register!(0x02, reg_bc, "BC", reg_af, read_higher, "A"),
    inc_u16!(0x03, reg_bc, "BC"),
    inc_u8!(0x04, reg_bc, write_higher, read_higher, "B"),
    dec_u8!(0x05, reg_bc, write_higher, read_higher, "B"),

    ld_8bit_register_immediate!(0x06, reg_bc, write_higher, "B"),

    ld_register_pointer!(0x0A, reg_af, write_a, "A", reg_bc, "BC"),
    dec_u16!(0x0B, reg_bc, "BC"),
    inc_u8!(0x0C, reg_bc, write_lower, read_lower, "C"),
    dec_u8!(0x0D, reg_bc, write_lower, read_lower, "C"),
    ld_8bit_register_immediate!(0x0E, reg_bc, write_lower, "C"),
    ld_16bit_register_immediate!(0x11, reg_de, "DE"),
    ld_pointer_register!(0x12, reg_de, "DE", reg_af, read_higher, "A"),
    inc_u16!(0x13, reg_de, "DE"),
    inc_u8!(0x14, reg_de, write_higher, read_higher, "D"),
    dec_u8!(0x15, reg_de, write_higher, read_higher, "D"),
    ld_8bit_register_immediate!(0x16, reg_de, write_higher, "D"),
    rotate_left_trough_carry!(0x17, reg_af, read_higher, write_higher, "A", fast),
    jump_relative!(0x18),
    ld_register_pointer!(0x1A, reg_af, write_a, "A", reg_de, "DE"),
    dec_u16!(0x1B, reg_de, "DE"),
    inc_u8!(0x1C, reg_de, write_lower, read_lower, "E"),
    dec_u8!(0x1D, reg_de, write_lower, read_lower, "E"),
    ld_8bit_register_immediate!(0x1E, reg_de, write_lower, "E"),

    jump_relative!(0x20, Flags::Z, false, "NZ"),

    ld_16bit_register_immediate!(0x21, reg_hl, "HL"),
    ld_pointer_register!(0x22, reg_hl, "HL", reg_af, read_higher, "A", 0x0001, "+"),
    inc_u16!(0x23, reg_hl, "HL"),
    inc_u8!(0x24, reg_hl, write_higher, read_higher, "H"),
    dec_u8!(0x25, reg_hl, write_higher, read_higher, "H"),
    ld_8bit_register_immediate!(0x26, reg_hl, write_higher, "H"),

    jump_relative!(0x28, Flags::Z, true, "Z"),

    ld_register_pointer!(0x2A, reg_af, write_a, "A", reg_hl, "HL", 0x0001, "+"),
    dec_u16!(0x2B, reg_hl, "HL"),
    inc_u8!(0x2C, reg_hl, write_lower, read_lower, "L"),
    dec_u8!(0x2D, reg_hl, write_lower, read_lower, "L"),
    ld_8bit_register_immediate!(0x2E, reg_hl, write_lower, "L"),

    jump_relative!(0x30, Flags::C, false, "NC"),

    Instruction{opcode: 0x31, mnemonic: "LD SP,d16", description: "Load immediate to SP",
        length_in_bytes: 3, cycles: "12", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 12;
            let immediate = cpu.pop_u16_from_pc();
            cpu.stack_pointer.write(immediate);
        } },

    ld_pointer_register!(0x32, reg_hl, "HL", reg_af, read_higher, "A", 0xFFFF, "-"),
    inc_u16!(0x33, stack_pointer, "SP"),

    jump_relative!(0x38, Flags::C, true, "C"),

    ld_register_pointer!(0x3A, reg_af, write_a, "A", reg_hl, "HL", 0xFFFF, "-"),
    dec_u16!(0x3B, stack_pointer, "SP"),
    inc_u8!(0x3C, reg_af, write_higher, read_higher, "A"),
    dec_u8!(0x3D, reg_af, write_higher, read_higher, "A"),

    ld_8bit_register_immediate!(0x3E, reg_af, write_higher, "A"),

    ld_8bit_register_register!(0x40, reg_bc, write_higher, "B",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x41, reg_bc, write_higher, "B",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x42, reg_bc, write_higher, "B",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x43, reg_bc, write_higher, "B",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x44, reg_bc, write_higher, "B",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x45, reg_bc, write_higher, "B",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x46, reg_bc, write_higher, "B", reg_hl, "HL"),
    ld_8bit_register_register!(0x47, reg_bc, write_higher, "B",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x48, reg_bc, write_lower, "C",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x49, reg_bc, write_lower, "C",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x4A, reg_bc, write_lower, "C",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x4B, reg_bc, write_lower, "C",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x4C, reg_bc, write_lower, "C",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x4D, reg_bc, write_lower, "C",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x4E, reg_bc, write_lower, "C", reg_hl, "HL"),
    ld_8bit_register_register!(0x4F, reg_bc, write_lower, "C",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x50, reg_de, write_higher, "D",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x51, reg_de, write_higher, "D",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x52, reg_de, write_higher, "D",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x53, reg_de, write_higher, "D",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x54, reg_de, write_higher, "D",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x55, reg_de, write_higher, "D",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x56, reg_de, write_higher, "D", reg_hl, "HL"),
    ld_8bit_register_register!(0x57, reg_de, write_higher, "D",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x58, reg_de, write_lower, "E",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x59, reg_de, write_lower, "E",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x5A, reg_de, write_lower, "E",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x5B, reg_de, write_lower, "E",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x5C, reg_de, write_lower, "E",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x5D, reg_de, write_lower, "E",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x5E, reg_de, write_lower, "E", reg_hl, "HL"),
    ld_8bit_register_register!(0x5F, reg_de, write_lower, "E",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x60, reg_hl, write_higher, "H",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x61, reg_hl, write_higher, "H",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x62, reg_hl, write_higher, "H",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x63, reg_hl, write_higher, "H",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x64, reg_hl, write_higher, "H",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x65, reg_hl, write_higher, "H",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x66, reg_hl, write_higher, "H", reg_hl, "HL"),
    ld_8bit_register_register!(0x67, reg_hl, write_higher, "H",  reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x68, reg_hl, write_lower, "L",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x69, reg_hl, write_lower, "L",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x6A, reg_hl, write_lower, "L",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x6B, reg_hl, write_lower, "L",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x6C, reg_hl, write_lower, "L",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x6D, reg_hl, write_lower, "L",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x6E, reg_hl, write_lower, "L", reg_hl, "HL"),
    ld_8bit_register_register!(0x6F, reg_hl, write_lower, "L",  reg_af, read_higher, "A"),

    ld_pointer_register!(0x70, reg_hl, "HL", reg_bc, read_higher, "B"),
    ld_pointer_register!(0x71, reg_hl, "HL", reg_bc, read_lower, "C"),
    ld_pointer_register!(0x72, reg_hl, "HL", reg_de, read_higher, "D"),
    ld_pointer_register!(0x73, reg_hl, "HL", reg_de, read_lower, "E"),
    ld_pointer_register!(0x74, reg_hl, "HL", reg_hl, read_higher, "H"),
    ld_pointer_register!(0x75, reg_hl, "HL", reg_hl, read_lower, "L"),
    ld_pointer_register!(0x77, reg_hl, "HL", reg_af, read_higher, "A"),

    ld_8bit_register_register!(0x78, reg_af, write_a, "A",  reg_bc, read_higher, "B"),
    ld_8bit_register_register!(0x79, reg_af, write_a, "A",  reg_bc, read_lower, "C"),
    ld_8bit_register_register!(0x7A, reg_af, write_a, "A",  reg_de, read_higher, "D"),
    ld_8bit_register_register!(0x7B, reg_af, write_a, "A",  reg_de, read_lower, "E"),
    ld_8bit_register_register!(0x7C, reg_af, write_a, "A",  reg_hl, read_higher, "H"),
    ld_8bit_register_register!(0x7D, reg_af, write_a, "A",  reg_hl, read_lower, "L"),
    ld_register_pointer!(0x7E, reg_af, write_a, "A", reg_hl, "HL"),
    ld_8bit_register_register!(0x7F, reg_af, write_a, "A",  reg_af, read_higher, "A"),

    add!(0x80, reg_bc, read_higher, "B"),
    add!(0x81, reg_bc, read_lower, "C"),
    add!(0x82, reg_de, read_higher, "D"),
    add!(0x83, reg_de, read_lower, "E"),
    add!(0x84, reg_hl, read_higher, "H"),
    add!(0x85, reg_hl, read_lower, "L"),
    add!(0x86, hl),
    add!(0x87, reg_af, read_a, "A"),

    sub!(0x90, reg_bc, read_higher, "B"),
    sub!(0x91, reg_bc, read_lower, "C"),
    sub!(0x92, reg_de, read_higher, "D"),
    sub!(0x93, reg_de, read_lower, "E"),
    sub!(0x94, reg_hl, read_higher, "H"),
    sub!(0x95, reg_hl, read_lower, "L"),
    sub!(0x96, hl),
    sub!(0x97, reg_af, read_a, "A"),

    Instruction{opcode: 0xAF, mnemonic: "XOR A", description: "XOR A with A (zeroes A)",
        length_in_bytes: 1, cycles: "4", flags_changed: "Z000",
        implementation: |cpu| {
            cpu.cycle_count += 4;
            cpu.reg_af.write_a(0);
            cpu.reg_af.flags.insert(Flags::Z);
        } },


    cp!(0xB8, reg_bc, read_higher, "B"),
    cp!(0xB9, reg_bc, read_lower, "C"),
    cp!(0xBA, reg_de, read_higher, "D"),
    cp!(0xBB, reg_de, read_lower, "E"),
    cp!(0xBC, reg_hl, read_higher, "H"),
    cp!(0xBD, reg_hl, read_lower, "L"),
    cp!(0xBE, hl),
    cp!(0xBF, reg_af, read_a, "A"),

    pop!(0xC1, reg_bc, "BC"),
    jump!(0xC2, Flags::Z, false, "NZ"),
    jump!(0xC3),
    push!(0xC5, reg_bc, "BC"),
    add!(0xC6, immediate),

    Instruction{opcode: 0xC9, mnemonic: "RET", description: "Return",
        length_in_bytes: 1, cycles: "16", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 16;
            let new_pc = cpu.pop_u16_from_stack();
            cpu.program_counter.write(new_pc);
        } },

    jump!(0xCA, Flags::Z, true, "Z"),

    Instruction{opcode: 0xCB, mnemonic: "CB", description: "CB prefix",
        length_in_bytes: 0, cycles: "0", flags_changed: "",
        implementation: |cpu| cpu.run_cb_op() },

    Instruction{opcode: 0xCD, mnemonic: "CALL", description: "Call",
        length_in_bytes: 3, cycles: "24", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 24;
            let new_pc = cpu.pop_u16_from_pc();
            cpu.push_u16_to_stack(cpu.program_counter.read());
            cpu.program_counter.write(new_pc);
        } },

    pop!(0xD1, reg_de, "DE"),
    jump!(0xD2, Flags::C, false, "NC"),
    push!(0xD5, reg_de, "DE"),
    sub!(0xD6, immediate),
    jump!(0xDA, Flags::C, true, "C"),

    Instruction{opcode: 0xE0, mnemonic: "LD ($FF00+imm), A", description: "Put A to pointer 0xFF00 + immediate",
        length_in_bytes: 2, cycles: "12", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 12;
            let address = 0xFF00 + (cpu.pop_u8_from_pc() as u16);
            cpu.bus.write(address, cpu.reg_af.read_a());
        } },

    pop!(0xE1, reg_hl, "HL"),

    Instruction{opcode: 0xE2, mnemonic: "LD ($FF00+C), A", description: "Put A to pointer 0xFF00 + C",
        length_in_bytes: 1, cycles: "8", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 8;
            let address = 0xFF00 + (cpu.reg_bc.read_lower() as u16);
            cpu.bus.write(address, cpu.reg_af.read_a());
        } },

    push!(0xE5, reg_hl, "HL"),

    jump!(0xE9, hl),

    Instruction{opcode: 0xEA, mnemonic: "LD (a16), A", description: "Load A to immediate pointer",
        length_in_bytes: 3, cycles: "16", flags_changed: "----",
        implementation: |cpu| {
            cpu.cycle_count += 16;
            let immediate = cpu.pop_u16_from_pc();
            cpu.bus.write(immediate, cpu.reg_af.read_a());
        } },

    Instruction{opcode: 0xF0, mnemonic: "LD A, ($FF00+imm)", description: "Put pointer 0xFF00 + immediate to A",
        length_in_bytes: 2, cycles: "12", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 12;
            let address = 0xFF00 + (cpu.pop_u8_from_pc() as u16);
            cpu.reg_af.write_a(cpu.bus.read(address));
        } },

    pop!(0xF1, reg_af, "AF"),

    Instruction{opcode: 0xF3, mnemonic: "DI", description: "Disable interrupts",
        length_in_bytes: 1, cycles: "4", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 4;
            cpu.interrupts_enabled = false;
        } },

    push!(0xF5, reg_af, "AF"),

    Instruction{opcode: 0xFB, mnemonic: "EI", description: "Enable interrupts",
        length_in_bytes: 1, cycles: "4", flags_changed: "",
        implementation: |cpu| {
            cpu.cycle_count += 4;
            cpu.interrupts_enabled = true;
        } },

    cp!(0xFE, immediate),
];

pub const INSTRUCTIONS_CB: [Instruction; 8] = [

    rotate_left_trough_carry!(0x10, reg_bc, read_higher, write_higher, "B", regular),
    rotate_left_trough_carry!(0x11, reg_bc, read_lower, write_lower, "C", regular),
    rotate_left_trough_carry!(0x12, reg_de, read_higher, write_higher, "D", regular),
    rotate_left_trough_carry!(0x13, reg_de, read_lower, write_lower, "E", regular),
    rotate_left_trough_carry!(0x14, reg_hl, read_higher, write_higher, "H", regular),
    rotate_left_trough_carry!(0x15, reg_hl, read_lower, write_lower, "L", regular),

    rotate_left_trough_carry!(0x17, reg_af, read_lower, write_lower, "A", regular),


    Instruction{opcode: 0x7C, mnemonic: "BIT 7,H", description: "Test bit 7 of H",
        length_in_bytes: 2, cycles: "8", flags_changed: "Z01-",
        implementation: |cpu| {
            cpu.cycle_count += 8;
            cpu.reg_af.flags.remove(Flags::N);
            cpu.reg_af.flags.insert(Flags::H);
            cpu.reg_af.flags.set(Flags::Z, (cpu.reg_hl.read_higher() & (1 << 7)) == 0)
        } },

];

#[cfg(test)]
mod tests {
    use super::CPU;
    use super::Flags;
    use crate::bus::Bus;
    use crate::cpu::register::DMGRegister;

    #[test]
    fn xor_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0xAF], vec![]));
        cpu.reg_af.write_a(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0);
        assert_eq!(cpu.reg_af.flags, Flags::Z)
    }

    #[test]
    fn inc_b() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x04], vec![]));
        cpu.reg_bc.write_higher(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn inc_c() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x0C], vec![]));
        cpu.reg_bc.write_lower(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_lower(), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn inc_d() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x14], vec![]));
        cpu.reg_de.write_higher(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_de.read_higher(), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn inc_e() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x1C], vec![]));
        cpu.reg_de.write_lower(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_de.read_lower(), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn inc_h() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x24], vec![]));
        cpu.reg_hl.write_higher(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_hl.read_higher(), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn inc_l() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x2C], vec![]));
        cpu.reg_hl.write_lower(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_hl.read_lower(), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn inc_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x3C], vec![]));
        cpu.reg_af.write_higher(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_af.read_higher(), 0x50);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn dec_b() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x05], vec![]));
        cpu.reg_bc.write_higher(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0x4E);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(cpu.reg_af.flags.contains(Flags::N));
        assert!(!cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn dec_c() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x0D], vec![]));
        cpu.reg_bc.write_lower(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_lower(), 0x4E);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(cpu.reg_af.flags.contains(Flags::N));
        assert!(!cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn dec_d() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x15], vec![]));
        cpu.reg_de.write_higher(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_de.read_higher(), 0x4E);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(cpu.reg_af.flags.contains(Flags::N));
        assert!(!cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn dec_e() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x1D], vec![]));
        cpu.reg_de.write_lower(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_de.read_lower(), 0x4E);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(cpu.reg_af.flags.contains(Flags::N));
        assert!(!cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn dec_h() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x25], vec![]));
        cpu.reg_hl.write_higher(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_hl.read_higher(), 0x4E);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(cpu.reg_af.flags.contains(Flags::N));
        assert!(!cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn dec_l() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x2D], vec![]));
        cpu.reg_hl.write_lower(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_hl.read_lower(), 0x4E);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(cpu.reg_af.flags.contains(Flags::N));
        assert!(!cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn dec_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x3D], vec![]));
        cpu.reg_af.write_higher(0x4F);
        cpu.step();
        assert_eq!(cpu.reg_af.read_higher(), 0x4E);
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
        assert!(cpu.reg_af.flags.contains(Flags::N));
        assert!(!cpu.reg_af.flags.contains(Flags::H));
    }

    #[test]
    fn inc_bc() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x03], vec![]));
        cpu.reg_bc.write(0x4F4F);
        cpu.step();
        assert_eq!(cpu.reg_bc.read(), 0x4F50);
    }

    #[test]
    fn inc_de() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x13], vec![]));
        cpu.reg_de.write(0x4F4F);
        cpu.step();
        assert_eq!(cpu.reg_de.read(), 0x4F50);
    }

    #[test]
    fn inc_hl() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x23], vec![]));
        cpu.reg_hl.write(0x4F4F);
        cpu.step();
        assert_eq!(cpu.reg_hl.read(), 0x4F50);
    }

    #[test]
    fn inc_sp() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x33], vec![]));
        cpu.stack_pointer.write(0x4F4F);
        cpu.step();
        assert_eq!(cpu.stack_pointer.read(), 0x4F50);
    }

    #[test]
    fn dec_bc() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x0B], vec![]));
        cpu.reg_bc.write(0x4F4F);
        cpu.step();
        assert_eq!(cpu.reg_bc.read(), 0x4F4E);
    }

    #[test]
    fn dec_de() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x1B], vec![]));
        cpu.reg_de.write(0x4F4F);
        cpu.step();
        assert_eq!(cpu.reg_de.read(), 0x4F4E);
    }

    #[test]
    fn dec_hl() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x2B], vec![]));
        cpu.reg_hl.write(0x4F4F);
        cpu.step();
        assert_eq!(cpu.reg_hl.read(), 0x4F4E);
    }

    #[test]
    fn dec_sp() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x3B], vec![]));
        cpu.stack_pointer.write(0x4F4F);
        cpu.step();
        assert_eq!(cpu.stack_pointer.read(), 0x4F4E);
    }

    #[test]
    fn ld_de_d16() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x11, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.reg_de.read(), 0x1234);
    }

    #[test]
    fn ld_hl_d16() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x21, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.reg_hl.read(), 0x1234);
    }

    #[test]
    fn ld_sp_d16() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x31, 0x34, 0x12], vec![]));
        cpu.step();
        assert_eq!(cpu.stack_pointer.read(), 0x1234);
    }

    #[test]
    fn ld_pointer_hl_a_and_decrement() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x32], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
        assert_eq!(cpu.reg_hl.read(), 0xC122);
    }

    #[test]
    fn ld_pointer_hl_a_and_increment() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x22], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
        assert_eq!(cpu.reg_hl.read(), 0xC124);
    }

    #[test]
    fn ld_pointer_bc_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x02], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.reg_bc.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
    }

    #[test]
    fn ld_pointer_de_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x12], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.reg_de.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
    }

    #[test]
    fn ld_pointer_hl_b() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x70], vec![]));
        cpu.reg_bc.write_higher(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
    }

    #[test]
    fn ld_pointer_hl_c() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x71], vec![]));
        cpu.reg_bc.write_lower(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
    }

    #[test]
    fn ld_pointer_hl_d() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x72], vec![]));
        cpu.reg_de.write_higher(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
    }

    #[test]
    fn ld_pointer_hl_e() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x73], vec![]));
        cpu.reg_de.write_lower(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
    }

    #[test]
    fn ld_pointer_hl_h() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x74], vec![]));
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xC1);
    }

    #[test]
    fn ld_pointer_hl_l() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x75], vec![]));
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0x23);
    }

    #[test]
    fn ld_pointer_hl_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x77], vec![]));
        cpu.reg_af.write_higher(0xF0);
        cpu.reg_hl.write(0xC123);
        cpu.step();
        assert_eq!(cpu.bus.read(0xC123), 0xF0);
    }

    #[test]
    fn ld_pointer_immediate_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0xEA, 0xC0, 0xC1], vec![]));
        cpu.reg_af.write_higher(0xF0);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0003);
        assert_eq!(cpu.bus.read(0xC1C0), 0xF0);
    }

    #[test]
    fn ld_a_pointer_de() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x1A, 0x55], vec![]));
        cpu.reg_de.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0x55);
    }

    #[test]
    fn ld_a_pointer_bc() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x0A, 0x55], vec![]));
        cpu.reg_bc.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0x55);
    }

    #[test]
    fn ld_a_pointer_hl_increment() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x2A, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0x55);
        assert_eq!(cpu.reg_hl.read(), 0x0002);
    }

    #[test]
    fn ld_a_pointer_hl_decrement() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x3A, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_af.read_a(), 0x55);
        assert_eq!(cpu.reg_hl.read(), 0x0000);
    }

    #[test]
    fn ld_b_pointer_hl() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x46, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0x55);
    }

    #[test]
    fn ld_c_pointer_hl() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x4E, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_lower(), 0x55);
    }

    #[test]
    fn ld_d_pointer_hl() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x56, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_de.read_higher(), 0x55);
    }

    #[test]
    fn ld_e_pointer_hl() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x5E, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_de.read_lower(), 0x55);
    }

    #[test]
    fn ld_h_pointer_hl() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x66, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_hl.read_higher(), 0x55);
    }

    #[test]
    fn ld_l_pointer_hl() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0x6E, 0x55], vec![]));
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.reg_hl.read_lower(), 0x55);
    }

    #[test]
    fn ld_high_immediate_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0xE0, 0xF5], vec![]));
        cpu.reg_af.write_a(0xF0);
        cpu.step();
        assert_eq!(cpu.cycle_count, 12);
        assert_eq!(cpu.bus.read(0xFFF5), 0xF0);
    }

    #[test]
    fn ld_a_high_immediate() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0xF0, 0xF5], vec![]));
        cpu.bus.write(0xFFF5, 0x12);
        cpu.step();
        assert_eq!(cpu.cycle_count, 12);
        assert_eq!(cpu.reg_af.read_a(), 0x12);
    }

    #[test]
    fn ld_pointer_c_a() {
        let mut cpu = CPU::new(
            Bus::new_from_vecs(vec![0xE2], vec![]));
        cpu.reg_af.write_a(0xCC);
        cpu.reg_bc.write_lower(0xF0);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.bus.read(0xFFF0), 0xCC);
    }

    #[test]
    fn bit_7_h_to_one() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xCB, 0x7C], vec![]));
        cpu.reg_hl.write(0xF000);
        cpu.step();
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
        assert!(!cpu.reg_af.flags.contains(Flags::Z));
    }

    #[test]
    fn bit_7_h_to_zero() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xCB, 0x7C], vec![]));
        cpu.reg_hl.write(0x0F00);
        cpu.step();
        assert!(!cpu.reg_af.flags.contains(Flags::N));
        assert!(cpu.reg_af.flags.contains(Flags::H));
        assert!(cpu.reg_af.flags.contains(Flags::Z));
    }

    #[test]
    fn jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xC3, 0x12, 0x34], vec![]));
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x3412);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jpnz() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xC2, 0x12, 0x34], vec![]));
        cpu.reg_af.flags.set(Flags::Z, false);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x3412);
        assert_eq!(cpu.cycle_count, 16);
    }

    #[test]
    fn jpnz_no_jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xC2, 0x12, 0x34], vec![]));
        cpu.reg_af.flags.set(Flags::Z, true);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x0003);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jpz() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xCA, 0x12, 0x34], vec![]));
        cpu.reg_af.flags.set(Flags::Z, true);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x3412);
        assert_eq!(cpu.cycle_count, 16);
    }

    #[test]
    fn jpz_no_jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xCA, 0x12, 0x34], vec![]));
        cpu.reg_af.flags.set(Flags::Z, false);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x0003);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jpnc() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xD2, 0x12, 0x34], vec![]));
        cpu.reg_af.flags.set(Flags::C, false);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x3412);
        assert_eq!(cpu.cycle_count, 16);
    }

    #[test]
    fn jpnc_no_jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xD2, 0x12, 0x34], vec![]));
        cpu.reg_af.flags.set(Flags::C, true);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x0003);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jpc() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xDA, 0x12, 0x34], vec![]));
        cpu.reg_af.flags.set(Flags::C, true);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x3412);
        assert_eq!(cpu.cycle_count, 16);
    }

    #[test]
    fn jpc_no_jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xDA, 0x12, 0x34], vec![]));
        cpu.reg_af.flags.set(Flags::C, false);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x0003);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jp_hl() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xE9, 0x12, 0x34], vec![]));
        cpu.reg_hl.write(0x0002);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x34);
        assert_eq!(cpu.cycle_count, 4);
    }

    #[test]
    fn jr() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x18, 0x33], vec![]));
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x35);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jrnz_no_jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x20, 0x33], vec![]));
        cpu.reg_af.flags.insert(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x02);
        assert_eq!(cpu.cycle_count, 8);
    }

    #[test]
    fn jrnz_jump_positive() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x20, 0x33], vec![]));
        cpu.reg_af.flags.remove(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x35);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jrnz_jump_negative() {
        // Jump -3
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x20, 0xFD], vec![]));
        cpu.reg_af.flags.remove(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0xFFFF);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jrz_no_jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x28, 0x33], vec![]));
        cpu.reg_af.flags.remove(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x02);
        assert_eq!(cpu.cycle_count, 8);
    }

    #[test]
    fn jrz_jump_positive() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x28, 0x33], vec![]));
        cpu.reg_af.flags.insert(Flags::Z);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x35);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jrnc_no_jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x30, 0x33], vec![]));
        cpu.reg_af.flags.insert(Flags::C);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x02);
        assert_eq!(cpu.cycle_count, 8);
    }

    #[test]
    fn jrnc_jump_positive() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x30, 0x33], vec![]));
        cpu.reg_af.flags.remove(Flags::C);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x35);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn jrc_no_jump() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x38, 0x33], vec![]));
        cpu.reg_af.flags.remove(Flags::C);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x02);
        assert_eq!(cpu.cycle_count, 8);
    }

    #[test]
    fn jrc_jump_positive() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x38, 0x33], vec![]));
        cpu.reg_af.flags.insert(Flags::C);
        cpu.step();
        assert_eq!(cpu.program_counter.read(), 0x35);
        assert_eq!(cpu.cycle_count, 12);
    }

    #[test]
    fn call() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xCD, 0x34, 0x12], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.step();
        assert_eq!(cpu.cycle_count, 24);
        assert_eq!(cpu.program_counter.read(), 0x1234);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.bus.read(0xCFFF), 0x03);
        assert_eq!(cpu.bus.read(0xCFFE), 0x00);
    }

    #[test]
    fn ret() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xC9], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.push_u16_to_stack(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x1234);
        assert_eq!(cpu.stack_pointer.read(), 0xD000);
    }

    #[test]
    fn ld_b_b() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x40], vec![]));
        cpu.reg_bc.write_higher(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5);
    }

    #[test]
    fn ld_b_c() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x41], vec![]));
        cpu.reg_bc.write_lower(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5);
    }

    #[test]
    fn ld_b_d() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x42], vec![]));
        cpu.reg_de.write_higher(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5);
    }

    #[test]
    fn ld_b_e() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x43], vec![]));
        cpu.reg_de.write_lower(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5);
    }

    #[test]
    fn ld_b_h() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x44], vec![]));
        cpu.reg_hl.write_higher(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5)
    }

    #[test]
    fn ld_b_l() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x45], vec![]));
        cpu.reg_hl.write_lower(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5)
    }

    #[test]
    fn ld_b_a() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x47], vec![]));
        cpu.reg_af.write_higher(0xF5);
        cpu.step();
        assert_eq!(cpu.reg_bc.read_higher(), 0xF5)
    }

    #[test]
    fn ld_b_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x06, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_bc.read_higher(), 0xBB);
    }

    #[test]
    fn ld_c_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x0E, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_bc.read_lower(), 0xBB);
    }

    #[test]
    fn ld_d_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x16, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_de.read_higher(), 0xBB);
    }

    #[test]
    fn ld_e_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x1E, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_de.read_lower(), 0xBB);
    }

    #[test]
    fn ld_h_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x26, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_hl.read_higher(), 0xBB);
    }

    #[test]
    fn ld_l_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x2E, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_hl.read_lower(), 0xBB);
    }

    #[test]
    fn ld_a_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x3E, 0xBB], vec![]));
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.reg_af.read_higher(), 0xBB);
    }

    #[test]
    fn push_bc() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xC5], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.reg_bc.write(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.bus.read(0xCFFF), 0x34);
        assert_eq!(cpu.bus.read(0xCFFE), 0x12);
    }

    #[test]
    fn push_de() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xD5], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.reg_de.write(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.bus.read(0xCFFF), 0x34);
        assert_eq!(cpu.bus.read(0xCFFE), 0x12);
    }

    #[test]
    fn push_hl() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xE5], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.reg_hl.write(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.bus.read(0xCFFF), 0x34);
        assert_eq!(cpu.bus.read(0xCFFE), 0x12);
    }

    #[test]
    fn push_af() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xF5], vec![]));
        cpu.stack_pointer.write(0xD000);
        cpu.reg_af.write(0x1234);
        cpu.step();
        assert_eq!(cpu.cycle_count, 16);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xCFFE);
        assert_eq!(cpu.bus.read(0xCFFF), 0x34);
        assert_eq!(cpu.bus.read(0xCFFE), 0x12);
    }

    #[test]
    fn pop_bc() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xC1], vec![]));
        cpu.stack_pointer.write(0xCFFE);
        cpu.bus.write(0xCFFF, 0x34);
        cpu.bus.write(0xCFFE, 0x12);
        cpu.step();
        assert_eq!(cpu.cycle_count, 12);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xD000);
        assert_eq!(cpu.reg_bc.read(), 0x1234);
    }

    #[test]
    fn pop_de() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xD1], vec![]));
        cpu.stack_pointer.write(0xCFFE);
        cpu.bus.write(0xCFFF, 0x34);
        cpu.bus.write(0xCFFE, 0x12);
        cpu.step();
        assert_eq!(cpu.cycle_count, 12);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xD000);
        assert_eq!(cpu.reg_de.read(), 0x1234);
    }

    #[test]
    fn pop_hl() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xE1], vec![]));
        cpu.stack_pointer.write(0xCFFE);
        cpu.bus.write(0xCFFF, 0x34);
        cpu.bus.write(0xCFFE, 0x12);
        cpu.step();
        assert_eq!(cpu.cycle_count, 12);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xD000);
        assert_eq!(cpu.reg_hl.read(), 0x1234);
    }

    #[test]
    fn pop_af() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xF1], vec![]));
        cpu.stack_pointer.write(0xCFFE);
        cpu.bus.write(0xCFFF, 0x34);
        cpu.bus.write(0xCFFE, 0x12);
        cpu.step();
        assert_eq!(cpu.cycle_count, 12);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.stack_pointer.read(), 0xD000);
        assert_eq!(cpu.reg_af.read(), 0x1234);
    }

    #[test]
    fn rl_c_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xCB, 0x11], vec![]));
        cpu.reg_bc.write_lower(0b01010010);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_bc.read_lower(), 0b10100100);
        assert_eq!(cpu.reg_af.flags, Flags::empty());
    }

    #[test]
    fn rl_c_to_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xCB, 0x11], vec![]));
        cpu.reg_bc.write_lower(0b11010010);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_bc.read_lower(), 0b10100100);
        assert_eq!(cpu.reg_af.flags, Flags::C);
    }

    #[test]
    fn rl_c_from_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xCB, 0x11], vec![]));
        cpu.reg_bc.write_lower(0);
        cpu.reg_af.flags.insert(Flags::C);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_bc.read_lower(), 1);
        assert_eq!(cpu.reg_af.flags, Flags::empty());
    }

    #[test]
    fn rla_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x17], vec![]));
        cpu.reg_af.write_higher(0b01010010);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.read_higher(), 0b10100100);
        assert_eq!(cpu.reg_af.flags, Flags::empty());
    }

    #[test]
    fn rla_to_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x17], vec![]));
        cpu.reg_af.write_higher(0b11010010);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.read_higher(), 0b10100100);
        assert_eq!(cpu.reg_af.flags, Flags::C);
    }

    #[test]
    fn rla_from_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x17], vec![]));
        cpu.reg_af.write_higher(0);
        cpu.reg_af.flags.insert(Flags::C);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.read_higher(), 1);
        assert_eq!(cpu.reg_af.flags, Flags::empty());
    }

    #[test]
    fn cp_a_zero() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xBF], vec![]));
        cpu.reg_af.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::Z | Flags::N);
    }

    #[test]
    fn cp_b_half_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xB8], vec![]));
        cpu.reg_af.write_higher(0x13);
        cpu.reg_bc.write_higher(0x04);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::N | Flags::H);
    }

    #[test]
    fn cp_c_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xB9], vec![]));
        cpu.reg_af.write_higher(0x11);
        cpu.reg_bc.write_lower(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::N);
    }

    #[test]
    fn cp_d_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xBA], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.reg_de.write_higher(0x06);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H | Flags::N);
    }

    #[test]
    fn cp_e_half_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xBB], vec![]));
        cpu.reg_af.write_higher(0x13);
        cpu.reg_de.write_lower(0x04);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::N | Flags::H);
    }

    #[test]
    fn cp_h_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xBC], vec![]));
        cpu.reg_af.write_higher(0x11);
        cpu.reg_hl.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::N);
    }

    #[test]
    fn cp_l_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xBD], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.reg_hl.write_lower(0x06);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H | Flags::N);
    }

    #[test]
    fn cp_hl_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xBE, 0x06], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H | Flags::N);
    }

    #[test]
    fn cp_immediate_zero() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xFE, 0x10], vec![]));
        cpu.reg_af.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_af.flags, Flags::Z | Flags::N);
    }

    #[test]
    fn cp_immediate_half_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xFE, 0x9], vec![]));
        cpu.reg_af.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_af.flags, Flags::N | Flags::H);
    }

    #[test]
    fn cp_immediate_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xFE, 0x1], vec![]));
        cpu.reg_af.write_higher(0x11);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_af.flags, Flags::N);
    }

    #[test]
    fn cp_immediate_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xFE, 0x11], vec![]));
        cpu.reg_af.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H | Flags::N);
    }

    #[test]
    fn sub_a_zero() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x97], vec![]));
        cpu.reg_af.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::Z | Flags::N);
        assert_eq!(cpu.reg_af.read_higher(), 0);
    }

    #[test]
    fn sub_b_half_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x90], vec![]));
        cpu.reg_af.write_higher(0x13);
        cpu.reg_bc.write_higher(0x04);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::N | Flags::H);
        assert_eq!(cpu.reg_af.read_higher(), 0x0F);
    }

    #[test]
    fn sub_c_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x91], vec![]));
        cpu.reg_af.write_higher(0x11);
        cpu.reg_bc.write_lower(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::N);
        assert_eq!(cpu.reg_af.read_higher(), 0x01);
    }

    #[test]
    fn sub_d_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x92], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.reg_de.write_higher(0x06);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H | Flags::N);
        assert_eq!(cpu.reg_af.read_higher(), 0xFF);
    }

    #[test]
    fn sub_e_half_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x93], vec![]));
        cpu.reg_af.write_higher(0x13);
        cpu.reg_de.write_lower(0x04);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::N | Flags::H);
        assert_eq!(cpu.reg_af.read_higher(), 0x0F);
    }

    #[test]
    fn sub_h_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x94], vec![]));
        cpu.reg_af.write_higher(0x11);
        cpu.reg_hl.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::N);
        assert_eq!(cpu.reg_af.read_higher(), 0x01);
    }

    #[test]
    fn sub_l_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x95], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.reg_hl.write_lower(0x06);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H | Flags::N);
        assert_eq!(cpu.reg_af.read_higher(), 0xFF);
    }

    #[test]
    fn sub_hl_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x96, 0x06], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H | Flags::N);
        assert_eq!(cpu.reg_af.read_higher(), 0xFF);
    }

    #[test]
    fn sub_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xD6, 0x06], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H | Flags::N);
        assert_eq!(cpu.reg_af.read_higher(), 0xFF);
    }

    #[test]
    fn add_a_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x87], vec![]));
        cpu.reg_af.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::default());
        assert_eq!(cpu.reg_af.read_higher(), 0x20);
    }

    #[test]
    fn add_b_half_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x80], vec![]));
        cpu.reg_af.write_higher(0x13);
        cpu.reg_bc.write_higher(0x0F);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::H);
        assert_eq!(cpu.reg_af.read_higher(), 0x22);
    }

    #[test]
    fn add_c_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x81], vec![]));
        cpu.reg_af.write_higher(0x11);
        cpu.reg_bc.write_lower(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::default());
        assert_eq!(cpu.reg_af.read_higher(), 0x21);
    }

    #[test]
    fn add_d_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x82], vec![]));
        cpu.reg_af.write_higher(0xFF);
        cpu.reg_de.write_higher(0x01);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::Z | Flags::C | Flags::H);
        assert_eq!(cpu.reg_af.read_higher(), 0x00);
    }

    #[test]
    fn add_e_half_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x83], vec![]));
        cpu.reg_af.write_higher(0x5F);
        cpu.reg_de.write_lower(0x12);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::H);
        assert_eq!(cpu.reg_af.read_higher(), 0x71);
    }

    #[test]
    fn add_h_no_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x84], vec![]));
        cpu.reg_af.write_higher(0x11);
        cpu.reg_hl.write_higher(0x10);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::default());
        assert_eq!(cpu.reg_af.read_higher(), 0x21);
    }

    #[test]
    fn add_l_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x85], vec![]));
        cpu.reg_af.write_higher(0xF5);
        cpu.reg_hl.write_lower(0x0F);
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H);
        assert_eq!(cpu.reg_af.read_higher(), 0x04);
    }

    #[test]
    fn add_hl_carry() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0x86, 0xFF], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.reg_hl.write(0x0001);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.reg_af.flags, Flags::C | Flags::H);
        assert_eq!(cpu.reg_af.read_higher(), 0x04);
    }

    #[test]
    fn add_immediate() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xC6, 0x06], vec![]));
        cpu.reg_af.write_higher(0x05);
        cpu.step();
        assert_eq!(cpu.cycle_count, 8);
        assert_eq!(cpu.program_counter.read(), 0x0002);
        assert_eq!(cpu.reg_af.flags, Flags::default());
        assert_eq!(cpu.reg_af.read_higher(), 0x0B);
    }

    #[test]
    fn disable_interrupts() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xF3], vec![]));
        cpu.interrupts_enabled = true;
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.interrupts_enabled, false);
    }

    #[test]
    fn enable_interrupts() {
        let mut cpu = CPU::new(Bus::new_from_vecs(vec![0xFB], vec![]));
        cpu.interrupts_enabled = false;
        cpu.step();
        assert_eq!(cpu.cycle_count, 4);
        assert_eq!(cpu.program_counter.read(), 0x0001);
        assert_eq!(cpu.interrupts_enabled, true);
    }

}
