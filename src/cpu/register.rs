use bitflags::bitflags;

pub enum Subregister { Higher, Lower }

pub trait DMGRegister {
    fn read(&self) -> u16;
    fn write(&mut self, value: u16);
    fn inc(&mut self);
    fn overflowing_add(&mut self, value: u16);
    fn read_lower(&self) -> u8;
    fn write_lower(&mut self, value: u8);
    fn read_higher(&self) -> u8;
    fn write_higher(&mut self, value: u8);
    fn read_subreg(&self, subregister: Subregister) -> u8;
    fn write_subreg(&mut self, subregister: Subregister, value: u8);
}

pub struct Register16bit { value: u16 }

impl Register16bit {
    pub fn new() -> Register16bit { Register16bit{value: 0} }
}

impl DMGRegister for Register16bit {
    fn read(&self) -> u16 { self.value }
    fn write(&mut self, value: u16) { self.value = value }
    fn inc(&mut self) { self.value += 1 }
    fn overflowing_add(&mut self, value: u16) { self.value = self.value.overflowing_add(value).0 }
    fn read_lower(&self) -> u8 { self.value as u8 }
    fn write_lower(&mut self, value: u8) { self.value = (self.value & 0xFF00) + (value as u16) }
    fn read_higher(&self) -> u8 { (self.value >> 8) as u8 }
    fn write_higher(&mut self, value: u8) {
        self.value = (self.value & 0x00FF) + ((value as u16) << 8);
    }
    fn read_subreg(&self, subregister: Subregister) -> u8 {
        match subregister {
            Subregister::Higher => self.read_higher(),
            Subregister::Lower => self.read_lower(),
        }
    }
    fn write_subreg(&mut self, subregister: Subregister, value: u8) {
        match subregister {
            Subregister::Higher => self.write_higher(value),
            Subregister::Lower => self.write_lower(value)
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Flags: u8 {
        const Z = 0b10000000;
        const N = 0b01000000;
        const H = 0b00100000;
        const C = 0b00010000;
    }
}

impl Flags {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

pub struct AFRegister {
    a: u8,
    pub flags: Flags
}

impl AFRegister {
    pub fn new() -> AFRegister { AFRegister{a: 0, flags: Flags{bits:0} } }
    pub fn read_a(&self) -> u8 { self.read_higher() }
    pub fn write_a(&mut self, value: u8) { self.write_higher(value) }
}

impl DMGRegister for AFRegister {
    fn read(&self) -> u16 { ((self.a as u16) << 8) + (self.flags.bits as u16) }
    fn write(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.flags.bits = value as u8;
    }
    fn inc(&mut self) { panic!("Called inc on AF register") }
    fn overflowing_add(&mut self, _value: u16) { panic!() }
    fn read_lower(&self) -> u8 { self.flags.bits }
    fn write_lower(&mut self, value: u8) { self.flags.bits = value; }
    fn read_higher(&self) -> u8 { self.a }
    fn write_higher(&mut self, value: u8) { self.a = value; }
    fn read_subreg(&self, subregister: Subregister) -> u8 {
        match subregister {
            Subregister::Higher => self.read_higher(),
            Subregister::Lower => self.read_lower(),
        }
    }
    fn write_subreg(&mut self, subregister: Subregister, value: u8) {
        match subregister {
            Subregister::Higher => self.write_higher(value),
            Subregister::Lower => self.write_lower(value)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_16_bit() {
        let mut reg = Register16bit{value: 0};
        reg.write(0x1234);
        assert_eq!(reg.read_subreg(Subregister::Higher), 0x12);
        assert_eq!(reg.read_subreg(Subregister::Lower), 0x34);
        assert_eq!(reg.read(), 0x1234);
    }

    #[test]
    fn write_8_bit() {
        let mut reg = Register16bit{value: 0};
        reg.write_subreg(Subregister::Higher, 0x12);
        assert_eq!(reg.read(), 0x1200);
        assert_eq!(reg.read_subreg(Subregister::Higher), 0x12);
        assert_eq!(reg.read_subreg(Subregister::Lower), 0x00);
        reg.write_subreg(Subregister::Lower, 0x34);
        assert_eq!(reg.read(), 0x1234);
        assert_eq!(reg.read_subreg(Subregister::Higher), 0x12);
        assert_eq!(reg.read_subreg(Subregister::Lower), 0x34);
    }

    #[test]
    fn clear_flags() {
        let mut reg = Flags::Z;
        assert_eq!(reg.bits(), 0b10000000);
        reg.clear();
        assert_eq!(reg.bits(), 0);
    }
}
