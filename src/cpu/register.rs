pub struct Register16bit {pub value: u16}
pub struct Register8bit {pub value: u8}

impl Register16bit {
    pub fn lower_8bit(&self) -> Register8bit { Register8bit{value: self.value as u8} }
    pub fn higher_8bit(&self) -> Register8bit { Register8bit{value: (self.value >> 8) as u8} }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_lower() {
        let mut reg = Register16bit{value: 0x1234};
        assert_eq!(reg.lower_8bit().value, 0x34);
    }

    #[test]
    fn get_higher() {
        let mut reg = Register16bit{value: 0x1234};
        assert_eq!(reg.higher_8bit().value, 0x12);
    }
}
