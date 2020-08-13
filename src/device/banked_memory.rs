use super::Device;
use crate::device::memory::Memory;

pub struct BankedMemory {
    mb: u16,
    banks: Vec<Memory>,
    size: u16,
}

impl BankedMemory {
    pub fn new(count: u8, size: u16) -> BankedMemory {
        let mut banks = Vec::with_capacity(count as usize);
        for _ in 0..count {
            banks.push(Memory::new(size))
        }
        BankedMemory { mb: 0, banks, size }
    }
}

impl Device for BankedMemory {
    fn get_u16(&self, address: usize) -> u16 {
        self.banks[self.mb as usize].get_u16(address)
    }

    fn get_u8(&self, address: usize) -> u8 {
        self.banks[self.mb as usize].get_u8(address)
    }

    fn set_u16(&mut self, address: usize, value: u16) {
        self.banks[self.mb as usize].set_u16(address, value)
    }

    fn set_u8(&mut self, address: usize, value: u8) {
        self.banks[self.mb as usize].set_u8(address, value)
    }

    fn len(&self) -> usize {
        self.size as usize
    }

    fn set_mb(&mut self, mb: u16) {
        self.mb = mb;
    }
}

#[cfg(test)]
mod tests {}
