use crate::device::Device;

#[derive(Debug)]
pub struct Memory {
    memory: Box<[u8]>,
}
impl Memory {
    pub fn new(size: u16) -> Memory {
        Memory {
            memory: vec![0; size as usize].into_boxed_slice(),
        }
    }
}
impl Device for Memory {
    fn get_u8(&self, address: usize) -> u8 {
        self.memory[address]
    }
    fn set_u8(&mut self, address: usize, value: u8) {
        self.memory[address] = value;
    }
    fn get_u16(&self, address: usize) -> u16 {
        u16::from_be_bytes([self.memory[address], self.memory[address + 1]])
    }
    fn set_u16(&mut self, address: usize, value: u16) {
        for (offset, &byte) in value.to_be_bytes().iter().enumerate() {
            self.memory[address + offset] = byte;
        }
    }
    fn len(&self) -> usize {
        self.memory.len()
    }

    fn set_mb(&mut self, _: u16) {}
}

#[cfg(test)]
mod tests {
    use super::Device;
    use super::Memory;

    #[test]
    fn test_memory() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, 12);
        assert_eq!(mem.get_u8(0), 12);
        mem.set_u16(2, 0x1234);
        assert_eq!(mem.get_u8(2), 0x12);
        assert_eq!(mem.get_u8(3), 0x34);
        assert_eq!(mem.get_u16(2), 0x1234);
    }
}
