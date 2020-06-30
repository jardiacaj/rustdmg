use super::*;


pub struct RAMBank {
    pub data: Vec<u8>,
    pub base_address: u16,
}

impl MemoryZone for RAMBank {
    fn read(&self, address: u16) -> u8 {
        self.data[self.global_address_to_local_address(address) as usize]
    }
    fn write(&mut self, address: u16, value: u8) {
        let local_address = self.global_address_to_local_address(address) as usize;
        self.data[local_address] = value;
    }
}

impl RAMBank {
    fn global_address_to_local_address(&self, address: u16) -> u16 { address - self.base_address }
}